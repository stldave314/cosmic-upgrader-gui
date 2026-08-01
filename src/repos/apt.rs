// SPDX-License-Identifier: GPL-3.0

//! Reading and changing apt's repository lists.
//!
//! Two formats are in use at once on a modern Debian or Ubuntu system, and this
//! machine has both: the classic one-line `.list` entries, and the deb822
//! `.sources` stanzas that are replacing them. Neither is going away soon, so
//! both are read, and each is changed in its own idiom rather than converted.
//!
//! ## Why this is careful
//!
//! A malformed sources file does not degrade gracefully — `apt update` fails
//! outright and every upgrade path on the system stops working, which for an
//! application whose whole job is upgrades would be a spectacular own goal. So:
//!
//! * Entries are **disabled rather than deleted** by default. Commenting a line
//!   out is reversible by hand with a text editor; deleting a file is not.
//! * A deb822 stanza is changed by setting one field. The rest of the stanza —
//!   including an inline PGP key running to dozens of lines — is copied through
//!   untouched, because rewriting a key is a good way to lose one.
//! * Files apt itself leaves behind (`.save`, `.distUpgrade`, `.bak`) are not
//!   repositories and are not shown. They exist precisely because something
//!   else already edited these files, and offering them for editing again would
//!   be offering to edit a backup.

use std::path::{Path, PathBuf};

/// Where apt keeps additional repository files.
pub const SOURCES_DIR: &str = "/etc/apt/sources.list.d";

/// The single legacy file, still used by some systems.
pub const SOURCES_LIST: &str = "/etc/apt/sources.list";

/// Suffixes on files that are not live repository lists.
///
/// apt and `software-properties` write these when they rewrite a file; showing
/// them would mean offering to edit a backup of something already edited.
const NOT_A_SOURCE: [&str; 5] = [".save", ".distUpgrade", ".bak", ".orig", ".dpkg-dist"];

/// Which of the two formats an entry is written in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    /// `deb [options] URI suite components`
    OneLine,
    /// A deb822 stanza of `Field: value` lines.
    Deb822,
}

/// One repository apt knows about.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    /// The file it lives in.
    pub file: PathBuf,
    /// Which entry within that file, counted from zero.
    ///
    /// For a one-line file this is the line number; for deb822 it is the stanza
    /// index. Either way it is what identifies the entry when writing back.
    pub index: usize,
    pub format: Format,
    pub enabled: bool,
    /// Where the packages come from.
    pub uri: String,
    /// The release, and the parts of it wanted.
    pub suites: String,
    pub components: String,
    /// `deb` or `deb-src`.
    pub types: String,
}

impl Entry {
    /// A one-line summary for the interface.
    pub fn describe(&self) -> String {
        let mut text = self.uri.clone();
        if !self.suites.is_empty() {
            text.push(' ');
            text.push_str(&self.suites);
        }
        if !self.components.is_empty() {
            text.push(' ');
            text.push_str(&self.components);
        }
        text
    }

    /// The file's name without its directory, which is what identifies it to a
    /// person.
    pub fn file_name(&self) -> String {
        self.file
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

/// Whether a path is a live repository list rather than something left beside
/// one.
pub fn is_source_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if NOT_A_SOURCE.iter().any(|suffix| name.ends_with(suffix)) {
        return false;
    }
    name.ends_with(".list") || name.ends_with(".sources")
}

/// Read the classic one-line format.
///
/// A commented-out `deb` line is a disabled repository, which is how everything
/// from `add-apt-repository` to hand editing turns one off — so it is read back
/// as one rather than ignored. A comment that is not a `deb` line is just a
/// comment.
pub fn parse_list(file: &Path, text: &str) -> Vec<Entry> {
    let mut entries = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        let (enabled, body) = match trimmed.strip_prefix('#') {
            // Any number of leading hashes and spaces: `### deb ...` is still a
            // commented-out entry.
            Some(rest) => (false, rest.trim_start_matches(['#', ' ']).trim()),
            None => (true, trimmed),
        };

        let mut fields = body.split_whitespace();
        let Some(types) = fields.next() else {
            continue;
        };
        if types != "deb" && types != "deb-src" {
            continue;
        }

        // Options come in brackets between the type and the URI, and may
        // contain spaces: `[arch=amd64 signed-by=/path]`.
        let mut rest: Vec<&str> = fields.collect();
        if rest.first().is_some_and(|field| field.starts_with('[')) {
            let end = rest
                .iter()
                .position(|field| field.ends_with(']'))
                .unwrap_or(0);
            rest.drain(..=end);
        }

        let Some(uri) = rest.first() else {
            continue;
        };

        entries.push(Entry {
            file: file.to_path_buf(),
            index,
            format: Format::OneLine,
            enabled,
            types: types.to_owned(),
            uri: (*uri).to_owned(),
            suites: rest.get(1).copied().unwrap_or_default().to_owned(),
            components: rest.get(2..).map(|rest| rest.join(" ")).unwrap_or_default(),
        });
    }

    entries
}

