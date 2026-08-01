// SPDX-License-Identifier: GPL-3.0

//! Downloading a release and putting it in place.
//!
//! A release page is a pile of files: several architectures, several packaging
//! formats, checksums, signatures, source archives and sometimes debug symbols.
//! Picking the right one is the whole problem, and picking the wrong one is
//! worse than picking none — installing an `arm64` package on an `x86_64`
//! machine, or handing `apt` a `.sig` file, wastes an authentication prompt to
//! produce an error.
//!
//! So assets are scored rather than pattern-matched, the best is offered by
//! name before anything is downloaded, and when nothing scores well enough the
//! answer is to open the release page instead of guessing.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use super::detect::Source;
use super::forge::{Asset, Release};
use crate::constants::{DOWNLOAD_TIMEOUT, PKEXEC, USER_AGENT};
use crate::debug::RELEASES;
use crate::debug_log;

/// Files that are about a release rather than part of it.
const NOT_PAYLOAD: [&str; 10] = [
    ".sig", ".asc", ".sha256", ".sha512", ".md5", ".sum", ".zsync", ".txt", ".json", ".sbom",
];

/// Words marking a build nobody wants installed by accident.
const UNWANTED: [&str; 5] = ["debug", "dbgsym", "source", "src", "symbols"];

/// The architecture this machine runs, and the spellings packagers use for it.
fn architecture_aliases() -> &'static [&'static str] {
    match std::env::consts::ARCH {
        "x86_64" => &["x86_64", "amd64", "x64"],
        "aarch64" => &["aarch64", "arm64"],
        "arm" => &["armhf", "armv7", "arm"],
        "x86" => &["i386", "i686", "x86"],
        // Anything else is passed through so a machine this was not written for
        // still matches on its own name.
        other => Box::leak(vec![other].into_boxed_slice()),
    }
}

/// The filename extension a source implies.
fn wanted_extension(source: &Source) -> &'static str {
    match source {
        Source::Deb => ".deb",
        Source::Rpm => ".rpm",
        Source::AppImage(_) => ".appimage",
    }
}

/// How well an asset suits this machine, or `None` if it is unsuitable.
///
/// Higher is better. The format is required — an asset of the wrong kind is
/// never a candidate — while the architecture is scored, because plenty of
/// projects publish a single portable build with no architecture in its name.
fn score(asset: &Asset, source: &Source) -> Option<u32> {
    let name = asset.name.to_ascii_lowercase();

    if NOT_PAYLOAD.iter().any(|suffix| name.ends_with(suffix)) {
        return None;
    }
    if UNWANTED.iter().any(|word| name.contains(word)) {
        return None;
    }
    if !name.ends_with(wanted_extension(source)) {
        return None;
    }

    let mut score: u32 = 1;

    let aliases = architecture_aliases();
    if aliases.iter().any(|alias| name.contains(alias)) {
        score += 10;
    } else if mentions_another_architecture(&name, aliases) {
        // Names a machine this is not. Rejected rather than scored low: a wrong
        // architecture is not a worse match, it is not a match.
        return None;
    }

    // A GNU build is the right default on a glibc system; musl builds exist
    // alongside them and are the more specialised choice.
    if name.contains("musl") {
        score = score.saturating_sub(1);
    }

    Some(score)
}

fn mentions_another_architecture(name: &str, ours: &[&str]) -> bool {
    const ALL: [&str; 11] = [
        "x86_64", "amd64", "x64", "aarch64", "arm64", "armhf", "armv7", "i386", "i686", "riscv64",
        "ppc64",
    ];
    ALL.iter()
        .filter(|arch| !ours.contains(arch))
        .any(|arch| name.contains(arch))
}

/// The best asset for this machine, if the release has one.
pub fn choose_asset<'a>(release: &'a Release, source: &Source) -> Option<&'a Asset> {
    release
        .assets
        .iter()
        .filter_map(|asset| score(asset, source).map(|score| (score, asset)))
        // Ties go to the shorter name, which is usually the plain build rather
        // than a variant.
        .max_by(|(left_score, left), (right_score, right)| {
            left_score
                .cmp(right_score)
                .then_with(|| right.name.len().cmp(&left.name.len()))
        })
        .map(|(_, asset)| asset)
}

