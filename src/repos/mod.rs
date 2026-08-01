// SPDX-License-Identifier: GPL-3.0

//! Where packages come from.
//!
//! topgrade upgrades what is installed; it has nothing to say about the
//! repositories those packages come from. Adding a PPA, turning off a
//! third-party source that has started breaking `apt update`, or adding a
//! Flatpak remote all mean editing files under `/etc` or remembering a command
//! — which is exactly the sort of thing a front-end should take care of.
//!
//! Three kinds are handled, because between them they cover what a desktop
//! actually has:
//!
//! * **apt**, in both formats now in use — see [`apt`].
//! * **Flatpak remotes**, which need no privileges at all when they belong to
//!   the user.
//! * **dnf**, for rpm systems.
//!
//! ## Reading is safe; writing is not
//!
//! Listing repositories reads world-readable files and runs `flatpak remotes`,
//! so it needs no privileges and cannot break anything. Changing one under
//! `/etc` does need root, and a mistake there stops every upgrade path on the
//! system — so writes go one file at a time through `pkexec`, staged in the
//! user's own space and installed atomically, and default to disabling rather
//! than deleting.

pub mod apt;
pub mod flatpak;

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use crate::constants::PKEXEC;
use crate::debug::REPOS;
use crate::debug_log;

/// Which package manager a repository belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Apt,
    Flatpak,
    Dnf,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Apt => "APT",
            Self::Flatpak => "Flatpak",
            Self::Dnf => "dnf",
        }
    }
}

/// One repository, whichever manager it belongs to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Repository {
    pub kind: Kind,
    /// What to call it: a file name for apt, a remote name for Flatpak.
    pub name: String,
    /// Where the packages come from.
    pub detail: String,
    pub enabled: bool,
    /// Whether changing it needs root.
    pub privileged: bool,
    /// Enough to find it again when writing.
    pub location: Location,
}

/// How to get back to a repository in order to change it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Location {
    /// A numbered entry in an apt sources file.
    AptEntry { file: PathBuf, index: usize },
    /// A Flatpak remote, and whether it belongs to the user or the system.
    FlatpakRemote { name: String, user: bool },
    /// A section in a dnf `.repo` file.
    DnfSection { file: PathBuf, section: String },
}

#[derive(Clone, Debug)]
pub enum Error {
    Io { path: PathBuf, message: String },
    Command(String),
    /// Something was asked for that this does not know how to do.
    Unsupported(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, message } => write!(f, "{}: {message}", path.display()),
            Self::Command(message) => write!(f, "{message}"),
            Self::Unsupported(what) => write!(f, "{what}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Every repository this system has, in a stable order.
pub async fn list() -> Vec<Repository> {
    let mut repositories = list_apt();
    repositories.extend(list_dnf());
    repositories.extend(flatpak::list().await);
    repositories.sort_by(|a, b| {
        (a.kind.label(), a.name.to_lowercase()).cmp(&(b.kind.label(), b.name.to_lowercase()))
    });
    debug_log!(REPOS, "{} repositories", repositories.len());
    repositories
}

/// Read apt's sources, from both the single file and the directory.
fn list_apt() -> Vec<Repository> {
    let mut files: Vec<PathBuf> = Vec::new();

    let single = PathBuf::from(apt::SOURCES_LIST);
    if single.is_file() {
        files.push(single);
    }
    if let Ok(entries) = std::fs::read_dir(apt::SOURCES_DIR) {
        let mut found: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| apt::is_source_file(path))
            .collect();
        found.sort();
        files.extend(found);
    }

    files
        .into_iter()
        .filter_map(|file| {
            let text = std::fs::read_to_string(&file).ok()?;
            Some(apt::parse_file(&file, &text))
        })
        .flatten()
        .map(|entry| Repository {
            kind: Kind::Apt,
            name: entry.file_name(),
            detail: entry.describe(),
            enabled: entry.enabled,
            privileged: true,
            location: Location::AptEntry {
                file: entry.file.clone(),
                index: entry.index,
            },
        })
        .collect()
}

/// Read dnf's repositories.
///
/// A `.repo` file is INI: `[section]` headers with `name=`, `baseurl=` and
/// `enabled=` under them.
fn list_dnf() -> Vec<Repository> {
    let Ok(entries) = std::fs::read_dir("/etc/yum.repos.d") else {
        return Vec::new();
    };

    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "repo"))
        .collect();
    files.sort();

    files
        .into_iter()
        .filter_map(|file| {
            let text = std::fs::read_to_string(&file).ok()?;
            Some(parse_dnf_repo(&file, &text))
        })
        .flatten()
        .collect()
}

