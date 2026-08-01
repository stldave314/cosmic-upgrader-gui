// SPDX-License-Identifier: GPL-3.0

//! What each step can actually do on this machine.
//!
//! topgrade's step list is the same everywhere — 17.9.0 offers `winget` and
//! `macports` on Linux just as readily as on Windows or macOS — so the list
//! alone says nothing about what is worth showing. Presenting all 174 would bury
//! the handful that matter: on a typical desktop only a dozen or so steps have
//! anything to do.
//!
//! The answer is to ask topgrade. Running `--only <step>` in dry-run mode with
//! `--show-skipped` makes it do its own detection and report the outcome, which
//! is authoritative in a way that guessing from `PATH` could never be — the
//! `restarts` step defers to the package manager, `clam_av_db` checks whether a
//! systemd timer already handles it, and no amount of looking for binaries would
//! discover either.
//!
//! Dry-run is used rather than a real run because it stops short of doing
//! anything: topgrade prints the commands it would execute instead of executing
//! them. A full scan of every step is roughly 175 short-lived processes and
//! finishes in a couple of seconds when run [a few at a
//! time](crate::constants::probe_concurrency).
//!
//! ## Reading the result
//!
//! `--only` is what makes this work: it constrains the run to one step, so
//! whatever appears in the summary belongs to that step. The summary itself
//! could not be used unattributed, because the names it prints are display
//! names rather than identifiers — `git_repos` reports as "Git Repositories",
//! `vscode` as "Visual Studio Code extensions" — and matching those back to
//! identifiers by shape is guesswork that would quietly mis-file steps.
//!
//! A step is also not one result. `shell` reports seventeen, one per plugin
//! manager it knows about; `vim` reports four. And silence is ambiguous on its
//! own: `winget` on Linux prints nothing because it does not apply, but
//! `claude_code_plugins` also prints no summary line while plainly running. The
//! step heading separates those two cases, which is why it is parsed as well.
//!
//! ## What probing one step at a time cannot see
//!
//! Isolating a step is what makes its result attributable, but it also takes it
//! out of the context of a whole run, and a few steps behave differently there.
//! `restarts` is the clear example: probed alone it is ready to run, but in a
//! full run it stands down with "needrestart will be ran by the package
//! manager", because the `system` step is going to do the job.
//!
//! This is the right way round. A step reported here as available may turn out
//! to have nothing to do once its neighbours have run, which costs the user
//! nothing; the reverse — hiding a step that would in fact have run — would
//! quietly narrow what the application offers, and that is the failure worth
//! avoiding. The run summary shows what actually happened either way.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinSet;

use super::discover::StepId;
use super::Topgrade;
use crate::constants::{
    probe_concurrency, PROBE_TIMEOUT, STATUS_FAILED, STATUS_OK, STATUS_SKIPPED, SUMMARY_HEADING,
};
use crate::debug::PROBE;
use crate::debug_log;

/// What a step reported about one of the things it looks after.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Ok,
    Skipped,
    Failed,
}

/// One line of topgrade's summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Component {
    /// topgrade's own display name, shown as-is. These are already written for
    /// people — "Flatpak", "JetBrains RustRover Plugins" — and rewriting them
    /// would only introduce a second vocabulary for the same thing.
    pub name: String,
    pub status: Status,
    /// Why it was skipped or how it failed, when topgrade said. Worth surfacing
    /// verbatim: "Cannot find \"flatpak\" in PATH" tells the user exactly what
    /// to do, and no paraphrase would improve on it.
    pub reason: Option<String>,
}

/// Whether a step is worth offering on this machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Availability {
    /// Has something to do and would run.
    Available,
    /// Applies here, but cannot run as things stand — the tool is missing, or a
    /// path it needs does not exist. Still shown, because the reason is usually
    /// actionable.
    Unavailable { reason: String },
    /// Produced no output at all. Either it does not apply to this platform
    /// (`winget`, `macports`) or it has nothing configured yet
    /// (`custom_commands`, `remotes`). These are not the same thing, and
    /// topgrade gives no way to tell them apart, so they share a state and the
    /// interface leaves room for the user to configure one.
    Inactive,
    /// topgrade warned that the step is on its way out.
    Deprecated { note: String },
}

impl Availability {
    /// Whether the step should be offered as something to run.
    pub fn is_runnable(&self) -> bool {
        matches!(self, Self::Available)
    }
}

