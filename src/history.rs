// SPDX-License-Identifier: GPL-3.0

//! Keeping what happened, so it can be looked at afterwards.
//!
//! An upgrade is exactly the kind of thing you want to read back later: it ran
//! for ten minutes, it touched forty packages, and the interesting part scrolled
//! past while you were doing something else. A scheduled run is worse, because
//! nobody was watching at all.
//!
//! So every run — started from the window or by a timer — writes two files into
//! `~/.local/share/cosmic-upgrader-gui/runs/`:
//!
//! * `<id>.json`, a small record of when it ran, how it was started and what the
//!   summary said, which is what the history list is built from.
//! * `<id>.log`, the full output.
//!
//! They are split so that listing the history reads a few kilobytes rather than
//! every transcript ever written; a busy machine's log runs to megabytes and
//! there is no reason to touch it until somebody asks to see it.
//!
//! The identifier is a sortable UTC timestamp, so the directory sorts into
//! chronological order with no index to keep consistent and nothing to rebuild
//! if a file is deleted by hand.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::constants::HISTORY_DIR;
use crate::debug::HISTORY;
use crate::debug_log;
use crate::topgrade::probe::{Component, Status};

#[derive(Clone, Debug)]
pub enum Error {
    NoDataDirectory,
    Io { path: PathBuf, message: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDataDirectory => write!(f, "no data directory could be determined"),
            Self::Io { path, message } => write!(f, "{}: {message}", path.display()),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// How a run was started.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Origin {
    /// Started from the window.
    #[default]
    Manual,
    /// Started by a timer, with nobody watching — which is why the record
    /// matters more for these.
    Scheduled,
}

/// How a run ended, in the form the history list shows.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Outcome {
    #[default]
    Succeeded,
    /// Finished, but at least one step failed.
    Failed,
    Cancelled,
}

/// One component's result, flattened for storage.
///
/// [`Component`](crate::topgrade::probe::Component) is not stored directly
/// because it belongs to the topgrade layer and would tie the on-disk format to
/// that type's shape.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ComponentRecord {
    pub name: String,
    /// `"ok"`, `"skipped"` or `"failed"`.
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl ComponentRecord {
    fn from_component(component: &Component) -> Self {
        Self {
            name: component.name.clone(),
            status: match component.status {
                Status::Ok => "ok",
                Status::Skipped => "skipped",
                Status::Failed => "failed",
            }
            .to_owned(),
            reason: component.reason.clone(),
        }
    }

    pub fn failed(&self) -> bool {
        self.status == "failed"
    }
}

/// What is known about one past run.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Record {
    /// Sortable UTC identifier, and the stem of both files.
    pub id: String,
    /// Seconds since the Unix epoch. Stored as a number rather than a formatted
    /// string so the display can follow the machine's timezone and locale
    /// rather than whatever they were when it was written.
    pub started: i64,
    #[serde(default)]
    pub finished: i64,
    #[serde(default)]
    pub origin: Origin,
    #[serde(default)]
    pub outcome: Outcome,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub ok: usize,
    #[serde(default)]
    pub skipped: usize,
    #[serde(default)]
    pub failed: usize,
    #[serde(default)]
    pub components: Vec<ComponentRecord>,
}

impl Record {
    /// The steps that failed, which is what a notification needs to name.
    pub fn failures(&self) -> Vec<&ComponentRecord> {
        self.components.iter().filter(|c| c.failed()).collect()
    }

    /// How long the run took, in seconds.
    pub fn duration(&self) -> i64 {
        (self.finished - self.started).max(0)
    }

    /// When it started, in the machine's own timezone.
    pub fn started_local(&self) -> String {
        format_timestamp(self.started)
    }
}

/// Render a Unix timestamp as a local date and time.
///
/// Falls back to the raw number rather than failing: a history entry with an
/// odd timestamp should still be listed and readable.
pub fn format_timestamp(seconds: i64) -> String {
    jiff::Timestamp::from_second(seconds)
        .map(|timestamp| {
            timestamp
                .to_zoned(jiff::tz::TimeZone::system())
                .strftime("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|_| seconds.to_string())
}

/// Where run records are kept.
pub fn directory() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
        })
        .ok_or(Error::NoDataDirectory)?;
    Ok(base.join(HISTORY_DIR))
}

/// A sortable identifier for a run starting now.
///
/// UTC rather than local time, so the ordering does not go backwards when the
/// clocks change — which would otherwise put an autumn run in the wrong place
/// in the list once a year.
fn new_id(started: i64) -> String {
    jiff::Timestamp::from_second(started)
        .map(|timestamp| timestamp.strftime("%Y%m%dT%H%M%SZ").to_string())
        .unwrap_or_else(|_| format!("{started}"))
}

