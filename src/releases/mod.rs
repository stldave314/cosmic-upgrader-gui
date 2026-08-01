// SPDX-License-Identifier: GPL-3.0

//! Watching projects for releases newer than what is installed.
//!
//! topgrade covers everything with a package manager behind it. What it cannot
//! cover is software installed by downloading a `.deb`, an `.rpm` or an AppImage
//! from a project's releases page — there is no manager holding a repository to
//! ask. Those are exactly the things that quietly fall years behind.
//!
//! So this asks the projects directly. [`detect`] proposes candidates from what
//! the packages already say about themselves, the user confirms which to watch,
//! and [`forge`] asks each project's own host for its releases.
//!
//! ## No HTTP client
//!
//! Requests go through `curl`, and through `gh` for GitHub when it is installed.
//! Two reasons, in order of importance:
//!
//! * **Rate limits.** GitHub allows 60 unauthenticated requests an hour, which a
//!   watch list of any size exhausts immediately. `gh` carries the user's own
//!   credentials and gets 5000. Reimplementing its token discovery would be
//!   worse than calling it.
//! * **Weight.** An async HTTP stack with TLS is around a hundred crates, for an
//!   application whose entire design is driving external tools. `curl` is
//!   already a dependency of the desktop it runs on.
//!
//! Both are detected rather than assumed, and their absence is reported as such.

pub mod detect;
pub mod forge;
pub mod install;
pub mod version;

use std::collections::HashMap;
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::constants::{RELEASE_CHECK_TIMEOUT, USER_AGENT};
use crate::debug::RELEASES;
use crate::debug_log;
use forge::{Kind, Release, Repo};

/// How often releases may be checked without being asked.
///
/// Forges are other people's servers, and a watch list of a few hundred
/// projects polled on every launch is impolite at best and rate-limited at
/// worst. This caps the automatic check; the button on the page is a deliberate
/// act and is never blocked by it.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum CheckInterval {
    /// Only ever when asked.
    Manual,
    SixHourly,
    #[default]
    Daily,
    Weekly,
}

impl CheckInterval {
    pub const ALL: [Self; 4] = [Self::Manual, Self::SixHourly, Self::Daily, Self::Weekly];

    /// Seconds between automatic checks, or `None` for never.
    pub fn seconds(self) -> Option<i64> {
        match self {
            Self::Manual => None,
            Self::SixHourly => Some(6 * 60 * 60),
            Self::Daily => Some(24 * 60 * 60),
            Self::Weekly => Some(7 * 24 * 60 * 60),
        }
    }

    /// Whether an automatic check is due.
    ///
    /// A `last` of zero means none has ever run, which *is* due: the first
    /// launch after adding projects should say something about them rather than
    /// waiting a day to do it.
    pub fn is_due(self, last: i64, now: i64) -> bool {
        match self.seconds() {
            None => false,
            Some(_) if last == 0 => true,
            Some(interval) => now.saturating_sub(last) >= interval,
        }
    }

    /// When the next automatic check falls due.
    pub fn next_due(self, last: i64) -> Option<i64> {
        self.seconds().map(|interval| last.saturating_add(interval))
    }
}

/// Which releases count.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Channel {
    /// Finished releases only.
    #[default]
    Stable,
    /// Also release candidates and betas — anything the project has published.
    IncludePreRelease,
}

impl Channel {
    pub const ALL: [Self; 2] = [Self::Stable, Self::IncludePreRelease];

    /// Whether a release should be offered on this channel.
    ///
    /// Both signals are read: the forge's own flag, and the tag. A project that
    /// tags `v2.0.0-rc1` without ticking the box on its release page is still
    /// publishing a release candidate, and somebody who asked for stable
    /// versions should not be shown one.
    pub fn accepts(self, release: &Release) -> bool {
        match self {
            Self::IncludePreRelease => true,
            Self::Stable => !release.pre_release && !version::is_pre_release(&release.tag),
        }
    }
}

/// A project the user has chosen to watch.
///
/// Stored rather than re-detected each time, because the watch list is a
/// decision the user made and re-deriving it would silently drop anything they
/// added by hand and re-add anything they rejected.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Watch {
    pub name: String,
    /// `github`, `gitlab` or `gitea`.
    pub kind: String,
    pub host: String,
    pub path: String,
    /// The version recorded when the watch was created or last updated.
    pub installed: String,
    /// `deb`, `rpm` or a path to an AppImage.
    pub source: String,
    /// The newest tag seen the last time this was checked, so the page can say
    /// something after a restart without going back to the network.
    #[serde(default)]
    pub latest_tag: String,
    /// When it was last checked, in seconds since the Unix epoch.
    #[serde(default)]
    pub checked: i64,
}

