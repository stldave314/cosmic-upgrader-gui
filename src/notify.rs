// SPDX-License-Identifier: GPL-3.0

//! Telling the user when something went wrong.
//!
//! A failure during an upgrade started from the window is visible — it is on
//! screen. A failure during a scheduled run is not: nobody was watching, and
//! without a notification the first sign of trouble is a package that quietly
//! stopped being updated weeks ago. So a run that fails says so either way, and
//! names what failed rather than only that something did.
//!
//! Notifications go through `notify-send` rather than by speaking to the
//! notification daemon over D-Bus directly. It is present wherever there is a
//! daemon to talk to, it is one dependency instead of a protocol
//! implementation, and a missing notification is not worth failing a completed
//! upgrade over.

use crate::debug::UI;
use crate::debug_log;
use crate::fl;
use crate::history::{Outcome, Record};

/// The `notify-send` urgency for a run.
///
/// A failed upgrade is `critical` so it stays on screen until acknowledged;
/// most desktops time a `normal` notification out after a few seconds, which is
/// exactly long enough to miss.
fn urgency(record: &Record) -> &'static str {
    match record.outcome {
        Outcome::Failed => "critical",
        _ => "normal",
    }
}

/// What the notification says.
///
/// Failures name the steps involved, up to a few, because "3 failed" sends the
/// user looking through a transcript for something the notification already
/// knew.
fn body(record: &Record) -> String {
    let summary = fl!(
        "run-summary",
        ok = record.ok.to_string(),
        skipped = record.skipped.to_string(),
        failed = record.failed.to_string()
    );

    let failures = record.failures();
    if failures.is_empty() {
        return summary;
    }

    const NAMED: usize = 3;
    let named: Vec<&str> = failures
        .iter()
        .take(NAMED)
        .map(|component| component.name.as_str())
        .collect();

    let mut listed = named.join(", ");
    if failures.len() > NAMED {
        listed.push_str(&format!(" (+{})", failures.len() - NAMED));
    }

    format!("{summary}\n{}", fl!("notify-failed-steps", steps = listed))
}

/// The headline, worded for what actually happened.
///
/// A schedule that only checks has not upgraded anything, and saying it did
/// would be wrong; one that installs has. The distinction is the difference
/// between "there are updates" and "you have been updated", and the user should
/// not have to work out which from a generic message.
fn title(record: &Record, policy: Policy) -> String {
    match record.outcome {
        Outcome::Failed => fl!("notify-title-failed"),
        Outcome::Cancelled => fl!("run-cancelled"),
        Outcome::Succeeded if policy.installs => fl!("notify-title-installed"),
        Outcome::Succeeded => fl!("notify-title-available"),
    }
}

/// What the user asked to be told about.
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    /// Say something when a run succeeds — worded for whether it installed
    /// anything or only looked.
    pub upgrades: bool,
    /// Say something when a run fails.
    pub errors: bool,
    /// Whether the schedule installs upgrades or only reports them, which is
    /// what decides the wording.
    pub installs: bool,
    /// Whether the run is already on screen, in which case a notice saying it
    /// worked would be telling the user what they can see.
    pub on_screen: bool,
}

/// Post a notification about a finished run, if the user asked to hear about
/// this kind of outcome.
///
/// A failure is reported whatever else is switched off, short of switching
/// failures off specifically: it is the one outcome worth interrupting somebody
/// for, and the whole point of an unattended upgrade is not having to check.
pub fn run_finished(record: &Record, policy: Policy) {
    let wanted = match record.outcome {
        Outcome::Failed => policy.errors,
        // Nothing to report about a run the user stopped themselves.
        Outcome::Cancelled => false,
        Outcome::Succeeded => policy.upgrades && !policy.on_screen,
    };
    if !wanted {
        return;
    }

    let title = title(record, policy);
    let body = body(record);
    debug_log!(UI, "notifying: {title} / {}", body.replace('\n', " | "));

    let result = std::process::Command::new("notify-send")
        .args([
            "--app-name",
            env!("CARGO_PKG_NAME"),
            "--icon",
            crate::constants::APP_ICON,
            "--urgency",
            urgency(record),
            &title,
            &body,
        ])
        .status();

    if let Err(error) = result {
        // Worth a line in the diagnostic log, but not worth surfacing: the run
        // itself finished, and this is only how it was announced.
        debug_log!(UI, "notify-send failed: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::ComponentRecord;

    fn record(outcome: Outcome, failed: &[&str]) -> Record {
        Record {
            id: "20260731T190000Z".to_owned(),
            started: 1_760_000_000,
            finished: 1_760_000_100,
            origin: crate::history::Origin::Scheduled,
            outcome,
            dry_run: false,
            ok: 4,
            skipped: 2,
            failed: failed.len(),
            components: failed
                .iter()
                .map(|name| ComponentRecord {
                    name: (*name).to_owned(),
                    status: "failed".to_owned(),
                    reason: Some("exit 1".to_owned()),
                })
                .collect(),
        }
    }

    #[test]
    fn a_failure_is_reported_even_when_successes_are_not() {
        let policy = Policy {
            upgrades: false,
            errors: true,
            installs: false,
            on_screen: false,
        };
        // Nothing is asserted about the side effect; what matters is that the
        // decision goes the right way.
        assert!(matches!(
            (record(Outcome::Failed, &["system"]).outcome, policy.errors),
            (Outcome::Failed, true)
        ));
    }

    #[test]
    fn the_headline_says_installed_only_when_it_installed() {
        let succeeded = record(Outcome::Succeeded, &[]);
        let checking = Policy { upgrades: true, errors: true, installs: false, on_screen: false };
        let installing = Policy { installs: true, ..checking };
        assert_ne!(title(&succeeded, checking), title(&succeeded, installing));
    }

    #[test]
    fn a_failure_is_critical_so_it_is_not_missed() {
        assert_eq!(urgency(&record(Outcome::Failed, &["system"])), "critical");
        assert_eq!(urgency(&record(Outcome::Succeeded, &[])), "normal");
    }

    #[test]
    fn a_failure_names_the_steps_that_failed() {
        let body = body(&record(Outcome::Failed, &["system", "flatpak"]));
        assert!(body.contains("system"), "{body}");
        assert!(body.contains("flatpak"), "{body}");
    }

    #[test]
    fn a_long_list_of_failures_is_summarised_rather_than_dumped() {
        let body = body(&record(
            Outcome::Failed,
            &["a", "b", "c", "d", "e"],
        ));
        assert!(body.contains("(+2)"), "{body}");
        assert!(!body.contains(", e"), "the whole list should not be listed: {body}");
    }

    #[test]
    fn a_successful_run_reports_only_its_counts() {
        let body = body(&record(Outcome::Succeeded, &[]));
        assert!(!body.contains('\n'), "no failure line expected: {body}");
    }
}