/// A run being recorded as it happens.
///
/// Output is appended to the transcript as it arrives rather than held until
/// the end, so a run that is killed — or that takes the machine down with it —
/// still leaves everything up to that point on disk.
pub struct Recorder {
    record: Record,
    json_path: PathBuf,
    log: Option<std::fs::File>,
}

impl Recorder {
    /// Begin recording. Returns `None` if the directory cannot be created, in
    /// which case the run still happens — it is simply not written down.
    pub fn start(origin: Origin, dry_run: bool, started: i64) -> Option<Self> {
        Self::start_in(&directory().ok()?, origin, dry_run, started)
    }

    /// As [`start`](Self::start), against a given directory.
    ///
    /// The directory is a parameter rather than always being looked up so the
    /// tests can use a temporary one. Pointing `XDG_DATA_HOME` at a scratch
    /// directory would do the same job, but environment variables are shared by
    /// the whole process and the test harness runs tests in parallel — so they
    /// would overwrite each other's answers.
    pub fn start_in(
        directory: &Path,
        origin: Origin,
        dry_run: bool,
        started: i64,
    ) -> Option<Self> {
        if let Err(error) = std::fs::create_dir_all(directory) {
            debug_log!(HISTORY, "cannot create {}: {error}", directory.display());
            return None;
        }

        let id = new_id(started);
        let json_path = directory.join(format!("{id}.json"));

        let log = std::fs::File::create(directory.join(format!("{id}.log")))
            .map_err(|error| debug_log!(HISTORY, "cannot open transcript: {error}"))
            .ok();

        debug_log!(HISTORY, "recording run {id}");

        Some(Self {
            record: Record {
                id,
                started,
                finished: started,
                origin,
                outcome: Outcome::Succeeded,
                dry_run,
                ok: 0,
                skipped: 0,
                failed: 0,
                components: Vec::new(),
            },
            json_path,
            log,
        })
    }

    /// Append one line of output.
    pub fn write_line(&mut self, line: &str) {
        use std::io::Write;
        if let Some(log) = self.log.as_mut() {
            let _ = writeln!(log, "{line}");
        }
    }

    /// Close the record out and write the summary beside the transcript.
    pub fn finish(
        mut self,
        components: &[Component],
        cancelled: bool,
        finished: i64,
    ) -> Option<Record> {
        use std::io::Write;
        if let Some(log) = self.log.as_mut() {
            let _ = log.flush();
        }

        self.record.finished = finished;
        self.record.components = components.iter().map(ComponentRecord::from_component).collect();
        self.record.ok = components.iter().filter(|c| c.status == Status::Ok).count();
        self.record.skipped = components
            .iter()
            .filter(|c| c.status == Status::Skipped)
            .count();
        self.record.failed = components
            .iter()
            .filter(|c| c.status == Status::Failed)
            .count();

        self.record.outcome = if cancelled {
            Outcome::Cancelled
        } else if self.record.failed > 0 {
            Outcome::Failed
        } else {
            Outcome::Succeeded
        };

        match serde_json::to_string_pretty(&self.record) {
            Ok(json) => {
                if let Err(error) = std::fs::write(&self.json_path, json) {
                    debug_log!(HISTORY, "cannot write record: {error}");
                    return None;
                }
            }
            Err(error) => {
                debug_log!(HISTORY, "cannot serialize record: {error}");
                return None;
            }
        }

        debug_log!(
            HISTORY,
            "run {} recorded: {:?}, {} failed",
            self.record.id,
            self.record.outcome,
            self.record.failed
        );
        Some(self.record)
    }
}

/// Every recorded run, newest first.
///
/// A record that cannot be read is skipped rather than failing the listing: one
/// truncated file — from a run interrupted by a power cut, say — should not hide
/// the rest of the history.
pub fn list() -> Vec<Record> {
    match directory() {
        Ok(directory) => list_in(&directory),
        Err(_) => Vec::new(),
    }
}

/// As [`list`], against a given directory.
pub fn list_in(directory: &Path) -> Vec<Record> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut records: Vec<Record> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "json"))
        .filter_map(|path| match std::fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Record>(&text) {
                Ok(record) => Some(record),
                Err(error) => {
                    debug_log!(HISTORY, "skipping {}: {error}", path.display());
                    None
                }
            },
            Err(error) => {
                debug_log!(HISTORY, "cannot read {}: {error}", path.display());
                None
            }
        })
        .collect();

    // Newest first, which is the order the history list wants.
    records.sort_by_key(|record| std::cmp::Reverse(record.started));
    records
}