/// Read the deb822 format.
///
/// Stanzas are separated by blank lines and fields may continue onto following
/// lines that start with whitespace — which is how an inline PGP key is carried,
/// and why a continuation is not mistaken for a new field.
pub fn parse_sources(file: &Path, text: &str) -> Vec<Entry> {
    let mut entries = Vec::new();

    for (index, stanza) in text.split("\n\n").enumerate() {
        if stanza.trim().is_empty() {
            continue;
        }

        let field = |name: &str| -> String {
            stanza
                .lines()
                .find(|line| {
                    line.to_ascii_lowercase()
                        .starts_with(&format!("{}:", name.to_ascii_lowercase()))
                })
                .and_then(|line| line.split_once(':'))
                .map(|(_, value)| value.trim().to_owned())
                .unwrap_or_default()
        };

        let types = field("Types");
        let uri = field("URIs");
        if types.is_empty() || uri.is_empty() {
            continue;
        }

        // Absent means enabled; apt only treats an explicit no as off.
        let enabled = !matches!(
            field("Enabled").to_ascii_lowercase().as_str(),
            "no" | "false"
        );

        entries.push(Entry {
            file: file.to_path_buf(),
            index,
            format: Format::Deb822,
            enabled,
            types,
            uri,
            suites: field("Suites"),
            components: field("Components"),
        });
    }

    entries
}

/// Read whichever format a file is in.
pub fn parse_file(file: &Path, text: &str) -> Vec<Entry> {
    let is_deb822 = file
        .extension()
        .is_some_and(|extension| extension == "sources");
    if is_deb822 {
        parse_sources(file, text)
    } else {
        parse_list(file, text)
    }
}

/// Turn one entry in a one-line file on or off.
///
/// Only the line in question is touched; everything else in the file is
/// returned byte for byte, including comments explaining why somebody disabled
/// something.
pub fn set_list_enabled(text: &str, index: usize, enabled: bool) -> String {
    let mut out: Vec<String> = Vec::new();

    for (number, line) in text.lines().enumerate() {
        if number != index {
            out.push(line.to_owned());
            continue;
        }

        let trimmed = line.trim_start();
        if enabled {
            // Strip the comment markers, keeping the entry itself as written.
            out.push(
                trimmed
                    .trim_start_matches(['#', ' '])
                    .to_owned(),
            );
        } else if trimmed.starts_with('#') {
            out.push(line.to_owned());
        } else {
            out.push(format!("# {line}"));
        }
    }

    let mut joined = out.join("\n");
    // A sources file that does not end in a newline is a common way to make apt
    // ignore its last entry.
    if text.ends_with('\n') || !joined.ends_with('\n') {
        joined.push('\n');
    }
    joined
}

