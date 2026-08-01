// SPDX-License-Identifier: GPL-3.0

//! Build-time diagnostic logging.
//!
//! Almost everything interesting in this application happens in a child
//! process: discovering what topgrade can do, probing each step, and then
//! driving a long upgrade run under a pseudo-terminal. When something goes
//! wrong the useful detail is which command ran, what it printed and what it
//! exited with — far too noisy for stderr, which in a desktop-launched app is
//! usually discarded anyway. Everything here goes to a file instead.
//!
//! Logging is gated on the [`ENABLED`] constant so it can be compiled out
//! entirely: when it is `false` the `debug_log!` macro's body is unreachable and
//! the optimiser removes it, leaving no formatting cost and no file I/O. The
//! arguments are still type-checked either way, so disabled call sites can't rot.
//!
//! ```ignore
//! debug_log!(PROBE, "{} steps discovered", steps.len());
//! ```

use std::fmt::Arguments;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// Developer switch — flip this to turn diagnostic logging on or off locally.
///
/// This is *not* the final word: see [`ENABLED`], which additionally forces
/// logging off for release builds.
const DEVELOPER_LOGGING: bool = false;

/// Whether logging actually happens.
///
/// Release packages are built with the `release-build` feature (see the
/// packaging targets in `install.sh`), which forces this to `false` no matter
/// what [`DEVELOPER_LOGGING`] says — so a release can never ship with diagnostic
/// logging left switched on by accident.
pub const ENABLED: bool = DEVELOPER_LOGGING && !cfg!(feature = "release-build");

/// Where the log is written. Truncated once per process launch.
pub const PATH: &str = "/tmp/cosmic-upgrader-gui.log";

// ── Categories ──────────────────────────────────────────────────────────────
// Short tags so a run can be filtered with `grep`.

/// Locating the topgrade binary and reading its version.
pub const LOCATE: &str = "loc";
/// Step and configuration-schema discovery.
pub const DISCOVER: &str = "disc";
/// Per-step capability probing.
pub const PROBE: &str = "probe";
/// Reading and writing topgrade's configuration file.
pub const SETTINGS: &str = "set";
/// Driving an upgrade run and parsing its output.
pub const RUN: &str = "run";
/// Scheduled runs and the systemd units behind them.
pub const SCHEDULE: &str = "sched";
/// External command invocations and their exit status.
pub const EXEC: &str = "exec";
/// Application configuration and UI state.
pub const UI: &str = "ui";
/// Recording and reading back past runs.
pub const HISTORY: &str = "hist";
/// Discovering projects and checking them for newer releases.
pub const RELEASES: &str = "rel";

/// Process start, so each line can be stamped with elapsed time rather than a
/// wall clock the reader has to subtract by hand.
static START: OnceLock<Instant> = OnceLock::new();

/// The open log file. Held behind a mutex so lines from the probe pool — which
/// is genuinely concurrent — don't interleave mid-line.
static FILE: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();

/// Write one line. Called only through [`debug_log!`], never directly.
///
/// Failures here are deliberately silent: diagnostic logging that itself
/// reports errors would be noise on exactly the systems where the log file
/// cannot be written, and it is not what the user asked the application to do.
#[doc(hidden)]
pub fn write(category: &str, args: Arguments<'_>) {
    let start = START.get_or_init(Instant::now);
    let file = FILE.get_or_init(|| {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(PATH)
            .ok()
            .map(Mutex::new)
    });

    let Some(file) = file.as_ref() else {
        return;
    };
    let Ok(mut file) = file.lock() else {
        return;
    };

    let elapsed = start.elapsed();
    let _ = writeln!(
        file,
        "[{:>8.3}] {:<5} {}",
        elapsed.as_secs_f64(),
        category,
        args
    );
    let _ = file.flush();
}

/// Record a diagnostic line, if logging is compiled in.
///
/// The `if ENABLED` is a compile-time constant, so with logging off the whole
/// body is dead code and disappears — but the arguments are still type-checked,
/// which is what stops a disabled call site from silently rotting.
#[macro_export]
macro_rules! debug_log {
    ($category:expr, $($arg:tt)*) => {{
        if $crate::debug::ENABLED {
            $crate::debug::write($category, format_args!($($arg)*));
        }
    }};
}
