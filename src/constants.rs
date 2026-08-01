// SPDX-License-Identifier: GPL-3.0

//! Compile-time tuning values, gathered in one place.
//!
//! These are *implementation* tuning knobs, not user settings — anything the
//! user should be able to change lives in [`crate::config`] and is persisted via
//! `cosmic-config`, or in topgrade's own configuration file and is written
//! through [`crate::topgrade::settings_file`]. Keeping these compile-time avoids
//! a third configuration mechanism, a startup file read, and a class of
//! "malformed config" failures, while still giving one obvious place to find
//! and adjust them.

use std::time::Duration;

// ── Identity ────────────────────────────────────────────────────────────────

/// D-Bus / desktop-entry identifier. Must match the `.desktop` file name
/// installed by the packaging targets, or the window will not pick up its icon.
pub const APP_ID: &str = "com.github.cosmic_upgrader_gui";

/// Icon name shipped alongside the desktop entry. Matches [`APP_ID`] because
/// the packaging targets install the icon under that name.
pub const APP_ICON: &str = APP_ID;

/// Consulted by the About dialog, derived from the `repository` field in
/// Cargo.toml so the URL has a single source of truth.
pub const REPOSITORY_URL: &str = env!("CARGO_PKG_REPOSITORY");

/// Where users are sent to report a problem.
pub const ISSUES_URL: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/issues");

// ── Window ──────────────────────────────────────────────────────────────────

/// Initial window size. Wide enough for the sidebar plus a two-column step
/// list, tall enough to show a category's steps without scrolling.
pub const WINDOW_WIDTH: f32 = 1100.0;
pub const WINDOW_HEIGHT: f32 = 750.0;

/// Below this libcosmic folds the sidebar into an overlay of its own accord.
pub const WINDOW_MIN_WIDTH: f32 = 460.0;
pub const WINDOW_MIN_HEIGHT: f32 = 400.0;

/// Content is centred and capped at this width so step rows and log lines don't
/// stretch into unreadably long lines on a maximised window.
pub const MAX_CONTENT_WIDTH: f32 = 1000.0;

/// Size in pixels of the small icons used in step and category rows.
pub const ICON_SIZE_ROW: u16 = 16;

// ── Locating topgrade ───────────────────────────────────────────────────────

/// Executable name looked up on `PATH`.
pub const TOPGRADE_BIN: &str = "topgrade";

/// Where the packaging targets put the bundled copy, used only when the system
/// has no topgrade of its own. Built by `install.sh` under the
/// `bundled-topgrade` feature.
pub const TOPGRADE_BUNDLED_PATH: &str = "/usr/libexec/cosmic-upgrader-gui/topgrade";

/// Lowest topgrade version whose `--config-reference` and `--only` output this
/// application knows how to read. Older releases are reported as unsupported
/// rather than parsed on a hope.
pub const TOPGRADE_MIN_VERSION: (u32, u32) = (16, 0);

// ── Introspection ───────────────────────────────────────────────────────────

/// Longest `--version`, `--help` or `--config-reference` may take. These only
/// format text that is compiled into the binary, so anything beyond this means
/// the process is wedged and the user is better served by an error.
pub const INTROSPECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Longest a single-step capability probe may take.
///
/// Most return in a few milliseconds, but a handful genuinely reach out — the
/// container step enumerates images, the JetBrains steps stat a directory tree —
/// so this is generous enough not to punish a slow disk while still bounding a
/// step that hangs on an unreachable network mount.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(30);

/// Upper bound on concurrent probe processes.
///
/// A full scan is roughly 175 short-lived processes. Running them one at a time
/// takes long enough to be noticeable; running all of them at once buries a
/// small machine under process spawns for no gain, since the work is dominated
/// by `exec` and directory stats rather than CPU. The effective figure is this
/// capped by available parallelism — see [`probe_concurrency`].
pub const PROBE_MAX_CONCURRENCY: usize = 16;

/// How many probes to run at once on this machine.
pub fn probe_concurrency() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(2, PROBE_MAX_CONCURRENCY)
}

// ── Parsing topgrade's output ───────────────────────────────────────────────

/// Characters topgrade rules its step headings with.
///
/// Which one it picks depends on what it believes it is writing to, and this
/// application uses both channels: a capability probe reads piped output, where
/// a heading is `―― 17:11:57 - Cargo ――` in `U+2015 HORIZONTAL BAR`, while a
/// real run happens under a pseudo-terminal, where the same heading arrives as
/// `── 17:11:57 - Cargo ─────…` in `U+2500 BOX DRAWINGS LIGHT HORIZONTAL`,
/// padded out to the terminal width.
///
/// Matching only the first form is a quiet failure rather than a loud one — the
/// run still works, but it never reports which step it is on and never finds
/// its own summary. The near neighbours are accepted too, so a release that
/// changes its mind again does not break this.
pub const HEADING_RULE_CHARS: [char; 4] = ['\u{2015}', '\u{2500}', '\u{2014}', '\u{2501}'];