impl Watch {
    pub fn repo(&self) -> Repo {
        Repo {
            kind: match self.kind.as_str() {
                "gitlab" => Kind::GitLab,
                "gitea" => Kind::Gitea,
                _ => Kind::GitHub,
            },
            host: self.host.clone(),
            path: self.path.clone(),
        }
    }

    pub fn from_candidate(candidate: &detect::Candidate) -> Option<Self> {
        let repo = candidate.repo.clone()?;
        Some(Self {
            name: candidate.name.clone(),
            kind: match repo.kind {
                Kind::GitHub => "github",
                Kind::GitLab => "gitlab",
                Kind::Gitea => "gitea",
            }
            .to_owned(),
            host: repo.host,
            path: repo.path,
            installed: candidate.version.clone(),
            source: match &candidate.source {
                detect::Source::Deb => "deb".to_owned(),
                detect::Source::Rpm => "rpm".to_owned(),
                detect::Source::AppImage(path) => path.display().to_string(),
            },
            latest_tag: String::new(),
            checked: 0,
        })
    }

    pub fn source(&self) -> detect::Source {
        match self.source.as_str() {
            "deb" => detect::Source::Deb,
            "rpm" => detect::Source::Rpm,
            path => detect::Source::AppImage(std::path::PathBuf::from(path)),
        }
    }
}

/// What checking one watched project found.
#[derive(Clone, Debug)]
pub struct Status {
    pub watch: Watch,
    /// The newest release that is not a pre-release, where there was one.
    pub latest: Option<Release>,
    pub comparison: version::Ordering,
    /// Why the check failed, if it did.
    pub error: Option<String>,
}

impl Status {
    /// Whether this is worth drawing attention to.
    pub fn is_update(&self) -> bool {
        self.latest.is_some() && self.comparison == version::Ordering::Newer
    }
}

/// How requests are made.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transport {
    /// `gh api`, which carries the user's GitHub credentials and so is not
    /// limited to 60 requests an hour.
    GitHubCli,
    Curl,
}

/// What is available for making requests on this machine.
#[derive(Clone, Copy, Debug, Default)]
pub struct Client {
    pub have_gh: bool,
    pub have_curl: bool,
}

impl Client {
    pub async fn detect() -> Self {
        let client = Self {
            have_gh: which("gh").await,
            have_curl: which("curl").await,
        };
        debug_log!(
            RELEASES,
            "transports: gh={} curl={}",
            client.have_gh,
            client.have_curl
        );
        client
    }

    /// Whether anything at all can be fetched.
    pub fn is_usable(&self) -> bool {
        self.have_curl || self.have_gh
    }

    fn transport_for(&self, repo: &Repo) -> Option<Transport> {
        // `gh` only speaks to GitHub, but for GitHub it is much the better
        // choice — an unauthenticated watch list of more than sixty projects
        // cannot be checked in an hour otherwise.
        if repo.kind == Kind::GitHub && repo.host.ends_with("github.com") && self.have_gh {
            return Some(Transport::GitHubCli);
        }
        self.have_curl.then_some(Transport::Curl)
    }