/// Everything learned about one step.
#[derive(Clone, Debug)]
pub struct StepReport {
    pub id: StepId,
    pub availability: Availability,
    /// The individual results, for steps that report more than one. Empty for a
    /// step that reported nothing.
    pub components: Vec<Component>,
}

/// Progress through a scan, for the interface to show while it runs.
#[derive(Clone, Debug)]
pub struct ScanProgress {
    pub completed: usize,
    pub total: usize,
    /// The step just finished, so the interface can name what it is doing.
    pub last: StepId,
}

/// The outcome of scanning every step.
#[derive(Clone, Debug, Default)]
pub struct Capabilities {
    reports: HashMap<StepId, StepReport>,
}

impl Capabilities {
    pub fn get(&self, id: &StepId) -> Option<&StepReport> {
        self.reports.get(id)
    }

    pub fn len(&self) -> usize {
        self.reports.len()
    }

    /// How many steps have something to do, which is the figure worth showing:
    /// "12 of 174 apply to this system" is the honest summary of a scan.
    pub fn runnable_count(&self) -> usize {
        self.reports
            .values()
            .filter(|report| report.availability.is_runnable())
            .count()
    }
}

impl FromIterator<StepReport> for Capabilities {
    fn from_iter<T: IntoIterator<Item = StepReport>>(iter: T) -> Self {
        Self {
            reports: iter
                .into_iter()
                .map(|report| (report.id.clone(), report))
                .collect(),
        }
    }
}

/// Probe every step, a few at a time, reporting progress as results arrive.
///
/// Results come back in completion order, so `progress` is what drives the
/// interface's counter; the returned [`Capabilities`] is keyed by step and does
/// not depend on ordering.
pub async fn scan(
    topgrade: &Topgrade,
    steps: &[StepId],
    progress: Option<mpsc::UnboundedSender<ScanProgress>>,
) -> Capabilities {
    let total = steps.len();
    let permits = Arc::new(Semaphore::new(probe_concurrency()));
    let mut tasks = JoinSet::new();

    debug_log!(
        PROBE,
        "scanning {total} steps, {} at a time",
        probe_concurrency()
    );

    for id in steps {
        let permits = Arc::clone(&permits);
        let topgrade = topgrade.clone();
        let id = id.clone();
        tasks.spawn(async move {
            // Dropped when the probe finishes, which is what bounds how many
            // topgrade processes exist at once.
            let _permit = permits.acquire_owned().await;
            probe_step(&topgrade, &id).await
        });
    }

    let mut reports = Vec::with_capacity(total);
    let mut completed = 0;
    while let Some(joined) = tasks.join_next().await {
        // A panicking probe task should not take the scan down with it: the
        // step it was looking at simply goes unreported, and every other step
        // still gets an answer.
        let Ok(report) = joined else {
            debug_log!(PROBE, "a probe task panicked, skipping that step");
            continue;
        };

        completed += 1;
        if let Some(progress) = progress.as_ref() {
            let _ = progress.send(ScanProgress {
                completed,
                total,
                last: report.id.clone(),
            });
        }
        reports.push(report);
    }

    let capabilities: Capabilities = reports.into_iter().collect();
    debug_log!(
        PROBE,
        "scan complete: {} runnable of {}",
        capabilities.runnable_count(),
        capabilities.len()
    );
    capabilities
}

/// Probe one step.
///
/// A step that cannot be probed at all — the process failed to start, or hung
/// past the timeout — is reported [`Inactive`](Availability::Inactive) rather
/// than dropped. Losing a step from the interface because its probe misbehaved
/// would be worse than showing it as having nothing to do, and the interface
/// offers a rescan.
async fn probe_step(topgrade: &Topgrade, id: &StepId) -> StepReport {
    let args = ["--dry-run", "--show-skipped", "--only", id.as_str()];

    let output = match tokio::time::timeout(PROBE_TIMEOUT, topgrade.output(&args)).await {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => {
            debug_log!(PROBE, "{id}: probe failed: {error}");
            String::new()
        }
        Err(_) => {
            debug_log!(PROBE, "{id}: probe timed out");
            String::new()
        }
    };

    parse_probe(id.clone(), &output)
}

