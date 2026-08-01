// SPDX-License-Identifier: GPL-3.0

//! Scanning after the virus database changes.
//!
//! topgrade already keeps ClamAV's database current — its `clam_av_db` step
//! runs `freshclam`, and stands down when systemd's `clamav-freshclam` service
//! is doing it instead, which on most systems it is. What neither of them does
//! is *use* the new database. A signature published this morning finds nothing
//! until something scans with it.
//!
//! So this watches the database rather than the updater. Comparing the
//! database directory either side of a run detects a change however it
//! happened — by topgrade, by the systemd service, or by somebody running
//! `freshclam` by hand — which is the only way to catch it on a system where
//! topgrade's own step correctly reports "SKIPPED: freshclam autoupdate is
//! active via systemd".
//!
//! The scan itself is deliberately not comprehensive by default. A full-system
//! scan takes hours and reads every mounted disk; somebody agreeing to "scan
//! after an update" is not asking for that.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use crate::constants::{CLAMAV_DATABASE_DIR, CLAMSCAN_TIMEOUT};
use crate::debug::CLAMAV;
use crate::debug_log;

/// A summary of the database directory, for spotting that it changed.
///
/// Names, sizes and modification times rather than contents: the databases run
/// to hundreds of megabytes, and hashing them to notice an update would cost
/// more than the scan it triggers.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Fingerprint(Vec<(String, u64, i64)>);

impl Fingerprint {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Read the current state of the database directory.
pub fn fingerprint() -> Fingerprint {
    fingerprint_of(Path::new(CLAMAV_DATABASE_DIR))
}

fn fingerprint_of(directory: &Path) -> Fingerprint {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Fingerprint::default();
    };

    let mut files: Vec<(String, u64, i64)> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            // Only the signature databases; freshclam also leaves locks and
            // partial downloads behind, and those change constantly.
            if !name.ends_with(".cvd") && !name.ends_with(".cld") {
                return None;
            }
            let meta = entry.metadata().ok()?;
            let modified = meta
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|elapsed| elapsed.as_secs() as i64)
                .unwrap_or(0);
            Some((name, meta.len(), modified))
        })
        .collect();

    files.sort();
    Fingerprint(files)
}

/// Whether the database changed between two readings.
pub fn changed(before: &Fingerprint, after: &Fingerprint) -> bool {
    // An empty reading either side means the directory could not be read, and
    // "it might have changed" is not a good enough reason to spend an hour
    // scanning.
    !before.is_empty() && !after.is_empty() && before != after
}

/// What a scan found.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    /// How many files matched a signature.
    pub infected: usize,
    /// How many were looked at.
    pub scanned: usize,
    /// The paths that matched, for the notification to name.
    pub found: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum Error {
    NotInstalled,
    Failed(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInstalled => write!(f, "clamscan is not installed"),
            Self::Failed(message) => write!(f, "{message}"),
        }
    }
}

/// Read what `clamscan` reported.
///
/// Its summary is stable across versions and its per-file hits are the lines
/// ending in `FOUND`, which is what `--infected` leaves in the output.
pub fn parse_output(output: &str) -> Report {
    let mut infected = 0;
    let mut scanned = 0;
    let mut found = Vec::new();

    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("Infected files: ") {
            infected = rest.trim().parse().unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("Scanned files: ") {
            scanned = rest.trim().parse().unwrap_or(0);
        } else if let Some(hit) = line.strip_suffix(" FOUND") {
            // `path: Signature.Name FOUND`; the path may itself contain a
            // colon, so the split is from the right.
            if let Some((path, _)) = hit.rsplit_once(": ") {
                found.push(path.to_owned());
            }
        }
    }

    Report {
        infected,
        scanned,
        found,
    }
}

