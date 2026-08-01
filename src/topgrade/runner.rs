// SPDX-License-Identifier: GPL-3.0

//! Driving an actual upgrade and reporting what happens.
//!
//! topgrade is a terminal program, and not an incidentally terminal one. It asks
//! what to do after a step fails, the package managers it drives ask for a sudo
//! password, and most of them decide whether to draw a progress bar by asking
//! whether their output is a terminal. Run behind an ordinary pipe, all of that
//! goes wrong at once: the password prompt is written to a stream nobody is
//! reading, the run stalls with no visible reason, and the output arrives as an
//! undifferentiated block at the end.
//!
//! So topgrade is given a real pseudo-terminal. It behaves exactly as it does
//! in a terminal window, and this module sits on the other end reading what it
//! writes and typing back when asked.
//!
//! ## Privileges
//!
//! The `system` step runs the distribution's package manager under `sudo`,
//! which needs a password this application does not have. There are two ways to
//! deal with that, and the user picks:
//!
//! * **Ask in this window.** `sudo` prompts on the pseudo-terminal, the prompt
//!   is recognised here, and the interface asks for the password and types it
//!   back. Nothing is stored and the password never reaches this process's own
//!   memory for longer than it takes to forward it.
//! * **System dialog.** topgrade's own `misc.sudo_command` is set to `pkexec`,
//!   so the desktop's polkit agent asks instead.
//!
//! The second is arranged by writing that key into topgrade's configuration
//! rather than by passing anything here, because topgrade has no command-line
//! equivalent for it, and because an `[include]` file cannot be used to override
//! it — included files take precedence over the file that includes them, which
//! is the opposite of what that arrangement would need. Writing the real key is
//! also the honest option: it is visible on the configuration page alongside
//! every other setting, and it applies equally when the user runs topgrade
//! themselves.

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tokio::sync::mpsc;

use super::discover::StepId;
use super::probe::{parse_summary_line, Component, Status};
use super::Topgrade;
use crate::constants::{
    PASSWORD_PROMPT_MARKERS, PTY_COLS, PTY_ROWS, SUMMARY_HEADING,
};
use crate::debug::RUN;
use crate::debug_log;

/// What to run.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Print what would be done without doing it.
    pub dry_run: bool,
    /// Restrict the run to these steps. Empty means every step the
    /// configuration leaves enabled, which is topgrade's own behaviour.
    pub only: Vec<StepId>,
    /// Answer package managers' prompts affirmatively.
    ///
    /// Without this an unattended run stops at the first "do you want to
    /// continue?" and waits for a keypress that is not coming.
    pub assume_yes: bool,
}

impl Options {
    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.dry_run {
            args.push("--dry-run".to_owned());
        }
        if self.assume_yes {
            args.push("--yes".to_owned());
        }

        // Failures are reported rather than negotiated: the interactive
        // "retry/skip/quit?" prompt has no good answer in a window that is
        // showing a progress list, and the summary names what failed either way.
        args.push("--no-ask-retry".to_owned());
        // Skip reasons are what the interface shows against each step.
        args.push("--show-skipped".to_owned());
        // topgrade posts its own notification when a run ends. This application
        // posts a better one — it knows whether the run was scheduled, and names
        // the steps that failed rather than only that something did — so two
        // would be one too many.
        args.push("--notify-end".to_owned());
        args.push("never".to_owned());

        if !self.only.is_empty() {
            args.push("--only".to_owned());
            args.extend(self.only.iter().map(|id| id.as_str().to_owned()));
        }

        args
    }
}

/// Something that happened during a run.
#[derive(Clone, Debug)]
pub enum Event {
    /// A step heading was printed. Carries topgrade's display name for it,
    /// which is what the interface shows as the current activity.
    StepStarted(String),
    /// One line of output, with terminal escapes removed.
    Output(String),
    /// Something is asking for a password, and the run is stopped until one is
    /// sent back through [`Handle::send_password`].
    PasswordRequested { prompt: String },
    /// The run ended.
    Finished(Outcome),
}

/// How a run ended.
#[derive(Clone, Debug)]
pub struct Outcome {
    /// Whether topgrade exited cleanly.
    pub success: bool,
    /// Whether it was stopped from the interface rather than finishing.
    pub cancelled: bool,
    /// The per-step results from topgrade's closing summary.
    pub components: Vec<Component>,
}

impl Outcome {
    pub fn counts(&self) -> (usize, usize, usize) {
        let mut ok = 0;
        let mut skipped = 0;
        let mut failed = 0;
        for component in &self.components {
            match component.status {
                Status::Ok => ok += 1,
                Status::Skipped => skipped += 1,
                Status::Failed => failed += 1,
            }
        }
        (ok, skipped, failed)
    }
}

