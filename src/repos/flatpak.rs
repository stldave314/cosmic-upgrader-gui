// SPDX-License-Identifier: GPL-3.0

//! Flatpak remotes.
//!
//! The pleasant case: `flatpak` has a proper command-line interface for this,
//! remotes belonging to the user need no privileges at all, and adding one back
//! after removing it is a single command — so unlike apt, removing really can
//! mean removing.
//!
//! System-wide remotes still need root, and `flatpak` asks for it through polkit
//! itself, so nothing here has to.

use std::process::Stdio;

use tokio::process::Command;

use super::{Error, Kind, Location, Repository, Result};
use crate::debug::REPOS;
use crate::debug_log;

/// Read the remotes `flatpak` knows about.
///
/// `--columns` gives a stable tab-separated output rather than the aligned
/// table the bare command prints, which is meant for people.
pub async fn list() -> Vec<Repository> {
    let output = Command::new("flatpak")
        .args(["remotes", "--columns=name,title,url,options"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await;

    let Ok(output) = output else {
        debug_log!(REPOS, "flatpak not available");
        return Vec::new();
    };

    parse_remotes(&String::from_utf8_lossy(&output.stdout))
}

/// Read `flatpak remotes --columns=name,title,url,options`.
///
/// The same remote can appear twice — once for the user and once for the system
/// — and they are genuinely different things: removing one leaves the other. So
/// the scope is part of what identifies a remote here, not a detail.
pub fn parse_remotes(text: &str) -> Vec<Repository> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let name = fields.next()?.trim();
            if name.is_empty() {
                return None;
            }
            let title = fields.next().unwrap_or_default().trim();
            let url = fields.next().unwrap_or_default().trim();
            let options = fields.next().unwrap_or_default();

            let user = options.split(',').any(|option| option.trim() == "user");
            // `flatpak` marks a disabled remote in its options rather than
            // omitting it.
            let enabled = !options.split(',').any(|option| option.trim() == "disabled");

            let scope = if user { "user" } else { "system" };
            Some(Repository {
                kind: Kind::Flatpak,
                name: format!("{name} ({scope})"),
                detail: if title.is_empty() || title == name {
                    url.to_owned()
                } else {
                    format!("{title} — {url}")
                },
                enabled,
                // A user remote is the user's own; only a system one needs root,
                // and flatpak asks for that itself.
                privileged: !user,
                location: Location::FlatpakRemote {
                    name: name.to_owned(),
                    user,
                },
            })
        })
        .collect()
}

fn scope(user: bool) -> &'static str {
    if user {
        "--user"
    } else {
        "--system"
    }
}

async fn run(args: &[&str]) -> Result<()> {
    debug_log!(REPOS, "flatpak {}", args.join(" "));

    let output = Command::new("flatpak")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|error| Error::Command(error.to_string()))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    Err(Error::Command(if stderr.is_empty() {
        format!("flatpak {} failed", args.join(" "))
    } else {
        stderr.to_owned()
    }))
}

/// Turn a remote on or off without removing it.
pub async fn set_enabled(name: &str, user: bool, enabled: bool) -> Result<()> {
    let flag = if enabled { "--enable" } else { "--disable" };
    run(&["remote-modify", scope(user), flag, name]).await
}

/// Remove a remote.
pub async fn remove(name: &str, user: bool) -> Result<()> {
    // `--force` so a remote that still has applications installed from it can
    // be removed; without it flatpak refuses and the button does nothing
    // visible.
    run(&["remote-delete", scope(user), "--force", name]).await
}

/// Add a remote from a URL.
///
/// Added for the user rather than the system, so it needs no password. A
/// `.flatpakrepo` URL carries the remote's own name, title and signing key,
/// which is why that is the form asked for.
pub async fn add(name: &str, url: &str) -> Result<()> {
    let name = name.trim();
    let url = url.trim();
    if name.is_empty() || url.is_empty() {
        return Err(Error::Unsupported(
            "a remote needs a name and a URL".to_owned(),
        ));
    }
    // Rejected rather than passed through: a name with a slash or a leading
    // dash would be read by flatpak as a path or an option.
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        || name.starts_with('-')
    {
        return Err(Error::Unsupported(
            "a remote name may only contain letters, digits, dashes, dots and underscores"
                .to_owned(),
        ));
    }

    run(&[
        "remote-add",
        "--user",
        "--if-not-exists",
        name,
        url,
    ])
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Captured from `flatpak remotes --columns=name,title,url,options` on the
    /// machine this was written on, where the same remote exists twice.
    const REMOTES: &str = "flathub\tFlathub\thttps://dl.flathub.org/repo/\tsystem\n\
appcenter\tAppCenter\thttps://flatpak.elementary.io/repo\tuser\n\
cosmic\tCOSMIC\thttps://apt.pop-os.org/cosmic/\tuser\n\
fedora\tFedora\toci+https://registry.fedoraproject.org\tuser,oci\n\
flathub\tFlathub\thttps://dl.flathub.org/repo/\tuser\n";

    #[test]
    fn reads_every_remote() {
        assert_eq!(parse_remotes(REMOTES).len(), 5);
    }

    #[test]
    fn the_same_remote_in_both_scopes_is_two_entries() {
        // Removing the user one leaves the system one; they are not the same
        // thing and must not collapse into one row.
        let found = parse_remotes(REMOTES);
        let flathubs: Vec<&Repository> = found
            .iter()
            .filter(|remote| remote.name.starts_with("flathub"))
            .collect();
        assert_eq!(flathubs.len(), 2);
        assert!(flathubs.iter().any(|remote| remote.name.contains("system")));
        assert!(flathubs.iter().any(|remote| remote.name.contains("user")));
    }

    #[test]
    fn only_system_remotes_need_privileges() {
        let found = parse_remotes(REMOTES);
        let user = found.iter().find(|r| r.name == "appcenter (user)").expect("appcenter");
        let system = found.iter().find(|r| r.name == "flathub (system)").expect("flathub");
        assert!(!user.privileged);
        assert!(system.privileged);
    }

    #[test]
    fn extra_options_do_not_confuse_the_scope() {
        // `user,oci` is still a user remote.
        let found = parse_remotes(REMOTES);
        let fedora = found.iter().find(|r| r.name.starts_with("fedora")).expect("fedora");
        assert!(!fedora.privileged);
        assert!(fedora.name.contains("user"));
    }

    #[test]
    fn a_disabled_remote_is_reported_as_such() {
        let found = parse_remotes("x\tX\thttps://x\tuser,disabled\n");
        assert!(!found[0].enabled);
    }

    #[test]
    fn a_remote_whose_title_repeats_its_name_shows_only_the_url() {
        let found = parse_remotes("flathub\tflathub\thttps://dl.flathub.org/repo/\tuser\n");
        assert_eq!(found[0].detail, "https://dl.flathub.org/repo/");
    }

    #[tokio::test]
    async fn a_remote_name_that_flatpak_would_misread_is_refused() {
        for bad in ["", "--delete-everything", "a/b", "a b"] {
            assert!(add(bad, "https://example.com/x.flatpakrepo").await.is_err(), "{bad:?}");
        }
    }

    #[tokio::test]
    async fn a_remote_needs_a_url() {
        assert!(add("name", "  ").await.is_err());
    }
}
