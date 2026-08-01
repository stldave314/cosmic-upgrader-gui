// SPDX-License-Identifier: GPL-3.0

//! Working out which projects on this machine have releases worth watching.
//!
//! Nothing here asks the network. It reads what the packages already say about
//! themselves and proposes a list, which the user then confirms — the confirming
//! matters, because a `Homepage:` field is a hint about where a project lives,
//! not a promise that its releases are what got installed.
//!
//! ## Only what nothing else will update
//!
//! A package that came from a distribution repository is already covered: apt
//! or dnf will update it, and topgrade drives them. Offering it here as well
//! would be noise — on the machine this was written against, 552 installed
//! packages name a forge in their metadata, and all but six of them arrive from
//! a repository.
//!
//! Those six are the point: software installed by downloading a `.deb` from a
//! releases page, which no package manager will ever update again. The test is
//! whether any remote source offers the package — `apt-cache policy` says so
//! directly — and it is what turns a list of 360 into a list of 6.
//!
//! Three sources, in descending order of how much they can be trusted:
//!
//! 1. **An AppImage's embedded update information.** Type-2 AppImages carry a
//!    `.upd_info` ELF section holding exactly the string needed to find newer
//!    builds — `gh-releases-zsync|owner|repo|latest|App-*.AppImage.zsync`. That
//!    is the archive stating where it came from, so it needs no guessing at all.
//! 2. **`Homepage:` from the package databases.** On this machine that is 2310
//!    entries, of which a third point at a forge. Reliable enough to propose.
//! 3. **The filename**, for an AppImage with no update information — which is
//!    most of them in practice. It gives a name and a version but no project, so
//!    those are offered for the user to point at a repository themselves.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::forge::Repo;
use crate::debug::RELEASES;
use crate::debug_log;

/// Where a candidate was found, which decides how an update is installed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Source {
    Deb,
    Rpm,
    /// The file to replace in place.
    AppImage(PathBuf),
}

impl Source {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Deb => "deb",
            Self::Rpm => "rpm",
            Self::AppImage(_) => "AppImage",
        }
    }
}

/// Something installed that might have newer releases.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    pub name: String,
    /// The version as installed, for comparison against a release tag.
    pub version: String,
    /// The project, where the metadata named one. `None` for something whose
    /// origin could not be worked out — offered so the user can supply it.
    pub repo: Option<Repo>,
    pub source: Source,
}

// ── Package databases ───────────────────────────────────────────────────────

/// Read `Package`/`Version`/`Homepage` out of dpkg's status file.
///
/// Parsed directly rather than by running `dpkg-query`, which on a full system
/// means one process and a megabyte of output for information already sitting in
/// a file. Only installed packages are taken: the status file also carries
/// entries for packages that were removed but left their configuration behind,
/// and offering an update for something uninstalled would be nonsense.
pub fn parse_dpkg_status(text: &str) -> Vec<Candidate> {
    let mut candidates = Vec::new();

    for stanza in text.split("\n\n") {
        let mut name = None;
        let mut version = None;
        let mut homepage = None;
        let mut installed = false;

        for line in stanza.lines() {
            if let Some(value) = line.strip_prefix("Package: ") {
                name = Some(value.trim().to_owned());
            } else if let Some(value) = line.strip_prefix("Version: ") {
                version = Some(value.trim().to_owned());
            } else if let Some(value) = line.strip_prefix("Homepage: ") {
                homepage = Some(value.trim().to_owned());
            } else if let Some(value) = line.strip_prefix("Status: ") {
                // "install ok installed" — anything else is not on the system.
                installed = value.trim().ends_with(" installed");
            }
        }

        let (Some(name), Some(version), true) = (name, version, installed) else {
            continue;
        };
        let Some(repo) = homepage.as_deref().and_then(Repo::from_url) else {
            continue;
        };

        candidates.push(Candidate {
            name,
            version,
            repo: Some(repo),
            source: Source::Deb,
        });
    }

    candidates
}