/// Turn one stanza in a deb822 file on or off.
///
/// Done by setting the `Enabled` field and copying everything else through, so
/// an inline signing key is never rewritten — losing one would leave a
/// repository that cannot be verified and an `apt update` that fails.
pub fn set_sources_enabled(text: &str, index: usize, enabled: bool) -> String {
    let value = if enabled { "yes" } else { "no" };

    let stanzas: Vec<String> = text
        .split("\n\n")
        .enumerate()
        .map(|(number, stanza)| {
            if number != index || stanza.trim().is_empty() {
                return stanza.to_owned();
            }

            let has_field = stanza
                .lines()
                .any(|line| line.to_ascii_lowercase().starts_with("enabled:"));

            if has_field {
                stanza
                    .lines()
                    .map(|line| {
                        if line.to_ascii_lowercase().starts_with("enabled:") {
                            format!("Enabled: {value}")
                        } else {
                            line.to_owned()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                // Added at the top rather than the bottom: a stanza ending in a
                // multi-line key would otherwise get the field folded into the
                // key as a continuation line.
                format!("Enabled: {value}\n{stanza}")
            }
        })
        .collect();

    stanzas.join("\n\n")
}

/// A new one-line entry, for adding a repository.
pub fn new_list_entry(uri: &str, suite: &str, components: &str) -> String {
    format!(
        "deb {} {} {}\n",
        uri.trim(),
        suite.trim(),
        components.trim()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(name: &str) -> PathBuf {
        PathBuf::from(SOURCES_DIR).join(name)
    }

    const DOCKER: &str = "deb [arch=amd64 signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian trixie stable\n";

    /// Taken from this machine: a package that ships its entry commented out.
    const CLAUDE: &str = "\
### Managed by the claude-desktop package.
### Set CLAUDE_DESKTOP_ADD_REPO=\"true\"|\"false\" to force this entry on or off.
# deb [signed-by=/usr/share/keyrings/claude.gpg] https://example.com/apt stable main
";

    /// deb822 with an inline key, as on this machine.
    const KOBUK: &str = "\
Types: deb
URIs: https://ppa.launchpadcontent.net/kobuk-team/intel-graphics/ubuntu/
Suites: noble
Components: main
Signed-By: -----BEGIN PGP PUBLIC KEY BLOCK-----
 .
 mQINBGTSU84BEADVSs1CRYCbfu4xppSmtCntU5KeefhklqbmGRBLJzqHGLlX7snZ
 -----END PGP PUBLIC KEY BLOCK-----
";

    #[test]
    fn reads_a_one_line_entry_past_its_options() {
        // The bracketed options contain a space, so a naive split puts the URI
        // in the wrong field.
        let entries = parse_list(&path("docker.list"), DOCKER);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].uri, "https://download.docker.com/linux/debian");
        assert_eq!(entries[0].suites, "trixie");
        assert_eq!(entries[0].components, "stable");
        assert!(entries[0].enabled);
    }

    #[test]
    fn a_commented_entry_is_a_disabled_repository() {
        let entries = parse_list(&path("claude-desktop.list"), CLAUDE);
        assert_eq!(entries.len(), 1, "{entries:?}");
        assert!(!entries[0].enabled);
        assert_eq!(entries[0].uri, "https://example.com/apt");
    }

    #[test]
    fn prose_comments_are_not_repositories() {
        // The first two lines of that file are explanation, not entries.
        let entries = parse_list(&path("claude-desktop.list"), CLAUDE);
        assert!(entries.iter().all(|entry| entry.uri.starts_with("https")));
    }

    #[test]
    fn reads_a_deb822_stanza() {
        let entries = parse_sources(&path("kobuk.sources"), KOBUK);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].suites, "noble");
        assert_eq!(entries[0].components, "main");
        assert!(entries[0].enabled, "absent Enabled means enabled");
        assert_eq!(entries[0].format, Format::Deb822);
    }

    #[test]
    fn an_explicit_no_disables_a_stanza() {
        let text = format!("Enabled: no\n{KOBUK}");
        let entries = parse_sources(&path("x.sources"), &text);
        assert!(!entries[0].enabled);
    }

    #[test]
    fn backups_are_not_repositories() {
        // This machine has a .save beside almost every list.
        assert!(is_source_file(&path("docker.list")));
        assert!(is_source_file(&path("pop-os-apps.sources")));
        assert!(!is_source_file(&path("docker.list.save")));
        assert!(!is_source_file(&path("system.sources.save")));
        assert!(!is_source_file(&path("something.txt")));
    }

    #[test]
    fn disabling_a_line_comments_only_that_line() {
        let text = "deb https://a b c\ndeb https://d e f\n";
        let out = set_list_enabled(text, 1, false);
        assert_eq!(out, "deb https://a b c\n# deb https://d e f\n");
    }

    #[test]
    fn enabling_a_line_removes_its_comment_markers() {
        let out = set_list_enabled(CLAUDE, 2, true);
        assert!(
            out.contains("deb [signed-by=/usr/share/keyrings/claude.gpg] https://example.com/apt stable main"),
            "{out}"
        );
        // The prose above it is untouched.
        assert!(out.contains("### Managed by the claude-desktop package."), "{out}");
    }

    #[test]
    fn toggling_is_reversible() {
        let disabled = set_list_enabled(DOCKER, 0, false);
        let enabled = set_list_enabled(&disabled, 0, true);
        assert_eq!(enabled, DOCKER);
    }

    #[test]
    fn a_deb822_stanza_keeps_its_inline_key_when_disabled() {
        // Rewriting a key would leave a repository that cannot be verified.
        let out = set_sources_enabled(KOBUK, 0, false);
        assert!(out.contains("Enabled: no"), "{out}");
        assert!(out.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----"), "{out}");
        assert!(out.contains("mQINBGTSU84BEADVSs1CRYCbfu4xppSmtCntU5KeefhklqbmGRBLJzqHGLlX7snZ"), "{out}");
        assert!(out.contains("-----END PGP PUBLIC KEY BLOCK-----"), "{out}");
    }

    #[test]
    fn the_enabled_field_goes_above_a_multi_line_key() {
        // Appended at the end it would be read as a continuation of the key.
        let out = set_sources_enabled(KOBUK, 0, false);
        assert!(out.starts_with("Enabled: no\n"), "{out}");
    }

    #[test]
    fn an_existing_enabled_field_is_replaced_rather_than_duplicated() {
        let text = format!("Enabled: no\n{KOBUK}");
        let out = set_sources_enabled(&text, 0, true);
        assert_eq!(out.matches("Enabled:").count(), 1, "{out}");
        assert!(out.contains("Enabled: yes"), "{out}");
    }

    #[test]
    fn only_the_named_stanza_changes() {
        let text = format!("{KOBUK}\nTypes: deb\nURIs: https://other\nSuites: x\n");
        let out = set_sources_enabled(&text, 1, false);
        let entries = parse_sources(&path("x.sources"), &out);
        assert!(entries[0].enabled, "the first stanza should be untouched");
        assert!(!entries[1].enabled);
    }

    #[test]
    fn a_new_entry_is_well_formed() {
        let line = new_list_entry(" https://example.com/apt ", " stable ", " main ");
        assert_eq!(line, "deb https://example.com/apt stable main\n");
        let entries = parse_list(&path("new.list"), &line);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].enabled);
    }

    #[test]
    fn a_file_ends_with_a_newline_after_editing() {
        // apt ignores a final entry with no newline after it.
        let out = set_list_enabled("deb https://a b c", 0, false);
        assert!(out.ends_with('\n'), "{out:?}");
    }
}