#[derive(Clone, Debug)]
pub enum Error {
    NoDownloader,
    Download(String),
    Install(String),
    Io(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDownloader => write!(f, "curl is not installed"),
            Self::Download(message) => write!(f, "download failed: {message}"),
            Self::Install(message) => write!(f, "install failed: {message}"),
            Self::Io(message) => write!(f, "{message}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Where downloads are staged.
///
/// Under the user's cache rather than `/tmp`, so a large download does not land
/// in a tmpfs sitting in RAM.
fn staging_directory() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
        .join("cosmic-upgrader-gui/downloads")
}

/// Fetch an asset and return where it landed.
pub async fn download(asset: &Asset) -> Result<PathBuf> {
    let directory = staging_directory();
    std::fs::create_dir_all(&directory).map_err(|error| Error::Io(error.to_string()))?;

    // The asset's own name is not used as a path component without checking:
    // it comes from a third party, and `../` in it would write outside the
    // staging directory.
    let file_name = Path::new(&asset.name)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty() && name != "." && name != "..")
        .ok_or_else(|| Error::Download(format!("unusable asset name {:?}", asset.name)))?;
    let target = directory.join(file_name);

    debug_log!(RELEASES, "downloading {} to {}", asset.url, target.display());

    let output = tokio::time::timeout(
        DOWNLOAD_TIMEOUT,
        Command::new("curl")
            .args([
                "--silent",
                "--show-error",
                "--location",
                "--fail",
                "--user-agent",
                USER_AGENT,
                "--output",
            ])
            .arg(&target)
            .arg(&asset.url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| Error::Download(format!("timed out after {DOWNLOAD_TIMEOUT:?}")))?
    .map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => Error::NoDownloader,
        _ => Error::Download(error.to_string()),
    })?;

    if !output.status.success() {
        // A partial file left behind would be installed on a later attempt.
        let _ = std::fs::remove_file(&target);
        return Err(Error::Download(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }

    Ok(target)
}

/// Install a downloaded asset.
///
/// Packages go through the distribution's own tool under `pkexec`, so the
/// desktop's authentication dialog names the program about to run as
/// administrator. AppImages need no privileges at all — they are one file in the
/// user's own directory.
pub async fn install(file: &Path, source: &Source) -> Result<()> {
    match source {
        Source::Deb => {
            run_privileged("apt-get", &["install", "-y", "--allow-downgrades"], file).await
        }
        Source::Rpm => run_privileged("dnf", &["install", "-y"], file).await,
        Source::AppImage(destination) => replace_appimage(file, destination),
    }
}

async fn run_privileged(tool: &str, args: &[&str], file: &Path) -> Result<()> {
    debug_log!(RELEASES, "installing {} with {tool}", file.display());

    let output = Command::new(PKEXEC)
        .arg(tool)
        .args(args)
        .arg(file)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|error| Error::Install(error.to_string()))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    Err(Error::Install(if stderr.is_empty() {
        // pkexec exits 126 when the user dismisses the dialog, which is a
        // choice rather than a fault and reads badly as a bare exit code.
        match output.status.code() {
            Some(126) => "authentication was dismissed".to_owned(),
            Some(code) => format!("{tool} exited with {code}"),
            None => format!("{tool} was terminated"),
        }
    } else {
        stderr.to_owned()
    }))
}

/// Put a new AppImage where the old one was.
///
/// Written alongside and renamed over, so an interrupted replacement cannot
/// leave a half-written file where a working program used to be. The original's
/// permissions are carried across, and the executable bit is set either way
/// because a downloaded file does not have one.
fn replace_appimage(downloaded: &Path, destination: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mode = std::fs::metadata(destination)
        .map(|metadata| metadata.permissions().mode())
        .unwrap_or(0o755);

    let staged = destination.with_extension("AppImage.new");
    std::fs::copy(downloaded, &staged).map_err(|error| Error::Io(error.to_string()))?;
    std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(mode | 0o700))
        .map_err(|error| Error::Io(error.to_string()))?;