/// The packages `apt-cache policy` shows no remote source for.
///
/// Such a package was installed from a file and apt has nowhere to get a newer
/// one, which is exactly the case release tracking exists for. Everything else
/// is left to the package manager that owns it.
///
/// The output lists each package, then its version table; an origin line either
/// names a URL or names dpkg's own status file. A package with only the latter
/// is not coming from anywhere.
pub fn parse_apt_policy(output: &str) -> HashSet<String> {
    let mut unmanaged = HashSet::new();
    let mut package: Option<&str> = None;
    let mut remote = false;
    let mut local = false;

    let mut finish = |package: &mut Option<&str>, remote: &mut bool, local: &mut bool| {
        if let Some(name) = package.take() {
            if *local && !*remote {
                unmanaged.insert(name.to_owned());
            }
        }
        *remote = false;
        *local = false;
    };

    for line in output.lines() {
        // A package header is unindented and ends in a colon.
        if !line.starts_with(char::is_whitespace) && line.ends_with(':') {
            finish(&mut package, &mut remote, &mut local);
            package = Some(&line[..line.len() - 1]);
            continue;
        }

        let trimmed = line.trim_start();
        if trimmed.contains("://") {
            remote = true;
        } else if trimmed.contains("/var/lib/dpkg/status") {
            local = true;
        }
    }
    finish(&mut package, &mut remote, &mut local);

    unmanaged
}

/// The packages dnf shows as installed from somewhere other than a repository.
///
/// `from_repo` is the repository a package came from; one installed from a file
/// on the command line reports `@commandline` or nothing at all.
pub fn parse_dnf_installed(output: &str) -> HashSet<String> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let name = fields.next()?;
            let from = fields.next().unwrap_or_default();
            (from.is_empty() || from == "@commandline" || from == "@System")
                .then(|| name.to_owned())
        })
        .collect()
}

/// Read the output of `rpm -qa --qf '%{NAME}\t%{VERSION}\t%{URL}\n'`.
pub fn parse_rpm_output(text: &str) -> Vec<Candidate> {
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let name = fields.next()?.trim();
            let version = fields.next()?.trim();
            let url = fields.next()?.trim();
            if name.is_empty() || version.is_empty() {
                return None;
            }
            // rpm prints this for a package with no URL.
            if url == "(none)" {
                return None;
            }
            Some(Candidate {
                name: name.to_owned(),
                version: version.to_owned(),
                repo: Some(Repo::from_url(url)?),
                source: Source::Rpm,
            })
        })
        .collect()
}

// ── AppImages ───────────────────────────────────────────────────────────────

/// Pull the contents of an ELF section out of a file's bytes.
///
/// Written out rather than taking an ELF-parsing dependency: only the section
/// table is needed, and shelling out to `objcopy` would make this depend on
/// binutils being installed, which on a desktop it often is not.
///
/// Returns `None` for anything that is not a 64-bit little-endian ELF, which is
/// the only shape AppImages are built in.
fn elf_section<'a>(bytes: &'a [u8], want: &str) -> Option<&'a [u8]> {
    // Header: magic, class (2 = 64-bit), data (1 = little-endian).
    if bytes.len() < 64 || &bytes[..4] != b"\x7fELF" || bytes[4] != 2 || bytes[5] != 1 {
        return None;
    }

    let read_u16 = |offset: usize| -> Option<usize> {
        Some(u16::from_le_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?) as usize)
    };
    let read_u64 = |offset: usize| -> Option<usize> {
        Some(u64::from_le_bytes(bytes.get(offset..offset + 8)?.try_into().ok()?) as usize)
    };

    let section_offset = read_u64(0x28)?;
    let entry_size = read_u16(0x3A)?;
    let count = read_u16(0x3C)?;
    let name_index = read_u16(0x3E)?;
    if entry_size < 64 || count == 0 || name_index >= count {
        return None;
    }

    // Each entry: name(4) type(4) flags(8) addr(8) offset(8) size(8) ...
    let entry = |index: usize| -> Option<(usize, usize, usize)> {
        let base = section_offset.checked_add(index.checked_mul(entry_size)?)?;
        let name = u32::from_le_bytes(bytes.get(base..base + 4)?.try_into().ok()?) as usize;
        Some((name, read_u64(base + 0x18)?, read_u64(base + 0x20)?))
    };

    // The string table holding section names, so they can be compared.
    let (_, strings_offset, strings_size) = entry(name_index)?;
    let strings = bytes.get(strings_offset..strings_offset.checked_add(strings_size)?)?;

    for index in 0..count {
        let (name_offset, offset, size) = entry(index)?;
        let name = strings
            .get(name_offset..)?
            .split(|byte| *byte == 0)
            .next()?;
        if name == want.as_bytes() {
            return bytes.get(offset..offset.checked_add(size)?);
        }
    }

    None
}

