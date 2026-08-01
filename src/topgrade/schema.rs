// SPDX-License-Identifier: GPL-3.0

//! The shape of topgrade's configuration, learned from `--config-reference`.
//!
//! topgrade ships its own documentation: `--config-reference` prints a complete,
//! commented example configuration — 509 lines across 36 sections in 17.9.0 —
//! describing every option it accepts, with prose explaining what each one does
//! and, usually, its default. That is a far better source than anything that
//! could be written here, because it comes from the binary being driven and so
//! is never out of date with it.
//!
//! What this module does is read that back into something an interface can be
//! built from: a list of sections, each holding settings with a key, a kind of
//! value, a default and the documentation topgrade wrote for it. A release that
//! adds an option gets a control for it here with no change to this crate; the
//! prose becomes the tooltip.
//!
//! ## What the reference looks like
//!
//! Options are presented commented out, preceded by their documentation:
//!
//! ```text
//! [misc]
//! # Periodically runs `sudo -n -v` to avoid password re-prompts (default: false)
//! # WARNING: This is a potential security risk; if you walk away from the
//! # computer while topgrade is running, another person can gain access.
//! # sudo_loop = true
//! ```
//!
//! So a run of comment lines is documentation until one of them turns out to be
//! an assignment, at which point the run so far belongs to it. The kind of
//! control to show is inferred from the example value, and refined by anything
//! the prose says about allowed values.

use std::fmt;

use super::{Error, Result, Topgrade};
use crate::debug::DISCOVER;
use crate::debug_log;

/// What kind of value a setting takes, and so what control represents it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueKind {
    /// A toggle.
    Bool,
    /// A whole number — an interval, a retry count.
    Integer,
    /// Free text: a command name, a path, an argument string.
    Text,
    /// One of a fixed set the documentation spells out.
    Enum { options: Vec<String> },
    /// A list of arbitrary strings.
    StringList,
    /// A list of step identifiers. Presented as a picker over the steps
    /// actually discovered, rather than a text box, since it is the same
    /// vocabulary `--only` and `--disable` use.
    StepList,
}

/// One configurable option.
#[derive(Clone, Debug)]
pub struct Setting {
    /// The TOML key, used verbatim when writing the file.
    pub key: String,
    /// The section it belongs to. Together with `key` this identifies it.
    pub section: String,
    pub kind: ValueKind,
    /// The value the reference shows, kept as written. Used to seed a control
    /// when the option is absent from the file and has no stated default.
    pub example: String,
    /// The default the documentation states, where it states one.
    pub default: Option<String>,
    /// topgrade's own prose, shown as help text. Kept whole rather than
    /// summarised: some of it is a security warning, and shortening that would
    /// be the wrong kind of tidy.
    pub doc: String,
}

impl Setting {
    /// A readable label, derived from the key.
    pub fn label(&self) -> String {
        let mut out = String::with_capacity(self.key.len());
        for (index, word) in self.key.split('_').enumerate() {
            if index > 0 {
                out.push(' ');
            }
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                out.extend(first.to_uppercase());
                out.push_str(chars.as_str());
            }
        }
        out
    }

    /// The first sentence of the documentation, for a one-line summary under
    /// the control. The rest stays available as the full tooltip.
    pub fn summary(&self) -> &str {
        let doc = self.doc.trim();
        match doc.find(". ") {
            Some(end) => &doc[..=end],
            None => doc,
        }
    }
}

/// A group of options, as topgrade's own file groups them.
#[derive(Clone, Debug)]
pub struct Section {
    /// The TOML table name — `misc`, `linux`, `git`.
    pub name: String,
    /// Prose introducing the section, where there is any.
    pub doc: String,
    pub settings: Vec<Setting>,
    /// Whether the section is a free-form map of user-chosen names to values
    /// rather than a fixed set of options.
    ///
    /// `[commands]`, `[pre_commands]` and `[post_commands]` are these: the keys
    /// are labels the user invents. They need an editor that adds and removes
    /// rows, not a form of known controls.
    pub free_form: bool,
}

/// Everything topgrade will accept in its configuration file.
#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub sections: Vec<Section>,
}

impl Schema {
    pub fn section(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|section| section.name == name)
    }

    pub fn setting(&self, section: &str, key: &str) -> Option<&Setting> {
        self.section(section)?
            .settings
            .iter()
            .find(|setting| setting.key == key)
    }

    pub fn settings(&self) -> impl Iterator<Item = &Setting> {
        self.sections
            .iter()
            .flat_map(|section| section.settings.iter())
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} sections, {} settings",
            self.sections.len(),
            self.settings().count()
        )
    }
}

