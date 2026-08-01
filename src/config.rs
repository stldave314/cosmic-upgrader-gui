// SPDX-License-Identifier: GPL-3.0

//! This application's own settings, stored and loaded via `cosmic-config`.
//!
//! Only things about *this application* live here. Everything that configures
//! the upgrade itself belongs to topgrade and is written to its own file
//! through [`crate::topgrade::settings_file`] — putting a second copy here
//! would mean two sources of truth for the same setting and a way for them to
//! disagree. Values that exist to tune the implementation are in
//! [`crate::constants`] instead.

use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

use crate::constants::DEFAULT_KEEP_RUNS;
use crate::constants::{APPIMAGE_SEARCH_DIRS, CLAMSCAN_DEFAULT_OPTIONS, CLAMSCAN_DEFAULT_TARGET};
use crate::releases::{Channel, CheckInterval, Watch};
use crate::schedule::Schedule;

/// Bumped when a field is removed or its meaning changes, so `cosmic-config`
/// discards an incompatible stored config rather than mis-reading it.
pub const CONFIG_VERSION: u64 = 2;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum AppTheme {
    Dark,
    Light,
    #[default]
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => theme::Theme::dark(),
            Self::Light => theme::Theme::light(),
            Self::System => theme::system_preference(),
        }
    }
}

/// How the password for privileged steps is obtained.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum PrivilegeMode {
    /// Run topgrade under a pseudo-terminal and answer `sudo`'s prompt from
    /// this window. One prompt for the whole run, and the password goes
    /// straight to the terminal without being stored.
    #[default]
    AskInWindow,
    /// Let topgrade call `pkexec`, so the desktop's own authentication dialog
    /// asks instead.
    ///
    /// Chosen by some people because the polkit dialog is the familiar one and
    /// this application never sees the password at all. The cost is that
    /// `pkexec` authenticates per command, and the system step runs the package
    /// manager several times.
    ///
    /// Selecting this writes `misc.sudo_command` into topgrade's own
    /// configuration, because that is the only place topgrade reads it from —
    /// there is no command-line equivalent. It is visible and editable on the
    /// configuration page like any other setting.
    SystemDialog,
}

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[version = 2]
pub struct Config {
    pub app_theme: AppTheme,
    /// How privileged steps get their authorization.
    pub privilege_mode: PrivilegeMode,
    /// Ask before starting an upgrade.
    ///
    /// On by default: an upgrade is not something to begin by mis-clicking, and
    /// once a package transaction has started it cannot be cleanly undone.
    pub confirm_before_running: bool,
    /// Answer package managers' prompts affirmatively during a run.
    ///
    /// On by default because the alternative, in a window with no terminal to
    /// type into, is a run that stops at the first prompt and appears to hang.
    pub assume_yes: bool,
    /// Show steps whose tools are not installed.
    ///
    /// Off by default: on a typical system that is around 140 of the 174 steps,
    /// and burying the dozen that apply under them is the main thing that makes
    /// topgrade's own step list hard to work with.
    pub show_unavailable_steps: bool,
    /// Say something when upgrades are found or installed.
    ///
    /// What it says depends on what the schedule does: a run that installs
    /// reports what it installed, one that only checks reports what is
    /// available. Both are the same wish — "tell me about upgrades" — so they
    /// are one setting rather than two the user has to keep consistent.
    pub notify_upgrades: bool,
    /// Say something when an upgrade fails.
    ///
    /// Separate, and on by default, because a failure is the one thing worth
    /// interrupting somebody for. Turning it off is possible but deliberate.
    pub notify_errors: bool,
    /// Run a virus scan after the ClamAV database changes.
    pub clamav_scan: bool,
    /// Options handed to `clamscan`.
    pub clamscan_options: String,
    /// What the scan looks at.
    pub clamscan_target: String,
    /// When upgrades run unattended.
    pub schedule: Schedule,
    /// Whether an icon is shown in the panel's status area.
    ///
    /// It raises the window, starts an upgrade without opening it, and quits.
    /// It deliberately does not hide the window: Wayland has no way for a client
    /// to undo minimizing itself, so an icon that put the window away could not
    /// bring it back.
    pub show_tray_icon: bool,
    /// Whether the first-run questions have been answered.
    ///
    /// Asking once is helpful; asking every launch is not, so this records that
    /// the conversation happened — including when it was dismissed without
    /// choosing anything.
    pub first_run_completed: bool,
    /// How many past runs to keep transcripts for.
    pub keep_run_logs: usize,
    /// Projects watched for new releases.
    ///
    /// Held here rather than re-detected each launch because it is a decision
    /// the user made: re-deriving it would drop anything they added by hand and
    /// re-add everything they rejected.
    pub watches: Vec<Watch>,
    /// How often releases are checked without being asked.
    pub release_check_interval: CheckInterval,
    /// Whether release candidates and betas count as updates.
    pub release_channel: Channel,
    /// Directories searched for downloaded applications.
    ///
    /// Relative entries are taken from the home directory; absolute ones are
    /// used as given, so somewhere outside home can be added.
    pub appimage_dirs: Vec<String>,
    /// When the last release check ran, in seconds since the Unix epoch.
    pub last_release_check: i64,
    /// When the in-app fallback scheduler last started a run, in seconds since
    /// the Unix epoch, or zero if it never has.
    ///
    /// Only the fallback uses this. Under systemd the timer's own `Persistent=`
    /// state is the record of when it last fired, and keeping a second copy
    /// here would be one more thing able to disagree with it.
    pub last_fallback_run: i64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            privilege_mode: PrivilegeMode::AskInWindow,
            confirm_before_running: true,
            assume_yes: true,
            show_unavailable_steps: false,
            notify_upgrades: true,
            notify_errors: true,
            clamav_scan: false,
            clamscan_options: CLAMSCAN_DEFAULT_OPTIONS.to_owned(),
            clamscan_target: CLAMSCAN_DEFAULT_TARGET.to_owned(),
            show_tray_icon: false,
            first_run_completed: false,
            keep_run_logs: DEFAULT_KEEP_RUNS,
            watches: Vec::new(),
            release_check_interval: CheckInterval::default(),
            release_channel: Channel::default(),
            appimage_dirs: APPIMAGE_SEARCH_DIRS
                .iter()
                .map(|directory| (*directory).to_owned())
                .collect(),
            last_release_check: 0,
            schedule: Schedule::default(),
            last_fallback_run: 0,
        }
    }
}
