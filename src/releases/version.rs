// SPDX-License-Identifier: GPL-3.0

//! Deciding whether one version is newer than another.
//!
//! This has to cope with what forges and packagers actually produce rather than
//! with a specification. Release tags in the wild are `v1.2.3`, `1.2.3`,
//! `release-1.2.3`, `qFlipper-1.3.3`, `2024-01-15` and `1.2.3-rc1`; the
//! installed side adds Debian and RPM decorations like `1.2.3-2ubuntu0.1`,
//! `2:1.2.3+really1.2.2-1` and `1.2.3~beta1`.
//!
//! A strict semver parse rejects most of that, so versions are reduced to the
//! numbers in them and a remainder. Two consequences are worth stating:
//!
//! * Comparison is *advisory*. Getting it wrong shows a release as new when it
//!   is not, or fails to show one that is — neither of which changes anything on
//!   the system by itself, because an update is something the user then chooses.
//! * When the comparison cannot be trusted at all, that is reported as
//!   [`Ordering::Unknown`] rather than guessed, so the interface can say "a
//!   release exists" instead of claiming it is newer.

use std::cmp;

/// How two versions relate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ordering {
    Older,
    Same,
    Newer,
    /// Neither could be reduced to something comparable.
    Unknown,
}

/// A version reduced to the parts that can be compared.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Parsed {
    /// The dotted numbers, most significant first.
    numbers: Vec<u64>,
    /// What followed them — `rc1`, `beta`, `alpha2`. Empty for a plain release.
    ///
    /// Its presence lowers the version: `1.2.3-rc1` precedes `1.2.3`, which is
    /// what every convention in use agrees on even when they agree on nothing
    /// else.
    pre_release: String,
}

/// Strip the decoration around a version and reduce it to numbers.
///
/// Returns `None` when there is no number in the string at all, which is the
/// only case that cannot be worked with.
fn parse(raw: &str) -> Option<Parsed> {
    let text = raw.trim();

    // A Debian epoch (`2:1.2.3`) orders above anything without one, but it is a
    // packaging artefact rather than part of the upstream version, and the two
    // sides of this comparison come from different places. Dropping it compares
    // like with like.
    let text = match text.split_once(':') {
        Some((epoch, rest)) if epoch.chars().all(|c| c.is_ascii_digit()) => rest,
        _ => text,
    };

    // A leading `v` is so nearly universal that it is stripped before looking
    // for the version, rather than being left for the scan below to trip over.
    let text = match text.strip_prefix(['v', 'V']) {
        Some(rest) if rest.starts_with(|c: char| c.is_ascii_digit()) => rest,
        _ => text,
    };

    // Find where the version proper starts: the first digit that is not part of
    // a word. That is what removes `release-` and a project name used as a tag
    // prefix, while leaving `x264-1.2` to start at the `1` rather than the `2`.
    let chars: Vec<char> = text.chars().collect();
    let start = (0..chars.len()).find(|&index| {
        chars[index].is_ascii_digit()
            && (index == 0 || !chars[index - 1].is_ascii_alphanumeric())
    })?;

    let mut numbers: Vec<u64> = Vec::new();
    let mut current = String::new();
    let mut remainder = String::new();
    let mut seen_dot = false;
    let mut index = start;

    while index < chars.len() {
        let c = chars[index];

        if c.is_ascii_digit() {
            current.push(c);
            index += 1;
            continue;
        }

        if c == '.' && !current.is_empty() {
            numbers.push(current.parse().unwrap_or(0));
            current.clear();
            seen_dot = true;
            index += 1;
            continue;
        }

        // A dash separates components in a date-style version (`2024-01-15`)
        // but ends the version in a packaged one (`1.2.3-2ubuntu0.1`), where
        // what follows is the packager's revision rather than upstream's. A
        // dotted version has already said which convention it is using, so the
        // dash only separates when no dot has been seen.
        let next_is_digit = chars.get(index + 1).is_some_and(char::is_ascii_digit);
        if (c == '-' || c == '_') && !seen_dot && !current.is_empty() && next_is_digit {
            numbers.push(current.parse().unwrap_or(0));
            current.clear();
            index += 1;
            continue;
        }

        break;
    }

    if !current.is_empty() {
        numbers.push(current.parse().unwrap_or(0));
    }

    // Whatever follows the numbers, minus the separator that introduced it.
    if index < chars.len() {
        let tail: String = chars[index..].iter().collect();
        remainder = tail
            .trim_start_matches(['-', '_', '+', '~', '.'])
            .to_owned();
    }

    if numbers.is_empty() {
        return None;
    }

    Some(Parsed {
        numbers,
        pre_release: normalize_pre_release(&remainder),
    })
}

/// Reduce a trailing fragment to a pre-release marker, or nothing.
///
/// Packagers append revisions that are not pre-releases — `1.2.3-2ubuntu0.1` is
/// the *same* upstream version as `1.2.3`, not an earlier one — so only the
/// recognised pre-release words count. Anything else is discarded rather than
/// compared, which is what stops a Debian revision from making an installed
/// package look older than itself.
fn normalize_pre_release(remainder: &str) -> String {
    const MARKERS: [&str; 7] = ["rc", "alpha", "beta", "pre", "dev", "snapshot", "nightly"];

    let lower = remainder.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return String::new();
    }

    for marker in MARKERS {
        if let Some(position) = lower.find(marker) {
            // Kept whole from the marker on, so `rc1` and `rc2` still order.
            return lower[position..].to_owned();
        }
    }

    String::new()
}

