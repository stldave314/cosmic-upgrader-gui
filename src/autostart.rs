// SPDX-License-Identifier: GPL-3.0

//! Starting with the desktop session.
//!
//! The freedesktop autostart specification is a directory of `.desktop` files:
//! anything in `~/.config/autostart/` is launched when the session begins. That
//! is all this is — writing one file and removing it again.
//!
//! A file of our own is written rather than copying the installed desktop entry,
//! because the autostart copy needs an argument the menu entry must not have.
//! Starting into a visible window every login is not what "start automatically"
//! means for an application whose point is to sit quietly and check for updates,
//! so the autostart entry passes `--minimized`.

use std::path::PathBuf;

use crate::constants::{APP_ICON, APP_ID, AUTOSTART_DIR, MINIMIZED_FLAG};
use crate::debug::UI;
use crate::debug_log;

#[derive(Clone, Debug)]
pub enum Error {
    NoConfigDirectory,
    Io { path: PathBuf, message: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoConfigDirectory => write!(f, "no configuration directory could be determined"),
            Self::Io { path, message } => write!(f, "{}: {message}", path.display()),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

fn directory() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or(Error::NoConfigDirectory)?;
    Ok(base.join(AUTOSTART_DIR))
}

fn entry_path() -> Result<PathBuf> {
    Ok(directory()?.join(format!("{APP_ID}.desktop")))
}

/// Whether the session is currently set to start this application.
pub fn is_enabled() -> bool {
    entry_path().map(|path| path.is_file()).unwrap_or(false)
}

/// The desktop entry written into the autostart directory.
///
/// `X-GNOME-Autostart-enabled` is honoured by several session managers that
/// otherwise have no way to express a disabled-but-present entry. It is
/// harmless where it is not understood.
fn entry(executable: &str) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Upgrader\n\
         Comment=Check for system upgrades in the background\n\
         Exec={executable} {MINIMIZED_FLAG}\n\
         Icon={APP_ICON}\n\
         Terminal=false\n\
         X-GNOME-Autostart-enabled=true\n\
         # Written by cosmic-upgrader-gui. Removing this file, or turning the\n\
         # setting off in the application, stops it starting with the session.\n"
    )
}

/// Turn starting with the session on or off.
pub fn set_enabled(enabled: bool) -> Result<()> {
    let path = entry_path()?;

    if !enabled {
        match std::fs::remove_file(&path) {
            Ok(()) => debug_log!(UI, "autostart entry removed"),
            // Already absent is the state that was asked for.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(Error::Io {
                    path,
                    message: error.to_string(),
                })
            }
        }
        return Ok(());
    }

    let directory = directory()?;
    std::fs::create_dir_all(&directory).map_err(|error| Error::Io {
        path: directory,
        message: error.to_string(),
    })?;

    // The running binary's own path, so an autostart entry written from a build
    // in a home directory starts that build rather than a packaged copy that
    // may not exist.
    let executable = std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_owned());

    std::fs::write(&path, entry(&executable)).map_err(|error| Error::Io {
        path,
        message: error.to_string(),
    })?;

    debug_log!(UI, "autostart entry written");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_entry_starts_minimized() {
        // Opening a window on every login is not what enabling this means.
        let entry = entry("/usr/bin/cosmic-upgrader-gui");
        assert!(
            entry.contains(&format!("Exec=/usr/bin/cosmic-upgrader-gui {MINIMIZED_FLAG}")),
            "{entry}"
        );
    }

    #[test]
    fn the_entry_is_a_valid_desktop_file() {
        let entry = entry("/usr/bin/app");
        assert!(entry.starts_with("[Desktop Entry]\n"), "{entry}");
        for key in ["Type=Application", "Name=", "Icon=", "Terminal=false"] {
            assert!(entry.contains(key), "missing {key} in:\n{entry}");
        }
    }

    #[test]
    fn the_entry_says_where_it_came_from() {
        // So somebody finding it in ~/.config/autostart knows what wrote it.
        assert!(entry("/usr/bin/app").contains("cosmic-upgrader-gui"));
    }
}