/// Ask topgrade to describe its configuration, and read the answer.
pub async fn load(topgrade: &Topgrade) -> Result<Schema> {
    let reference = topgrade.output(&["--config-reference"]).await?;
    let schema = parse(&reference)?;
    debug_log!(DISCOVER, "config schema: {schema}");
    Ok(schema)
}

/// Sections whose keys are chosen by the user rather than by topgrade.
///
/// Detected from the documentation rather than listed, so a release that adds
/// another such section is handled without a change here. The reference
/// introduces each of them with prose about running commands, and the examples
/// underneath use quoted names — `"Emacs Snapshot" = "…"` — which is what
/// [`parse_assignment`] declines to read as a setting.
fn is_free_form(name: &str, settings: &[Setting]) -> bool {
    settings.is_empty() && (name.ends_with("commands") || name == "commands")
}

/// Read the reference into a schema.
fn parse(reference: &str) -> Result<Schema> {
    let mut sections: Vec<Section> = Vec::new();
    let mut docs: Vec<String> = Vec::new();

    let mut lines = reference.lines().peekable();
    while let Some(raw) = lines.next() {
        let line = raw.trim();

        // A blank line ends a run of documentation. Without this, prose from
        // one option would drift down onto whichever option came next.
        if line.is_empty() {
            docs.clear();
            continue;
        }

        if let Some(name) = parse_section_header(line) {
            // Prose immediately above a header introduces the section rather
            // than any one option in it.
            let doc = join_docs(&docs);
            docs.clear();
            sections.push(Section {
                name: name.to_owned(),
                doc,
                settings: Vec::new(),
                free_form: false,
            });
            continue;
        }

        let Some(body) = line.strip_prefix('#') else {
            // The reference comments everything out. An uncommented line is not
            // something this knows how to read, so it is left alone.
            continue;
        };
        let body = body.trim();
        if body.is_empty() {
            docs.clear();
            continue;
        }

        match parse_assignment(body) {
            Some((key, value)) => {
                // A list example can be spread over several comment lines, as
                // conda's `env_names` is. Pull in the continuation before
                // deciding what kind of value this is.
                let value = if starts_unclosed_list(&value) {
                    let mut value = value;
                    while let Some(next) = lines.peek() {
                        let Some(more) = next.trim().strip_prefix('#') else {
                            break;
                        };
                        value.push(' ');
                        value.push_str(more.trim());
                        lines.next();
                        if !starts_unclosed_list(&value) {
                            break;
                        }
                    }
                    value
                } else {
                    value
                };

                let doc = join_docs(&docs);
                docs.clear();

                let Some(section) = sections.last_mut() else {
                    // An assignment before any section header would belong to
                    // TOML's implicit root table, which topgrade does not use.
                    continue;
                };

                let kind = infer_kind(&key, &value, &doc);
                section.settings.push(Setting {
                    key,
                    section: section.name.clone(),
                    kind,
                    default: stated_default(&doc),
                    example: value,
                    doc,
                });
            }
            None => docs.push(body.to_owned()),
        }
    }

    for section in &mut sections {
        section.free_form = is_free_form(&section.name, &section.settings);
    }

    if sections.is_empty() {
        return Err(Error::Parse {
            what: "the configuration reference",
            detail: "no sections found".to_owned(),
        });
    }

    Ok(Schema { sections })
}

/// Read `[name]` as a section header.
fn parse_section_header(line: &str) -> Option<&str> {
    let inner = line.strip_prefix('[')?.strip_suffix(']')?;
    let inner = inner.trim();
    // Reject `[[array]]` tables and anything with structure this does not model.
    if inner.is_empty() || !inner.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    Some(inner)
}

/// Read `key = value`, but only where `key` is a bare TOML identifier.
///
/// Quoted keys are how the free-form sections write their user-chosen names —
/// `"Emacs Snapshot" = "…"` — and those are examples of what a user might add
/// rather than options to offer a control for, so they are declined here and
/// the section is marked free-form instead.
fn parse_assignment(body: &str) -> Option<(String, String)> {
    let (key, value) = body.split_once('=')?;
    let key = key.trim();
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return None;
    }

    let value = strip_trailing_comment(value.trim());
    Some((key.to_owned(), value.trim().to_owned()))
}

/// Drop an end-of-line comment from an example value.
///
/// The reference writes them: `sudo_loop_interval = 240 # 4 minutes (240s)`.
/// Only `#` outside a quoted string counts, so a value that legitimately
/// contains one survives.
fn strip_trailing_comment(value: &str) -> &str {
    let mut in_string = false;
    let mut previous = '\0';
    for (index, c) in value.char_indices() {
        match c {
            '"' if previous != '\\' => in_string = !in_string,
            '#' if !in_string => return &value[..index],
            _ => {}
        }
        previous = c;
    }
    value
}

