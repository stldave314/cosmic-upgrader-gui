// SPDX-License-Identifier: GPL-3.0

//! The list of steps this topgrade knows about.
//!
//! topgrade declares its steps to clap as an enumeration, and clap prints them
//! in the help text for every option that takes one — `--disable`, `--only` and
//! `--yes` all carry the same `[possible values: …]` list. Reading it back is
//! how this application learns the step list of the binary it is actually
//! driving, rather than of whatever release happened to be current when this
//! was written.

use std::collections::BTreeSet;
use std::fmt;

use super::{Error, Result, Topgrade};
use crate::debug::DISCOVER;
use crate::debug_log;

/// Introduces the enumerated values in clap's help output.
///
/// Note the leading bracket: the text reads `[possible values: am, …]`, so the
/// bracket comes *before* the label rather than after it.
const POSSIBLE_VALUES: &str = "[possible values:";

/// A topgrade step identifier, exactly as the command line accepts it.
///
/// Kept as an owned string rather than an enumeration for the reason the module
/// documentation gives: the set is whatever the binary says it is, and a Rust
/// enum would have to be edited every time topgrade gained a step.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StepId(String);

impl StepId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// A readable name for a step, derived from its identifier.
    ///
    /// Only used for steps [`categories`](super::categories) has no entry for —
    /// which is to say, steps added to topgrade after this application was last
    /// updated. `pip_review_local` becomes "Pip Review Local": not as good as a
    /// hand-written name, but far better than showing the raw identifier, and it
    /// costs nothing to be reasonable about the unknown.
    pub fn humanized(&self) -> String {
        let mut out = String::with_capacity(self.0.len());
        for (index, word) in self.0.split('_').enumerate() {
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
}

impl fmt::Display for StepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Ask topgrade for every step it supports.
pub async fn steps(topgrade: &Topgrade) -> Result<Vec<StepId>> {
    let help = topgrade.output(&["--help"]).await?;
    let steps = parse_steps(&help)?;
    debug_log!(DISCOVER, "{} steps discovered", steps.len());
    Ok(steps)
}

/// Pull the step list out of clap's help text.
///
/// Every `[possible values: …]` block in the help is collected and merged
/// rather than just the first one taken. They are expected to be identical —
/// they come from one enumeration — but merging costs nothing and means a
/// release where one option accepts a step another does not still yields the
/// full set.
fn parse_steps(help: &str) -> Result<Vec<StepId>> {
    let mut found = BTreeSet::new();

    let mut rest = help;
    while let Some(start) = rest.find(POSSIBLE_VALUES) {
        let after = &rest[start + POSSIBLE_VALUES.len()..];
        let Some(end) = after.find(']') else {
            // An unterminated block means the help was truncated. Stop here and
            // use whatever earlier blocks yielded rather than discarding them.
            debug_log!(DISCOVER, "unterminated possible-values block, stopping");
            break;
        };

        for value in after[..end].split(',') {
            // clap wraps long lists, so a value can carry a newline and the
            // indentation of the continuation line.
            let value = value.trim();
            if !value.is_empty() && is_step_identifier(value) {
                found.insert(StepId::new(value));
            }
        }

        rest = &after[end..];
    }

    if found.is_empty() {
        return Err(Error::Parse {
            what: "the step list in --help",
            detail: format!("no {POSSIBLE_VALUES} block found"),
        });
    }

    Ok(found.into_iter().collect())
}

/// Whether a value looks like a step identifier rather than some other
/// enumeration's member.
///
/// The help text also enumerates values for `--run-type` and `--notify-end`,
/// and those blocks are picked up by the same scan. Step identifiers are
/// lowercase ASCII words joined by underscores; so are `dry`, `wet` and
/// `always`, so this cannot tell them apart by shape alone — but those three
/// blocks list a handful of values each and are filtered out by
/// [`parse_steps`]'s caller having no use for them. What this rejects is
/// anything containing whitespace or punctuation, which is what a wrapped or
/// malformed block produces.
fn is_step_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_single_line_block() {
        let help = "  --only <STEP>...\n      [possible values: am, cargo, system]\n";
        let steps = parse_steps(help).expect("should parse");
        let names: Vec<_> = steps.iter().map(StepId::as_str).collect();
        assert_eq!(names, ["am", "cargo", "system"]);
    }

    #[test]
    fn merges_repeated_blocks_without_duplicating() {
        // --disable, --only and --yes each print the same list.
        let help = "[possible values: am, cargo]\n[possible values: cargo, system]\n";
        let steps = parse_steps(help).expect("should parse");
        assert_eq!(steps.len(), 3, "expected the union, got {steps:?}");
    }

    #[test]
    fn reads_a_block_clap_wrapped_across_lines() {
        let help = "[possible values: am, android_studio,\n        cargo, system]";
        let steps = parse_steps(help).expect("should parse");
        let names: Vec<_> = steps.iter().map(StepId::as_str).collect();
        assert_eq!(names, ["am", "android_studio", "cargo", "system"]);
    }

    #[test]
    fn errors_when_help_carries_no_block() {
        assert!(parse_steps("Usage: topgrade [OPTIONS]\n").is_err());
    }

    #[test]
    fn keeps_earlier_blocks_when_a_later_one_is_truncated() {
        let help = "[possible values: am, cargo]\n[possible values: system, flat";
        let steps = parse_steps(help).expect("should parse");
        assert_eq!(steps.len(), 2);
    }

    #[test]
    fn humanizes_an_unknown_identifier() {
        assert_eq!(StepId::new("pip_review_local").humanized(), "Pip Review Local");
        assert_eq!(StepId::new("cargo").humanized(), "Cargo");
    }
}