/// The heading that introduces the per-step result list at the end of a run.
pub const SUMMARY_HEADING: &str = "Summary";

/// Status words topgrade writes in the summary, in the form `Name: STATUS`.
pub const STATUS_OK: &str = "OK";
pub const STATUS_SKIPPED: &str = "SKIPPED";
pub const STATUS_FAILED: &str = "FAILED";

// ── Running an upgrade ──────────────────────────────────────────────────────

/// Terminal size reported to the pseudo-terminal topgrade runs under.
///
/// Wide enough that the tools it drives don't wrap their progress bars into
/// unreadable fragments, which is what happens at the 80×24 default.
pub const PTY_COLS: u16 = 200;
pub const PTY_ROWS: u16 = 50;

/// Longest the run log is kept in memory, in lines. A full upgrade of a busy
/// machine produces tens of thousands; keeping all of them costs memory and
/// makes the scroll view sluggish for output nobody reads.
pub const RUN_LOG_MAX_LINES: usize = 20_000;

/// Substrings that mark a line as a password prompt from `sudo` or a tool it
/// invokes, in the locales topgrade is likely to be run under.
///
/// Matched case-insensitively against the tail of the pseudo-terminal's output.
/// A false positive shows a password field the user can dismiss; a false
/// negative leaves the run stalled with no visible reason, so this leans
/// towards matching.
pub const PASSWORD_PROMPT_MARKERS: [&str; 4] =
    ["password for", "password:", "mot de passe", "passwort"];

/// The `pkexec` binary, used when the user has chosen that privilege transport.
pub const PKEXEC: &str = "/usr/bin/pkexec";

// ── Session integration ─────────────────────────────────────────────────────

/// Directory, relative to the user's config directory, that the session reads
/// autostart entries from.
pub const AUTOSTART_DIR: &str = "autostart";

/// Passed by the autostart entry so a login does not open a window.
///
/// Not a documented interface — it is written by this application into its own
/// autostart file and read back by the same binary.
pub const MINIMIZED_FLAG: &str = "--minimized";

// ── Run history ─────────────────────────────────────────────────────────────

/// Directory, relative to the user's data directory, holding run records.
pub const HISTORY_DIR: &str = "cosmic-upgrader-gui/runs";

/// How many past runs to keep before the oldest are discarded.
///
/// Each run keeps its full transcript, which for a busy machine is megabytes,
/// so these cannot accumulate for ever. Fifty is a couple of months of nightly
/// upgrades — long enough to answer "when did this break?".
pub const DEFAULT_KEEP_RUNS: usize = 50;

// ── Release tracking ────────────────────────────────────────────────────────

/// Sent when asking a forge for its releases.
///
/// Several forges reject requests without one, and an identifiable agent is the
/// courteous thing to send to a service being polled.
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Longest a single release check may take. These are small JSON responses, so
/// anything beyond this is a host that is not answering.
pub const RELEASE_CHECK_TIMEOUT: Duration = Duration::from_secs(20);

/// Longest a release download may take. Generous: some releases are hundreds of
/// megabytes and some connections are slow.
pub const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// How many release checks to run at once.
///
/// Lower than the step probe's concurrency because these are requests to other
/// people's servers rather than local processes, and several forges rate-limit
/// per client.
pub const RELEASE_CHECK_CONCURRENCY: usize = 4;

/// Directories, relative to the user's home, searched for AppImages.
pub const APPIMAGE_SEARCH_DIRS: [&str; 5] = [
    "Applications",
    "Downloads",
    ".local/bin",
    "bin",
    "AppImages",
];

// ── Scheduling ──────────────────────────────────────────────────────────────

/// Base name of the systemd user units written for scheduled runs. The timer
/// and service share it, as systemd expects.
pub const SCHEDULE_UNIT_NAME: &str = "cosmic-upgrader-gui-scheduled";

/// Directory, relative to the user's config directory, that systemd reads user
/// units from.
pub const SYSTEMD_USER_UNIT_DIR: &str = "systemd/user";

/// How often the in-app fallback scheduler checks whether a run is due.
///
/// Only used where systemd is unavailable. A minute is frequent enough that a
/// scheduled time is not missed by a noticeable margin, and idle enough not to
/// matter.
pub const FALLBACK_SCHEDULER_TICK: Duration = Duration::from_secs(60);