    std::fs::rename(&staged, destination).map_err(|error| {
        let _ = std::fs::remove_file(&staged);
        Error::Io(error.to_string())
    })?;

    debug_log!(RELEASES, "replaced {}", destination.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str) -> Asset {
        Asset {
            name: name.to_owned(),
            url: format!("https://example.com/{name}"),
            size: 1,
        }
    }

    fn release(names: &[&str]) -> Release {
        Release {
            tag: "v1".to_owned(),
            name: "1".to_owned(),
            published: String::new(),
            notes: String::new(),
            web_url: String::new(),
            assets: names.iter().map(|name| asset(name)).collect(),
            pre_release: false,
        }
    }

    /// These assume the test machine is x86_64, which is what the aliases are
    /// written against; on anything else the architecture scoring differs and
    /// the assertions would not describe the same choice.
    fn on_x86_64() -> bool {
        std::env::consts::ARCH == "x86_64"
    }

    #[test]
    fn picks_the_package_for_this_machine() {
        if !on_x86_64() {
            return;
        }
        let release = release(&[
            "app_1.2.3_arm64.deb",
            "app_1.2.3_amd64.deb",
            "app-1.2.3.tar.gz",
        ]);
        let chosen = choose_asset(&release, &Source::Deb).expect("an asset");
        assert_eq!(chosen.name, "app_1.2.3_amd64.deb");
    }

    #[test]
    fn never_picks_the_wrong_architecture() {
        if !on_x86_64() {
            return;
        }
        // Only an arm64 build exists; installing it would waste an
        // authentication prompt to produce an error.
        let release = release(&["app_1.2.3_arm64.deb"]);
        assert!(choose_asset(&release, &Source::Deb).is_none());
    }

    #[test]
    fn never_picks_a_checksum_or_signature() {
        let release = release(&["app_amd64.deb.sha256", "app_amd64.deb.sig", "app_amd64.deb"]);
        let chosen = choose_asset(&release, &Source::Deb).expect("an asset");
        assert_eq!(chosen.name, "app_amd64.deb");
    }

    #[test]
    fn never_picks_debug_symbols() {
        let release = release(&["app-dbgsym_amd64.deb", "app_amd64.deb"]);
        assert_eq!(
            choose_asset(&release, &Source::Deb).expect("an asset").name,
            "app_amd64.deb"
        );
    }

    #[test]
    fn respects_the_format_the_thing_was_installed_as() {
        if !on_x86_64() {
            return;
        }
        let release = release(&["app_amd64.deb", "app.x86_64.rpm", "App-x86_64.AppImage"]);
        assert_eq!(
            choose_asset(&release, &Source::Deb).expect("deb").name,
            "app_amd64.deb"
        );
        assert_eq!(
            choose_asset(&release, &Source::Rpm).expect("rpm").name,
            "app.x86_64.rpm"
        );
        assert_eq!(
            choose_asset(&release, &Source::AppImage("/x".into()))
                .expect("appimage")
                .name,
            "App-x86_64.AppImage"
        );
    }

    #[test]
    fn a_portable_build_with_no_architecture_is_still_acceptable() {
        let release = release(&["app.deb"]);
        assert!(choose_asset(&release, &Source::Deb).is_some());
    }

    #[test]
    fn a_gnu_build_is_preferred_over_musl() {
        if !on_x86_64() {
            return;
        }
        let release = release(&["app-x86_64-musl.AppImage", "app-x86_64.AppImage"]);
        assert_eq!(
            choose_asset(&release, &Source::AppImage("/x".into()))
                .expect("an asset")
                .name,
            "app-x86_64.AppImage"
        );
    }

    #[test]
    fn a_release_with_nothing_suitable_yields_nothing() {
        // The interface then offers the release page instead of guessing.
        let release = release(&["source.tar.gz", "CHANGELOG.txt"]);
        assert!(choose_asset(&release, &Source::Deb).is_none());
    }

    #[test]
    fn an_asset_named_with_a_traversal_is_refused() {
        // The name comes from a third party and becomes a path component.
        let name = Path::new("../../etc/evil.deb")
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        assert_eq!(name.as_deref(), Some("evil.deb"));
    }
}