/// Whether a list example has been opened but not closed on this line.
fn starts_unclosed_list(value: &str) -> bool {
    let opens = value.matches('[').count();
    let closes = value.matches(']').count();
    opens > closes
}

fn join_docs(docs: &[String]) -> String {
    docs.join("\n").trim().to_owned()
}

/// Decide which control a setting needs.
fn infer_kind(key: &str, example: &str, doc: &str) -> ValueKind {
    let value = example.trim();

    if value.starts_with('[') {
        // The step-taking options — `disable`, `only`, `first`, `last`,
        // `ignore_failures` — are recognised by what their documentation says
        // rather than by name, so a release that adds another gets the step
        // picker too. The reference describes each of them as taking steps.
        return if mentions_steps(doc) {
            ValueKind::StepList
        } else {
            ValueKind::StringList
        };
    }

    if value == "true" || value == "false" {
        return ValueKind::Bool;
    }

    if value.parse::<i64>().is_ok() {
        return ValueKind::Integer;
    }

    if let Some(options) = allowed_values(doc) {
        // Only trust a stated set that actually contains the example, so prose
        // that happens to use the phrase for something else does not turn a
        // text field into a dropdown missing its own current value.
        let unquoted = unquote(value);
        if options.iter().any(|option| option == &unquoted) {
            return ValueKind::Enum { options };
        }
        debug_log!(
            DISCOVER,
            "{key}: stated options {options:?} exclude example {unquoted:?}, using text"
        );
    }

    ValueKind::Text
}

/// Whether documentation describes an option as taking steps.
fn mentions_steps(doc: &str) -> bool {
    let doc = doc.to_ascii_lowercase();
    doc.contains("steps") || doc.contains("same options as the command line flag")
}

/// Pull a stated set of permitted values out of the documentation.
///
/// The reference writes these two ways, both of which appear in 17.9.0:
///
/// ```text
/// # (default: "attach_if_not_in_session", allowed values: "attach_if_not_in_session", "attach_always")
/// ```
///
/// ```text
/// # Allowed values:
/// #   autodetect, nh, vanilla
/// ```
fn allowed_values(doc: &str) -> Option<Vec<String>> {
    let lower = doc.to_ascii_lowercase();
    let marker = lower.find("allowed values")?;

    // Work from the original text so the values keep their case.
    let after = &doc[marker + "allowed values".len()..];
    let after = after.trim_start_matches([':', ' ', '\n']);

    // Inline form: the values sit on the same line, usually inside the closing
    // parenthesis of a `(default: …, allowed values: …)` note.
    let segment = after.lines().next().unwrap_or_default().trim();
    let segment = segment.trim_end_matches(')');

    let mut values = split_values(segment);

    // Continuation form: the label is alone on its line and the values are on
    // the next one.
    if values.is_empty() {
        let next = after.lines().nth(1).unwrap_or_default().trim();
        values = split_values(next.trim_end_matches(')'));
    }

    (!values.is_empty()).then_some(values)
}

/// Split a comma-separated list of possibly-quoted values.
fn split_values(segment: &str) -> Vec<String> {
    segment
        .split(',')
        .map(|value| unquote(value.trim()))
        .filter(|value| {
            !value.is_empty()
                && value
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        })
        .collect()
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_owned()
}