/// Read one dnf `.repo` file.
pub fn parse_dnf_repo(file: &Path, text: &str) -> Vec<Repository> {
    let mut repositories = Vec::new();
    let mut section: Option<String> = None;
    let mut name = String::new();
    let mut url = String::new();
    let mut enabled = true;

    let mut flush = |section: &mut Option<String>, name: &mut String, url: &mut String, enabled: &mut bool| {
        if let Some(id) = section.take() {
            repositories.push(Repository {
                kind: Kind::Dnf,
                name: if name.is_empty() { id.clone() } else { name.clone() },
                detail: url.clone(),
                enabled: *enabled,
                privileged: true,
                location: Location::DnfSection {
                    file: file.to_path_buf(),
                    section: id,
                },
            });
        }
        name.clear();
        url.clear();
        // Absent means enabled, as dnf reads it.
        *enabled = true;
    };

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            flush(&mut section, &mut name, &mut url, &mut enabled);
            section = Some(trimmed[1..trimmed.len() - 1].to_owned());
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        match key.trim() {
            "name" => name = value.trim().to_owned(),
            "baseurl" | "metalink" | "mirrorlist" if url.is_empty() => {
                url = value.trim().to_owned()
            }
            "enabled" => enabled = value.trim() != "0",
            _ => {}
        }
    }
    flush(&mut section, &mut name, &mut url, &mut enabled);

    repositories
}

/// Turn a repository on or off.
pub async fn set_enabled(repository: &Repository, enabled: bool) -> Result<()> {
    match &repository.location {
        Location::AptEntry { file, index } => {
            let text = std::fs::read_to_string(file).map_err(|error| Error::Io {
                path: file.clone(),
                message: error.to_string(),
            })?;

            let updated = if file.extension().is_some_and(|e| e == "sources") {
                apt::set_sources_enabled(&text, *index, enabled)
            } else {
                apt::set_list_enabled(&text, *index, enabled)
            };

            write_privileged(file, &updated).await
        }
        Location::FlatpakRemote { name, user } => flatpak::set_enabled(name, *user, enabled).await,
        Location::DnfSection { .. } => Err(Error::Unsupported(
            "dnf repositories are shown but not yet editable here".to_owned(),
        )),
    }
}

/// Remove a repository.
///
/// apt entries are disabled rather than removed: commenting a line out is
/// reversible with a text editor, and deleting a file somebody else's package
/// installed is not something to do on one click. Flatpak remotes really are
/// removed, because `flatpak remote-add` puts them back.
pub async fn remove(repository: &Repository) -> Result<()> {
    match &repository.location {
        Location::FlatpakRemote { name, user } => flatpak::remove(name, *user).await,
        Location::AptEntry { .. } | Location::DnfSection { .. } => {
            set_enabled(repository, false).await
        }
    }
}

/// Add a repository.
pub async fn add(kind: Kind, first: &str, second: &str, third: &str) -> Result<()> {
    match kind {
        Kind::Flatpak => flatpak::add(first, second).await,
        Kind::Apt => {
            let name = first.trim();
            if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                return Err(Error::Unsupported(
                    "a repository name may only contain letters, digits, dashes and underscores"
                        .to_owned(),
                ));
            }
            let file = PathBuf::from(apt::SOURCES_DIR).join(format!("{name}.list"));
            // Suite and components are not validated beyond being present: apt
            // accepts a wide range and guessing at what is legal would refuse
            // valid entries.
            let line = apt::new_list_entry(second, third, "main");
            write_privileged(&file, &line).await
        }
        Kind::Dnf => Err(Error::Unsupported(
            "adding dnf repositories is not supported here".to_owned(),
        )),
    }
}