/// Turn one probe's output into a report.
///
/// Split out from [`probe_step`] so it can be tested against output captured
/// from a real topgrade, which is the only way to be confident about a format
/// that is not a documented interface.
fn parse_probe(id: StepId, output: &str) -> StepReport {
    let (body, summary) = split_at_summary(output);

    let components: Vec<Component> = summary.lines().filter_map(parse_summary_line).collect();

    // A heading means topgrade entered the step, which distinguishes a step
    // that runs without reporting from one that does not apply here at all.
    let ran = body
        .lines()
        .any(|line| super::heading_name(line).is_some());

    let availability = if components.iter().any(|c| c.status == Status::Ok) {
        Availability::Available
    } else if let Some(reason) = collective_reason(&components) {
        Availability::Unavailable { reason }
    } else if ran {
        Availability::Available
    } else if let Some(note) = deprecation_note(body) {
        Availability::Deprecated { note }
    } else {
        Availability::Inactive
    };

    StepReport {
        id,
        availability,
        components,
    }
}

/// Split output into what came before the summary and the summary itself.
///
/// The summary heading is a step heading like any other, so it is read with the
/// same parser and compared to the word — rather than looking for the word
/// anywhere, which would also match a step whose output happened to mention it.
pub(super) fn split_at_summary(output: &str) -> (&str, &str) {
    for (offset, line) in line_offsets(output) {
        if super::heading_name(line) == Some(SUMMARY_HEADING) {
            let after = offset + line.len();
            return (&output[..offset], &output[after..]);
        }
    }
    (output, "")
}

/// Byte offset and content of each line, so a split can be made without
/// allocating a `Vec` of the whole output.
fn line_offsets(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0;
    text.split_inclusive('\n').map(move |line| {
        let start = offset;
        offset += line.len();
        (start, line.trim_end_matches(['\r', '\n']))
    })
}

/// Read one `Name: STATUS` or `Name: STATUS: reason` line.
///
/// The name is taken up to the first `": "` and the status from what follows,
/// because a reason can itself contain colons — pip's is "Skip pip3 update as it
/// is externally managed and global.break-system-packages is not true" — and
/// splitting on the last one would take the sentence apart in the wrong place.
pub(super) fn parse_summary_line(line: &str) -> Option<Component> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let (name, rest) = line.split_once(": ")?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    let (status, reason) = match rest.split_once(": ") {
        Some((word, reason)) => (word.trim(), Some(reason.trim().to_owned())),
        None => (rest.trim(), None),
    };

    let status = match status {
        STATUS_OK => Status::Ok,
        STATUS_SKIPPED => Status::Skipped,
        STATUS_FAILED => Status::Failed,
        // Not a summary line — topgrade prints other colon-separated text, and
        // anything unrecognised is safer ignored than guessed at.
        _ => return None,
    };

    Some(Component {
        name: name.to_owned(),
        status,
        reason,
    })
}

/// One reason standing for a whole step's worth of skipped components.
///
/// When every component was skipped for the same reason — all seventeen of
/// `shell`'s plugin managers need a shell that is not installed — that reason
/// describes the step, and repeating it seventeen times in the interface would
/// be noise. When they differ, no single reason is honest, so the count is
/// given instead and the detail is left to the expanded view.
fn collective_reason(components: &[Component]) -> Option<String> {
    if components.is_empty() {
        return None;
    }

    let mut reasons = components.iter().filter_map(|c| c.reason.as_deref());
    let first = reasons.next()?;
    if reasons.all(|reason| reason == first) {
        Some(first.to_owned())
    } else {
        Some(format!("{} components unavailable", components.len()))
    }
}