    /// Fetch a URL, returning its body.
    async fn fetch(&self, repo: &Repo, url: &str) -> Result<String, String> {
        let Some(transport) = self.transport_for(repo) else {
            return Err("neither curl nor gh is installed".to_owned());
        };

        let mut command = match transport {
            Transport::GitHubCli => {
                let mut command = Command::new("gh");
                // The path after the host is what `gh api` takes; it adds the
                // host and the credentials itself.
                let path = url
                    .split_once("api.github.com/")
                    .map(|(_, rest)| rest)
                    .unwrap_or(url);
                command.args(["api", path]);
                command
            }
            Transport::Curl => {
                let mut command = Command::new("curl");
                command.args([
                    "--silent",
                    "--location",
                    "--fail",
                    "--max-time",
                    &RELEASE_CHECK_TIMEOUT.as_secs().to_string(),
                    "--user-agent",
                    USER_AGENT,
                    url,
                ]);
                command
            }
        };

        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let output = tokio::time::timeout(RELEASE_CHECK_TIMEOUT, command.output())
            .await
            .map_err(|_| format!("timed out after {RELEASE_CHECK_TIMEOUT:?}"))?
            .map_err(|error| error.to_string())?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr = stderr.trim();
            return Err(if stderr.is_empty() {
                format!("request failed ({})", output.status)
            } else {
                stderr.to_owned()
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Ask a project's host for its releases.
    ///
    /// A host whose software was recognised by name is asked once. One that was
    /// not is asked in each shape until one answers, and the shape that worked
    /// is returned so the caller can remember it — which is what lets this work
    /// against a self-hosted instance it has never seen.
    pub async fn releases(&self, repo: &Repo) -> Result<(Kind, Vec<Release>), String> {
        let mut last = String::new();

        for kind in std::iter::once(repo.kind).chain(
            Kind::PROBE_ORDER
                .into_iter()
                .filter(|probe| *probe != repo.kind),
        ) {
            let attempt = repo.as_kind(kind);
            match self.fetch(&attempt, &attempt.releases_url()).await {
                Ok(body) => match forge::parse_releases(kind, &body, &attempt) {
                    Ok(releases) => {
                        debug_log!(
                            RELEASES,
                            "{} answered as {} with {} release(s)",
                            repo.display(),
                            kind.label(),
                            releases.len()
                        );
                        return Ok((kind, releases));
                    }
                    Err(error) => last = error,
                },
                Err(error) => last = error,
            }
        }

        Err(if last.is_empty() {
            "no response".to_owned()
        } else {
            last
        })
    }

    /// Check one watched project.
    pub async fn check(&self, watch: &Watch, channel: Channel) -> Status {
        let repo = watch.repo();
        match self.releases(&repo).await {
            Ok((_, releases)) => {
                // A project whose only releases are pre-releases reports as
                // having none on the stable channel, which is the honest answer
                // rather than quietly offering one anyway.
                let latest = releases.into_iter().find(|release| channel.accepts(release));
                let comparison = latest
                    .as_ref()
                    .map(|release| version::compare(&watch.installed, &release.tag))
                    .unwrap_or(version::Ordering::Unknown);

                Status {
                    watch: watch.clone(),
                    latest,
                    comparison,
                    error: None,
                }
            }
            Err(error) => {
                debug_log!(RELEASES, "{}: {error}", repo.display());
                Status {
                    watch: watch.clone(),
                    latest: None,
                    comparison: version::Ordering::Unknown,
                    error: Some(error),
                }
            }
        }
    }
}

/// This application's own project, as something to watch.
///
/// Synthesized from what is compiled in rather than discovered, so the check
/// works however this was installed — from a package, from a downloaded
/// archive, or built from source, where there is no file for detection to find
/// and nothing in any package database to read.
pub fn self_watch() -> Option<Watch> {
    let repo = Repo::from_url(crate::constants::REPOSITORY_URL)?;
    Some(Watch {
        name: env!("CARGO_PKG_NAME").to_owned(),
        kind: match repo.kind {
            Kind::GitHub => "github",
            Kind::GitLab => "gitlab",
            Kind::Gitea => "gitea",
        }
        .to_owned(),
        host: repo.host,
        path: repo.path,
        installed: env!("CARGO_PKG_VERSION").to_owned(),
        source: "deb".to_owned(),
        latest_tag: String::new(),
        checked: 0,
    })
}

/// The key identifying this application's own watch, so the interface can pin
/// it and refuse to remove it.
pub fn self_key() -> Option<String> {
    self_watch().map(|watch| format!("{}/{}", watch.host, watch.path))
}

/// Propose everything on this machine that has a project behind it.
///
/// Deduplicated by project, because several packages routinely share one — a
/// library and its `-dev` package point at the same repository, and offering
/// both would be offering the same update twice.
pub async fn discover(appimage_dirs: &[String]) -> Vec<detect::Candidate> {
    let mut candidates = Vec::new();

    match tokio::fs::read_to_string("/var/lib/dpkg/status").await {
        Ok(text) => {
            let named = detect::parse_dpkg_status(&text);
            // Everything a repository offers is already covered by the package
            // manager, and topgrade drives that. Only what nothing will update
            // is worth tracking here.
            let unmanaged = apt_unmanaged(&named).await;
            let before = named.len();
            candidates.extend(
                named
                    .into_iter()
                    .filter(|candidate| unmanaged.contains(&candidate.name)),
            );
            debug_log!(
                RELEASES,
                "{before} packages name a forge, {} are not offered by any repository",
                candidates.len()
            );
        }
        Err(error) => debug_log!(RELEASES, "no dpkg database: {error}"),
    }

    if which("rpm").await {
        let unmanaged = dnf_unmanaged().await;
        let output = Command::new("rpm")
            .args(["-qa", "--qf", "%{NAME}\\t%{VERSION}\\t%{URL}\\n"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .output()
            .await;
        if let Ok(output) = output {
            candidates.extend(
                detect::parse_rpm_output(&String::from_utf8_lossy(&output.stdout))
                    .into_iter()
                    // As with apt: a package a repository offers is already
                    // covered, and only what nothing will update belongs here.
                    .filter(|candidate| {
                        unmanaged
                            .as_ref()
                            .is_none_or(|known| known.contains(&candidate.name))
                    }),
            );
        }
    }

    candidates.extend(detect::find_appimages(appimage_dirs));

    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut unique: Vec<detect::Candidate> = Vec::new();
    for candidate in candidates {
        // An AppImage with no project is keyed by its path, so several
        // unidentified ones do not collapse into a single entry.
        let key = match &candidate.repo {
            Some(repo) => format!("{}/{}", repo.host, repo.path),
            None => format!("?{}", candidate.name),
        };
        match seen.get(&key) {
            // Prefer the shortest package name, which is almost always the
            // actual program rather than one of its satellites.
            Some(&index) if unique[index].name.len() <= candidate.name.len() => {}
            Some(&index) => unique[index] = candidate,
            None => {
                seen.insert(key, unique.len());
                unique.push(candidate);
            }
        }
    }

    unique.sort_by(|a, b| a.name.cmp(&b.name));
    debug_log!(RELEASES, "{} candidate project(s)", unique.len());
    unique
}

/// Ask apt which of these packages no repository offers.
///
/// Only the packages that got this far are asked about — a few hundred rather
/// than every installed package — which keeps this to well under a second.
async fn apt_unmanaged(candidates: &[detect::Candidate]) -> std::collections::HashSet<String> {
    let names: Vec<&str> = candidates
        .iter()
        .map(|candidate| candidate.name.as_str())
        .collect();

    if names.is_empty() || !which("apt-cache").await {
        // Without apt there is nothing to ask, so nothing is excluded: a system
        // with a different package manager should not silently lose its list.
        return candidates
            .iter()
            .map(|candidate| candidate.name.clone())
            .collect();
    }

    let output = Command::new("apt-cache")
        .arg("policy")
        .args(&names)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await;

    match output {
        Ok(output) => detect::parse_apt_policy(&String::from_utf8_lossy(&output.stdout)),
        Err(error) => {
            debug_log!(RELEASES, "apt-cache failed: {error}");
            candidates
                .iter()
                .map(|candidate| candidate.name.clone())
                .collect()
        }
    }
}

/// Ask dnf which installed packages did not come from a repository.
///
/// `None` when dnf cannot be asked, which means nothing is excluded rather
/// than everything.
async fn dnf_unmanaged() -> Option<std::collections::HashSet<String>> {
    if !which("dnf").await {
        return None;
    }
    let output = Command::new("dnf")
        .args([
            "repoquery",
            "--installed",
            "--queryformat",
            "%{name} %{from_repo}",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await
        .ok()?;
    Some(detect::parse_dnf_installed(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

async fn which(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|directory| directory.join(name).is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(name: &str, host: &str, path: &str) -> detect::Candidate {
        detect::Candidate {
            name: name.to_owned(),
            version: "1.0".to_owned(),
            repo: Repo::from_url(&format!("https://{host}/{path}")),
            source: detect::Source::Deb,
        }
    }

    fn release(tag: &str, flagged: bool) -> Release {
        Release {
            tag: tag.to_owned(),
            name: tag.to_owned(),
            published: String::new(),
            notes: String::new(),
            web_url: String::new(),
            assets: Vec::new(),
            pre_release: flagged,
        }
    }

    #[test]
    fn the_stable_channel_declines_a_flagged_pre_release() {
        assert!(!Channel::Stable.accepts(&release("v2.0.0", true)));
    }

    #[test]
    fn the_stable_channel_declines_a_pre_release_the_forge_did_not_flag() {
        // The case that matters: plenty of projects tag `-rc1` and never tick
        // the box on the release page.
        assert!(!Channel::Stable.accepts(&release("v2.0.0-rc1", false)));
        assert!(!Channel::Stable.accepts(&release("v2.0.0-beta", false)));
    }

    #[test]
    fn the_stable_channel_accepts_a_finished_release() {
        assert!(Channel::Stable.accepts(&release("v2.0.0", false)));
        // A Debian-style revision is not a pre-release.
        assert!(Channel::Stable.accepts(&release("1.2.3-2ubuntu0.1", false)));
    }

    #[test]
    fn the_pre_release_channel_accepts_everything() {
        assert!(Channel::IncludePreRelease.accepts(&release("v2.0.0-rc1", true)));
        assert!(Channel::IncludePreRelease.accepts(&release("v2.0.0", false)));
    }

    #[test]
    fn this_application_watches_itself() {
        // Synthesized, so it works however this was installed — including from
        // source, where nothing on disk would identify it.
        let watch = self_watch().expect("the compiled-in repository should parse");
        assert_eq!(watch.installed, env!("CARGO_PKG_VERSION"));
        assert_eq!(watch.repo().kind, Kind::GitHub);
        assert!(watch.path.ends_with("cosmic-upgrader-gui"), "{}", watch.path);
        assert_eq!(self_key().as_deref(), Some(format!("{}/{}", watch.host, watch.path).as_str()));
    }

    #[test]
    fn a_check_is_due_the_first_time() {
        // Otherwise adding projects would say nothing about them for a day.
        assert!(CheckInterval::Daily.is_due(0, 1_000_000));
    }

    #[test]
    fn a_check_is_not_due_again_until_the_interval_has_passed() {
        let day = 24 * 60 * 60;
        assert!(!CheckInterval::Daily.is_due(1_000_000, 1_000_000 + day - 1));
        assert!(CheckInterval::Daily.is_due(1_000_000, 1_000_000 + day));
    }

    #[test]
    fn manual_never_falls_due() {
        assert!(!CheckInterval::Manual.is_due(0, i64::MAX));
        assert_eq!(CheckInterval::Manual.next_due(0), None);
    }

    #[test]
    fn a_clock_that_went_backwards_does_not_make_a_check_due() {
        assert!(!CheckInterval::Daily.is_due(2_000_000, 1_000_000));
    }

    #[test]
    fn the_intervals_are_ordered_as_named() {
        assert!(CheckInterval::SixHourly.seconds() < CheckInterval::Daily.seconds());
        assert!(CheckInterval::Daily.seconds() < CheckInterval::Weekly.seconds());
    }

    #[test]
    fn a_watch_round_trips_through_its_stored_form() {
        let watch = Watch::from_candidate(&candidate("kwin", "invent.kde.org", "plasma/kwin"))
            .expect("watch");
        let repo = watch.repo();
        assert_eq!(repo.kind, Kind::GitLab);
        assert_eq!(repo.host, "invent.kde.org");
        assert_eq!(repo.path, "plasma/kwin");
    }

    #[test]
    fn a_candidate_with_no_project_cannot_be_watched_yet() {
        let mut unidentified = candidate("mystery", "example.com", "a/b");
        unidentified.repo = None;
        assert!(Watch::from_candidate(&unidentified).is_none());
    }

    #[test]
    fn an_appimage_watch_remembers_which_file_to_replace() {
        let candidate = detect::Candidate {
            name: "qFlipper".to_owned(),
            version: "1.3.3".to_owned(),
            repo: Repo::from_url("https://github.com/Flipper-Devices/qFlipper"),
            source: detect::Source::AppImage("/home/x/qFlipper.AppImage".into()),
        };
        let watch = Watch::from_candidate(&candidate).expect("watch");
        assert_eq!(
            watch.source(),
            detect::Source::AppImage("/home/x/qFlipper.AppImage".into())
        );
    }

    #[test]
    fn an_update_is_only_reported_when_the_release_is_newer() {
        let watch = Watch::from_candidate(&candidate("a", "github.com", "a/b")).expect("watch");
        let release = Release {
            tag: "v2.0".to_owned(),
            name: "2.0".to_owned(),
            published: String::new(),
            notes: String::new(),
            web_url: String::new(),
            assets: Vec::new(),
            pre_release: false,
        };

        let newer = Status {
            watch: watch.clone(),
            latest: Some(release.clone()),
            comparison: version::Ordering::Newer,
            error: None,
        };
        assert!(newer.is_update());

        let same = Status {
            watch: watch.clone(),
            latest: Some(release),
            comparison: version::Ordering::Same,
            error: None,
        };
        assert!(!same.is_update());

        // An unreadable comparison is not an update: the interface says a
        // release exists rather than claiming it is newer.
        let unknown = Status {
            watch,
            latest: None,
            comparison: version::Ordering::Unknown,
            error: None,
        };
        assert!(!unknown.is_update());
    }

    #[tokio::test]
    async fn discovery_offers_one_entry_per_project() {
        // A library and its -dev package share a repository; both would be the
        // same update.
        let candidates = vec![
            candidate("libfoo-dev", "github.com", "foo/foo"),
            candidate("libfoo", "github.com", "foo/foo"),
            candidate("bar", "github.com", "bar/bar"),
        ];

        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut unique: Vec<detect::Candidate> = Vec::new();
        for candidate in candidates {
            let key = match &candidate.repo {
                Some(repo) => format!("{}/{}", repo.host, repo.path),
                None => format!("?{}", candidate.name),
            };
            match seen.get(&key) {
                Some(&index) if unique[index].name.len() <= candidate.name.len() => {}
                Some(&index) => unique[index] = candidate,
                None => {
                    seen.insert(key, unique.len());
                    unique.push(candidate);
                }
            }
        }

        assert_eq!(unique.len(), 2);
        // The shorter name is the program rather than its satellite package.
        assert!(unique.iter().any(|c| c.name == "libfoo"));
        assert!(!unique.iter().any(|c| c.name == "libfoo-dev"));
    }

    #[test]
    fn github_prefers_the_cli_when_it_is_available() {
        let both = Client {
            have_gh: true,
            have_curl: true,
        };
        let github = Repo::from_url("https://github.com/a/b").expect("repo");
        assert_eq!(both.transport_for(&github), Some(Transport::GitHubCli));

        // But the CLI cannot speak to anything else.
        let gitlab = Repo::from_url("https://invent.kde.org/a/b").expect("repo");
        assert_eq!(both.transport_for(&gitlab), Some(Transport::Curl));
    }

    #[test]
    fn with_no_transport_nothing_can_be_checked() {
        let none = Client::default();
        assert!(!none.is_usable());
        let repo = Repo::from_url("https://github.com/a/b").expect("repo");
        assert_eq!(none.transport_for(&repo), None);
    }
}

/// Checks that reach the network and this machine's own package database.
///
/// Ignored by default: they need connectivity and take a few seconds.
/// Run with `cargo test -- --ignored live_`.
#[cfg(test)]
mod live_tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn live_discovers_projects_from_this_machine() {
        let directories: Vec<String> = crate::constants::APPIMAGE_SEARCH_DIRS
            .iter()
            .map(|d| (*d).to_owned())
            .collect();
        let found = discover(&directories).await;
        println!("{} candidates", found.len());
        let forges: HashMap<String, usize> =
            found.iter().filter_map(|c| c.repo.as_ref()).fold(
                HashMap::new(),
                |mut counts, repo| {
                    *counts.entry(format!("{:?} {}", repo.kind, repo.host)).or_insert(0) += 1;
                    counts
                },
            );
        let mut summary: Vec<_> = forges.into_iter().collect();
        summary.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        for (host, count) in summary.iter().take(8) {
            println!("  {count:>4}  {host}");
        }
        for candidate in found.iter().filter(|c| c.repo.is_none()).take(5) {
            println!("  unidentified: {} {}", candidate.name, candidate.version);
        }
        assert!(!found.is_empty(), "nothing was discovered");
    }

    #[tokio::test]
    #[ignore]
    async fn live_reads_releases_from_every_forge_kind() {
        let client = Client::detect().await;
        assert!(client.is_usable(), "no transport available");

        for url in [
            "https://github.com/topgrade-rs/topgrade",
            "https://gitlab.com/gitlab-org/gitlab-runner",
            "https://codeberg.org/forgejo/forgejo",
        ] {
            let repo = Repo::from_url(url).expect("repo");
            match client.releases(&repo).await {
                Ok((kind, releases)) => {
                    let newest = releases.first().expect("at least one release");
                    println!(
                        "{:<12} {:<14} {} ({} asset(s))",
                        kind.label(),
                        newest.tag,
                        newest.published,
                        newest.assets.len()
                    );
                }
                Err(error) => panic!("{url}: {error}"),
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn live_probes_a_host_whose_software_is_not_known_by_name() {
        // Told it is GitHub, which it is not: the probe has to find GitLab.
        let client = Client::detect().await;
        let mut repo = Repo::from_url("https://gitlab.com/gitlab-org/gitlab-runner").expect("repo");
        repo.kind = forge::Kind::GitHub;
        let (kind, releases) = client.releases(&repo).await.expect("probe should recover");
        println!("recovered as {} with {} release(s)", kind.label(), releases.len());
        assert_eq!(kind, forge::Kind::GitLab);
    }
}