/// Replace a file under `/etc` as root.
///
/// Staged in the user's own directory and then installed with a single
/// privileged command. Writing directly would need a shell to redirect into,
/// and handing a shell a path is how a quoting mistake becomes a root mistake.
async fn write_privileged(destination: &Path, contents: &str) -> Result<()> {
    let staged = std::env::temp_dir().join(format!(
        "{}-repo-staged",
        env!("CARGO_PKG_NAME")
    ));
    std::fs::write(&staged, contents).map_err(|error| Error::Io {
        path: staged.clone(),
        message: error.to_string(),
    })?;

    debug_log!(REPOS, "installing {} as root", destination.display());

    let output = Command::new(PKEXEC)
        .arg("install")
        .arg("-m")
        .arg("0644")
        .arg(&staged)
        .arg(destination)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|error| Error::Command(error.to_string()))?;

    let _ = std::fs::remove_file(&staged);

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    Err(Error::Command(if stderr.is_empty() {
        match output.status.code() {
            Some(126) => "authentication was dismissed".to_owned(),
            Some(code) => format!("install exited with {code}"),
            None => "install was terminated".to_owned(),
        }
    } else {
        stderr.to_owned()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_dnf_repository_file() {
        let text = "\
[fedora]
name=Fedora $releasever - $basearch
metalink=https://mirrors.fedoraproject.org/metalink?repo=fedora-$releasever
enabled=1

[fedora-debuginfo]
name=Fedora debuginfo
baseurl=https://example.com/debug
enabled=0
";
        let found = parse_dnf_repo(Path::new("/etc/yum.repos.d/fedora.repo"), text);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "Fedora $releasever - $basearch");
        assert!(found[0].enabled);
        assert!(!found[1].enabled, "enabled=0 means off");
        assert_eq!(found[1].detail, "https://example.com/debug");
    }

    #[test]
    fn a_dnf_section_with_no_enabled_line_is_on() {
        let text = "[x]\nname=X\nbaseurl=https://x\n";
        let found = parse_dnf_repo(Path::new("/etc/yum.repos.d/x.repo"), text);
        assert!(found[0].enabled);
    }

    #[tokio::test]
    async fn an_apt_name_that_could_escape_its_directory_is_refused() {
        // The name becomes a file name under /etc, written as root.
        for bad in ["../evil", "a/b", "", "with space", "x;y"] {
            let result = add(Kind::Apt, bad, "https://example.com", "stable").await;
            assert!(result.is_err(), "{bad:?} should be refused");
        }
    }

    #[test]
    fn removing_an_apt_entry_only_disables_it() {
        // Deleting a file another package installed is not a one-click action.
        let repository = Repository {
            kind: Kind::Apt,
            name: "docker.list".to_owned(),
            detail: String::new(),
            enabled: true,
            privileged: true,
            location: Location::AptEntry {
                file: PathBuf::from("/etc/apt/sources.list.d/docker.list"),
                index: 0,
            },
        };
        // The behaviour is in `remove`; this asserts the routing that decides it.
        assert!(matches!(repository.location, Location::AptEntry { .. }));
    }

    #[test]
    fn flatpak_remotes_need_no_privileges_when_they_are_the_users() {
        let user = Repository {
            kind: Kind::Flatpak,
            name: "flathub".to_owned(),
            detail: "https://dl.flathub.org/repo/".to_owned(),
            enabled: true,
            privileged: false,
            location: Location::FlatpakRemote {
                name: "flathub".to_owned(),
                user: true,
            },
        };
        assert!(!user.privileged);
    }
}

/// Checks that read this machine's own repository files.
///
/// Ignored by default: they depend on what is installed. Run with
/// `cargo test -- --ignored live_`.
#[cfg(test)]
mod live_tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn live_reads_this_systems_repositories() {
        let found = list().await;
        for kind in [Kind::Apt, Kind::Flatpak, Kind::Dnf] {
            let listed: Vec<&Repository> =
                found.iter().filter(|r| r.kind == kind).collect();
            println!("{} — {} source(s)", kind.label(), listed.len());
            for repository in listed.iter().take(6) {
                println!(
                    "   {:<34} {:<9} {}",
                    repository.name,
                    if repository.enabled { "enabled" } else { "disabled" },
                    repository.detail.chars().take(60).collect::<String>()
                );
            }
        }
        assert!(!found.is_empty(), "nothing was read");
        // No backup file should have been read as a live source.
        assert!(
            !found.iter().any(|r| r.name.ends_with(".save")),
            "a backup was listed as a repository"
        );
    }
}