/// A running upgrade.
///
/// Dropping this does not stop the run — an upgrade halfway through a package
/// transaction should not be killed because a window closed. [`cancel`](Self::cancel)
/// is explicit for that reason.
pub struct Handle {
    events: mpsc::UnboundedReceiver<Event>,
    /// Writes to the pseudo-terminal, for answering prompts.
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
}

impl Handle {
    /// Take the next event, or `None` once the run has ended and every event has
    /// been delivered.
    pub async fn next_event(&mut self) -> Option<Event> {
        self.events.recv().await
    }

    /// Answer a [`PasswordRequested`](Event::PasswordRequested).
    ///
    /// The newline matters: `sudo` reads a line, and without it the password
    /// sits in the terminal's buffer and the run stays stopped.
    pub fn send_password(&self, password: &str) {
        let Ok(mut writer) = self.writer.lock() else {
            return;
        };
        if let Err(error) = writeln!(writer, "{password}") {
            debug_log!(RUN, "could not send password: {error}");
        }
        let _ = writer.flush();
        debug_log!(RUN, "password sent");
    }

    /// Stop the run.
    pub fn cancel(&self) {
        debug_log!(RUN, "cancelling");
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
    }
}

/// Start an upgrade.
pub fn start(topgrade: &Topgrade, options: &Options) -> std::io::Result<Handle> {
    let pty = native_pty_system();
    let pair = pty
        .openpty(PtySize {
            rows: PTY_ROWS,
            cols: PTY_COLS,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(std::io::Error::other)?;

    let mut command = CommandBuilder::new(topgrade.path());
    for arg in options.to_args() {
        command.arg(arg);
    }
    // Without a sensible TERM the tools topgrade drives fall back to their
    // dumbest output mode — which is where PowerShell's "Cannot detect
    // PowerShell in a dumb terminal" comes from.
    command.env("TERM", "xterm-256color");
    command.env("COLUMNS", PTY_COLS.to_string());
    command.env("LINES", PTY_ROWS.to_string());
    if let Some(home) = std::env::var_os("HOME") {
        command.env("HOME", home);
    }
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    command.cwd(std::env::var_os("HOME").unwrap_or_else(|| "/".into()));

    debug_log!(RUN, "starting topgrade with {:?}", options.to_args());

    let child = pair
        .slave
        .spawn_command(command)
        .map_err(std::io::Error::other)?;
    // The slave side has been handed to the child. Holding our copy open would
    // keep the pseudo-terminal from reporting end-of-file when the child exits,
    // and the reader below would block for ever.
    drop(pair.slave);

    let reader = pair
        .master
        .try_clone_reader()
        .map_err(std::io::Error::other)?;
    let writer = pair.master.take_writer().map_err(std::io::Error::other)?;

    let (events, receiver) = mpsc::unbounded_channel();
    let child = Arc::new(Mutex::new(child));

    // Reading a pseudo-terminal is a blocking operation with no async
    // equivalent, so it gets a thread of its own rather than occupying one of
    // the runtime's workers indefinitely.
    let reader_child = Arc::clone(&child);
    std::thread::Builder::new()
        .name("topgrade-reader".to_owned())
        .spawn(move || read_output(reader, &events, &reader_child))?;

    Ok(Handle {
        events: receiver,
        writer: Arc::new(Mutex::new(writer)),
        child,
    })
}

/// Consume the pseudo-terminal until the run ends, reporting as it goes.
fn read_output(
    mut reader: Box<dyn Read + Send>,
    events: &mpsc::UnboundedSender<Event>,
    child: &Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
) {
    let mut buffer = [0u8; 4096];
    // Output arrives in arbitrary chunks, so a line can be split across reads
    // and has to be carried over.
    let mut pending = String::new();
    let mut transcript = String::new();
    let mut in_summary = false;

    loop {
        let read = match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => read,
            Err(error) => {
                debug_log!(RUN, "read failed: {error}");
                break;
            }
        };

        pending.push_str(&String::from_utf8_lossy(&buffer[..read]));

        // Complete lines are reported; whatever is left is a partial line.
        while let Some(newline) = pending.find('\n') {
            let line: String = pending.drain(..=newline).collect();
            let line = strip_ansi(line.trim_end_matches(['\r', '\n']));

            transcript.push_str(&line);
            transcript.push('\n');

            if let Some(name) = super::heading_name(&line) {
                if name == SUMMARY_HEADING {
                    in_summary = true;
                } else {
                    let _ = events.send(Event::StepStarted(name.to_owned()));
                }
            }

            let _ = events.send(Event::Output(line));
        }

        // A password prompt is written without a trailing newline — that is how
        // it leaves the cursor after the colon — so it never becomes a complete
        // line and would be missed entirely by the loop above.
        if !pending.is_empty() {
            let partial = strip_ansi(&pending);
            if is_password_prompt(&partial) {
                debug_log!(RUN, "password prompt: {partial:?}");
                let _ = events.send(Event::PasswordRequested {
                    prompt: partial.trim().to_owned(),
                });
                // Cleared so the same prompt is not reported again on the next
                // read; the reply arrives through the writer, not through here.
                pending.clear();
            }
        }

        let _ = in_summary;
    }

    // Anything left without a trailing newline is still output worth showing.
    if !pending.trim().is_empty() {
        let line = strip_ansi(pending.trim_end());
        transcript.push_str(&line);
        transcript.push('\n');
        let _ = events.send(Event::Output(line));
    }

    let status = child.lock().ok().and_then(|mut child| child.wait().ok());
    let success = status.map(|status| status.success()).unwrap_or(false);

    // The summary is parsed from the transcript rather than accumulated as the
    // lines went past, so the same code reads it here and in a capability probe.
    let (_, summary) = super::probe::split_at_summary(&transcript);
    let components: Vec<Component> = summary.lines().filter_map(parse_summary_line).collect();

    debug_log!(
        RUN,
        "finished: success={success}, {} components",
        components.len()
    );

    let _ = events.send(Event::Finished(Outcome {
        success,
        // A killed child exits unsuccessfully, and there is no way to tell that
        // apart from a failed run at this level. The interface knows whether it
        // asked for a cancellation and corrects this.
        cancelled: false,
        components,
    }));
}

/// Whether a partial line looks like something waiting for a password.
fn is_password_prompt(partial: &str) -> bool {
    let tail = partial.trim();
    if tail.is_empty() {
        return false;
    }
    // Only the last line matters: earlier ones are output that has already been
    // read past, and matching them would fire on a step that merely printed the
    // word.
    let last = tail.lines().next_back().unwrap_or(tail).to_ascii_lowercase();
    PASSWORD_PROMPT_MARKERS
        .iter()
        .any(|marker| last.contains(marker))
}

/// Remove terminal escape sequences.
///
/// The output is shown in a text view rather than a terminal emulator, so
/// colour and cursor-movement sequences would appear as literal noise. Only the
/// text is kept.
fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            // A carriage return is how progress bars redraw a line in place.
            // Keeping only what follows the last one leaves the final state,
            // which is the useful one.
            if c == '\r' {
                out.clear();
            } else {
                out.push(c);
            }
            continue;
        }

        match chars.next() {
            // Control Sequence Introducer: parameters, then a letter that ends it.
            Some('[') => {
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() || c == '@' || c == '~' {
                        break;
                    }
                }
            }
            // Operating System Command: runs until BEL or ESC \.
            Some(']') => {
                while let Some(c) = chars.next() {
                    if c == '\u{7}' {
                        break;
                    }
                    if c == '\u{1b}' && chars.peek() == Some(&'\\') {
                        chars.next();
                        break;
                    }
                }
            }
            // Two-character sequences, already consumed.
            _ => {}
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_summary_heading_is_recognised_through_the_shared_parser() {
        assert_eq!(
            super::super::heading_name("―― 17:11:57 - Summary ――"),
            Some(SUMMARY_HEADING)
        );
    }

    #[test]
    fn a_title_sequence_before_a_heading_is_stripped_away() {
        // Under a pseudo-terminal topgrade sets the window title on the same
        // line as the heading, so the escape has to go before it can be read.
        let raw = "\u{1b}]0;Topgrade - Cargo\u{7}―― 18:59:41 - Cargo ――";
        assert_eq!(super::super::heading_name(&strip_ansi(raw)), Some("Cargo"));
    }

    #[test]
    fn recognises_a_sudo_prompt() {
        assert!(is_password_prompt("[sudo] password for dave: "));
        assert!(is_password_prompt("Password:"));
    }

    #[test]
    fn matches_only_the_last_line_of_the_pending_buffer() {
        // Output that mentioned a password earlier must not keep matching.
        assert!(!is_password_prompt(
            "Changing the password for nobody\nUpgrading packages"
        ));
    }

    #[test]
    fn ordinary_output_is_not_a_password_prompt() {
        assert!(!is_password_prompt("Upgrading 42 packages"));
        assert!(!is_password_prompt(""));
    }

    #[test]
    fn strips_colour_sequences() {
        assert_eq!(strip_ansi("\u{1b}[32mOK\u{1b}[0m"), "OK");
        assert_eq!(strip_ansi("\u{1b}[1;31mFAILED\u{1b}[0m: nope"), "FAILED: nope");
    }

    #[test]
    fn strips_window_title_sequences() {
        assert_eq!(strip_ansi("\u{1b}]0;topgrade\u{7}done"), "done");
    }

    #[test]
    fn keeps_the_final_state_of_a_redrawn_progress_line() {
        assert_eq!(strip_ansi("10%\r50%\r100%"), "100%");
    }

    #[test]
    fn leaves_plain_text_untouched() {
        assert_eq!(strip_ansi("cargo: OK"), "cargo: OK");
    }

    #[test]
    fn topgrades_own_end_of_run_notification_is_turned_off() {
        // This application posts its own, which knows more.
        let args = Options::default().to_args();
        let at = args.iter().position(|a| a == "--notify-end").expect("--notify-end");
        assert_eq!(args.get(at + 1).map(String::as_str), Some("never"));
    }

    #[test]
    fn builds_arguments_that_will_not_stall_an_unattended_run() {
        let args = Options {
            assume_yes: true,
            ..Options::default()
        }
        .to_args();
        assert!(args.contains(&"--yes".to_owned()));
        assert!(args.contains(&"--no-ask-retry".to_owned()));
    }

    #[test]
    fn restricting_to_steps_passes_them_after_one_flag() {
        // `--only` takes several values, so each step is its own argument and
        // there is exactly one flag.
        let args = Options {
            only: vec![StepId::new("cargo"), StepId::new("flatpak")],
            ..Options::default()
        }
        .to_args();
        let position = args.iter().position(|a| a == "--only").expect("--only");
        assert_eq!(&args[position + 1..position + 3], ["cargo", "flatpak"]);
        assert_eq!(args.iter().filter(|a| *a == "--only").count(), 1);
    }

    #[test]
    fn an_unrestricted_run_does_not_pass_only() {
        let args = Options::default().to_args();
        assert!(!args.contains(&"--only".to_owned()));
    }

    #[test]
    fn counts_the_summary_by_status() {
        let outcome = Outcome {
            success: true,
            cancelled: false,
            components: vec![
                Component {
                    name: "cargo".into(),
                    status: Status::Ok,
                    reason: None,
                },
                Component {
                    name: "vim".into(),
                    status: Status::Skipped,
                    reason: Some("not installed".into()),
                },
                Component {
                    name: "system".into(),
                    status: Status::Failed,
                    reason: Some("exit 1".into()),
                },
            ],
        };
        assert_eq!(outcome.counts(), (1, 1, 1));
    }
}