/// Compare an installed version with a release tag.
pub fn compare(installed: &str, candidate: &str) -> Ordering {
    let (Some(installed), Some(candidate)) = (parse(installed), parse(candidate)) else {
        return Ordering::Unknown;
    };

    let width = installed.numbers.len().max(candidate.numbers.len());
    for index in 0..width {
        // A missing component is zero, so `1.2` and `1.2.0` are the same
        // version written two ways.
        let left = installed.numbers.get(index).copied().unwrap_or(0);
        let right = candidate.numbers.get(index).copied().unwrap_or(0);
        match right.cmp(&left) {
            cmp::Ordering::Greater => return Ordering::Newer,
            cmp::Ordering::Less => return Ordering::Older,
            cmp::Ordering::Equal => {}
        }
    }

    match (installed.pre_release.is_empty(), candidate.pre_release.is_empty()) {
        (true, true) => Ordering::Same,
        // A release beats the pre-release of the same version.
        (false, true) => Ordering::Newer,
        (true, false) => Ordering::Older,
        (false, false) => match candidate.pre_release.cmp(&installed.pre_release) {
            cmp::Ordering::Greater => Ordering::Newer,
            cmp::Ordering::Less => Ordering::Older,
            cmp::Ordering::Equal => Ordering::Same,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_higher_number_is_newer() {
        assert_eq!(compare("1.2.3", "1.2.4"), Ordering::Newer);
        assert_eq!(compare("1.2.3", "1.3.0"), Ordering::Newer);
        assert_eq!(compare("1.2.3", "2.0.0"), Ordering::Newer);
    }

    #[test]
    fn a_lower_number_is_older() {
        assert_eq!(compare("2.0.0", "1.9.9"), Ordering::Older);
    }

    #[test]
    fn the_same_version_written_differently_is_the_same() {
        assert_eq!(compare("1.2.3", "v1.2.3"), Ordering::Same);
        assert_eq!(compare("1.2", "1.2.0"), Ordering::Same);
        assert_eq!(compare("1.2.3", "release-1.2.3"), Ordering::Same);
    }

    #[test]
    fn numbers_compare_numerically_not_as_text() {
        // The comparison this replaces would put 9 above 10.
        assert_eq!(compare("1.9.0", "1.10.0"), Ordering::Newer);
        assert_eq!(compare("1.10.0", "1.9.0"), Ordering::Older);
    }

    #[test]
    fn a_project_name_in_the_tag_is_ignored() {
        assert_eq!(compare("1.3.3", "qFlipper-1.3.4"), Ordering::Newer);
    }

    #[test]
    fn a_debian_revision_is_not_a_pre_release() {
        // `1.2.3-2ubuntu0.1` is the same upstream version as `1.2.3`; treating
        // the revision as a pre-release would report a downgrade as an upgrade.
        assert_eq!(compare("1.2.3-2ubuntu0.1", "1.2.3"), Ordering::Same);
        assert_eq!(compare("1.2.3-2ubuntu0.1", "1.2.4"), Ordering::Newer);
    }

    #[test]
    fn a_debian_epoch_is_ignored() {
        assert_eq!(compare("2:1.2.3", "1.2.3"), Ordering::Same);
    }

    #[test]
    fn a_release_beats_its_own_pre_release() {
        assert_eq!(compare("1.2.3-rc1", "1.2.3"), Ordering::Newer);
        assert_eq!(compare("1.2.3", "1.2.3-rc1"), Ordering::Older);
    }

    #[test]
    fn pre_releases_of_the_same_version_order_among_themselves() {
        assert_eq!(compare("1.2.3-rc1", "1.2.3-rc2"), Ordering::Newer);
        assert_eq!(compare("1.2.3-beta", "1.2.3-rc1"), Ordering::Newer);
    }

    #[test]
    fn a_pre_release_of_a_higher_version_still_wins_on_the_numbers() {
        assert_eq!(compare("1.2.3", "1.3.0-rc1"), Ordering::Newer);
    }

    #[test]
    fn date_style_versions_compare_sensibly() {
        assert_eq!(compare("2024-01-15", "2024-02-01"), Ordering::Newer);
        assert_eq!(compare("2024-02-01", "2024-01-15"), Ordering::Older);
    }

    #[test]
    fn something_with_no_number_cannot_be_compared() {
        // Reported rather than guessed: the interface says a release exists
        // instead of claiming it is newer.
        assert_eq!(compare("1.2.3", "latest"), Ordering::Unknown);
        assert_eq!(compare("nightly", "1.2.3"), Ordering::Unknown);
        assert_eq!(compare("", "1.2.3"), Ordering::Unknown);
    }

    #[test]
    fn a_tilde_pre_release_is_read() {
        // Debian writes pre-releases as `1.2.3~beta1`.
        assert_eq!(compare("1.2.3~beta1", "1.2.3"), Ordering::Newer);
    }

    #[test]
    fn a_four_component_version_is_handled() {
        assert_eq!(compare("1.2.3.4", "1.2.3.5"), Ordering::Newer);
        assert_eq!(compare("1.2.3.4", "1.2.3.4"), Ordering::Same);
    }
}
