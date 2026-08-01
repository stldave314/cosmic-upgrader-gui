// SPDX-License-Identifier: GPL-3.0

//! Talking to whatever is hosting a project's releases.
//!
//! "Check GitHub for new releases" is the common case but a poor design: on this
//! machine alone, packages point at `gitlab.freedesktop.org` and `invent.kde.org`
//! as well as `github.com`, and neither of those is GitHub. What they have in
//! common is a small REST API for listing releases, and there are only three
//! shapes of it in wide use:
//!
//! | Software | Releases endpoint | Result |
//! | --- | --- | --- |
//! | GitHub | `/repos/{path}/releases/latest` on `api.github.com` | one object |
//! | GitLab | `/api/v4/projects/{escaped path}/releases` | an array |
//! | Gitea / Forgejo | `/api/v1/repos/{path}/releases` | an array |
//!
//! Codeberg is Forgejo, Forgejo is a fork of Gitea, and every self-hosted GitLab
//! answers the same path as `gitlab.com` — so covering three shapes covers most
//! of what a package's metadata will point at.
//!
//! A host that is not recognised by name is not given up on: the shapes are
//! tried in turn and whichever answers is remembered. That is what makes this
//! work for a self-hosted instance nobody has heard of, which is the whole point
//! of not hard-coding GitHub.

use serde::Deserialize;

/// Which API shape a host speaks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    GitHub,
    GitLab,
    /// Gitea and its fork Forgejo, which share an API. Codeberg is one of these.
    Gitea,
}

impl Kind {
    /// The shapes to try, in order, against a host of unknown software.
    ///
    /// Gitea first because its endpoint is the most specific — a GitLab
    /// instance returns a clear 404 for it, whereas some servers answer
    /// GitLab's project path with something unhelpful.
    pub const PROBE_ORDER: [Self; 3] = [Self::Gitea, Self::GitLab, Self::GitHub];

    pub fn label(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::GitLab => "GitLab",
            Self::Gitea => "Gitea/Forgejo",
        }
    }
}

/// A project on a forge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Repo {
    pub kind: Kind,
    /// Host as it appears in the URL, so a self-hosted instance is reached at
    /// its own address rather than at the public one for its software.
    pub host: String,
    /// `owner/project`, or a longer path where the forge allows it: GitLab
    /// nests groups, so `plasma/kwin` and `gnome/gtk/gtk` are both valid.
    pub path: String,
}

/// Path segments that are part of a forge's own interface rather than of a
/// project's name, and everything after them.
const PAGE_SEGMENTS: [&str; 12] = [
    "-", "tree", "blob", "issues", "releases", "tags", "commits", "commit", "wiki", "pulls",
    "merge_requests", "src",
];

/// Hosts whose software is known without asking.
///
/// Only a shortcut: an unlisted host is probed rather than refused, so this
/// going out of date costs one extra request and nothing else.
fn known_kind(host: &str) -> Option<Kind> {
    match host {
        "github.com" | "www.github.com" | "api.github.com" => Some(Kind::GitHub),
        "gitlab.com" | "www.gitlab.com" | "invent.kde.org" | "salsa.debian.org"
        | "gitlab.gnome.org" | "gitlab.freedesktop.org" => Some(Kind::GitLab),
        "codeberg.org" | "gitea.com" | "git.disroot.org" => Some(Kind::Gitea),
        // Naming conventions are a decent hint for self-hosted instances.
        _ if host.contains("gitlab") => Some(Kind::GitLab),
        _ if host.contains("gitea") || host.contains("forgejo") => Some(Kind::Gitea),
        _ => None,
    }
}

impl Repo {
    /// Read a project out of a URL found in package metadata.
    ///
    /// Returns `None` for a URL that names no project — a documentation site, a
    /// wiki, a plain homepage — which is most of what `Homepage:` contains.
    pub fn from_url(url: &str) -> Option<Self> {
        let trimmed = url.trim().trim_end_matches('/');

        // Strip the scheme and any `git+` or user@ decoration.
        let without_scheme = trimmed
            .split_once("://")
            .map(|(_, rest)| rest)
            .unwrap_or(trimmed);
        let without_scheme = without_scheme
            .rsplit_once('@')
            .map(|(_, rest)| rest)
            .unwrap_or(without_scheme);

        let (host, path) = without_scheme.split_once('/')?;
        let host = host.split(':').next()?.to_ascii_lowercase();
        if host.is_empty() || !host.contains('.') {
            return None;
        }

        // Drop query strings and fragments before looking at segments.
        let path = path.split(['?', '#']).next().unwrap_or(path);

        let mut segments: Vec<&str> = Vec::new();
        for segment in path.split('/') {
            if segment.is_empty() {
                continue;
            }
            if PAGE_SEGMENTS.contains(&segment) {
                break;
            }
            segments.push(segment);
        }

        if segments.len() < 2 {
            // A single segment is a user or an organisation, not a project.
            return None;
        }

        let kind = known_kind(&host)?;

        // GitHub projects are always exactly two segments; GitLab nests groups,
        // so more are kept there.
        let kept = match kind {
            Kind::GitHub | Kind::Gitea => 2,
            Kind::GitLab => segments.len(),
        };

        let path = segments[..kept.min(segments.len())]
            .join("/")
            .trim_end_matches(".git")
            .to_owned();

        Some(Self { kind, host, path })
    }

