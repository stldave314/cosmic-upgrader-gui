// SPDX-License-Identifier: GPL-3.0

//! Everything this application knows about topgrade, learned from topgrade.
//!
//! The guiding rule here is that no list of steps, categories or settings is
//! written down in this crate. topgrade gains and loses steps in most releases —
//! 17.9.0 has 174 — and a hard-coded list would be wrong the day after it was
//! written, in the quiet way where the application keeps working but silently
//! omits whatever is new. So the step list comes from `--help`, the
//! configuration schema comes from `--config-reference`, and what each step can
//! actually do on this machine comes from running it in dry-run mode.
//!
//! The one thing topgrade does not supply is presentation: it has no notion of
//! categories, and its step identifiers are terse (`gnome_shell_extensions`,
//! `pip_review_local`). [`categories`] adds that on top, but only as a lookup —
//! a step it has never heard of is still discovered, still shown and still
//! runnable, it simply lands in the catch-all category.

pub mod categories;
pub mod discover;
pub mod probe;
pub mod runner;
pub mod schema;
pub mod settings_file;

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use crate::constants::{
    HEADING_RULE_CHARS, INTROSPECT_TIMEOUT, TOPGRADE_BIN, TOPGRADE_BUNDLED_PATH,
    TOPGRADE_MIN_VERSION,
};
use crate::debug::{EXEC, LOCATE};
use crate::debug_log;

/// What went wrong talking to topgrade.
#[derive(Clone, Debug)]
pub enum Error {
    /// No topgrade on `PATH` and no bundled copy either.
    NotFound,
    /// A topgrade was found but is too old for its output to be read reliably.
    TooOld { found: Version },
    /// `--version` printed something this doesn't recognise.
    UnreadableVersion { output: String },
    /// A command could not be started, or was killed by the timeout.
    Exec { command: String, message: String },
    /// A command ran but its output could not be made sense of.
    Parse { what: &'static str, detail: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "topgrade is not installed"),
            Self::TooOld { found } => write!(
                f,
                "topgrade {found} is older than the required {}.{}",
                TOPGRADE_MIN_VERSION.0, TOPGRADE_MIN_VERSION.1
            ),
            Self::UnreadableVersion { output } => {
                write!(f, "could not read topgrade's version from {output:?}")
            }
            Self::Exec { command, message } => write!(f, "{command} failed: {message}"),
            Self::Parse { what, detail } => write!(f, "could not parse {what}: {detail}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// A topgrade release, compared by precedence rather than by string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Version {
    /// Read a version out of `topgrade 17.9.0`.
    ///
    /// Trailing detail after the patch number — a `-dev` suffix, a git hash on a
    /// self-built copy — is ignored rather than rejected, since it does not
    /// change how the output is read.
    fn parse(output: &str) -> Option<Self> {
        let field = output.split_whitespace().nth(1)?;
        let mut parts = field.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        // A release with no patch component is legitimate; treat it as zero.
        let patch = parts
            .next()
            .map(|p| {
                p.split(|c: char| !c.is_ascii_digit())
                    .next()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        Some(Self {
            major,
            minor,
            patch,
        })
    }

    fn is_supported(self) -> bool {
        (self.major, self.minor) >= TOPGRADE_MIN_VERSION
    }
}

/// Which topgrade is being driven.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Source {
    /// Found on `PATH` — the user's own installation, and the one preferred, so
    /// that upgrading topgrade upgrades what this application can do.
    System,
    /// The copy shipped inside our own package, used only when the system has
    /// none. Built by `install.sh` under the `bundled-topgrade` feature.
    Bundled,
}

/// A located, version-checked topgrade.
///
/// Held for the life of the process. Every other module in [`crate::topgrade`]
/// takes one of these rather than looking the binary up again, so the whole
/// application is guaranteed to be talking to the same executable — which
/// matters, because the discovered step list and the run that uses it have to
/// agree.
#[derive(Clone, Debug)]
pub struct Topgrade {
    path: PathBuf,
    version: Version,
    source: Source,
}

impl Topgrade {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn source(&self) -> Source {
        self.source
    }

    /// Find a usable topgrade, preferring the system's own.
    ///
    /// The preference matters beyond politeness: the system copy is the one the
    /// user updates, and because everything this application shows is
    /// discovered from the binary, using it means a newer topgrade's new steps
    /// appear here without this application changing at all.
    pub async fn locate() -> Result<Self> {
        let mut candidates: Vec<(PathBuf, Source)> = Vec::new();
        if let Some(path) = which(TOPGRADE_BIN).await {
            candidates.push((path, Source::System));
        }
        let bundled = PathBuf::from(TOPGRADE_BUNDLED_PATH);
        if bundled.is_file() {
            candidates.push((bundled, Source::Bundled));
        }

        debug_log!(LOCATE, "{} candidate(s)", candidates.len());

        // Remember the newest too-old candidate so the error names the version
        // actually present, rather than just saying "not found" for a machine
        // that plainly has one.
        let mut too_old: Option<Version> = None;

        for (path, source) in candidates {
            match read_version(&path).await {
                Ok(version) if version.is_supported() => {
                    debug_log!(LOCATE, "using {source:?} topgrade {version} at {path:?}");
                    return Ok(Self {
                        path,
                        version,
                        source,
                    });
                }
                Ok(version) => {
                    debug_log!(LOCATE, "{path:?} is topgrade {version}, too old");
                    too_old = Some(too_old.map_or(version, |best: Version| best.max(version)));
                }
                Err(error) => debug_log!(LOCATE, "{path:?} rejected: {error}"),
            }
        }

        Err(match too_old {
            Some(found) => Error::TooOld { found },
            None => Error::NotFound,
        })
    }

    /// Start a topgrade command with this binary, stdin closed and both output
    /// streams captured.
    ///
    /// Used for every introspection call. Runs are started by
    /// [`runner`](crate::topgrade::runner) instead, which needs a terminal on
    /// the other end and so cannot go through here.
    pub(crate) fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(&self.path);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // topgrade colours its output when it believes it is talking to a
        // terminal and honours this variable when it is not. Introspection
        // output is parsed, so escape sequences would be nothing but noise to
        // strip back out again.
        command.env("NO_COLOR", "1");
        command.kill_on_drop(true);
        command
    }

    /// Run a topgrade command to completion and return its combined output.
    ///
    /// Both streams are wanted: topgrade writes step output to stdout but puts
    /// warnings — including the deprecation notices that are the only trace
    /// some steps leave — on stderr, and a probe that read only one of them
    /// would draw the wrong conclusion.
    pub(crate) async fn output(&self, args: &[&str]) -> Result<String> {
        let described = || format!("{} {}", self.path.display(), args.join(" "));

        let child = self.command(args).spawn().map_err(|error| Error::Exec {
            command: described(),
            message: error.to_string(),
        })?;

        let output = match tokio::time::timeout(INTROSPECT_TIMEOUT, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(error)) => {
                return Err(Error::Exec {
                    command: described(),
                    message: error.to_string(),
                })
            }
            Err(_) => {
                return Err(Error::Exec {
                    command: described(),
                    message: format!("timed out after {INTROSPECT_TIMEOUT:?}"),
                })
            }
        };

        let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            combined.push('\n');
            combined.push_str(&stderr);
        }

        debug_log!(
            EXEC,
            "{} -> {} ({} bytes)",
            described(),
            output.status,
            combined.len()
        );

        Ok(combined)
    }
}