/// The update-information string an AppImage was built with, if any.
///
/// The section is a fixed 1 KiB and is zero-filled when the builder did not set
/// it — which is common — so an empty result is the ordinary case rather than a
/// malformed file.
pub fn appimage_update_info(bytes: &[u8]) -> Option<String> {
    let section = elf_section(bytes, ".upd_info")?;
    let text = section
        .split(|byte| *byte == 0)
        .next()
        .and_then(|slice| std::str::from_utf8(slice).ok())?
        .trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// Read a project out of an AppImage's update information.
///
/// Two transports are defined. `gh-releases-zsync` names the project directly,
/// which is as good as metadata gets. Plain `zsync` gives a URL that may or may
/// not be on a forge, so it is put through the same URL reader as everything
/// else and discarded if it names no project.
pub fn parse_update_info(info: &str) -> Option<Repo> {
    let mut fields = info.split('|');
    match fields.next()?.trim() {
        "gh-releases-zsync" | "gh-releases-direct" => {
            let owner = fields.next()?.trim();
            let project = fields.next()?.trim();
            if owner.is_empty() || project.is_empty() {
                return None;
            }
            Repo::from_url(&format!("https://github.com/{owner}/{project}"))
        }
        "zsync" => Repo::from_url(fields.next()?.trim()),
        _ => None,
    }
}

/// Split an AppImage filename into a project name and a version.
///
/// Used only when the file carries no update information. `qFlipper-x86_64-1.3.3.AppImage`
/// becomes `qFlipper` and `1.3.3`; architecture words are dropped because they
/// are not part of either.
pub fn parse_appimage_filename(file_name: &str) -> (String, String) {
    const ARCHITECTURES: [&str; 6] = ["x86_64", "amd64", "i386", "i686", "aarch64", "arm64"];

    let stem = file_name
        .trim_end_matches(".AppImage")
        .trim_end_matches(".appimage");

    // `x86_64` is the one architecture whose name contains a separator, so it
    // is set aside before the name is split apart — otherwise it becomes `x86`
    // and `64`, neither of which is recognised and both of which end up in the
    // project name.
    const PLACEHOLDER: &str = "\u{1}";
    let protected = stem.replace("x86_64", PLACEHOLDER).replace("X86_64", PLACEHOLDER);

    let parts: Vec<String> = protected
        .split(['-', '_'])
        .map(|part| part.replace(PLACEHOLDER, "x86_64"))
        .filter(|part| {
            !part.is_empty() && !ARCHITECTURES.contains(&part.to_ascii_lowercase().as_str())
        })
        .collect();

    // The version is the last part that starts with a digit; everything before
    // it is the name.
    let version_at = parts
        .iter()
        .rposition(|part| part.starts_with(|c: char| c.is_ascii_digit()));

    match version_at {
        Some(index) => (parts[..index].join("-"), parts[index..].join("-")),
        None => (parts.join("-"), String::new()),
    }
}

/// Look through the given directories for AppImages and read what each one says
/// about itself.
///
/// The directories are a parameter rather than a fixed list because where
/// people keep downloaded applications is a matter of habit — `~/Applications`,
/// `~/Downloads`, `/opt`, an external drive — and guessing wrongly means
/// silently finding nothing.
pub fn find_appimages(directories: &[String]) -> Vec<Candidate> {
    let home = std::env::var_os("HOME").map(PathBuf::from);

    let mut candidates = Vec::new();
    for entry in directories {
        // An absolute path is used as given; a relative one is taken from home,
        // which is what the defaults are written as.
        let directory = match (Path::new(entry).is_absolute(), &home) {
            (true, _) => PathBuf::from(entry),
            (false, Some(home)) => home.join(entry),
            (false, None) => continue,
        };
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };

        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !file_name.to_ascii_lowercase().ends_with(".appimage") {
                continue;
            }
            // A downloaded file that was never made runnable is not something
            // in use, and this is an updater rather than an installer.
            if !is_executable(&path) {
                debug_log!(RELEASES, "{file_name}: not executable, not in use");
                continue;
            }

            candidates.push(read_appimage(&path, file_name));
        }
    }

    debug_log!(RELEASES, "{} AppImage(s) found", candidates.len());
    candidates
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn read_appimage(path: &Path, file_name: &str) -> Candidate {
    let (name, version) = parse_appimage_filename(file_name);

    // Only the head of the file is read: the section table lives in the ELF
    // header region, and an AppImage is a whole filesystem — often hundreds of
    // megabytes — that there is no reason to pull into memory.
    let repo = read_head(path, ELF_HEAD_BYTES)
        .as_deref()
        .and_then(appimage_update_info)
        .as_deref()
        .and_then(parse_update_info);

    if repo.is_some() {
        debug_log!(RELEASES, "{file_name}: repository from embedded update info");
    }

    Candidate {
        name,
        version,
        repo,
        source: Source::AppImage(path.to_path_buf()),
    }
}

/// How much of an AppImage to read to reach its section table.
const ELF_HEAD_BYTES: usize = 512 * 1024;

