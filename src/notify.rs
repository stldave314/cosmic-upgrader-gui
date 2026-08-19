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

use crate::constants::SCHEDULED_FLAG;
use crate::debug::UI;
use crate::debug_log;
use crate::fl;
use crate::history::{Outcome, Record};

/// Whether a window of this application is already open.
///
/// A scheduled run is a separate process started by systemd, and it has no way
/// to ask the interface whether anybody is looking. What it can do is notice
/// that the interface is there at all — and for this application that is the
/// same question, because it has no windowless mode. `--minimized` is written
/// into the autostart entry but nothing acts on it: Wayland gives a client no
/// way to un-minimize itself, so nothing here minimizes. A copy of this running
/// is a window on screen.
///
/// This matters at login. The timer is `Persistent`, so a run missed while the
/// machine was off starts as the session comes up — exactly when the user is
/// likely to be opening the window — and announcing a successful check to
/// somebody already looking at the application is noise.
///
/// Other windowless copies do not count: a check and an upgrade can both be due
/// at once, and neither of those is anybody watching.
pub fn window_is_open() -> bool {
    // Truncated to fifteen characters by the kernel, but ours is read the same
    // way, so both ends are truncated alike and still compare equal.
    let Ok(ours) = std::fs::read_to_string("/proc/self/comm") else {
        // Without `/proc` there is no way to tell, and silence is the damaging
        // guess: a scheduled run nobody hears about is the failure this module
        // exists to prevent.
        return false;
    };
    let ours = ours.trim().to_owned();
    let mine = std::process::id().to_string();

    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };

    entries.flatten().any(|entry| {
        let name = entry.file_name();
        let Some(pid) = name.to_str() else {
            return false;
        };
        // `/proc` holds more than processes; anything not all-digits is not one.
        if pid == mine || pid.is_empty() || !pid.bytes().all(|byte| byte.is_ascii_digit()) {
            return false;
        }

        let path = entry.path();
        // A process can exit between listing and reading, which is a missing
        // file rather than an error worth reporting.
        let (Ok(comm), Ok(cmdline)) = (
            std::fs::read_to_string(path.join("comm")),
            std::fs::read(path.join("cmdline")),
        ) else {
            return false;
        };

        is_open_window(&comm, &ours, &cmdline)
    })
}

/// Whether one `/proc` entry is a copy of this application showing a window.
///
/// Split out from [`window_is_open`] so the decision can be tested without a
/// process tree arranged to point it at.
fn is_open_window(comm: &str, ours: &str, cmdline: &[u8]) -> bool {
    if comm.trim() != ours {
        return false;
    }
    // Arguments are NUL-separated in `/proc`, and the trailing NUL leaves an
    // empty final field, which matches nothing.
    !cmdline
        .split(|byte| *byte == 0)
        .any(|arg| arg == SCHEDULED_FLAG.as_bytes())
}

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

/// Whether this outcome is worth saying something about under this policy.
///
/// Separate from the posting so the decision can be tested on its own — it is
/// the part with rules in it, and the part a change is likely to get wrong.
fn would_notify(record: &Record, policy: Policy) -> bool {
    match record.outcome {
        Outcome::Failed => policy.errors,
        // Nothing to report about a run the user stopped themselves.
        Outcome::Cancelled => false,
        // A success the user can already see is not news.
        Outcome::Succeeded => policy.upgrades && !policy.on_screen,
    }
}

/// Post a notification about a finished run, if the user asked to hear about
/// this kind of outcome.
///
/// A failure is reported whatever else is switched off, short of switching
/// failures off specifically: it is the one outcome worth interrupting somebody
/// for, and the whole point of an unattended upgrade is not having to check.
pub fn run_finished(record: &Record, policy: Policy) {
    if !would_notify(record, policy) {
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
    use crate::constants::MINIMIZED_FLAG;
    use crate::history::ComponentRecord;

    /// Argument lists are NUL-separated in `/proc`, with a trailing NUL.
    fn cmdline(args: &[&str]) -> Vec<u8> {
        let mut out = Vec::new();
        for arg in args {
            out.extend_from_slice(arg.as_bytes());
            out.push(0);
        }
        out
    }

    /// What the kernel puts in `comm`: truncated to fifteen characters, with a
    /// trailing newline.
    const COMM: &str = "cosmic-upgrader\n";

    #[test]
    fn an_open_window_is_somebody_watching() {
        assert!(is_open_window(
            COMM,
            COMM.trim(),
            &cmdline(&["/usr/bin/cosmic-upgrader-gui"])
        ));
    }

    #[test]
    fn another_scheduled_run_is_not_somebody_watching() {
        // A check and an upgrade can both come due, and neither has a window.
        assert!(!is_open_window(
            COMM,
            COMM.trim(),
            &cmdline(&["/usr/bin/cosmic-upgrader-gui", SCHEDULED_FLAG, "--check"])
        ));
    }

    #[test]
    fn an_autostarted_copy_still_counts_as_a_window() {
        // The autostart entry passes `--minimized`, but nothing acts on it —
        // Wayland gives a client no way to un-minimize itself — so that copy has
        // a window like any other, and its user can see the run.
        assert!(is_open_window(
            COMM,
            COMM.trim(),
            &cmdline(&["/usr/bin/cosmic-upgrader-gui", MINIMIZED_FLAG])
        ));
    }

    #[test]
    fn some_other_program_is_not_this_one() {
        assert!(!is_open_window(
            "cosmic-files\n",
            COMM.trim(),
            &cmdline(&["/usr/bin/cosmic-files"])
        ));
    }

    #[test]
    fn a_successful_scheduled_run_stays_quiet_when_the_window_is_open() {
        // The whole point: at login, a missed run finishing while the user is
        // looking at the application should not announce itself.
        let watched = Policy {
            upgrades: true,
            errors: true,
            installs: false,
            on_screen: true,
        };
        let unwatched = Policy {
            on_screen: false,
            ..watched
        };
        let succeeded = record(Outcome::Succeeded, &[]);

        assert!(!would_notify(&succeeded, watched));
        assert!(would_notify(&succeeded, unwatched));
        // A failure is still worth interrupting for, either way.
        let failed = record(Outcome::Failed, &["system"]);
        assert!(would_notify(&failed, watched));
        assert!(would_notify(&failed, unwatched));
    }

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
        assert!(would_notify(&record(Outcome::Failed, &["system"]), policy));
        assert!(!would_notify(&record(Outcome::Succeeded, &[]), policy));
        // Not a run the user stopped themselves, whatever else is switched on.
        assert!(!would_notify(&record(Outcome::Cancelled, &[]), policy));
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
