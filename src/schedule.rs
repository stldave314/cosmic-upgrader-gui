// SPDX-License-Identifier: GPL-3.0

//! Running upgrades on a schedule.
//!
//! "Check for upgrades regularly" only means anything if it happens when the
//! window is closed, which rules out a timer living inside the application. So
//! the schedule is handed to systemd as a pair of user units — a timer and the
//! service it starts — written into the user's own unit directory. They survive
//! logout and reboot, they can be inspected with `systemctl --user`, and they
//! keep working if this application is never opened again.
//!
//! Where systemd is not available — a container, a BSD, a session put together
//! by hand — there is a fallback that ticks inside the running application. It
//! is genuinely worse, and the interface says so rather than implying a
//! schedule is being kept that is not.
//!
//! The scheduled unit runs this same binary with `--scheduled`, rather than
//! running topgrade directly, so that the run goes through the same
//! configuration and reporting as one started from the window, and so there is
//! somewhere to put the notification afterwards.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::constants::{SCHEDULE_UNIT_NAME, SYSTEMD_USER_UNIT_DIR, SYSTEM_UNIT_DIR};
use crate::debug::SCHEDULE;
use crate::debug_log;

/// How often a scheduled run happens.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Frequency {
    Hourly,
    #[default]
    Daily,
    Weekly,
    Monthly,
}

impl Frequency {
    pub const ALL: [Self; 4] = [Self::Hourly, Self::Daily, Self::Weekly, Self::Monthly];

    /// The `OnCalendar=` expression for this frequency at a given time.
    ///
    /// Hourly ignores the hour, since "every hour at half past" is what an
    /// hourly schedule with a chosen minute means.
    fn on_calendar(self, hour: u32, minute: u32) -> String {
        match self {
            Self::Hourly => format!("*-*-* *:{minute:02}:00"),
            Self::Daily => format!("*-*-* {hour:02}:{minute:02}:00"),
            Self::Weekly => format!("Mon *-*-* {hour:02}:{minute:02}:00"),
            Self::Monthly => format!("*-*-01 {hour:02}:{minute:02}:00"),
        }
    }
}

/// What the user asked for.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Schedule {
    pub enabled: bool,
    pub frequency: Frequency,
    pub hour: u32,
    pub minute: u32,
    /// Whether to install what is found, or only report it.
    ///
    /// Off by default. An upgrade that runs unattended can restart services and
    /// replace a running kernel, and that should be something the user turns on
    /// deliberately rather than something they discover has been happening.
    pub automatic: bool,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: Frequency::Daily,
            // Late enough to be after a working day, early enough that a
            // machine suspended overnight has probably not been shut down yet.
            hour: 18,
            minute: 0,
            automatic: false,
        }
    }
}

impl Schedule {
    /// How long between runs, for the in-app fallback.
    ///
    /// The systemd path does not use this — `OnCalendar` expresses "daily at
    /// 18:00" exactly, and systemd works out the next occurrence itself. The
    /// fallback has only a tick and a last-run time to work with, so it
    /// approximates the same schedule as an interval. Monthly is taken as 30
    /// days, which is the closest a fixed interval gets.
    pub fn interval(self) -> std::time::Duration {
        let hours = match self.frequency {
            Frequency::Hourly => 1,
            Frequency::Daily => 24,
            Frequency::Weekly => 24 * 7,
            Frequency::Monthly => 24 * 30,
        };
        std::time::Duration::from_secs(hours * 60 * 60)
    }

    /// Whether a run is due, given when the last one happened.
    ///
    /// A `last_run` of zero means none has ever happened. That deliberately
    /// does *not* count as due: starting an upgrade the moment the option is
    /// switched on would be a surprise, and the interval is measured from now
    /// instead.
    pub fn is_due(self, last_run: i64, now: i64) -> bool {
        if !self.enabled || last_run == 0 {
            return false;
        }
        now.saturating_sub(last_run) >= self.interval().as_secs() as i64
    }