/// Read the default out of a `(default: …)` note.
fn stated_default(doc: &str) -> Option<String> {
    let lower = doc.to_ascii_lowercase();
    let start = lower.find("default:")?;
    let after = &doc[start + "default:".len()..];

    // Stop at whatever ends the note: a comma introducing the allowed-values
    // clause, the closing parenthesis, or the end of the line.
    let end = after
        .find([',', ')', '\n'])
        .unwrap_or(after.len());
    let value = unquote(after[..end].trim());
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Excerpts from `topgrade --config-reference` on 17.9.0, kept verbatim.
    const REFERENCE: &str = r#"
# Include any additional configuration file(s)
[include]
# paths = ["/etc/topgrade.toml"]


[misc]
# On Unix systems, Topgrade should not be run as root, it
# will run commands with sudo or equivalent where needed.
# (default: false)
# allow_root = false

# Periodically runs `sudo -n -v` or `please -w` to avoid password re-prompts during updates (default: false)
# WARNING: This is a potential security risk; if you walk away from the computer while topgrade is running,
# another person can come by, CTRL+C, and gain access to a sudo session.
# sudo_loop = true
# sudo_loop_interval = 240 # 4 minutes (240s); default if sudo_loop enabled

# Sudo command to be used
# sudo_command = "sudo"

# Disable specific steps - same options as the command line flag
# disable = ["system", "emacs"]

# Changes the way topgrade interacts with
# the tmux session
# (default: "attach_if_not_in_session", allowed values: "attach_if_not_in_session", "attach_always")
# tmux_session_mode = "attach_if_not_in_session"

# For NixOS/home-manager, there are multiple ways to switch.
# When set to autodetect: use nh when available, fall back to vanilla
# Allowed values:
#   autodetect, nh, vanilla
# nix_handler = "autodetect"


# Commands to run before anything
[pre_commands]
# "Emacs Snapshot" = "rm -rf ~/.emacs.d/elpa.bak"


[conda]
# Additional named conda environments to update
# env_names = [
#     "Toolbox",
#     "PyTorch"
# ]
"#;

    fn schema() -> Schema {
        parse(REFERENCE).expect("the reference should parse")
    }

    fn setting(section: &str, key: &str) -> Setting {
        schema()
            .setting(section, key)
            .unwrap_or_else(|| panic!("{section}.{key} should be present"))
            .clone()
    }

    #[test]
    fn finds_every_section() {
        let schema = schema();
        let names: Vec<_> = schema.sections.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["include", "misc", "pre_commands", "conda"]);
    }

    #[test]
    fn a_boolean_example_becomes_a_toggle() {
        assert_eq!(setting("misc", "allow_root").kind, ValueKind::Bool);
    }

    #[test]
    fn an_integer_example_becomes_a_number() {
        assert_eq!(
            setting("misc", "sudo_loop_interval").kind,
            ValueKind::Integer
        );
    }

    #[test]
    fn a_trailing_comment_is_not_part_of_the_value() {
        // `sudo_loop_interval = 240 # 4 minutes (240s)` — reading the comment as
        // part of the value would make this text rather than a number.
        assert_eq!(setting("misc", "sudo_loop_interval").example, "240");
    }

    #[test]
    fn a_quoted_example_becomes_text() {
        assert_eq!(setting("misc", "sudo_command").kind, ValueKind::Text);
    }

    #[test]
    fn a_step_taking_list_is_recognised_from_its_documentation() {
        assert_eq!(setting("misc", "disable").kind, ValueKind::StepList);
    }

    #[test]
    fn an_ordinary_list_stays_a_string_list() {
        assert_eq!(setting("conda", "env_names").kind, ValueKind::StringList);
    }

    #[test]
    fn a_list_split_over_several_comment_lines_is_read_whole() {
        let example = setting("conda", "env_names").example;
        assert!(example.contains("Toolbox"), "{example:?}");
        assert!(example.contains("PyTorch"), "{example:?}");
        assert!(example.ends_with(']'), "list was left unclosed: {example:?}");
    }

    #[test]
    fn inline_allowed_values_become_a_dropdown() {
        match setting("misc", "tmux_session_mode").kind {
            ValueKind::Enum { options } => {
                assert_eq!(options, ["attach_if_not_in_session", "attach_always"]);
            }
            other => panic!("expected Enum, got {other:?}"),
        }
    }

    #[test]
    fn allowed_values_on_a_following_line_become_a_dropdown() {
        match setting("misc", "nix_handler").kind {
            ValueKind::Enum { options } => assert_eq!(options, ["autodetect", "nh", "vanilla"]),
            other => panic!("expected Enum, got {other:?}"),
        }
    }

    #[test]
    fn a_stated_default_is_captured() {
        assert_eq!(setting("misc", "allow_root").default.as_deref(), Some("false"));
    }

    #[test]
    fn a_default_stated_alongside_allowed_values_stops_at_the_comma() {
        assert_eq!(
            setting("misc", "tmux_session_mode").default.as_deref(),
            Some("attach_if_not_in_session")
        );
    }

    #[test]
    fn documentation_is_attached_to_the_option_it_precedes() {
        let doc = setting("misc", "sudo_loop").doc;
        assert!(doc.contains("WARNING"), "security warning was lost: {doc:?}");
    }

    #[test]
    fn documentation_does_not_drift_onto_the_next_option() {
        // `sudo_command` is separated from sudo_loop's warning by a blank line.
        let doc = setting("misc", "sudo_command").doc;
        assert!(!doc.contains("WARNING"), "prose leaked downwards: {doc:?}");
    }

    #[test]
    fn a_user_named_section_is_marked_free_form() {
        let section = schema()
            .section("pre_commands")
            .expect("pre_commands should be present")
            .clone();
        assert!(section.free_form);
        assert!(
            section.settings.is_empty(),
            "a user-chosen name is not an option: {:?}",
            section.settings
        );
    }

    #[test]
    fn keys_get_readable_labels() {
        assert_eq!(setting("misc", "sudo_loop_interval").label(), "Sudo Loop Interval");
    }

    #[test]
    fn errors_when_there_is_nothing_to_read() {
        assert!(parse("").is_err());
    }
}