/// The full transcript of a recorded run.
pub fn transcript(id: &str) -> Result<String> {
    transcript_in(&directory()?, id)
}

/// As [`transcript`], against a given directory.
pub fn transcript_in(directory: &Path, id: &str) -> Result<String> {
    let path = directory.join(format!("{id}.log"));
    std::fs::read_to_string(&path).map_err(|error| Error::Io {
        path,
        message: error.to_string(),
    })
}

/// Delete all but the newest `keep` runs.
///
/// Transcripts are the large part and grow without bound otherwise; a machine
/// upgraded nightly would accumulate them for years.
pub fn prune(keep: usize) {
    if let Ok(directory) = directory() {
        prune_in(&directory, keep);
    }
}

/// As [`prune`], against a given directory.
pub fn prune_in(directory: &Path, keep: usize) {
    for record in list_in(directory).into_iter().skip(keep.max(1)) {
        for extension in ["json", "log"] {
            let path = directory.join(format!("{}.{extension}", record.id));
            if let Err(error) = std::fs::remove_file(&path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    debug_log!(HISTORY, "cannot remove {}: {error}", path.display());
                }
            }
        }
    }
}

/// Remove one run's files.
pub fn remove(id: &str) -> Result<()> {
    remove_in(&directory()?, id)
}