    /// Clamp to a real time of day.
    ///
    /// The interface offers valid values, but the configuration is a file a
    /// user can edit, and an out-of-range hour would produce a unit systemd
    /// silently refuses to start.
    fn normalized(self) -> Self {
        Self {
            hour: self.hour.min(23),
            minute: self.minute.min(59),
            ..self
        }
    }
}

/// How the schedule is being kept.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Backend {
    /// systemd user units — the real thing.
    Systemd,
    /// A timer inside this process, which only runs while the window is open.
    InApp,
}

#[derive(Clone, Debug)]
pub enum Error {
    NoConfigDirectory,
    Io { path: PathBuf, message: String },
    Systemctl { message: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoConfigDirectory => write!(f, "no configuration directory could be determined"),
            Self::Io { path, message } => write!(f, "{}: {message}", path.display()),
            Self::Systemctl { message } => write!(f, "systemctl: {message}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// Whether systemd can keep the schedule.
///
/// Asks systemd itself rather than looking for its files: a `systemctl` binary
/// can be present on a machine booted with something else, and what matters is
/// whether there is a user manager listening now.
pub async fn detect_backend() -> Backend {
    let ran = Command::new("systemctl")
        .args(["--user", "--quiet", "is-system-running"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .status()
        .await;

    // `is-system-running` reports failure for a degraded manager, which is
    // still a manager that runs timers. Only being unable to talk to one at all
    // rules systemd out, and that shows up as the command not running.
    match ran {
        Ok(_) => {
            debug_log!(SCHEDULE, "systemd user manager present");
            Backend::Systemd
        }
        Err(error) => {
            debug_log!(SCHEDULE, "no systemd user manager ({error}), falling back");
            Backend::InApp
        }
    }
}

fn unit_directory() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or(Error::NoConfigDirectory)?;
    Ok(base.join(SYSTEMD_USER_UNIT_DIR))
}

fn service_path() -> Result<PathBuf> {
    Ok(unit_directory()?.join(format!("{SCHEDULE_UNIT_NAME}.service")))
}

fn timer_path() -> Result<PathBuf> {
    Ok(unit_directory()?.join(format!("{SCHEDULE_UNIT_NAME}.timer")))
}

/// The service unit, which runs this binary in its scheduled mode.
fn service_unit(executable: &str, automatic: bool) -> String {
    let mode = if automatic { "--upgrade" } else { "--check" };
    format!(
        "# Written by cosmic-upgrader-gui. Changes here are replaced when the\n\
         # schedule is next applied from the application.\n\
         [Unit]\n\
         Description=Scheduled system upgrade check\n\
         Documentation={}\n\
         \n\
         [Service]\n\
         Type=oneshot\n\
         ExecStart={executable} --scheduled {mode}\n\
         # A long upgrade should not be killed halfway through a package\n\
         # transaction, so no timeout is imposed.\n\
         TimeoutStartSec=infinity\n",
        crate::constants::REPOSITORY_URL
    )
}

/// The timer unit.
fn timer_unit(schedule: Schedule) -> String {
    let schedule = schedule.normalized();
    format!(
        "# Written by cosmic-upgrader-gui. Changes here are replaced when the\n\
         # schedule is next applied from the application.\n\
         [Unit]\n\
         Description=Scheduled system upgrade check\n\
         \n\
         [Timer]\n\
         OnCalendar={}\n\
         # Run after a missed occurrence rather than skipping it, so a machine\n\
         # that was asleep at the appointed time still gets its check.\n\
         Persistent=true\n\
         # Spread the start over a few minutes so every machine configured this\n\
         # way does not hit the same mirrors at once.\n\
         RandomizedDelaySec=300\n\
         \n\
         [Install]\n\
         WantedBy=timers.target\n",
        schedule.frequency.on_calendar(schedule.hour, schedule.minute)
    )
}

/// Where the units live, which follows what they have to be able to do.
///
/// A run that only checks needs nothing special and belongs to the user. One
/// that *installs* cannot ask for a password — nobody is there — so it has to
/// already have the rights, which means a system service running as root.
/// Nothing else in this application runs as root, and the interface says so
/// before this is switched on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scope {
    User,
    System,
}

impl Scope {
    fn for_schedule(schedule: Schedule) -> Self {
        if schedule.automatic {
            Self::System
        } else {
            Self::User
        }
    }

    fn directory(self) -> Result<PathBuf> {
        match self {
            Self::User => unit_directory(),
            Self::System => Ok(PathBuf::from(SYSTEM_UNIT_DIR)),
        }
    }

    fn systemctl_scope(self) -> &'static str {
        match self {
            Self::User => "--user",
            Self::System => "--system",
        }
    }
}

/// Write the units and enable or disable the timer to match.
///
/// Both scopes are cleaned up whichever is in use, so turning automatic
/// installation on or off moves the schedule rather than leaving a second one
/// running behind it.
pub async fn apply(schedule: Schedule) -> Result<()> {
    let scope = Scope::for_schedule(schedule);
    // Whatever is not wanted goes first, so the two can never both be live.
    let other = match scope {
        Scope::User => Scope::System,
        Scope::System => Scope::User,
    };
    let _ = disable(other).await;

    match scope {
        Scope::User => apply_user(schedule).await,
        Scope::System => apply_system(schedule).await,
    }
}

/// Stop and forget the timer in a scope, ignoring one that was never there.
async fn disable(scope: Scope) -> Result<()> {
    let timer = format!("{SCHEDULE_UNIT_NAME}.timer");
    match scope {
        Scope::User => {
            let _ = systemctl(&["disable", "--now", &timer]).await;
        }
        Scope::System => {
            // Only worth an authentication prompt if something is actually
            // there to remove.
            if Path::new(SYSTEM_UNIT_DIR)
                .join(format!("{SCHEDULE_UNIT_NAME}.timer"))
                .exists()
            {
                let _ = privileged(&[
                    "systemctl",
                    Scope::System.systemctl_scope(),
                    "disable",
                    "--now",
                    &timer,
                ])
                .await;
            }
        }
    }
    Ok(())
}

/// Install the units as root, so an unattended run can install upgrades.
async fn apply_system(schedule: Schedule) -> Result<()> {
    let executable = current_executable();
    let service = service_unit(&executable, schedule.automatic);
    let timer = timer_unit(schedule);

    // Staged in the user's own space and then installed with one privileged
    // command each: writing directly as root would need a shell to redirect
    // into, and handing a shell a path is how quoting bugs become root bugs.
    let staged_service = stage(&service, "service")?;
    let staged_timer = stage(&timer, "timer")?;

    let directory = Scope::System.directory()?;
    let destination = |extension: &str| {
        directory
            .join(format!("{SCHEDULE_UNIT_NAME}.{extension}"))
            .display()
            .to_string()
    };

    privileged(&[
        "install",
        "-m",
        "0644",
        &staged_service.display().to_string(),
        &destination("service"),
    ])
    .await?;
    privileged(&[
        "install",
        "-m",
        "0644",
        &staged_timer.display().to_string(),
        &destination("timer"),
    ])
    .await?;

    let _ = std::fs::remove_file(&staged_service);
    let _ = std::fs::remove_file(&staged_timer);

    // The scope is named rather than relied on: `systemctl` under pkexec runs
    // as root and would default to the system manager anyway, but saying which
    // one is meant leaves no room for that default to change.
    let scope = Scope::System.systemctl_scope();
    privileged(&["systemctl", scope, "daemon-reload"]).await?;

    let unit = format!("{SCHEDULE_UNIT_NAME}.timer");
    if schedule.enabled {
        privileged(&["systemctl", scope, "enable", "--now", &unit]).await?;
        debug_log!(SCHEDULE, "system timer enabled: {schedule:?}");
    } else {
        privileged(&["systemctl", scope, "disable", "--now", &unit]).await?;
    }
    Ok(())
}

fn stage(contents: &str, extension: &str) -> Result<PathBuf> {
    let path = std::env::temp_dir().join(format!("{SCHEDULE_UNIT_NAME}.{extension}"));
    std::fs::write(&path, contents).map_err(|error| Error::Io {
        path: path.clone(),
        message: error.to_string(),
    })?;
    Ok(path)
}

fn current_executable() -> String {
    std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_owned())
}

/// Run one command as root through the desktop's authentication dialog.
async fn privileged(args: &[&str]) -> Result<()> {
    let output = Command::new(crate::constants::PKEXEC)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|error| Error::Systemctl {
            message: error.to_string(),
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    Err(Error::Systemctl {
        message: if stderr.is_empty() {
            match output.status.code() {
                Some(126) => "authentication was dismissed".to_owned(),
                Some(code) => format!("{args:?} exited with {code}"),
                None => format!("{args:?} was terminated"),
            }
        } else {
            stderr.to_owned()
        },
    })
}

async fn apply_user(schedule: Schedule) -> Result<()> {
    let directory = unit_directory()?;
    std::fs::create_dir_all(&directory).map_err(|error| Error::Io {
        path: directory.clone(),
        message: error.to_string(),
    })?;

    let executable = current_executable();
    write_unit(service_path()?, &service_unit(&executable, schedule.automatic))?;
    write_unit(timer_path()?, &timer_unit(schedule))?;

    // systemd caches unit files, so a rewritten timer is not seen until it is
    // told to look again.
    systemctl(&["daemon-reload"]).await?;

    let timer = format!("{SCHEDULE_UNIT_NAME}.timer");
    if schedule.enabled {
        systemctl(&["enable", "--now", &timer]).await?;
        debug_log!(SCHEDULE, "timer enabled: {schedule:?}");
    } else {
        // A disabled schedule leaves the units in place but stopped, so turning
        // it back on does not have to rewrite them.
        systemctl(&["disable", "--now", &timer]).await?;
        debug_log!(SCHEDULE, "timer disabled");
    }

    Ok(())
}

fn write_unit(path: PathBuf, contents: &str) -> Result<()> {
    std::fs::write(&path, contents).map_err(|error| Error::Io {
        path,
        message: error.to_string(),
    })
}

async fn systemctl(args: &[&str]) -> Result<()> {
    let output = Command::new("systemctl")
        .arg("--user")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|error| Error::Systemctl {
            message: error.to_string(),
        })?;

    if output.status.success() {
        return Ok(());
    }

    let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(Error::Systemctl {
        message: if message.is_empty() {
            format!("{args:?} exited with {}", output.status)
        } else {
            message
        },
    })
}

/// When the timer will next fire, as systemd reports it.
///
/// Read from systemd rather than worked out here, so what is shown is what will
/// actually happen — including the randomized delay and any catch-up for a
/// missed occurrence.
pub async fn next_run() -> Option<String> {
    let output = Command::new("systemctl")
        .args([
            "--user",
            "show",
            &format!("{SCHEDULE_UNIT_NAME}.timer"),
            "--property=NextElapseUSecRealtime",
            "--value",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await
        .ok()?;

    let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    // An inactive timer reports an empty value rather than failing.
    (!value.is_empty() && value != "n/a").then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hourly_repeats_every_hour_at_the_chosen_minute() {
        assert_eq!(Frequency::Hourly.on_calendar(18, 30), "*-*-* *:30:00");
    }

    #[test]
    fn daily_uses_the_chosen_time() {
        assert_eq!(Frequency::Daily.on_calendar(18, 5), "*-*-* 18:05:00");
    }

    #[test]
    fn weekly_and_monthly_pin_a_day() {
        assert_eq!(Frequency::Weekly.on_calendar(9, 0), "Mon *-*-* 09:00:00");
        assert_eq!(Frequency::Monthly.on_calendar(9, 0), "*-*-01 09:00:00");
    }

    #[test]
    fn times_are_zero_padded_as_systemd_requires() {
        let expression = Frequency::Daily.on_calendar(7, 5);
        assert_eq!(expression, "*-*-* 07:05:00");
    }

    #[test]
    fn an_out_of_range_time_is_clamped_rather_than_written_out() {
        let schedule = Schedule {
            hour: 99,
            minute: 61,
            ..Schedule::default()
        }
        .normalized();
        assert_eq!((schedule.hour, schedule.minute), (23, 59));
    }

    #[test]
    fn the_timer_catches_up_after_a_missed_occurrence() {
        let unit = timer_unit(Schedule::default());
        assert!(unit.contains("Persistent=true"), "{unit}");
        assert!(unit.contains("WantedBy=timers.target"), "{unit}");
    }

    #[test]
    fn the_service_distinguishes_checking_from_upgrading() {
        let checking = service_unit("/usr/bin/app", false);
        assert!(checking.contains("--scheduled --check"), "{checking}");

        let upgrading = service_unit("/usr/bin/app", true);
        assert!(upgrading.contains("--scheduled --upgrade"), "{upgrading}");
    }

    #[test]
    fn a_long_upgrade_is_not_killed_by_a_timeout() {
        let unit = service_unit("/usr/bin/app", true);
        assert!(unit.contains("TimeoutStartSec=infinity"), "{unit}");
    }

    #[test]
    fn a_schedule_that_has_never_run_is_not_immediately_due() {
        // Otherwise switching the option on would start an upgrade at once.
        let schedule = Schedule {
            enabled: true,
            ..Schedule::default()
        };
        assert!(!schedule.is_due(0, 1_000_000));
    }

    #[test]
    fn a_run_becomes_due_once_the_interval_has_passed() {
        let schedule = Schedule {
            enabled: true,
            frequency: Frequency::Daily,
            ..Schedule::default()
        };
        let day: i64 = 24 * 60 * 60;
        assert!(!schedule.is_due(1_000_000, 1_000_000 + day - 1));
        assert!(schedule.is_due(1_000_000, 1_000_000 + day));
    }

    #[test]
    fn a_disabled_schedule_is_never_due() {
        let schedule = Schedule {
            enabled: false,
            ..Schedule::default()
        };
        assert!(!schedule.is_due(1, i64::MAX));
    }

    #[test]
    fn a_clock_that_went_backwards_does_not_make_a_run_due() {
        let schedule = Schedule {
            enabled: true,
            ..Schedule::default()
        };
        assert!(!schedule.is_due(2_000_000, 1_000_000));
    }

    #[test]
    fn a_checking_schedule_belongs_to_the_user() {
        let checking = Schedule {
            enabled: true,
            automatic: false,
            ..Schedule::default()
        };
        assert_eq!(Scope::for_schedule(checking), Scope::User);
        assert_eq!(Scope::for_schedule(checking).systemctl_scope(), "--user");
    }

    #[test]
    fn an_installing_schedule_needs_the_system_scope() {
        // Nobody is present to type a password, so the run has to already have
        // the rights.
        let installing = Schedule {
            enabled: true,
            automatic: true,
            ..Schedule::default()
        };
        assert_eq!(Scope::for_schedule(installing), Scope::System);
        assert_eq!(
            Scope::for_schedule(installing).directory().unwrap(),
            PathBuf::from(SYSTEM_UNIT_DIR)
        );
    }

    #[test]
    fn automatic_upgrades_are_off_unless_asked_for() {
        assert!(!Schedule::default().automatic);
        assert!(!Schedule::default().enabled);
    }
}