fn read_head(path: &Path, limit: usize) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut buffer = vec![0; limit];
    let read = file.read(&mut buffer).ok()?;
    buffer.truncate(read);
    Some(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DPKG: &str = "\
Package: brightnessctl
Status: install ok installed
Version: 0.5.1-4
Homepage: https://github.com/Hummer12007/brightnessctl

Package: removed-thing
Status: deinstall ok config-files
Version: 1.0
Homepage: https://github.com/a/b

Package: kwin
Status: install ok installed
Version: 4:5.27.5-2
Homepage: https://invent.kde.org/plasma/kwin

Package: coreutils
Status: install ok installed
Version: 9.4-3
Homepage: https://www.gnu.org/software/coreutils/
";

    #[test]
    fn reads_installed_packages_that_name_a_project() {
        let found = parse_dpkg_status(DPKG);
        let names: Vec<&str> = found.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["brightnessctl", "kwin"]);
    }

    #[test]
    fn a_removed_package_is_not_a_candidate() {
        // The status file keeps entries for packages that left configuration
        // behind; offering an update for one would be nonsense.
        assert!(!parse_dpkg_status(DPKG)
            .iter()
            .any(|c| c.name == "removed-thing"));
    }

    #[test]
    fn a_homepage_that_is_not_a_project_is_skipped() {
        assert!(!parse_dpkg_status(DPKG).iter().any(|c| c.name == "coreutils"));
    }

    #[test]
    fn a_self_hosted_forge_is_detected_from_a_homepage() {
        let kwin = parse_dpkg_status(DPKG)
            .into_iter()
            .find(|c| c.name == "kwin")
            .expect("kwin");
        let repo = kwin.repo.expect("repo");
        assert_eq!(repo.host, "invent.kde.org");
        assert_eq!(repo.kind, super::super::forge::Kind::GitLab);
        // The Debian epoch travels with the version and is dealt with when
        // comparing, not here.
        assert_eq!(kwin.version, "4:5.27.5-2");
    }

    #[test]
    fn a_package_from_a_repository_is_left_to_the_package_manager() {
        let output = "\
curl:
  Installed: 8.5.0
  Version table:
 *** 8.5.0 500
        500 http://apt.pop-os.org/ubuntu noble/main amd64 Packages
        100 /var/lib/dpkg/status
";
        assert!(parse_apt_policy(output).is_empty());
    }

    #[test]
    fn a_package_installed_from_a_file_is_a_candidate() {
        // Nothing offers it, so apt will never update it — which is the whole
        // reason for tracking releases.
        let output = "\
winboat:
  Installed: 1.2.3
  Version table:
 *** 1.2.3 100
        100 /var/lib/dpkg/status
";
        let unmanaged = parse_apt_policy(output);
        assert_eq!(unmanaged.len(), 1);
        assert!(unmanaged.contains("winboat"));
    }

    #[test]
    fn several_packages_are_told_apart() {
        let output = "\
curl:
  Installed: 8.5.0
  Version table:
 *** 8.5.0 500
        500 http://apt.pop-os.org/ubuntu noble/main amd64 Packages
        100 /var/lib/dpkg/status
yapcap:
  Installed: 0.4.0
  Version table:
 *** 0.4.0 100
        100 /var/lib/dpkg/status
bash:
  Installed: 5.2
  Version table:
 *** 5.2 500
        500 http://apt.pop-os.org/ubuntu noble/main amd64 Packages
        100 /var/lib/dpkg/status
";
        let unmanaged = parse_apt_policy(output);
        assert_eq!(unmanaged.len(), 1, "{unmanaged:?}");
        assert!(unmanaged.contains("yapcap"));
    }

    #[test]
    fn dnf_reports_command_line_installs_as_unmanaged() {
        let output = "curl fedora\nwinboat @commandline\nbash updates\nthing\n";
        let unmanaged = parse_dnf_installed(output);
        assert!(unmanaged.contains("winboat"));
        assert!(unmanaged.contains("thing"));
        assert!(!unmanaged.contains("curl"));
        assert!(!unmanaged.contains("bash"));
    }

    #[test]
    fn reads_rpm_output() {
        let text = "topgrade\t17.9.0\thttps://github.com/topgrade-rs/topgrade\n\
                    bash\t5.2\t(none)\n";
        let found = parse_rpm_output(text);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "topgrade");
        assert_eq!(found[0].source, Source::Rpm);
    }

    #[test]
    fn reads_github_update_information() {
        let repo = parse_update_info(
            "gh-releases-zsync|Flipper-Devices|qFlipper|latest|qFlipper-x86_64-*.AppImage.zsync",
        )
        .expect("repo");
        assert_eq!(repo.path, "Flipper-Devices/qFlipper");
        assert_eq!(repo.kind, super::super::forge::Kind::GitHub);
    }

    #[test]
    fn reads_generic_zsync_update_information() {
        let repo = parse_update_info("zsync|https://codeberg.org/a/b/releases/x.zsync")
            .expect("repo");
        assert_eq!(repo.host, "codeberg.org");
        assert_eq!(repo.path, "a/b");
    }

    #[test]
    fn unusable_update_information_is_declined() {
        assert!(parse_update_info("zsync|https://example.com/x.zsync").is_none());
        assert!(parse_update_info("something-else|a|b").is_none());
        assert!(parse_update_info("").is_none());
        assert!(parse_update_info("gh-releases-zsync||").is_none());
    }

    #[test]
    fn splits_an_appimage_filename() {
        assert_eq!(
            parse_appimage_filename("qFlipper-x86_64-1.3.3.AppImage"),
            ("qFlipper".to_owned(), "1.3.3".to_owned())
        );
        assert_eq!(
            parse_appimage_filename("VeraCrypt-1.26.24-x86_64.AppImage"),
            ("VeraCrypt".to_owned(), "1.26.24".to_owned())
        );
    }

    #[test]
    fn an_appimage_filename_with_no_version_still_yields_a_name() {
        assert_eq!(
            parse_appimage_filename("SomeApp.AppImage"),
            ("SomeApp".to_owned(), String::new())
        );
    }

    #[test]
    fn an_empty_update_section_reads_as_absent() {
        // The section is a fixed kilobyte and is usually zero-filled, which is
        // what both AppImages on the machine this was written on look like.
        let mut elf = minimal_elf_with_section(".upd_info", &[0; 64]);
        assert!(appimage_update_info(&elf).is_none());

        // And a populated one reads back.
        elf = minimal_elf_with_section(".upd_info", b"gh-releases-zsync|a|b|latest|x\0\0\0");
        assert_eq!(
            appimage_update_info(&elf).as_deref(),
            Some("gh-releases-zsync|a|b|latest|x")
        );
    }

    #[test]
    fn something_that_is_not_an_elf_is_declined() {
        assert!(appimage_update_info(b"not an elf at all").is_none());
        assert!(appimage_update_info(&[]).is_none());
    }

    /// Build the smallest ELF64 that has one named section, so the section
    /// reader can be tested without shipping a binary fixture.
    fn minimal_elf_with_section(name: &str, contents: &[u8]) -> Vec<u8> {
        let header = 64usize;
        let entry = 64usize;
        // Layout: header, two section headers, the name table, the contents.
        let table_offset = header;
        let names_offset = table_offset + entry * 2;
        let names = {
            let mut bytes = vec![0u8];
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(0);
            bytes.extend_from_slice(b".shstrtab\0");
            bytes
        };
        let contents_offset = names_offset + names.len();

        let mut elf = vec![0u8; contents_offset + contents.len()];
        elf[..4].copy_from_slice(b"\x7fELF");
        elf[4] = 2; // 64-bit
        elf[5] = 1; // little-endian
        elf[0x28..0x30].copy_from_slice(&(table_offset as u64).to_le_bytes());
        elf[0x3A..0x3C].copy_from_slice(&(entry as u16).to_le_bytes());
        elf[0x3C..0x3E].copy_from_slice(&2u16.to_le_bytes());
        elf[0x3E..0x40].copy_from_slice(&1u16.to_le_bytes());

        // Section 0: the one being looked for; its name is at offset 1.
        let base = table_offset;
        elf[base..base + 4].copy_from_slice(&1u32.to_le_bytes());
        elf[base + 0x18..base + 0x20].copy_from_slice(&(contents_offset as u64).to_le_bytes());
        elf[base + 0x20..base + 0x28].copy_from_slice(&(contents.len() as u64).to_le_bytes());

        // Section 1: the name table itself.
        let base = table_offset + entry;
        let shstrtab_at = 1 + name.len() + 1;
        elf[base..base + 4].copy_from_slice(&(shstrtab_at as u32).to_le_bytes());
        elf[base + 0x18..base + 0x20].copy_from_slice(&(names_offset as u64).to_le_bytes());
        elf[base + 0x20..base + 0x28].copy_from_slice(&(names.len() as u64).to_le_bytes());

        elf[names_offset..names_offset + names.len()].copy_from_slice(&names);
        elf[contents_offset..].copy_from_slice(contents);
        elf
    }
}