/// Checks that run against the topgrade actually installed.
///
/// Ignored by default, since they need a topgrade on the machine and take a
/// second to run:
///
/// ```sh
/// cargo test -- --ignored live_
/// ```
///
/// Worth keeping despite that. The heading format differs between piped output
/// and a pseudo-terminal, and the tests above — which work from captured piped
/// output — cannot see the difference. This is what caught it.
#[cfg(test)]
mod live_tests {
    use super::*;

    /// Drives a real topgrade dry-run through the pseudo-terminal and checks
    /// that output arrives as separate events, that step headings are
    /// recognised, and that the closing summary is parsed.
    #[tokio::test]
    #[ignore]
    async fn live_dry_run_emits_output_and_a_summary() {
        let Ok(topgrade) = Topgrade::locate().await else {
            eprintln!("topgrade not installed; skipping");
            return;
        };

        let options = Options {
            dry_run: true,
            only: vec![StepId::new("cargo"), StepId::new("flatpak")],
            assume_yes: true,
        };

        let mut handle = start(&topgrade, &options).expect("run should start");
        let mut lines = 0;
        let mut steps = Vec::new();
        let mut outcome = None;

        while let Some(event) = handle.next_event().await {
            match event {
                Event::Output(line) => {
                    lines += 1;
                    println!("out: {line}");
                }
                Event::StepStarted(name) => steps.push(name),
                Event::Finished(finished) => outcome = Some(finished),
                Event::PasswordRequested { prompt } => panic!("unexpected prompt: {prompt}"),
            }
        }

        let outcome = outcome.expect("a Finished event");
        println!("lines={lines} steps={steps:?} components={:?}", outcome.components);
        assert!(lines > 0, "no output was captured");
        assert!(!steps.is_empty(), "no step headings were seen");
        assert!(!outcome.components.is_empty(), "summary was not parsed");
    }
}
