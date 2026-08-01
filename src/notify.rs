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

fn title(record: &Record) -> String {
    match record.outcome {
        Outcome::Failed => fl!("notify-title-failed"),
        Outcome::Cancelled => fl!("run-cancelled"),
        Outcome::Succeeded => fl!("notify-title-succeeded"),
    }
}

/// Post a notification about a finished run.
///
/// `only_on_failure` is what a run started from the window passes: it is already
/// on screen, so a notification saying it worked would be telling the user
/// something they can see. A scheduled run reports either way.
pub fn run_finished(record: &Record, only_on_failure: bool) {
    if only_on_failure && record.outcome != Outcome::Failed {
        return;
    }

    let title = title(record);
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