/// The step name from a heading line, or `None` if the line is not one.
///
/// Lives here rather than in [`probe`] or [`runner`] because both read this
/// format from different channels — piped output and a pseudo-terminal — and
/// they disagreed about it once already. One parser means one thing to fix.
pub(crate) fn heading_name(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let stripped = trimmed.trim_start_matches(|c| HEADING_RULE_CHARS.contains(&c));
    if stripped.len() == trimmed.len() {
        // Nothing was stripped, so the line does not open with a rule.
        return None;
    }

    let inner = stripped
        .trim_end_matches(|c| HEADING_RULE_CHARS.contains(&c))
        .trim();
    if inner.is_empty() {
        // A rule with nothing in it is a separator, not a heading.
        return None;
    }

    // topgrade omits the timestamp when `display_time` is off, so the name is
    // whatever follows the separator when there is one and the whole of it
    // otherwise.
    Some(match inner.split_once(" - ") {
        Some((_timestamp, name)) => name.trim(),
        None => inner,
    })
}

/// Read `--version` from a candidate binary.
async fn read_version(path: &Path) -> Result<Version> {
    let mut command = Command::new(path);
    command
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);

    let child = command.spawn().map_err(|error| Error::Exec {
        command: format!("{} --version", path.display()),
        message: error.to_string(),
    })?;

    let output = tokio::time::timeout(INTROSPECT_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| Error::Exec {
            command: format!("{} --version", path.display()),
            message: "timed out".to_owned(),
        })?
        .map_err(|error| Error::Exec {
            command: format!("{} --version", path.display()),
            message: error.to_string(),
        })?;

    let text = String::from_utf8_lossy(&output.stdout);
    Version::parse(text.trim()).ok_or_else(|| Error::UnreadableVersion {
        output: text.trim().to_owned(),
    })
}

/// Resolve an executable name against `PATH`.
///
/// Written out rather than shelling out to `which`, which is one more process
/// and one more thing that can be absent on a minimal system.
async fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_piped_heading() {
        assert_eq!(heading_name("―― 17:11:57 - Cargo ――"), Some("Cargo"));
    }

    #[test]
    fn reads_a_pseudo_terminal_heading() {
        // What a real run actually produces: a different rule character, and
        // padded out to the width of the terminal.
        let line = "── 18:59:20 - Flatpak System Packages ──────────────────────────";
        assert_eq!(heading_name(line), Some("Flatpak System Packages"));
    }

    #[test]
    fn reads_a_heading_with_no_timestamp() {
        assert_eq!(heading_name("―― Cargo ――"), Some("Cargo"));
    }

    #[test]
    fn ordinary_output_is_not_a_heading() {
        assert_eq!(heading_name("Dry running: /usr/bin/cargo install-update"), None);
        assert_eq!(heading_name(""), None);
        assert_eq!(heading_name("cargo: OK"), None);
    }

    #[test]
    fn a_bare_rule_is_not_a_heading() {
        assert_eq!(heading_name("────────────────"), None);
    }

    #[test]
    fn parses_a_release_version() {
        let version = Version::parse("topgrade 17.9.0").expect("should parse");
        assert_eq!(
            version,
            Version {
                major: 17,
                minor: 9,
                patch: 0
            }
        );
    }

    #[test]
    fn tolerates_a_suffixed_patch_component() {
        let version = Version::parse("topgrade 18.0.1-dev").expect("should parse");
        assert_eq!(version.patch, 1);
    }

    #[test]
    fn treats_a_missing_patch_component_as_zero() {
        let version = Version::parse("topgrade 18.1").expect("should parse");
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn rejects_output_that_is_not_a_version() {
        assert!(Version::parse("command not found").is_none());
    }

    #[test]
    fn compares_by_precedence_not_lexically() {
        // The string comparison this replaces would put 9 above 17.
        let older = Version::parse("topgrade 9.0.0").expect("should parse");
        let newer = Version::parse("topgrade 17.0.0").expect("should parse");
        assert!(newer > older);
    }
}