/// The warning a deprecated step leaves behind instead of running.
fn deprecation_note(body: &str) -> Option<String> {
    body.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && line.to_ascii_lowercase().contains("deprecated"))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Captured from `topgrade -n --show-skipped --only cargo` on 17.9.0.
    const CARGO: &str = "―― 17:11:57 - Cargo ――\n\
        Dry running: /home/dave/.cargo/bin/cargo-install-update install-update --all --git\n\
        ―― 17:11:57 - Summary ――\n\
        cargo: OK\n";

    /// `--only vscode`, where the tool is absent.
    const VSCODE: &str = "―― 17:11:57 - Summary ――\n\
        Visual Studio Code extensions: SKIPPED: Cannot find \"code\" in PATH\n";

    /// `--only shell`, abridged — seventeen components, all skipped, and not all
    /// for the same reason.
    const SHELL: &str = "―― 17:11:57 - Summary ――\n\
        zr: SKIPPED: Cannot find \"zsh\" in PATH\n\
        oh-my-zsh: SKIPPED: Cannot find \"zsh\" in PATH\n\
        bash-it: SKIPPED: Path \"/home/dave/.bash_it\" doesn't exist\n";

    /// `--only claude_code_plugins` — runs, but reports nothing in the summary.
    const RUNS_SILENTLY: &str = "―― 17:19:49 - Claude Code Plugins ――\n\
        Dry running: /home/dave/.local/bin/claude plugin marketplace update\n";

    /// `--only nix_helper` on a release where it has been retired.
    const DEPRECATED: &str = "    `NixHelper` step is deprecated\n";

    fn probe(id: &str, output: &str) -> StepReport {
        parse_probe(StepId::new(id), output)
    }

    #[test]
    fn a_step_with_work_to_do_is_available() {
        let report = probe("cargo", CARGO);
        assert_eq!(report.availability, Availability::Available);
        assert_eq!(report.components.len(), 1);
        assert_eq!(report.components[0].status, Status::Ok);
    }

    #[test]
    fn a_skipped_step_keeps_topgrades_own_reason() {
        let report = probe("vscode", VSCODE);
        assert_eq!(
            report.availability,
            Availability::Unavailable {
                reason: "Cannot find \"code\" in PATH".to_owned()
            }
        );
    }

    #[test]
    fn the_display_name_is_kept_not_the_identifier() {
        // `vscode` reports as "Visual Studio Code extensions"; mapping display
        // names back to identifiers is exactly what --only exists to avoid.
        let report = probe("vscode", VSCODE);
        assert_eq!(report.components[0].name, "Visual Studio Code extensions");
    }

    #[test]
    fn a_multi_component_step_keeps_every_component() {
        let report = probe("shell", SHELL);
        assert_eq!(report.components.len(), 3);
    }

    #[test]
    fn differing_reasons_collapse_to_a_count_not_a_misleading_one() {
        let report = probe("shell", SHELL);
        match report.availability {
            Availability::Unavailable { reason } => {
                assert_eq!(reason, "3 components unavailable");
            }
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }

    #[test]
    fn identical_reasons_collapse_to_that_reason() {
        let output = "―― 17:11:57 - Summary ――\n\
            zr: SKIPPED: Cannot find \"zsh\" in PATH\n\
            oh-my-zsh: SKIPPED: Cannot find \"zsh\" in PATH\n";
        match probe("shell", output).availability {
            Availability::Unavailable { reason } => {
                assert_eq!(reason, "Cannot find \"zsh\" in PATH");
            }
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }

    #[test]
    fn a_step_that_runs_without_reporting_is_still_available() {
        // The distinction this rests on: a heading was printed, so topgrade
        // entered the step, even though nothing reached the summary.
        let report = probe("claude_code_plugins", RUNS_SILENTLY);
        assert_eq!(report.availability, Availability::Available);
        assert!(report.components.is_empty());
    }

    #[test]
    fn silence_is_inactive() {
        // `winget` on Linux: no heading, no summary, no warning.
        let report = probe("winget", "");
        assert_eq!(report.availability, Availability::Inactive);
    }

    #[test]
    fn a_deprecation_warning_is_reported_as_such() {
        match probe("nix_helper", DEPRECATED).availability {
            Availability::Deprecated { note } => assert!(note.contains("deprecated")),
            other => panic!("expected Deprecated, got {other:?}"),
        }
    }

    #[test]
    fn a_reason_containing_colons_survives_intact() {
        let output = "―― 17:11:57 - Summary ――\n\
            pip3: SKIPPED: Skip pip3 update as it is externally managed and \
            global.break-system-packages is not true\n";
        let report = probe("pip3", output);
        let reason = report.components[0].reason.as_deref().expect("a reason");
        assert!(
            reason.starts_with("Skip pip3 update"),
            "reason was cut short: {reason:?}"
        );
        assert!(reason.ends_with("is not true"), "reason was cut short: {reason:?}");
    }

    #[test]
    fn output_before_the_summary_is_not_read_as_results() {
        // "Dry running: …" lines are colon-separated but are not results.
        let report = probe("cargo", CARGO);
        assert_eq!(report.components.len(), 1, "{:?}", report.components);
    }

    #[test]
    fn a_failed_component_is_recognised() {
        let output = "―― 17:11:57 - Summary ――\nsystem: FAILED: exit code 1\n";
        let report = probe("system", output);
        assert_eq!(report.components[0].status, Status::Failed);
        assert!(!report.availability.is_runnable());
    }
}

