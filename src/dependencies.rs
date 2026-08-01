// SPDX-License-Identifier: GPL-3.0

//! What this application needs, and what it merely benefits from.
//!
//! Almost everything here is done by driving another program: topgrade for the
//! upgrades themselves, `curl` and `gh` for release checks, `pkexec` for
//! anything privileged, `notify-send` for reporting a scheduled run. That is a
//! deliberate design, but it has a cost — a missing tool turns into a feature
//! that quietly does nothing, and the user has no way to know which tool or
//! why.
//!
//! So the list is explicit and checked. Each entry says what it is for, whether
//! it is required or optional, and what stops working without it. Required
//! means the application cannot do its job at all; optional means one feature
//! is unavailable and everything else is fine. Nothing is installed without
//! being asked for.

use std::process::Stdio;

use tokio::process::Command;

use crate::debug::DEPS;
use crate::debug_log;
use crate::fl;

/// How badly a dependency is wanted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Requirement {
    /// Without it the application cannot do the thing it exists to do.
    Required,
    /// Without it one feature is unavailable; everything else works.
    Optional,
}

/// One thing this application drives.
#[derive(Clone, Debug)]
pub struct Dependency {
    /// The executable looked for on `PATH`.
    pub binary: &'static str,
    pub requirement: Requirement,
    /// Package names by package manager, for offering to install it.
    pub apt: &'static str,
    pub dnf: &'static str,
    pub pacman: &'static str,
}

impl Dependency {
    /// What it is for, and what is lost without it.
    pub fn purpose(&self) -> String {
        match self.binary {
            "topgrade" => fl!("dep-topgrade"),
            "curl" => fl!("dep-curl"),
            "gh" => fl!("dep-gh"),
            "pkexec" => fl!("dep-pkexec"),
            "notify-send" => fl!("dep-notify-send"),
            "systemctl" => fl!("dep-systemctl"),
            "xdg-open" => fl!("dep-xdg-open"),
            _ => String::new(),
        }
    }

    /// The package to install for the detected package manager.
    fn package(&self, manager: Manager) -> &'static str {
        match manager {
            Manager::Apt => self.apt,
            Manager::Dnf => self.dnf,
            Manager::Pacman => self.pacman,
        }
    }
}

/// Everything this application will look for.
///
/// Ordered required-first, since that is the order somebody reading the list
/// cares about.
pub const ALL: [Dependency; 7] = [
    Dependency {
        binary: "topgrade",
        requirement: Requirement::Required,
        apt: "topgrade",
        dnf: "topgrade",
        pacman: "topgrade",
    },
    Dependency {
        // Required rather than optional: without it the release page cannot
        // reach any forge, and that is half of what this application does.
        binary: "curl",
        requirement: Requirement::Required,
        apt: "curl",
        dnf: "curl",
        pacman: "curl",
    },
    Dependency {
        binary: "pkexec",
        requirement: Requirement::Optional,
        apt: "policykit-1",
        dnf: "polkit",
        pacman: "polkit",
    },
    Dependency {
        binary: "gh",
        requirement: Requirement::Optional,
        apt: "gh",
        dnf: "gh",
        pacman: "github-cli",
    },
    Dependency {
        binary: "notify-send",
        requirement: Requirement::Optional,
        apt: "libnotify-bin",
        dnf: "libnotify",
        pacman: "libnotify",
    },
    Dependency {
        binary: "systemctl",
        requirement: Requirement::Optional,
        apt: "systemd",
        dnf: "systemd",
        pacman: "systemd",
    },
    Dependency {
        binary: "xdg-open",
        requirement: Requirement::Optional,
        apt: "xdg-utils",
        dnf: "xdg-utils",
        pacman: "xdg-utils",
    },
];

/// What was found for one dependency.
#[derive(Clone, Debug)]
pub struct Report {
    pub dependency: Dependency,
    pub installed: bool,
    /// Where it was found, for the interface to show.
    pub path: Option<String>,
}

impl Report {
    /// Whether this is something the user should act on.
    pub fn is_problem(&self) -> bool {
        !self.installed && self.dependency.requirement == Requirement::Required
    }
}

/// The package manager this system installs with.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Manager {
    Apt,
    Dnf,
    Pacman,
}

impl Manager {
    /// Detected from which tool is present, rather than by reading
    /// `/etc/os-release` — a derivative distribution reports its own name but
    /// installs with its parent's tool, and it is the tool that matters here.
    pub fn detect() -> Option<Self> {
        for (binary, manager) in [
            ("apt-get", Self::Apt),
            ("dnf", Self::Dnf),
            ("pacman", Self::Pacman),
        ] {
            if which(binary).is_some() {
                return Some(manager);
            }
        }
        None
    }

    /// The command that installs a package without asking.
    fn install_command(self, package: &str) -> Vec<String> {
        match self {
            Self::Apt => vec![
                "apt-get".into(),
                "install".into(),
                "-y".into(),
                package.into(),
            ],
            Self::Dnf => vec!["dnf".into(), "install".into(), "-y".into(), package.into()],
            Self::Pacman => vec![
                "pacman".into(),
                "-S".into(),
                "--noconfirm".into(),
                package.into(),
            ],
        }
    }
}