/// Expand a leading `~` against the user's home directory.
///
/// The target is something the user typed, and `~/Downloads` is the obvious
/// thing to type; passing it through unexpanded would scan a directory called
/// `~` in the working directory, which is not what anybody means.
fn expand(target: &str) -> PathBuf {
    let target = target.trim();
    if let Some(rest) = target.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    if target == "~" {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(target)
}

/// Split a user-supplied option string into arguments.
///
/// Whitespace-separated, which is all `clamscan`'s options need. Deliberately
/// not a shell: the string is not run through one, so nothing in it can expand
/// into another command.
fn split_options(options: &str) -> Vec<String> {
    options
        .split_whitespace()
        .map(str::to_owned)
        .filter(|argument| !argument.is_empty())
        .collect()
}

/// Run a scan.
pub async fn scan(options: &str, target: &str) -> Result<Report, Error> {
    let target = expand(target);
    let arguments = split_options(options);
    debug_log!(CLAMAV, "scanning {} with {arguments:?}", target.display());

    let output = tokio::time::timeout(
        CLAMSCAN_TIMEOUT,
        Command::new("clamscan")
            .args(&arguments)
            .arg(&target)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| Error::Failed(format!("timed out after {CLAMSCAN_TIMEOUT:?}")))?
    .map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => Error::NotInstalled,
        _ => Error::Failed(error.to_string()),
    })?;

    let text = String::from_utf8_lossy(&output.stdout);
    let report = parse_output(&text);

    // clamscan exits 1 when it found something, which is a successful scan
    // rather than a failure; only 2 and above mean it could not do its job.
    if output.status.code().unwrap_or(0) >= 2 {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Failed(stderr.trim().to_owned()));
    }

    debug_log!(
        CLAMAV,
        "scanned {} file(s), {} infected",
        report.scanned,
        report.infected
    );
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLEAN: &str = "\
----------- SCAN SUMMARY -----------
Known viruses: 8703112
Scanned directories: 412
Scanned files: 1893
Infected files: 0
Time: 92.145 sec (1 m 32 s)
";

    const INFECTED: &str = "\
/home/dave/Downloads/thing.bin: Win.Test.EICAR_HDB-1 FOUND
/home/dave/Downloads/other.bin: Win.Test.EICAR_HDB-1 FOUND

----------- SCAN SUMMARY -----------
Scanned files: 1893
Infected files: 2
";

    #[test]
    fn reads_a_clean_scan() {
        let report = parse_output(CLEAN);
        assert_eq!(report.infected, 0);
        assert_eq!(report.scanned, 1893);
        assert!(report.found.is_empty());
    }

    #[test]
    fn reads_the_paths_that_matched() {
        let report = parse_output(INFECTED);
        assert_eq!(report.infected, 2);
        assert_eq!(
            report.found,
            [
                "/home/dave/Downloads/thing.bin",
                "/home/dave/Downloads/other.bin"
            ]
        );
    }

    #[test]
    fn a_home_relative_target_is_expanded() {
        // Left alone it would name a directory called `~`.
        std::env::set_var("HOME", "/home/test");
        assert_eq!(expand("~/Downloads"), PathBuf::from("/home/test/Downloads"));
        assert_eq!(expand("~"), PathBuf::from("/home/test"));
        assert_eq!(expand("/var/tmp"), PathBuf::from("/var/tmp"));
    }

    #[test]
    fn options_are_split_without_a_shell() {
        // Nothing in the string can become another command, because it is never
        // handed to one.
        let split = split_options("--infected --recursive --max-filesize=100M");
        assert_eq!(split, ["--infected", "--recursive", "--max-filesize=100M"]);
        assert!(split_options("   ").is_empty());
        assert_eq!(split_options("--a; rm -rf /"), ["--a;", "rm", "-rf", "/"]);
    }

    #[test]
    fn a_changed_database_is_noticed() {
        let before = Fingerprint(vec![("daily.cvd".into(), 100, 1_000)]);
        let after = Fingerprint(vec![("daily.cvd".into(), 120, 2_000)]);
        assert!(changed(&before, &after));
        assert!(!changed(&before, &before));
    }

    #[test]
    fn an_unreadable_directory_does_not_trigger_a_scan() {
        // "It might have changed" is not worth an hour of disk reads.
        let known = Fingerprint(vec![("daily.cvd".into(), 100, 1_000)]);
        assert!(!changed(&Fingerprint::default(), &known));
        assert!(!changed(&known, &Fingerprint::default()));
    }

    #[test]
    fn only_signature_databases_are_fingerprinted() {
        let directory = std::env::temp_dir().join("cosmic-upgrader-gui-clamav");
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("scratch");
        std::fs::write(directory.join("daily.cvd"), b"x").expect("write");
        std::fs::write(directory.join("main.cld"), b"y").expect("write");
        // freshclam leaves these behind and they change constantly.
        std::fs::write(directory.join("freshclam.dat"), b"z").expect("write");
        std::fs::write(directory.join("mirrors.dat"), b"w").expect("write");

        let print = fingerprint_of(&directory);
        let names: Vec<&str> = print.0.iter().map(|(name, _, _)| name.as_str()).collect();
        assert_eq!(names, ["daily.cvd", "main.cld"]);

        let _ = std::fs::remove_dir_all(&directory);
    }
}