/// As [`remove`], against a given directory.
pub fn remove_in(directory: &Path, id: &str) -> Result<()> {
    for extension in ["json", "log"] {
        let path = directory.join(format!("{id}.{extension}"));
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(Error::Io {
                    path,
                    message: error.to_string(),
                })
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component(name: &str, status: Status) -> Component {
        Component {
            name: name.to_owned(),
            status,
            reason: (status != Status::Ok).then(|| "because".to_owned()),
        }
    }

    /// A scratch directory of its own per test, removed on drop.
    ///
    /// Each test gets a distinct path so the harness can keep running them in
    /// parallel; nothing here reads or writes an environment variable.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("cosmic-upgrader-gui-test-{name}"));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("scratch directory");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn identifiers_sort_chronologically() {
        // The listing depends on this, and on it surviving a clock change: the
        // identifier is UTC, so an hour going backwards locally in autumn does
        // not reorder the history.
        let earlier = new_id(1_760_000_000);
        let later = new_id(1_760_003_600);
        assert!(earlier < later, "{earlier} should sort before {later}");
    }

    #[test]
    fn an_identifier_is_filesystem_safe() {
        let id = new_id(1_760_000_000);
        assert!(
            id.chars().all(|c| c.is_ascii_alphanumeric()),
            "identifier is used as a filename: {id}"
        );
    }

    #[test]
    fn a_timestamp_renders_as_a_readable_local_time() {
        let rendered = format_timestamp(1_760_000_000);
        assert!(rendered.starts_with("202"), "{rendered}");
        assert_eq!(rendered.len(), "2025-10-09 12:13:20".len(), "{rendered}");
    }

    #[test]
    fn an_impossible_timestamp_still_renders_something() {
        // Better a raw number in the list than a run that cannot be shown.
        assert!(!format_timestamp(i64::MAX).is_empty());
    }

    #[test]
    fn a_failed_component_makes_the_run_a_failure() {
        let scratch = Scratch::new("history-fail");
        let recorder =
            Recorder::start_in(scratch.path(), Origin::Scheduled, false, 1_760_000_000)
                .expect("recorder");
        let record = recorder
            .finish(
                &[
                    component("cargo", Status::Ok),
                    component("system", Status::Failed),
                    component("vim", Status::Skipped),
                ],
                false,
                1_760_000_100,
            )
            .expect("record");

        assert_eq!(record.outcome, Outcome::Failed);
        assert_eq!((record.ok, record.skipped, record.failed), (1, 1, 1));
        assert_eq!(record.failures().len(), 1);
        assert_eq!(record.failures()[0].name, "system");
        assert_eq!(record.duration(), 100);
    }

    #[test]
    fn a_cancelled_run_is_not_reported_as_a_failure() {
        let scratch = Scratch::new("history-cancel");
        let recorder = Recorder::start_in(scratch.path(), Origin::Manual, false, 1_760_000_000)
            .expect("recorder");
        let record = recorder
            .finish(&[component("cargo", Status::Ok)], true, 1_760_000_010)
            .expect("record");
        assert_eq!(record.outcome, Outcome::Cancelled);
    }

    #[test]
    fn a_run_round_trips_through_the_filesystem() {
        let scratch = Scratch::new("history-round");
        let mut recorder =
            Recorder::start_in(scratch.path(), Origin::Manual, true, 1_760_000_000)
                .expect("recorder");
        recorder.write_line("―― Cargo ――");
        recorder.write_line("Dry running: cargo install-update");
        let record = recorder
            .finish(&[component("cargo", Status::Ok)], false, 1_760_000_030)
            .expect("record");

        let listed = list_in(scratch.path());
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, record.id);
        assert!(listed[0].dry_run);
        assert_eq!(listed[0].origin, Origin::Manual);

        let text = transcript_in(scratch.path(), &record.id).expect("transcript");
        assert!(text.contains("Dry running: cargo install-update"), "{text}");
    }

    #[test]
    fn output_is_on_disk_before_the_run_ends() {
        // A run killed partway through should still leave its output behind.
        let scratch = Scratch::new("history-partial");
        let mut recorder =
            Recorder::start_in(scratch.path(), Origin::Manual, false, 1_760_000_000)
                .expect("recorder");
        recorder.write_line("first line");
        drop(recorder);

        let logs: Vec<_> = std::fs::read_dir(scratch.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "log"))
            .collect();
        assert_eq!(logs.len(), 1);
        let text = std::fs::read_to_string(logs[0].path()).unwrap();
        assert!(text.contains("first line"), "{text:?}");
    }

    #[test]
    fn the_newest_runs_are_listed_first() {
        let scratch = Scratch::new("history-order");
        for index in 0..3 {
            Recorder::start_in(
                scratch.path(),
                Origin::Manual,
                false,
                1_760_000_000 + index * 3_600,
            )
            .expect("recorder")
            .finish(&[component("cargo", Status::Ok)], false, 1_760_000_100)
            .expect("record");
        }
        let listed = list_in(scratch.path());
        assert_eq!(listed[0].started, 1_760_000_000 + 2 * 3_600);
        assert_eq!(listed[2].started, 1_760_000_000);
    }

    #[test]
    fn pruning_keeps_the_newest_runs_and_removes_their_transcripts() {
        let scratch = Scratch::new("history-prune");
        for index in 0..5 {
            let mut recorder = Recorder::start_in(
                scratch.path(),
                Origin::Manual,
                false,
                1_760_000_000 + index * 3_600,
            )
            .expect("recorder");
            recorder.write_line("some output");
            recorder
                .finish(&[component("cargo", Status::Ok)], false, 1_760_000_100)
                .expect("record");
        }
        assert_eq!(list_in(scratch.path()).len(), 5);

        prune_in(scratch.path(), 2);
        let remaining = list_in(scratch.path());
        assert_eq!(remaining.len(), 2);
        assert_eq!(remaining[0].started, 1_760_000_000 + 4 * 3_600);

        // The transcripts must go too — they are the part that grows.
        let logs = std::fs::read_dir(scratch.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "log"))
            .count();
        assert_eq!(logs, 2, "transcripts were left behind");
    }

    #[test]
    fn pruning_never_removes_everything() {
        let scratch = Scratch::new("history-prune-zero");
        Recorder::start_in(scratch.path(), Origin::Manual, false, 1_760_000_000)
            .expect("recorder")
            .finish(&[component("cargo", Status::Ok)], false, 1_760_000_010)
            .expect("record");
        prune_in(scratch.path(), 0);
        assert_eq!(list_in(scratch.path()).len(), 1);
    }

    #[test]
    fn removing_a_run_takes_both_of_its_files() {
        let scratch = Scratch::new("history-remove");
        let record = Recorder::start_in(scratch.path(), Origin::Manual, false, 1_760_000_000)
            .expect("recorder")
            .finish(&[component("cargo", Status::Ok)], false, 1_760_000_010)
            .expect("record");

        remove_in(scratch.path(), &record.id).expect("remove");
        assert!(list_in(scratch.path()).is_empty());
        assert_eq!(std::fs::read_dir(scratch.path()).unwrap().count(), 0);
    }

    #[test]
    fn an_unreadable_record_does_not_hide_the_rest() {
        let scratch = Scratch::new("history-bad");
        Recorder::start_in(scratch.path(), Origin::Manual, false, 1_760_000_000)
            .expect("recorder")
            .finish(&[component("cargo", Status::Ok)], false, 1_760_000_010)
            .expect("record");
        std::fs::write(scratch.path().join("broken.json"), "{ not json").expect("write");

        assert_eq!(
            list_in(scratch.path()).len(),
            1,
            "one truncated file should not hide the history"
        );
    }
}