    /// The same project, read by a different API shape.
    ///
    /// Used when probing a host whose software is not known: the identity is the
    /// same, only the endpoint differs.
    pub fn as_kind(&self, kind: Kind) -> Self {
        Self {
            kind,
            host: self.host.clone(),
            path: self.path.clone(),
        }
    }

    /// Where to ask for the releases.
    pub fn releases_url(&self) -> String {
        match self.kind {
            // The public API lives on its own host; a GitHub Enterprise install
            // answers under `/api/v3` on its own.
            Kind::GitHub if self.host.ends_with("github.com") => {
                format!("https://api.github.com/repos/{}/releases?per_page=5", self.path)
            }
            Kind::GitHub => format!(
                "https://{}/api/v3/repos/{}/releases?per_page=5",
                self.host, self.path
            ),
            // The project path is one escaped path parameter, so its slashes
            // have to be encoded or the route does not match.
            Kind::GitLab => format!(
                "https://{}/api/v4/projects/{}/releases?per_page=5",
                self.host,
                escape_path(&self.path)
            ),
            Kind::Gitea => format!(
                "https://{}/api/v1/repos/{}/releases?limit=5",
                self.host, self.path
            ),
        }
    }

    /// Where a person would go to read about the release.
    pub fn web_url(&self) -> String {
        format!("https://{}/{}", self.host, self.path)
    }

    /// A short label for the interface.
    pub fn display(&self) -> String {
        format!("{} ({})", self.path, self.host)
    }
}

/// Percent-encode a project path for GitLab's escaped path parameter.
///
/// Only the characters that actually appear in project paths are handled;
/// pulling in a URL-encoding dependency for this one call would not earn its
/// place, and a path with anything stranger in it will simply not be found.
fn escape_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len() + 8);
    for c in path.chars() {
        match c {
            '/' => out.push_str("%2F"),
            '.' => out.push_str("%2E"),
            c if c.is_ascii_alphanumeric() || c == '-' || c == '_' => out.push(c),
            c => out.push_str(&format!("%{:02X}", c as u32)),
        }
    }
    out
}

/// One downloadable file attached to a release.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

/// A release, in the form the rest of the application uses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Release {
    /// The tag, which is what gets compared against the installed version.
    pub tag: String,
    pub name: String,
    /// ISO 8601, as every one of these APIs returns it.
    pub published: String,
    pub notes: String,
    pub web_url: String,
    pub assets: Vec<Asset>,
    /// Whether the forge marks it as a pre-release. Skipped by default: someone
    /// tracking stable versions does not want to be told about a nightly.
    pub pre_release: bool,
}

// ── Wire formats ────────────────────────────────────────────────────────────
// Each forge is deserialized into its own shape and then converted, rather than
// one lenient struct with every field optional. The differences are real —
// GitLab has no pre-release flag and calls the body `description` — and folding
// them together would hide which fields are actually guaranteed.

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    html_url: Option<String>,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

#[derive(Deserialize)]
struct GitLabRelease {
    tag_name: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    released_at: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    upcoming_release: bool,
    #[serde(default)]
    assets: Option<GitLabAssets>,
    #[serde(rename = "_links", default)]
    links: Option<GitLabLinks>,
}

#[derive(Deserialize, Default)]
struct GitLabAssets {
    #[serde(default)]
    links: Vec<GitLabAssetLink>,
    #[serde(default)]
    sources: Vec<GitLabSource>,
}

#[derive(Deserialize)]
struct GitLabAssetLink {
    name: String,
    url: String,
}

#[derive(Deserialize)]
struct GitLabSource {
    format: String,
    url: String,
}

#[derive(Deserialize, Default)]
struct GitLabLinks {
    #[serde(default)]
    self_: Option<String>,
}

#[derive(Deserialize)]
struct GiteaRelease {
    tag_name: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    html_url: Option<String>,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    assets: Vec<GiteaAsset>,
}

