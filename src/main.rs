// SPDX-License-Identifier: GPL-3.0

//! A COSMIC Desktop front-end for topgrade.
//!
//! Shows what topgrade can upgrade on this machine, lets each step be turned on
//! or off, edits topgrade's own configuration, runs an upgrade with live
//! progress, and keeps a schedule for unattended runs.

mod app;
mod autostart;
mod clamav;
mod config;
mod constants;
mod debug;
mod dependencies;
mod history;
mod i18n;
mod notify;
mod releases;
mod tray;
mod schedule;
mod topgrade;

use cosmic::app::Settings;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::Limits;
use cosmic::Application;

use app::{App, Flags};
use config::{Config, CONFIG_VERSION};
use constants::{WINDOW_HEIGHT, WINDOW_MIN_HEIGHT, WINDOW_MIN_WIDTH, WINDOW_WIDTH};

fn main() -> cosmic::iced::Result {
    // The system's preferred languages, so the UI comes up localized.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);

    let (config_handler, config) = load_config();

    // The scheduled units run this same binary rather than topgrade directly,
    // so the unattended path goes through the same configuration and reporting
    // as a run started from the window.
    if let Some(mode) = scheduled_mode() {
        if let Err(error) = run_scheduled(mode, &config) {
            eprintln!("scheduled run failed: {error}");
            std::process::exit(1);
        }
        return Ok(());
    }

    let settings = Settings::default()
        .theme(config.app_theme.theme())
        .size_limits(
            Limits::NONE
                .min_width(WINDOW_MIN_WIDTH)
                .min_height(WINDOW_MIN_HEIGHT),
        )
        .size(cosmic::iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT));

    cosmic::app::run::<App>(
        settings,
        Flags {
            config_handler,
            config,
        },
    )
}

/// What a scheduled invocation was asked to do.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScheduledMode {
    /// Report what is available without changing anything.
    Check,
    /// Install what is available.
    Upgrade,
}

/// Read `--scheduled --check` / `--scheduled --upgrade` off the command line.
///
/// Deliberately minimal rather than a full argument parser: these are written
/// by this application into its own systemd units and are not a documented
/// interface for anyone else.
fn scheduled_mode() -> Option<ScheduledMode> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.iter().any(|arg| arg == "--scheduled") {
        return None;
    }
    Some(if args.iter().any(|arg| arg == "--upgrade") {
        ScheduledMode::Upgrade
    } else {
        ScheduledMode::Check
    })
}

/// Run without a window, for a systemd timer.
fn run_scheduled(mode: ScheduledMode, config: &Config) -> Result<(), String> {
    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;

    runtime.block_on(async move {
        let topgrade = topgrade::Topgrade::locate()
            .await
            .map_err(|error| error.to_string())?;

        let options = topgrade::runner::Options {
            dry_run: mode == ScheduledMode::Check,
            only: Vec::new(),
            // Nobody is present to answer a package manager's prompt.
            assume_yes: true,
        };

        let mut handle =
            topgrade::runner::start(&topgrade, &options).map_err(|error| error.to_string())?;

        let started = unix_now();
        let mut recorder =
            history::Recorder::start(history::Origin::Scheduled, options.dry_run, started);

        let mut outcome = None;
        while let Some(event) = handle.next_event().await {
            match event {
                topgrade::runner::Event::Output(line) => {
                    // Recorded first: the journal may not be persistent — it is
                    // not on every system — and the run record is then the only
                    // account of what happened.
                    if let Some(recorder) = recorder.as_mut() {
                        recorder.write_line(&line);
                    }
                    println!("{line}");
                }
                topgrade::runner::Event::Finished(finished) => outcome = Some(finished),
                // Nobody is here to answer. Saying so is more use than feeding
                // sudo an empty string, which would spend one of its attempts
                // and log an authentication failure.
                topgrade::runner::Event::PasswordRequested { prompt } => {
                    eprintln!("a password was requested but nobody is present: {prompt}");
                }
                topgrade::runner::Event::StepStarted(_) => {}
            }
        }

        let components = outcome
            .as_ref()
            .map(|outcome| outcome.components.clone())
            .unwrap_or_default();

        if let Some(recorder) = recorder {
            if let Some(record) = recorder.finish(&components, false, unix_now()) {
                // Nobody watched this run, so it reports either way rather than
                // only on failure — that is the whole point of a scheduled one.
                notify::run_finished(
                    &record,
                    notify::Policy {
                        upgrades: config.notify_upgrades,
                        errors: config.notify_errors,
                        installs: mode == ScheduledMode::Upgrade,
                        // Nobody watched this run, so a success is worth
                        // reporting rather than only a failure.
                        on_screen: false,
                    },
                );
                history::prune(config.keep_run_logs);

                if record.failed > 0 {
                    // A non-zero exit makes the failure visible to systemd, so
                    // `systemctl --user status` and `is-failed` agree with what
                    // the notification said.
                    return Err(format!("{} step(s) failed", record.failed));
                }
            }
        }

        Ok(())
    })
}

/// Seconds since the Unix epoch.
fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

/// Load the persisted configuration, falling back to defaults.
///
/// A config that fails to load is not fatal: the application starts with
/// defaults and reports the problem, rather than refusing to open.
fn load_config() -> (Option<cosmic_config::Config>, Config) {
    match cosmic_config::Config::new(App::APP_ID, CONFIG_VERSION) {
        Ok(handler) => {
            let config = match Config::get_entry(&handler) {
                Ok(config) => config,
                Err((errors, config)) => {
                    eprintln!("errors loading configuration: {errors:?}");
                    config
                }
            };
            (Some(handler), config)
        }
        Err(error) => {
            eprintln!("failed to create the configuration handler: {error}");
            (None, Config::default())
        }
    }
}