/// Look for an executable on `PATH`.
fn which(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .map(|found| found.display().to_string())
}

/// Check everything, in the order [`ALL`] lists it.
pub fn check() -> Vec<Report> {
    let reports: Vec<Report> = ALL
        .iter()
        .map(|dependency| {
            let path = which(dependency.binary);
            Report {
                dependency: dependency.clone(),
                installed: path.is_some(),
                path,
            }
        })
        .collect();

    debug_log!(
        DEPS,
        "{} of {} present, {} required missing",
        reports.iter().filter(|report| report.installed).count(),
        reports.len(),
        reports.iter().filter(|report| report.is_problem()).count()
    );
    reports
}

/// Whether anything required is missing, which is what decides if the first-run
/// dialog leads with dependencies rather than with preferences.
pub fn has_missing_required(reports: &[Report]) -> bool {
    reports.iter().any(Report::is_problem)
}

#[derive(Clone, Debug)]
pub enum Error {
    NoPackageManager,
    NoPkexec,
    Failed(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPackageManager => write!(f, "no supported package manager was found"),
            Self::NoPkexec => write!(f, "pkexec is not installed"),
            Self::Failed(message) => write!(f, "{message}"),
        }
    }
}

/// Install one dependency.
///
/// Through `pkexec` so the desktop's own dialog asks, and names the package
/// manager as the program about to run as administrator. Nothing is installed
/// without the user having asked for that specific package.
pub async fn install(dependency: &Dependency) -> Result<(), Error> {
    let manager = Manager::detect().ok_or(Error::NoPackageManager)?;
    if which("pkexec").is_none() {
        return Err(Error::NoPkexec);
    }

    let package = dependency.package(manager);
    let command = manager.install_command(package);
    debug_log!(DEPS, "installing {package} with {manager:?}");

    let output = Command::new(crate::constants::PKEXEC)
        .args(&command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|error| Error::Failed(error.to_string()))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    Err(Error::Failed(if stderr.is_empty() {
        match output.status.code() {
            // pkexec's code for a dismissed dialog, which is a choice rather
            // than a fault and reads badly as a bare number.
            Some(126) => fl!("dep-authentication-dismissed"),
            Some(code) => format!("{package}: exit {code}"),
            None => format!("{package}: terminated"),
        }
    } else {
        stderr.to_owned()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topgrade_is_required() {
        let topgrade = ALL
            .iter()
            .find(|dependency| dependency.binary == "topgrade")
            .expect("topgrade should be listed");
        assert_eq!(topgrade.requirement, Requirement::Required);
    }

    #[test]
    fn required_dependencies_come_first() {
        // So the list reads in the order somebody cares about.
        let first_optional = ALL
            .iter()
            .position(|dependency| dependency.requirement == Requirement::Optional)
            .expect("some are optional");
        assert!(
            ALL[first_optional..]
                .iter()
                .all(|dependency| dependency.requirement == Requirement::Optional),
            "a required dependency is listed after an optional one"
        );
    }

    #[test]
    fn every_dependency_names_a_package_for_every_manager() {
        // A blank name would produce an install command that does nothing, or
        // worse, one that installs whatever comes next on the command line.
        for dependency in &ALL {
            for manager in [Manager::Apt, Manager::Dnf, Manager::Pacman] {
                assert!(
                    !dependency.package(manager).is_empty(),
                    "{} has no package for {manager:?}",
                    dependency.binary
                );
            }
        }
    }

    #[test]
    fn every_dependency_explains_itself() {
        for dependency in &ALL {
            assert!(
                !dependency.purpose().is_empty(),
                "{} does not say what it is for",
                dependency.binary
            );
        }
    }

    #[test]
    fn install_commands_do_not_prompt() {
        // Nobody is at a terminal to answer, so a command that asks would hang
        // behind the authentication dialog with no way to see why.
        for manager in [Manager::Apt, Manager::Dnf, Manager::Pacman] {
            let command = manager.install_command("example");
            let joined = command.join(" ");
            assert!(
                joined.contains("-y") || joined.contains("--noconfirm"),
                "{manager:?} would prompt: {joined}"
            );
            assert_eq!(command.last().map(String::as_str), Some("example"));
        }
    }

    #[test]
    fn a_missing_optional_dependency_is_not_a_problem() {
        let report = Report {
            dependency: ALL
                .iter()
                .find(|dependency| dependency.requirement == Requirement::Optional)
                .expect("some are optional")
                .clone(),
            installed: false,
            path: None,
        };
        assert!(!report.is_problem());
    }

    #[test]
    fn a_missing_required_dependency_is_a_problem() {
        let report = Report {
            dependency: ALL[0].clone(),
            installed: false,
            path: None,
        };
        assert!(report.is_problem());
        assert!(has_missing_required(&[report]));
    }

    #[test]
    fn checking_reports_on_everything_listed() {
        let reports = check();
        assert_eq!(reports.len(), ALL.len());
        // This machine has topgrade, which is how everything else was tested.
        let topgrade = reports
            .iter()
            .find(|report| report.dependency.binary == "topgrade")
            .expect("topgrade should be reported");
        assert_eq!(topgrade.installed, which("topgrade").is_some());
    }
}