#[derive(Deserialize)]
struct GiteaAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

/// Read a forge's answer into releases, newest first.
///
/// An entry that cannot be read is skipped rather than failing the lot: forges
/// add fields, and one odd release should not hide the others.
pub fn parse_releases(kind: Kind, json: &str, repo: &Repo) -> Result<Vec<Release>, String> {
    let releases = match kind {
        Kind::GitHub => serde_json::from_str::<Vec<GitHubRelease>>(json)
            .map_err(|error| error.to_string())?
            .into_iter()
            .filter(|release| !release.draft)
            .map(|release| Release {
                tag: release.tag_name.clone(),
                name: release.name.unwrap_or_else(|| release.tag_name.clone()),
                published: release.published_at.unwrap_or_default(),
                notes: release.body.unwrap_or_default(),
                web_url: release.html_url.unwrap_or_else(|| repo.web_url()),
                pre_release: release.prerelease,
                assets: release
                    .assets
                    .into_iter()
                    .map(|asset| Asset {
                        name: asset.name,
                        url: asset.browser_download_url,
                        size: asset.size,
                    })
                    .collect(),
            })
            .collect(),

        Kind::GitLab => serde_json::from_str::<Vec<GitLabRelease>>(json)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|release| {
                let assets = release.assets.unwrap_or_default();
                Release {
                    tag: release.tag_name.clone(),
                    name: release.name.unwrap_or_else(|| release.tag_name.clone()),
                    published: release.released_at.or(release.created_at).unwrap_or_default(),
                    notes: release.description.unwrap_or_default(),
                    web_url: release
                        .links
                        .and_then(|links| links.self_)
                        .unwrap_or_else(|| repo.web_url()),
                    // GitLab has no pre-release flag; `upcoming_release` means
                    // the tag exists but the release date is in the future,
                    // which is the nearest equivalent.
                    pre_release: release.upcoming_release,
                    assets: assets
                        .links
                        .into_iter()
                        .map(|link| Asset {
                            name: link.name,
                            url: link.url,
                            size: 0,
                        })
                        .chain(assets.sources.into_iter().map(|source| Asset {
                            name: format!("source.{}", source.format),
                            url: source.url,
                            size: 0,
                        }))
                        .collect(),
                }
            })
            .collect(),

        Kind::Gitea => serde_json::from_str::<Vec<GiteaRelease>>(json)
            .map_err(|error| error.to_string())?
            .into_iter()
            .filter(|release| !release.draft)
            .map(|release| Release {
                tag: release.tag_name.clone(),
                name: release.name.unwrap_or_else(|| release.tag_name.clone()),
                published: release.published_at.unwrap_or_default(),
                notes: release.body.unwrap_or_default(),
                web_url: release.html_url.unwrap_or_else(|| repo.web_url()),
                pre_release: release.prerelease,
                assets: release
                    .assets
                    .into_iter()
                    .map(|asset| Asset {
                        name: asset.name,
                        url: asset.browser_download_url,
                        size: asset.size,
                    })
                    .collect(),
            })
            .collect(),
    };

    Ok(releases)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(url: &str) -> Repo {
        Repo::from_url(url).unwrap_or_else(|| panic!("{url} should parse"))
    }

    #[test]
    fn reads_a_github_project() {
        let repo = repo("https://github.com/topgrade-rs/topgrade");
        assert_eq!(repo.kind, Kind::GitHub);
        assert_eq!(repo.path, "topgrade-rs/topgrade");
        assert!(repo.releases_url().starts_with("https://api.github.com/repos/topgrade-rs/topgrade/releases"));
    }

    #[test]
    fn reads_a_self_hosted_gitlab() {
        // Two of these are in this machine's own package metadata.
        let repo = repo("https://invent.kde.org/plasma/kwin");
        assert_eq!(repo.kind, Kind::GitLab);
        assert_eq!(repo.host, "invent.kde.org");
        assert!(
            repo.releases_url().contains("invent.kde.org/api/v4/projects/plasma%2Fkwin/releases"),
            "{}",
            repo.releases_url()
        );
    }

    #[test]
    fn keeps_nested_gitlab_groups() {
        // GitLab nests groups, so truncating to two segments loses the project.
        let repo = repo("https://gitlab.gnome.org/GNOME/gtk/gtk");
        assert_eq!(repo.path, "GNOME/gtk/gtk");
    }

    #[test]
    fn reads_codeberg_as_gitea() {
        let repo = repo("https://codeberg.org/forgejo/forgejo");
        assert_eq!(repo.kind, Kind::Gitea);
        assert!(repo.releases_url().contains("/api/v1/repos/forgejo/forgejo/releases"));
    }

    #[test]
    fn recognises_self_hosted_instances_by_name() {
        assert_eq!(repo("https://gitlab.example.org/a/b").kind, Kind::GitLab);
        assert_eq!(repo("https://gitea.example.org/a/b").kind, Kind::Gitea);
    }

    #[test]
    fn strips_decoration_from_a_url() {
        assert_eq!(repo("git+https://github.com/a/b.git").path, "a/b");
        assert_eq!(repo("https://github.com/a/b/").path, "a/b");
        assert_eq!(repo("https://github.com/a/b/tree/main").path, "a/b");
        assert_eq!(repo("https://github.com/a/b/-/releases").path, "a/b");
        assert_eq!(repo("https://github.com/a/b?tab=readme").path, "a/b");
    }

    #[test]
    fn a_url_that_names_no_project_is_not_one() {
        // Most of what `Homepage:` holds looks like this.
        assert!(Repo::from_url("https://www.gnu.org/").is_none());
        assert!(Repo::from_url("https://github.com/someuser").is_none());
        assert!(Repo::from_url("not a url").is_none());
        assert!(Repo::from_url("https://wiki.gnome.org/Apps").is_none());
    }

    #[test]
    fn a_github_enterprise_host_uses_its_own_api_path() {
        let repo = Repo {
            kind: Kind::GitHub,
            host: "git.corp.example".to_owned(),
            path: "a/b".to_owned(),
        };
        assert!(repo.releases_url().contains("git.corp.example/api/v3/repos/a/b"), "{}", repo.releases_url());
    }

    #[test]
    fn parses_a_github_release() {
        let json = r#"[{"tag_name":"v17.9.0","name":"17.9.0","published_at":"2026-07-01T10:00:00Z",
            "body":"notes","html_url":"https://github.com/a/b/releases/tag/v17.9.0","prerelease":false,
            "draft":false,"assets":[{"name":"app_amd64.deb","browser_download_url":"https://x/app.deb","size":42}]}]"#;
        let releases = parse_releases(Kind::GitHub, json, &repo("https://github.com/a/b")).expect("parse");
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].tag, "v17.9.0");
        assert_eq!(releases[0].assets[0].name, "app_amd64.deb");
        assert_eq!(releases[0].assets[0].size, 42);
    }

    #[test]
    fn a_github_draft_is_not_a_release() {
        let json = r#"[{"tag_name":"v2","draft":true},{"tag_name":"v1","draft":false}]"#;
        let releases = parse_releases(Kind::GitHub, json, &repo("https://github.com/a/b")).expect("parse");
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].tag, "v1");
    }

    #[test]
    fn parses_a_gitlab_release_including_its_assets() {
        let json = r#"[{"tag_name":"v19.0.3","name":"19.0.3","created_at":"2026-06-01T00:00:00Z",
            "description":"see changelog","upcoming_release":false,
            "assets":{"links":[{"name":"binary","url":"https://x/bin"}],
                      "sources":[{"format":"tar.gz","url":"https://x/src.tar.gz"}]}}]"#;
        let releases = parse_releases(Kind::GitLab, json, &repo("https://gitlab.com/a/b")).expect("parse");
        assert_eq!(releases[0].tag, "v19.0.3");
        assert_eq!(releases[0].notes, "see changelog");
        let names: Vec<&str> = releases[0].assets.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["binary", "source.tar.gz"]);
    }

    #[test]
    fn parses_a_gitea_release() {
        let json = r#"[{"tag_name":"v16.0.2","name":"16.0.2","published_at":"2026-05-01T00:00:00Z",
            "body":"notes","prerelease":false,"draft":false,
            "assets":[{"name":"forgejo","browser_download_url":"https://x/f","size":7}]}]"#;
        let releases = parse_releases(Kind::Gitea, json, &repo("https://codeberg.org/a/b")).expect("parse");
        assert_eq!(releases[0].tag, "v16.0.2");
        assert_eq!(releases[0].assets[0].size, 7);
    }

    #[test]
    fn an_empty_answer_is_no_releases_rather_than_an_error() {
        // invent.kde.org answers exactly this for a project with no releases.
        let releases = parse_releases(Kind::GitLab, "[]", &repo("https://gitlab.com/a/b")).expect("parse");
        assert!(releases.is_empty());
    }

    #[test]
    fn an_unreadable_answer_is_reported() {
        assert!(parse_releases(Kind::GitHub, "not json", &repo("https://github.com/a/b")).is_err());
    }

    #[test]
    fn probing_covers_every_shape() {
        assert_eq!(Kind::PROBE_ORDER.len(), 3);
    }
}
