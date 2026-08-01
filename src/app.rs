// SPDX-License-Identifier: GPL-3.0

//! The window: what is shown, and what happens when it is used.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::Arc;

use cosmic::app::{context_drawer, Core, Task};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::widget::scrollable;
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::widget::{self, nav_bar};
use cosmic::{Application, ApplicationExt, Apply, Element};
use tokio::sync::Mutex;

use crate::autostart;
use crate::config::{AppTheme, Config, PrivilegeMode};
use crate::constants::{
    APP_ID, FALLBACK_SCHEDULER_TICK, ICON_SIZE_ROW, MAX_CONTENT_WIDTH, RUN_LOG_MAX_LINES,
};
use crate::debug::UI;
use crate::debug_log;
use crate::fl;
use crate::history::{self, Origin, Outcome as RunOutcome, Recorder};
use crate::dependencies::{self, Report, Requirement};
use crate::repos::{self, Repository};
use crate::releases::{
    self, detect::Candidate, Channel, CheckInterval, Status as ReleaseStatus, Watch,
};
use crate::schedule::{self, Backend, Frequency};
use crate::tray;
use crate::topgrade::{
    categories::Category,
    discover::StepId,
    probe::{Availability, Capabilities, Status},
    runner,
    schema::{Schema, Setting, ValueKind},
    settings_file::{SettingValue, SettingsFile},
    Source, Topgrade,
};

/// Theme choices, paired with `App::theme_labels` by index.
const THEME_OPTIONS: [AppTheme; 3] = [AppTheme::System, AppTheme::Light, AppTheme::Dark];

/// Privilege choices, paired with `App::privilege_labels` by index.
const PRIVILEGE_OPTIONS: [PrivilegeMode; 2] =
    [PrivilegeMode::AskInWindow, PrivilegeMode::SystemDialog];

/// How close to the bottom counts as "following" the log.
///
/// Not an exact 1.0: the offset is a ratio of floats, and a view sitting at the
/// bottom can report a hair under it, which would silently switch following off
/// and leave the log looking stuck.
const LOG_FOLLOW_THRESHOLD: f32 = 0.99;

/// How far the log may drift back before it counts as the user scrolling up
/// rather than the view settling, in pixels.
///
/// Comfortably under one notch of a mouse wheel, so a genuine scroll is caught,
/// and above the sub-pixel movement that rounding produces.
const LOG_SCROLL_TOLERANCE: f32 = 8.0;

/// topgrade's key for the step exclusion list.
///
/// Named here because it is deliberately *not* shown on the configuration page:
/// the step toggles already edit it, and offering a second control for the same
/// key would let the two disagree in front of the user.
const DISABLE_KEY: (&str, &str) = ("misc", "disable");

/// topgrade's key for the command used to gain administrator rights.
const SUDO_COMMAND_KEY: (&str, &str) = ("misc", "sudo_command");

/// Which page the sidebar has selected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Page {
    /// Shown until the first-run choices have been made, then gone.
    Welcome,
    Overview,
    Steps(Category),
    Run,
    Releases,
    /// Where packages come from — deliberately not called "Repositories", which
    /// in this application already means the git repositories topgrade pulls.
    Sources,
    History,
    Dependencies,
    Schedule,
    Configuration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextPage {
    Settings,
    About,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogPage {
    /// An upgrade changes the system and cannot be cleanly undone, so it is
    /// worth one confirmation.
    ConfirmRun,
    /// Something in the run is waiting for a password.
    Password { prompt: String },
}

pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
}

#[derive(Clone, Debug)]
pub enum Message {
    None,

    /// Startup finished, one way or the other.
    Loaded(Box<Result<Loaded, String>>),
    /// A capability scan reported progress or finished.
    ScanProgress(usize, usize, StepId),
    Scanned(Box<Capabilities>),
    Rescan,

    SelectPage(Page),
    ToggleNavBar,
    ToggleContextPage(ContextPage),
    DialogCancel,
    LaunchUrl(String),

    /// Include or exclude a step.
    ToggleStep(StepId, bool),
    SetCategoryEnabled(Category, bool),

    /// Begin an upgrade, possibly after confirming.
    RequestRun { dry_run: bool },
    StartRun { dry_run: bool },
    RunEvent(Box<runner::Event>),
    RunPumpEnded,
    /// The run log was scrolled, by the user or by the follow behaviour.
    RunLogScrolled(scrollable::Viewport),
    CancelRun,
    ClearLog,
    PasswordInput(String),
    PasswordSubmit,
    TogglePasswordVisible,

    /// A topgrade configuration value changed.
    EditSetting(String, String, SettingValue),
    EditText(String, String, String),
    SaveSettings,
    SettingsSaved(Box<Result<(), String>>),

    ScheduleEnabled(bool),
    ScheduleFrequency(usize),
    ScheduleHour(usize),
    ScheduleMinute(usize),
    ScheduleAutomatic(bool),
    ScheduleApply,
    /// The in-app fallback scheduler ticked.
    SchedulerTick,

    /// A past run was selected in the history.
    SelectHistory(String),
    HistoryTranscript(Box<Result<String, String>>),
    DeleteHistory(String),
    RefreshHistory,

    DiscoverProjects,
    ProjectsDiscovered(Vec<Candidate>),
    ToggleCandidate(usize, bool),
    AddSelectedWatches,
    CancelDiscovery,
    RemoveWatch(String),
    CheckReleases,
    ReleaseChecked(Box<ReleaseStatus>),
    ReleasesChecked,
    InstallRelease(String),
    ReleaseInstalled(Box<Result<(String, String), String>>),

    TrayStarted(Option<TrayHandles>),
    TrayCommand(Option<tray::Command>),
    ShowWindow,
    Quit,

    ToggleCategorySettings(Category),
    DraftCommandName(String, String),
    DraftCommandValue(String, String),
    AddCommand(String),
    EditCommand(String, String, String),
    RemoveCommand(String, String),

    FinishWelcome,
    ShowWelcome,
    ConfigNotifyUpgrades(bool),
    ConfigNotifyErrors(bool),
    ConfigClamavScan(bool),
    ScanFinished(Box<Result<crate::clamav::Report, String>>),
    ConfigClamscanOptions(String),
    ConfigClamscanTarget(String),
    ConfigAutostart(bool),
    ConfigShowTrayIcon(bool),
    ScheduleApplied(Box<Result<Option<String>, String>>),

    ConfigTheme(AppTheme),
    ConfigPrivilege(PrivilegeMode),
    ConfigConfirmBeforeRunning(bool),
    ConfigAssumeYes(bool),
    ConfigShowUnavailable(bool),
    ConfigCheckInterval(usize),
    ConfigChannel(usize),
    AddDirectory,
    DraftDirectory(String),
    RemoveDirectory(String),
    LoadSources,
    SourcesLoaded(Vec<Repository>),
    ToggleSource(String, bool),
    RemoveSource(String),
    AddSource(repos::Kind),
    DraftSource(usize, String),
    SourceChanged(Box<Result<(), String>>),
    RecheckDependencies,
    InstallDependency(String),
    DependencyInstalled(Box<Result<String, (String, String)>>),
    ConfigUpdated(Config),
}

/// A started status-area item and the channel it reports through.
///
/// Wrapped in one type so it can travel in a message; neither half is useful
/// without the other.
#[derive(Clone)]
pub struct TrayHandles {
    tray: Arc<tray::Tray>,
    commands: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<tray::Command>>>,
}

impl std::fmt::Debug for TrayHandles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TrayHandles")
    }
}

/// Everything discovered at startup.
#[derive(Clone, Debug)]
pub struct Loaded {
    pub topgrade: Topgrade,
    pub steps: Vec<StepId>,
    pub schema: Schema,
    pub backend: Backend,
    pub next_run: Option<String>,
    pub client: releases::Client,
}

/// What the window is doing.
enum State {
    Loading,
    /// topgrade could not be used at all. The only thing worth showing is why.
    Unusable(String),
    Ready(Box<Ready>),
}

struct Ready {
    topgrade: Topgrade,
    steps: Vec<StepId>,
    grouped: BTreeMap<Category, Vec<StepId>>,
    capabilities: Capabilities,
    schema: Schema,
    settings: SettingsFile,
    backend: Backend,
    next_run: Option<String>,
    /// Progress through a capability scan, while one is running: how many
    /// steps are done, how many there are, and the one just finished.
    scanning: Option<(usize, usize, StepId)>,
    /// In-progress text edits, keyed by `section.key`.
    ///
    /// Text and number fields need somewhere to hold a value that is not yet
    /// valid — an empty number box mid-edit — without that reaching the
    /// document and being written out.
    edits: HashMap<String, String>,
    status: Option<String>,
    /// Past runs, newest first. Re-read after each run rather than appended to,
    /// so a run recorded by a scheduled invocation while the window was open
    /// shows up too.
    history: Vec<history::Record>,
    /// The run whose transcript is on screen, and the transcript itself.
    viewing: Option<(String, String)>,
    /// Categories whose settings panel is expanded.
    category_settings: HashSet<Category>,
    /// Half-typed new commands, keyed by section: the name and the command.
    drafts: HashMap<String, (String, String)>,
    /// What is available for talking to forges.
    client: releases::Client,
    /// Candidates from the last discovery, with whether each is selected.
    /// `None` when the picker is not open.
    candidates: Option<Vec<(Candidate, bool)>>,
    discovering: bool,
    /// The result of the last release check, keyed by `host/path`.
    statuses: HashMap<String, ReleaseStatus>,
    /// Progress through a check, while one is running.
    checking: Option<(usize, usize)>,
    /// The project currently being downloaded and installed.
    installing: Option<String>,
    /// What was found when the tools this application drives were last checked.
    deps: Vec<Report>,
    /// The dependency currently being installed.
    installing_dep: Option<String>,
    /// Half-typed directory to add to the search list.
    directory_draft: String,
    /// Where packages come from, as last read.
    sources: Vec<Repository>,
    /// Half-typed new source: name, then URL, then suite.
    source_draft: (String, String, String),
    /// The source currently being changed, so its row can say so.
    changing_source: Option<String>,
}

/// A run in progress or just finished.
struct Run {
    handle: Arc<Mutex<runner::Handle>>,
    log: VecDeque<String>,
    current_step: Option<String>,
    outcome: Option<runner::Outcome>,
    cancelled: bool,
    dry_run: bool,
    /// Whether the log view should keep itself pinned to the newest line.
    ///
    /// True until the user scrolls up, which is taken as wanting to read
    /// something, and true again once they scroll back to the bottom. Scrolling
    /// unconditionally would drag the view away mid-sentence every time another
    /// line arrived — and during a system upgrade they arrive constantly.
    follow_log: bool,
    /// Writes this run to the history as it goes.
    recorder: Option<Recorder>,
    /// The virus database as it was when the run started, so a change can be
    /// spotted when it ends.
    clamav_before: crate::clamav::Fingerprint,
    /// The log's scroll position in pixels as of the last notification.
    ///
    /// Needed because a scroll notification does not say who caused it, and
    /// appending a line causes one: the widget re-notifies whenever the content
    /// bounds change, so every line of output arrives looking like a scroll
    /// event. Comparing positions is what separates the two — see
    /// [`App::update`]'s handling of [`Message::RunLogScrolled`].
    last_log_offset: f32,
}

pub struct App {
    core: Core,
    config: Config,
    config_handler: Option<cosmic_config::Config>,
    nav: nav_bar::Model,
    state: State,
    run: Option<Run>,
    context_page: Option<ContextPage>,
    dialog: Option<DialogPage>,
    password: String,
    password_visible: bool,
    /// The status-area item, when one is running.
    tray: Option<Arc<tray::Tray>>,
    /// Commands arriving from the status area.
    tray_commands: Option<Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<tray::Command>>>>,
    run_log_id: widget::Id,
    theme_labels: Vec<String>,
    privilege_labels: Vec<String>,
    frequency_labels: Vec<String>,
    interval_labels: Vec<String>,
    channel_labels: Vec<String>,
    hour_labels: Vec<String>,
    minute_labels: Vec<String>,
}

impl App {
    fn ready(&self) -> Option<&Ready> {
        match &self.state {
            State::Ready(ready) => Some(ready),
            _ => None,
        }
    }

    fn ready_mut(&mut self) -> Option<&mut Ready> {
        match &mut self.state {
            State::Ready(ready) => Some(ready),
            _ => None,
        }
    }

    fn page(&self) -> Page {
        self.nav
            .active_data::<Page>()
            .cloned()
            .unwrap_or(Page::Overview)
    }

    /// Rebuild the sidebar from what was discovered.
    ///
    /// Only categories that actually received a step appear, so the sidebar
    /// describes the machine rather than listing every heading the application
    /// knows how to draw.
    fn rebuild_nav(&mut self) {
        let previous = self.page();
        self.nav = nav_bar::Model::default();

        // Only until it has been through once: a permanent "Welcome" entry is
        // clutter, and these are all reachable in Settings afterwards.
        if !self.config.first_run_completed {
            self.nav
                .insert()
                .text(fl!("nav-welcome"))
                .icon(widget::icon::from_name("emblem-favorite-symbolic"))
                .data(Page::Welcome)
                .activate();
        }

        self.nav
            .insert()
            .text(fl!("nav-overview"))
            .icon(widget::icon::from_name("dialog-information-symbolic"))
            .data(Page::Overview);

        if let State::Ready(ready) = &self.state {
            for category in Category::ALL {
                let Some(steps) = ready.grouped.get(&category) else {
                    continue;
                };
                // The count shown is of steps that would actually do something,
                // which is the number worth knowing at a glance.
                let runnable = steps
                    .iter()
                    .filter(|id| {
                        ready
                            .capabilities
                            .get(id)
                            .is_some_and(|report| report.availability.is_runnable())
                    })
                    .count();
                self.nav
                    .insert()
                    .text(format!("{} ({runnable})", category.label()))
                    .icon(widget::icon::from_name(category.icon_name()))
                    .data(Page::Steps(category));
            }
        }

        for (label, icon, page) in [
            (fl!("nav-run"), "media-playback-start-symbolic", Page::Run),
            (
                fl!("nav-releases"),
                "folder-download-symbolic",
                Page::Releases,
            ),
            (
                fl!("nav-sources"),
                "network-workgroup-symbolic",
                Page::Sources,
            ),
            (
                fl!("nav-history"),
                "document-open-recent-symbolic",
                Page::History,
            ),
            (
                fl!("nav-dependencies"),
                "application-x-executable-symbolic",
                Page::Dependencies,
            ),
            (fl!("nav-schedule"), "alarm-symbolic", Page::Schedule),
            (
                fl!("nav-configuration"),
                "emblem-system-symbolic",
                Page::Configuration,
            ),
        ] {
            self.nav
                .insert()
                .text(label)
                .icon(widget::icon::from_name(icon))
                .data(page);
        }

        self.activate(&previous);
    }

    fn activate(&mut self, page: &Page) {
        // Resolved to an entity before activating, since the search holds a
        // borrow of the model that activation needs mutably.
        let found = self
            .nav
            .iter()
            .find(|entity| self.nav.data::<Page>(*entity) == Some(page))
            .or_else(|| self.nav.iter().next());
        if let Some(entity) = found {
            self.nav.activate(entity);
        }
    }

    /// Turn starting with the session on or off, reporting a failure rather
    /// than leaving the switch showing something untrue.
    fn apply_autostart(&mut self, enabled: bool) {
        if let Err(error) = autostart::set_enabled(enabled) {
            debug_log!(UI, "autostart change failed: {error}");
            if let Some(ready) = self.ready_mut() {
                ready.status = Some(error.to_string());
            }
        }
    }

    fn save_config(&mut self) {
        let Some(handler) = &self.config_handler else {
            return;
        };
        if let Err(errors) = self.config.write_entry(handler) {
            eprintln!("failed to save configuration: {errors:?}");
        }
    }

    /// Kick off a capability scan, reporting progress as it goes.
    fn start_scan(&mut self) -> Task<Message> {
        let Some(ready) = self.ready_mut() else {
            return Task::none();
        };
        ready.scanning = Some((0, ready.steps.len(), StepId::new("")));
        let topgrade = ready.topgrade.clone();
        let steps = ready.steps.clone();

        // Progress and the final result share one channel so they arrive in
        // order; a separate future for the result could otherwise land before
        // the last progress message.
        let (messages, receiver) = tokio::sync::mpsc::unbounded_channel::<Message>();
        let (progress, mut progress_receiver) = tokio::sync::mpsc::unbounded_channel();

        let forward = messages.clone();
        tokio::spawn(async move {
            while let Some(update) = progress_receiver.recv().await {
                let update: crate::topgrade::probe::ScanProgress = update;
                if forward
                    .send(Message::ScanProgress(
                        update.completed,
                        update.total,
                        update.last,
                    ))
                    .is_err()
                {
                    break;
                }
            }
        });

        tokio::spawn(async move {
            let capabilities =
                crate::topgrade::probe::scan(&topgrade, &steps, Some(progress)).await;
            let _ = messages.send(Message::Scanned(Box::new(capabilities)));
        });

        cosmic::task::stream(futures_util::stream::unfold(
            receiver,
            |mut receiver| async move { receiver.recv().await.map(|message| (message, receiver)) },
        ))
    }

    /// Scroll the run log to its newest line.
    ///
    /// Only the vertical offset is given, so a horizontal scroll the user has
    /// made to read a long line is left where they put it.
    fn snap_log_to_end(&self) -> Task<Message> {
        scrollable::snap_to(
            self.run_log_id.clone(),
            scrollable::RelativeOffset {
                x: None,
                y: Some(1.0),
            },
        )
    }

    /// Register the status-area item.
    fn start_tray() -> Task<Message> {
        cosmic::task::future(async move {
            let handles = tray::Tray::start().await.map(|(tray, commands)| TrayHandles {
                tray: Arc::new(tray),
                commands: Arc::new(Mutex::new(commands)),
            });
            Message::TrayStarted(handles)
        })
    }

    /// Start a virus scan if the run changed the signature database.
    ///
    /// Keyed off the database rather than off topgrade's step, because on a
    /// system where systemd's `clamav-freshclam` is enabled that step correctly
    /// stands down and the database still changes underneath it.
    fn scan_if_database_changed(&mut self) -> Task<Message> {
        if !self.config.clamav_scan {
            return Task::none();
        }
        let Some(run) = self.run.as_ref() else {
            return Task::none();
        };
        let before = run.clamav_before.clone();
        let after = crate::clamav::fingerprint();
        if !crate::clamav::changed(&before, &after) {
            return Task::none();
        }

        let options = self.config.clamscan_options.clone();
        let target = self.config.clamscan_target.clone();
        debug_log!(UI, "virus database changed; scanning");
        if let Some(ready) = self.ready_mut() {
            ready.status = Some(fl!("clamav-scanning"));
        }

        cosmic::task::future(async move {
            let result = crate::clamav::scan(&options, &target)
                .await
                .map_err(|error| error.to_string());
            Message::ScanFinished(Box::new(result))
        })
    }

    /// Tell the status-area item whether a run is under way.
    fn tray_running(&self, running: bool) -> Task<Message> {
        let Some(tray) = self.tray.clone() else {
            return Task::none();
        };
        cosmic::task::future(async move {
            tray.set_running(running).await;
            Message::None
        })
    }

    /// Await one command from the status area.
    ///
    /// Pumped one at a time for the same reason the run is: the receiver cannot
    /// be cloned, so it lives in one place and each message asks for the next.
    fn pump_tray(
        commands: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<tray::Command>>>,
    ) -> Task<Message> {
        cosmic::task::future(async move {
            let command = commands.lock().await.recv().await;
            Message::TrayCommand(command)
        })
    }

    /// Await one run event and turn it into a message.
    ///
    /// The run is pumped an event at a time rather than through a subscription
    /// because the handle owns a receiver that cannot be cloned, and this keeps
    /// it in one place.
    fn pump(handle: Arc<Mutex<runner::Handle>>) -> Task<Message> {
        cosmic::task::future(async move {
            let mut handle = handle.lock().await;
            match handle.next_event().await {
                Some(event) => Message::RunEvent(Box::new(event)),
                None => Message::RunPumpEnded,
            }
        })
    }

    /// Steps to pass to `--only`, or none for "everything still enabled".
    ///
    /// Only steps that are both runnable and enabled are listed. Passing the
    /// list explicitly rather than relying on the configuration keeps the run
    /// to what the user can see is selected.
    fn selected_steps(ready: &Ready) -> Vec<StepId> {
        ready
            .steps
            .iter()
            .filter(|id| {
                ready
                    .capabilities
                    .get(id)
                    .is_some_and(|report| report.availability.is_runnable())
                    && ready.settings.is_step_enabled(id)
            })
            .cloned()
            .collect()
    }
}

impl Application for App {
    type Executor = cosmic::executor::Default;
    type Flags = Flags;
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut app = App {
            core,
            config: flags.config,
            config_handler: flags.config_handler,
            nav: nav_bar::Model::default(),
            state: State::Loading,
            run: None,
            context_page: None,
            dialog: None,
            password: String::new(),
            password_visible: false,
            tray: None,
            tray_commands: None,
            run_log_id: widget::Id::new("run-log"),
            theme_labels: vec![fl!("theme-system"), fl!("theme-light"), fl!("theme-dark")],
            privilege_labels: vec![fl!("privilege-pty"), fl!("privilege-pkexec")],
            frequency_labels: vec![
                fl!("frequency-hourly"),
                fl!("frequency-daily"),
                fl!("frequency-weekly"),
                fl!("frequency-monthly"),
            ],
            interval_labels: vec![
                fl!("interval-manual"),
                fl!("interval-six-hourly"),
                fl!("interval-daily"),
                fl!("interval-weekly"),
            ],
            channel_labels: vec![fl!("channel-stable"), fl!("channel-pre-release")],
            hour_labels: (0..24).map(|hour| format!("{hour:02}")).collect(),
            minute_labels: (0..60).step_by(5).map(|m| format!("{m:02}")).collect(),
        };

        app.set_header_title(fl!("app-title"));
        app.rebuild_nav();

        // Registering the item is a D-Bus round trip, so it happens alongside
        // discovery rather than delaying the window.
        let tray_task = if app.config.show_tray_icon {
            Self::start_tray()
        } else {
            Task::none()
        };

        let task = cosmic::task::future(async move {
            let loaded = async {
                let topgrade = Topgrade::locate().await.map_err(|e| e.to_string())?;
                let steps = crate::topgrade::discover::steps(&topgrade)
                    .await
                    .map_err(|e| e.to_string())?;
                let schema = crate::topgrade::schema::load(&topgrade)
                    .await
                    .map_err(|e| e.to_string())?;
                let backend = schedule::detect_backend().await;
                let next_run = match backend {
                    Backend::Systemd => schedule::next_run().await,
                    Backend::InApp => None,
                };
                let client = releases::Client::detect().await;
                Ok::<_, String>(Loaded {
                    topgrade,
                    steps,
                    schema,
                    backend,
                    next_run,
                    client,
                })
            }
            .await;
            Message::Loaded(Box::new(loaded))
        });

        (app, Task::batch([task, tray_task]))
    }

    fn header_start(&self) -> Vec<Element<'_, Message>> {
        vec![widget::tooltip(
            widget::button::icon(widget::icon::from_name("sidebar-show-symbolic"))
                .on_press(Message::ToggleNavBar)
                .padding(8),
            widget::text(fl!("toggle-sidebar")),
            widget::tooltip::Position::Bottom,
        )
        .into()]
    }

    fn header_end(&self) -> Vec<Element<'_, Message>> {
        let mut elements: Vec<Element<'_, Message>> = Vec::new();

        elements.extend(vec![
            widget::button::icon(widget::icon::from_name("emblem-system-symbolic"))
                .on_press(Message::ToggleContextPage(ContextPage::Settings))
                .padding(8)
                .into(),
            widget::button::icon(widget::icon::from_name("help-about-symbolic"))
                .on_press(Message::ToggleContextPage(ContextPage::About))
                .padding(8)
                .into(),
        ]);

        elements
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Message> {
        self.nav.activate(id);
        // Read when the page is first opened rather than at startup: it walks a
        // directory and runs flatpak, and most sessions never look at it.
        if self.page() == Page::Sources
            && self.ready().is_some_and(|ready| ready.sources.is_empty())
        {
            return cosmic::task::message(Message::LoadSources);
        }
        Task::none()
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Message>> {
        if !self.core.window.show_context {
            return None;
        }
        Some(match self.context_page? {
            ContextPage::Settings => context_drawer::context_drawer(
                self.view_app_settings(),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title(fl!("settings")),
            ContextPage::About => context_drawer::context_drawer(
                self.view_about(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
        })
    }

    fn dialog(&self) -> Option<Element<'_, Message>> {
        match self.dialog.as_ref()? {
            DialogPage::ConfirmRun => Some(
                widget::dialog()
                    .icon(widget::icon::from_name("system-software-update-symbolic").size(64))
                    .title(fl!("run-now"))
                    .body(fl!("app-description"))
                    .primary_action(
                        widget::button::suggested(fl!("run-now"))
                            .on_press(Message::StartRun { dry_run: false }),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .into(),
            ),
            DialogPage::Password { prompt } => Some(
                widget::dialog()
                    .icon(widget::icon::from_name("dialog-password-symbolic").size(64))
                    .title(fl!("password-title"))
                    .body(fl!("password-body", command = prompt.as_str()))
                    .control(
                        widget::secure_input(
                            fl!("password-placeholder"),
                            self.password.clone(),
                            Some(Message::TogglePasswordVisible),
                            !self.password_visible,
                        )
                        .on_input(Message::PasswordInput)
                        .on_submit(|_| Message::PasswordSubmit),
                    )
                    .primary_action(
                        widget::button::suggested(fl!("authenticate"))
                            .on_press(Message::PasswordSubmit),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::CancelRun),
                    )
                    .into(),
            ),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![self
            .core
            .watch_config::<Config>(Self::APP_ID)
            .map(|update| Message::ConfigUpdated(update.config))];

        // Only ticks where there is no systemd user manager to keep the
        // schedule properly. Subscribing unconditionally would wake the process
        // every minute on the machines that need it least.
        let needs_fallback = self
            .ready()
            .is_some_and(|ready| ready.backend == Backend::InApp)
            && self.config.schedule.enabled;

        if needs_fallback {
            subscriptions.push(
                cosmic::iced::time::every(FALLBACK_SCHEDULER_TICK).map(|_| Message::SchedulerTick),
            );
        }

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => Task::none(),
            Message::Loaded(result) => {
                match *result {
                    Ok(loaded) => {
                        let settings = match SettingsFile::load() {
                            Ok(settings) => settings,
                            Err(error) => {
                                // A configuration that cannot be read is worth
                                // saying so about, but it does not stop the
                                // application: everything except the
                                // configuration page still works.
                                debug_log!(UI, "settings unreadable: {error}");
                                self.state = State::Unusable(error.to_string());
                                return Task::none();
                            }
                        };

                        let grouped = crate::topgrade::categories::group(&loaded.steps);
                        self.state = State::Ready(Box::new(Ready {
                            topgrade: loaded.topgrade,
                            steps: loaded.steps,
                            grouped,
                            capabilities: Capabilities::default(),
                            schema: loaded.schema,
                            settings,
                            backend: loaded.backend,
                            next_run: loaded.next_run,
                            scanning: None,
                            edits: HashMap::new(),
                            status: None,
                            history: history::list(),
                            viewing: None,
                            category_settings: HashSet::new(),
                            drafts: HashMap::new(),
                            client: loaded.client,
                            candidates: None,
                            discovering: false,
                            statuses: HashMap::new(),
                            checking: None,
                            installing: None,
                            deps: dependencies::check(),
                            installing_dep: None,
                            directory_draft: String::new(),
                            sources: Vec::new(),
                            source_draft: (String::new(), String::new(), String::new()),
                            changing_source: None,
                        }));
                        self.rebuild_nav();

                        let scan = self.start_scan();

                        // Only when it is actually due: a watch list of a few
                        // hundred projects polled on every launch would be
                        // impolite to the forges and rate-limited by some.
                        let check = if self
                                .config
                                .release_check_interval
                                .is_due(self.config.last_release_check, unix_now())
                        {
                            debug_log!(UI, "release check is due");
                            cosmic::task::message(Message::CheckReleases)
                        } else {
                            Task::none()
                        };

                        return Task::batch([scan, check]);
                    }
                    Err(error) => self.state = State::Unusable(error),
                }
                Task::none()
            }

            Message::ScanProgress(completed, total, last) => {
                if let Some(ready) = self.ready_mut() {
                    ready.scanning = Some((completed, total, last));
                }
                Task::none()
            }

            Message::Scanned(capabilities) => {
                if let Some(ready) = self.ready_mut() {
                    ready.capabilities = *capabilities;
                    ready.scanning = None;
                }
                self.rebuild_nav();
                Task::none()
            }

            Message::Rescan => self.start_scan(),

            Message::SelectPage(page) => {
                self.activate(&page);
                Task::none()
            }

            Message::ToggleNavBar => {
                self.core.nav_bar_toggle();
                Task::none()
            }

            Message::ToggleContextPage(page) => {
                if self.context_page == Some(page) && self.core.window.show_context {
                    self.core.window.show_context = false;
                } else {
                    self.context_page = Some(page);
                    self.core.window.show_context = true;
                }
                Task::none()
            }

            Message::DialogCancel => {
                self.dialog = None;
                Task::none()
            }

            Message::LaunchUrl(url) => {
                if let Err(error) = std::process::Command::new("xdg-open").arg(&url).spawn() {
                    eprintln!("failed to open {url}: {error}");
                }
                Task::none()
            }

            Message::ToggleStep(id, enabled) => {
                if let Some(ready) = self.ready_mut() {
                    ready.settings.set_step_enabled(&id, enabled);
                }
                Task::none()
            }

            Message::SetCategoryEnabled(category, enabled) => {
                if let Some(ready) = self.ready_mut() {
                    let steps = ready.grouped.get(&category).cloned().unwrap_or_default();
                    for id in steps {
                        ready.settings.set_step_enabled(&id, enabled);
                    }
                }
                Task::none()
            }

            Message::RequestRun { dry_run } => {
                // A preview changes nothing, so it never needs confirming.
                if dry_run || !self.config.confirm_before_running {
                    return cosmic::task::message(Message::StartRun { dry_run });
                }
                self.dialog = Some(DialogPage::ConfirmRun);
                Task::none()
            }

            Message::StartRun { dry_run } => {
                self.dialog = None;
                let Some(ready) = self.ready() else {
                    return Task::none();
                };

                let options = runner::Options {
                    dry_run,
                    only: Self::selected_steps(ready),
                    assume_yes: self.config.assume_yes,
                };

                match runner::start(&ready.topgrade, &options) {
                    Ok(handle) => {
                        let handle = Arc::new(Mutex::new(handle));
                        self.run = Some(Run {
                            handle: Arc::clone(&handle),
                            log: VecDeque::new(),
                            current_step: None,
                            outcome: None,
                            cancelled: false,
                            dry_run,
                            follow_log: true,
                            last_log_offset: 0.0,
                            recorder: Recorder::start(Origin::Manual, dry_run, unix_now()),
                            // Read before anything runs; compared afterwards.
                            clamav_before: if self.config.clamav_scan && !dry_run {
                                crate::clamav::fingerprint()
                            } else {
                                crate::clamav::Fingerprint::default()
                            },
                        });
                        self.activate(&Page::Run);
                        // The status-area menu reflects the run, so starting one
                        // from there does not offer to start another.
                        let mark = self.tray_running(true);
                        Task::batch([Self::pump(handle), mark])
                    }
                    Err(error) => {
                        if let Some(ready) = self.ready_mut() {
                            ready.status = Some(error.to_string());
                        }
                        Task::none()
                    }
                }
            }

            Message::RunEvent(event) => {
                let Some(run) = self.run.as_mut() else {
                    return Task::none();
                };

                match *event {
                    runner::Event::StepStarted(name) => run.current_step = Some(name),
                    runner::Event::Output(line) => {
                        if let Some(recorder) = run.recorder.as_mut() {
                            recorder.write_line(&line);
                        }
                        run.log.push_back(line);
                        // A long upgrade produces tens of thousands of lines,
                        // and keeping all of them costs memory for output
                        // nobody scrolls back to.
                        while run.log.len() > RUN_LOG_MAX_LINES {
                            run.log.pop_front();
                        }
                    }
                    runner::Event::PasswordRequested { prompt } => {
                        self.password.clear();
                        self.dialog = Some(DialogPage::Password { prompt });
                    }
                    runner::Event::Finished(mut outcome) => {
                        // The runner cannot tell a killed child from a failed
                        // one; this side knows which it asked for.
                        outcome.cancelled = run.cancelled;
                        run.current_step = None;

                        if let Some(recorder) = run.recorder.take() {
                            if let Some(record) =
                                recorder.finish(&outcome.components, outcome.cancelled, unix_now())
                            {
                                // Already visible on screen, so only a failure
                                // is worth interrupting the user for.
                                crate::notify::run_finished(
                                    &record,
                                    crate::notify::Policy {
                                        upgrades: self.config.notify_upgrades,
                                        errors: self.config.notify_errors,
                                        installs: self.config.schedule.automatic,
                                        // Started here, so success is already
                                        // visible; only a failure is news.
                                        on_screen: true,
                                    },
                                );
                                history::prune(self.config.keep_run_logs);
                                if let Some(ready) = match &mut self.state {
                                    State::Ready(ready) => Some(ready),
                                    _ => None,
                                } {
                                    ready.history = history::list();
                                }
                            }
                        }

                        // Re-borrowed because recording needed `self`.
                        if let Some(run) = self.run.as_mut() {
                            run.outcome = Some(outcome);
                        }
                        let scan = self.scan_if_database_changed();
                        return Task::batch([self.tray_running(false), scan]);
                    }
                }

                // Snapping is queued alongside the next pump rather than
                // instead of it, so following the log never costs an event.
                let follow = run.follow_log;
                let handle = Arc::clone(&run.handle);
                let pump = Self::pump(handle);

                if follow {
                    Task::batch([pump, self.snap_log_to_end()])
                } else {
                    pump
                }
            }

            Message::RunLogScrolled(viewport) => {
                let Some(run) = self.run.as_mut() else {
                    return Task::none();
                };

                // This fires for two quite different reasons, and reading the
                // relative offset alone cannot tell them apart.
                //
                // Appending a line grows the content, which the widget reports
                // as a viewport change. The *absolute* position is untouched by
                // that — the view is anchored to the top — but the position
                // expressed as a fraction of a now-taller content drops below
                // the bottom. Treating that as the user scrolling away would
                // switch following off on the very first line of output.
                //
                // A deliberate scroll upwards, by contrast, is the one thing
                // that actually moves the absolute position backwards. That
                // holds however fast output is arriving, so the two signals stay
                // distinguishable even mid-upgrade.
                let offset = viewport.absolute_offset().y;
                run.follow_log = should_follow_log(
                    run.follow_log,
                    viewport.relative_offset().y,
                    offset,
                    run.last_log_offset,
                );
                run.last_log_offset = offset;
                Task::none()
            }

            Message::RunPumpEnded => Task::none(),

            Message::CancelRun => {
                self.dialog = None;
                if let Some(run) = self.run.as_mut() {
                    run.cancelled = true;
                    let handle = Arc::clone(&run.handle);
                    return cosmic::task::future(async move {
                        handle.lock().await.cancel();
                        Message::None
                    });
                }
                Task::none()
            }

            Message::ClearLog => {
                self.run = None;
                Task::none()
            }

            Message::PasswordInput(password) => {
                self.password = password;
                Task::none()
            }

            Message::TogglePasswordVisible => {
                self.password_visible = !self.password_visible;
                Task::none()
            }

            Message::PasswordSubmit => {
                self.dialog = None;
                // Taken rather than copied so it is not left sitting in the
                // application's state after being sent.
                let password = std::mem::take(&mut self.password);
                self.password_visible = false;
                if let Some(run) = self.run.as_ref() {
                    let handle = Arc::clone(&run.handle);
                    return cosmic::task::future(async move {
                        handle.lock().await.send_password(&password);
                        Message::None
                    });
                }
                Task::none()
            }

            Message::EditSetting(section, key, value) => {
                if let Some(ready) = self.ready_mut() {
                    ready.settings.set(&section, &key, &value);
                }
                Task::none()
            }

            Message::EditText(section, key, text) => {
                let Some(ready) = self.ready_mut() else {
                    return Task::none();
                };

                let kind = ready
                    .schema
                    .setting(&section, &key)
                    .map(|setting| setting.kind.clone());
                ready.edits.insert(format!("{section}.{key}"), text.clone());

                // An empty box means "no value", which is how a setting is
                // returned to topgrade's own default rather than pinned to an
                // empty string.
                if text.trim().is_empty() {
                    ready.settings.unset(&section, &key);
                    return Task::none();
                }

                let value = match kind {
                    Some(ValueKind::Integer) => match text.trim().parse::<i64>() {
                        Ok(number) => Some(SettingValue::Integer(number)),
                        // Mid-edit text that is not yet a number is kept in the
                        // box but not written to the document.
                        Err(_) => None,
                    },
                    Some(ValueKind::StringList | ValueKind::StepList) => {
                        Some(SettingValue::List(
                            text.split(',')
                                .map(|entry| entry.trim().to_owned())
                                .filter(|entry| !entry.is_empty())
                                .collect(),
                        ))
                    }
                    _ => Some(SettingValue::Text(text)),
                };

                if let Some(value) = value {
                    ready.settings.set(&section, &key, &value);
                }
                Task::none()
            }

            Message::SaveSettings => {
                let Some(ready) = self.ready_mut() else {
                    return Task::none();
                };
                let result = ready.settings.save().map_err(|error| error.to_string());
                cosmic::task::message(Message::SettingsSaved(Box::new(result)))
            }

            Message::SettingsSaved(result) => {
                if let Some(ready) = self.ready_mut() {
                    ready.status = Some(match *result {
                        Ok(()) => fl!("configuration-saved"),
                        Err(error) => error,
                    });
                }
                Task::none()
            }

            Message::SchedulerTick => {
                let now = unix_now();
                if !self.config.schedule.is_due(self.config.last_fallback_run, now) {
                    return Task::none();
                }
                // Recorded before the run rather than after, so a run that takes
                // longer than the interval cannot immediately trigger another.
                self.config.last_fallback_run = now;
                self.save_config();
                debug_log!(UI, "fallback scheduler starting a run");
                cosmic::task::message(Message::StartRun {
                    dry_run: !self.config.schedule.automatic,
                })
            }

            Message::ScheduleEnabled(enabled) => {
                self.config.schedule.enabled = enabled;
                if enabled && self.config.last_fallback_run == 0 {
                    // `is_due` treats zero as "never run" and declines to fire,
                    // so the interval is measured from the moment the schedule
                    // was switched on.
                    self.config.last_fallback_run = unix_now();
                }
                self.save_config();
                Task::none()
            }

            Message::ScheduleFrequency(index) => {
                if let Some(frequency) = Frequency::ALL.get(index) {
                    self.config.schedule.frequency = *frequency;
                    self.save_config();
                }
                Task::none()
            }

            Message::ScheduleHour(index) => {
                self.config.schedule.hour = index as u32;
                self.save_config();
                Task::none()
            }

            Message::ScheduleMinute(index) => {
                self.config.schedule.minute = (index as u32) * 5;
                self.save_config();
                Task::none()
            }

            Message::ScheduleAutomatic(automatic) => {
                self.config.schedule.automatic = automatic;
                self.save_config();
                Task::none()
            }

            Message::ScheduleApply => {
                let schedule = self.config.schedule;
                cosmic::task::future(async move {
                    let result = match schedule::apply(schedule).await {
                        Ok(()) => Ok(schedule::next_run().await),
                        Err(error) => Err(error.to_string()),
                    };
                    Message::ScheduleApplied(Box::new(result))
                })
            }

            Message::ScheduleApplied(result) => {
                if let Some(ready) = self.ready_mut() {
                    match *result {
                        Ok(next) => {
                            ready.next_run = next;
                            ready.status = Some(fl!("schedule-applied"));
                        }
                        Err(message) => {
                            ready.status = Some(fl!("schedule-error", message = message))
                        }
                    }
                }
                Task::none()
            }

            Message::ConfigTheme(theme) => {
                self.config.app_theme = theme;
                self.save_config();
                cosmic::command::set_theme(theme.theme())
            }

            Message::ConfigPrivilege(mode) => {
                self.config.privilege_mode = mode;
                self.save_config();

                // topgrade has no command-line equivalent for this, so the
                // choice is recorded in its own configuration — where it is
                // also visible on the configuration page, and where it applies
                // equally when topgrade is run from a terminal.
                if let Some(ready) = self.ready_mut() {
                    let (section, key) = SUDO_COMMAND_KEY;
                    match mode {
                        PrivilegeMode::SystemDialog => ready.settings.set(
                            section,
                            key,
                            &SettingValue::Text(crate::constants::PKEXEC.to_owned()),
                        ),
                        // Removed rather than set back to "sudo", so topgrade
                        // keeps using whatever it would have chosen by default.
                        PrivilegeMode::AskInWindow => ready.settings.unset(section, key),
                    }
                    let _ = ready.settings.save();
                }
                Task::none()
            }

            Message::ConfigConfirmBeforeRunning(value) => {
                self.config.confirm_before_running = value;
                self.save_config();
                Task::none()
            }

            Message::ConfigAssumeYes(value) => {
                self.config.assume_yes = value;
                self.save_config();
                Task::none()
            }

            Message::ConfigShowUnavailable(value) => {
                self.config.show_unavailable_steps = value;
                self.save_config();
                Task::none()
            }

            Message::DiscoverProjects => {
                if let Some(ready) = self.ready_mut() {
                    ready.discovering = true;
                }
                let directories = self.config.appimage_dirs.clone();
                cosmic::task::future(async move {
                    Message::ProjectsDiscovered(releases::discover(&directories).await)
                })
            }

            Message::ProjectsDiscovered(found) => {
                let watched: HashSet<String> = self
                    .config
                    .watches
                    .iter()
                    .map(|watch| format!("{}/{}", watch.host, watch.path))
                    .collect();

                if let Some(ready) = self.ready_mut() {
                    ready.discovering = false;
                    ready.candidates = Some(
                        found
                            .into_iter()
                            // Already-watched projects are not offered again,
                            // and one with no repository cannot be watched yet.
                            .filter(|candidate| {
                                candidate.repo.as_ref().is_some_and(|repo| {
                                    !watched.contains(&format!("{}/{}", repo.host, repo.path))
                                })
                            })
                            .map(|candidate| (candidate, false))
                            .collect(),
                    );
                }
                Task::none()
            }

            Message::ToggleCandidate(index, selected) => {
                if let Some(ready) = self.ready_mut() {
                    if let Some(candidates) = ready.candidates.as_mut() {
                        if let Some(entry) = candidates.get_mut(index) {
                            entry.1 = selected;
                        }
                    }
                }
                Task::none()
            }

            Message::AddSelectedWatches => {
                let Some(ready) = self.ready_mut() else {
                    return Task::none();
                };
                let Some(candidates) = ready.candidates.take() else {
                    return Task::none();
                };
                let added: Vec<Watch> = candidates
                    .into_iter()
                    .filter(|(_, selected)| *selected)
                    .filter_map(|(candidate, _)| Watch::from_candidate(&candidate))
                    .collect();

                self.config.watches.extend(added);
                self.config
                    .watches
                    .sort_by_key(|watch| watch.name.to_lowercase());
                self.save_config();
                Task::none()
            }

            Message::CancelDiscovery => {
                if let Some(ready) = self.ready_mut() {
                    ready.candidates = None;
                    ready.discovering = false;
                }
                Task::none()
            }

            Message::RemoveWatch(key) => {
                self.config
                    .watches
                    .retain(|watch| format!("{}/{}", watch.host, watch.path) != key);
                self.save_config();
                if let Some(ready) = self.ready_mut() {
                    ready.statuses.remove(&key);
                }
                Task::none()
            }

            Message::CheckReleases => {
                // This application's own project is always checked, however it
                // was installed — there is nothing on disk to discover it from
                // when it was built from source.
                let mut watches = self.config.watches.clone();
                if let Some(own) = releases::self_watch() {
                    let key = format!("{}/{}", own.host, own.path);
                    if !watches
                        .iter()
                        .any(|watch| format!("{}/{}", watch.host, watch.path) == key)
                    {
                        watches.insert(0, own);
                    }
                }
                let channel = self.config.release_channel;
                if watches.is_empty() {
                    return Task::none();
                }
                let Some(client) = self.ready().map(|ready| ready.client) else {
                    return Task::none();
                };

                if let Some(ready) = self.ready_mut() {
                    ready.checking = Some((0, watches.len()));
                    ready.statuses.clear();
                }

                // Checks share one channel so progress and results arrive in
                // order, and run a few at a time: these are requests to other
                // people's servers, several of which rate-limit per client.
                let (messages, receiver) = tokio::sync::mpsc::unbounded_channel::<Message>();
                tokio::spawn(async move {
                    let permits = Arc::new(tokio::sync::Semaphore::new(
                        crate::constants::RELEASE_CHECK_CONCURRENCY,
                    ));
                    let mut tasks = tokio::task::JoinSet::new();
                    for watch in watches {
                        let permits = Arc::clone(&permits);
                        tasks.spawn(async move {
                            let _permit = permits.acquire_owned().await;
                            client.check(&watch, channel).await
                        });
                    }
                    while let Some(joined) = tasks.join_next().await {
                        if let Ok(status) = joined {
                            let _ = messages.send(Message::ReleaseChecked(Box::new(status)));
                        }
                    }
                    let _ = messages.send(Message::ReleasesChecked);
                });

                cosmic::task::stream(futures_util::stream::unfold(
                    receiver,
                    |mut receiver| async move {
                        receiver.recv().await.map(|message| (message, receiver))
                    },
                ))
            }

            Message::ReleaseChecked(status) => {
                let key = format!("{}/{}", status.watch.host, status.watch.path);
                let now = unix_now();

                // This application's own watch is synthesized rather than
                // stored, so the first time it is checked it is added to the
                // list — otherwise its result would be forgotten on every
                // restart and the entry would sit there looking inert.
                if releases::self_key().as_deref() == Some(key.as_str())
                    && !self
                        .config
                        .watches
                        .iter()
                        .any(|watch| format!("{}/{}", watch.host, watch.path) == key)
                {
                    self.config.watches.insert(0, status.watch.clone());
                }

                // Remembered on the watch itself so the page can say something
                // after a restart without asking the forges again.
                if let Some(watch) = self
                    .config
                    .watches
                    .iter_mut()
                    .find(|watch| format!("{}/{}", watch.host, watch.path) == key)
                {
                    watch.latest_tag = status
                        .latest
                        .as_ref()
                        .map(|release| release.tag.clone())
                        .unwrap_or_default();
                    watch.checked = now;
                }

                if let Some(ready) = self.ready_mut() {
                    ready.statuses.insert(key, *status);
                    if let Some((done, total)) = ready.checking {
                        ready.checking = Some((done + 1, total));
                    }
                }
                Task::none()
            }

            Message::ReleasesChecked => {
                if let Some(ready) = self.ready_mut() {
                    ready.checking = None;
                }
                // Stamped at the end rather than the start, so a check that was
                // interrupted does not count as one that happened.
                self.config.last_release_check = unix_now();
                self.save_config();
                Task::none()
            }

            Message::InstallRelease(key) => {
                let Some(ready) = self.ready() else {
                    return Task::none();
                };
                let Some(status) = ready.statuses.get(&key).cloned() else {
                    return Task::none();
                };
                let Some(release) = status.latest.clone() else {
                    return Task::none();
                };
                let source = status.watch.source();
                let name = status.watch.name.clone();
                let tag = release.tag.clone();

                if let Some(ready) = self.ready_mut() {
                    ready.installing = Some(key);
                }

                cosmic::task::future(async move {
                    let result = async {
                        let asset = releases::install::choose_asset(&release, &source)
                            .ok_or_else(|| fl!("releases-no-asset"))?;
                        let file = releases::install::download(asset)
                            .await
                            .map_err(|error| error.to_string())?;
                        releases::install::install(&file, &source)
                            .await
                            .map_err(|error| error.to_string())?;
                        Ok::<_, String>((name, tag))
                    }
                    .await;
                    Message::ReleaseInstalled(Box::new(result))
                })
            }

            Message::ReleaseInstalled(result) => {
                let Some(ready) = self.ready_mut() else {
                    return Task::none();
                };
                let key = ready.installing.take();

                match *result {
                    Ok((name, version)) => {
                        ready.status = Some(fl!(
                            "releases-installed",
                            name = name,
                            version = version.clone()
                        ));
                        // The recorded version moves forward so the entry stops
                        // reporting an update it has already applied.
                        if let Some(key) = key {
                            if let Some(watch) = self
                                .config
                                .watches
                                .iter_mut()
                                .find(|watch| format!("{}/{}", watch.host, watch.path) == key)
                            {
                                watch.installed = version;
                            }
                            self.save_config();
                        }
                    }
                    Err(message) => {
                        ready.status = Some(fl!(
                            "releases-install-failed",
                            name = String::new(),
                            message = message
                        ));
                    }
                }
                Task::none()
            }

            Message::TrayStarted(handles) => {
                let Some(handles) = handles else {
                    // No status area on this desktop. Said once in the settings
                    // rather than as an error: it is a normal state.
                    debug_log!(UI, "no status area; continuing without an icon");
                    return Task::none();
                };
                self.tray = Some(Arc::clone(&handles.tray));
                self.tray_commands = Some(Arc::clone(&handles.commands));
                Self::pump_tray(handles.commands)
            }

            Message::TrayCommand(command) => {
                let Some(command) = command else {
                    return Task::none();
                };
                let next = match self.tray_commands.clone() {
                    Some(commands) => Self::pump_tray(commands),
                    None => Task::none(),
                };
                let action = match command {
                    tray::Command::Show => cosmic::task::message(Message::ShowWindow),
                    tray::Command::Run => {
                        cosmic::task::message(Message::RequestRun { dry_run: false })
                    }
                    tray::Command::Quit => cosmic::task::message(Message::Quit),
                };
                Task::batch([action, next])
            }

            Message::ShowWindow => {
                let Some(id) = self.core.main_window_id() else {
                    return Task::none();
                };
                Task::batch([
                    // Raises and focuses an ordinary window. It cannot recover
                    // a minimized one — Wayland has no request for that — which
                    // is why nothing here minimizes.
                    cosmic::iced::window::gain_focus(id),
                ])
            }

            Message::Quit => {
                let tray = self.tray.take();
                cosmic::task::future(async move {
                    // Removed explicitly, so the icon does not linger in the
                    // panel after the process has gone.
                    if let Some(tray) = tray {
                        tray.shutdown().await;
                    }
                    cosmic::action::app(Message::None)
                })
                .chain(cosmic::iced::exit())
            }

            Message::ToggleCategorySettings(category) => {
                if let Some(ready) = self.ready_mut() {
                    if !ready.category_settings.remove(&category) {
                        ready.category_settings.insert(category);
                    }
                }
                Task::none()
            }

            Message::DraftCommandName(section, name) => {
                if let Some(ready) = self.ready_mut() {
                    ready.drafts.entry(section).or_default().0 = name;
                }
                Task::none()
            }

            Message::DraftCommandValue(section, command) => {
                if let Some(ready) = self.ready_mut() {
                    ready.drafts.entry(section).or_default().1 = command;
                }
                Task::none()
            }

            Message::AddCommand(section) => {
                if let Some(ready) = self.ready_mut() {
                    let Some((name, command)) = ready.drafts.get(&section).cloned() else {
                        return Task::none();
                    };
                    if ready.settings.set_free_form(&section, &name, &command) {
                        // Cleared only on success, so a refused entry keeps what
                        // was typed rather than throwing it away.
                        ready.drafts.remove(&section);
                    }
                }
                Task::none()
            }

            Message::EditCommand(section, name, command) => {
                if let Some(ready) = self.ready_mut() {
                    ready.settings.set_free_form(&section, &name, &command);
                }
                Task::none()
            }

            Message::RemoveCommand(section, name) => {
                if let Some(ready) = self.ready_mut() {
                    ready.settings.remove_free_form(&section, &name);
                }
                Task::none()
            }

            Message::FinishWelcome => {
                // Whatever was chosen is already saved — these are ordinary
                // settings, changed as they were touched. This only records
                // that the screen has been seen.
                self.config.first_run_completed = true;
                self.save_config();
                self.rebuild_nav();

                // Something required is missing, so the next thing shown is
                // what and why rather than an application that half works.
                let missing = self
                    .ready()
                    .is_some_and(|ready| dependencies::has_missing_required(&ready.deps));
                self.activate(if missing {
                    &Page::Dependencies
                } else {
                    &Page::Overview
                });
                Task::none()
            }

            Message::ShowWelcome => {
                // The screen is only in the sidebar while it has not been
                // finished, so bringing it back means marking it unfinished.
                // Nothing it sets is lost — those are ordinary settings.
                self.config.first_run_completed = false;
                self.save_config();
                self.rebuild_nav();
                self.activate(&Page::Welcome);
                self.core.window.show_context = false;
                Task::none()
            }

            Message::ConfigNotifyUpgrades(value) => {
                self.config.notify_upgrades = value;
                self.save_config();
                Task::none()
            }

            Message::ConfigNotifyErrors(value) => {
                self.config.notify_errors = value;
                self.save_config();
                Task::none()
            }

            Message::ScanFinished(result) => {
                if let Some(ready) = self.ready_mut() {
                    ready.status = Some(match *result {
                        Ok(report) if report.infected == 0 => {
                            fl!("clamav-clean", scanned = report.scanned)
                        }
                        Ok(report) => {
                            fl!("clamav-infected", infected = report.infected)
                        }
                        Err(message) => fl!("clamav-failed", message = message),
                    });
                }
                Task::none()
            }

            Message::ConfigClamavScan(value) => {
                self.config.clamav_scan = value;
                self.save_config();
                Task::none()
            }

            Message::ConfigClamscanOptions(value) => {
                self.config.clamscan_options = value;
                self.save_config();
                Task::none()
            }

            Message::ConfigClamscanTarget(value) => {
                self.config.clamscan_target = value;
                self.save_config();
                Task::none()
            }

            Message::ConfigAutostart(value) => {
                self.apply_autostart(value);
                Task::none()
            }

            Message::ConfigShowTrayIcon(value) => {
                self.config.show_tray_icon = value;
                self.save_config();

                if value {
                    if self.tray.is_none() {
                        return Self::start_tray();
                    }
                    return Task::none();
                }

                self.tray_commands = None;
                let tray = self.tray.take();
                cosmic::task::future(async move {
                    if let Some(tray) = tray {
                        tray.shutdown().await;
                    }
                    Message::None
                })
            }

            Message::RefreshHistory => {
                if let Some(ready) = self.ready_mut() {
                    ready.history = history::list();
                    ready.viewing = None;
                }
                Task::none()
            }

            Message::SelectHistory(id) => {
                cosmic::task::future(async move {
                    let result = history::transcript(&id)
                        .map(|text| format!("{id}\n{text}"))
                        .map_err(|error| error.to_string());
                    Message::HistoryTranscript(Box::new(result))
                })
            }

            Message::HistoryTranscript(result) => {
                if let Some(ready) = self.ready_mut() {
                    match *result {
                        // The identifier is carried back on the first line so
                        // the reply can be matched to the run without keeping a
                        // pending-request field for a single in-flight read.
                        Ok(payload) => {
                            let (id, text) = payload.split_once('\n').unwrap_or((&payload, ""));
                            ready.viewing = Some((id.to_owned(), text.to_owned()));
                        }
                        Err(error) => ready.status = Some(error),
                    }
                }
                Task::none()
            }

            Message::DeleteHistory(id) => {
                if let Some(ready) = self.ready_mut() {
                    if let Err(error) = history::remove(&id) {
                        ready.status = Some(error.to_string());
                    }
                    ready.history = history::list();
                    if ready.viewing.as_ref().is_some_and(|(shown, _)| *shown == id) {
                        ready.viewing = None;
                    }
                }
                Task::none()
            }

            Message::ConfigCheckInterval(index) => {
                if let Some(interval) = CheckInterval::ALL.get(index) {
                    self.config.release_check_interval = *interval;
                    self.save_config();
                }
                Task::none()
            }

            Message::ConfigChannel(index) => {
                if let Some(channel) = Channel::ALL.get(index) {
                    self.config.release_channel = *channel;
                    self.save_config();
                }
                Task::none()
            }

            Message::DraftDirectory(text) => {
                if let Some(ready) = self.ready_mut() {
                    ready.directory_draft = text;
                }
                Task::none()
            }

            Message::AddDirectory => {
                let Some(ready) = self.ready_mut() else {
                    return Task::none();
                };
                let directory = ready.directory_draft.trim().to_owned();
                if directory.is_empty() || self.config.appimage_dirs.contains(&directory) {
                    return Task::none();
                }
                if let Some(ready) = self.ready_mut() {
                    ready.directory_draft.clear();
                }
                self.config.appimage_dirs.push(directory);
                self.save_config();
                Task::none()
            }

            Message::RemoveDirectory(directory) => {
                self.config.appimage_dirs.retain(|entry| *entry != directory);
                self.save_config();
                Task::none()
            }

            Message::LoadSources => cosmic::task::future(async move {
                Message::SourcesLoaded(repos::list().await)
            }),

            Message::SourcesLoaded(sources) => {
                if let Some(ready) = self.ready_mut() {
                    ready.sources = sources;
                    ready.changing_source = None;
                }
                Task::none()
            }

            Message::DraftSource(field, text) => {
                if let Some(ready) = self.ready_mut() {
                    match field {
                        0 => ready.source_draft.0 = text,
                        1 => ready.source_draft.1 = text,
                        _ => ready.source_draft.2 = text,
                    }
                }
                Task::none()
            }

            Message::ToggleSource(name, enabled) => {
                let Some(ready) = self.ready_mut() else {
                    return Task::none();
                };
                let Some(repository) = ready
                    .sources
                    .iter()
                    .find(|source| source.name == name)
                    .cloned()
                else {
                    return Task::none();
                };
                ready.changing_source = Some(name);

                cosmic::task::future(async move {
                    let result = repos::set_enabled(&repository, enabled)
                        .await
                        .map_err(|error| error.to_string());
                    Message::SourceChanged(Box::new(result))
                })
            }

            Message::RemoveSource(name) => {
                let Some(ready) = self.ready_mut() else {
                    return Task::none();
                };
                let Some(repository) = ready
                    .sources
                    .iter()
                    .find(|source| source.name == name)
                    .cloned()
                else {
                    return Task::none();
                };
                ready.changing_source = Some(name);

                cosmic::task::future(async move {
                    let result = repos::remove(&repository)
                        .await
                        .map_err(|error| error.to_string());
                    Message::SourceChanged(Box::new(result))
                })
            }

            Message::AddSource(kind) => {
                let Some(ready) = self.ready() else {
                    return Task::none();
                };
                let (name, url, suite) = ready.source_draft.clone();
                if let Some(ready) = self.ready_mut() {
                    ready.source_draft = (String::new(), String::new(), String::new());
                }

                cosmic::task::future(async move {
                    let result = repos::add(kind, &name, &url, &suite)
                        .await
                        .map_err(|error| error.to_string());
                    Message::SourceChanged(Box::new(result))
                })
            }

            Message::SourceChanged(result) => {
                if let Some(ready) = self.ready_mut() {
                    ready.changing_source = None;
                    if let Err(message) = *result {
                        ready.status = Some(message);
                    } else {
                        ready.status = None;
                    }
                }
                // Re-read rather than assuming: the file on disk is the truth,
                // and a change that was authenticated away should show as not
                // having happened.
                cosmic::task::message(Message::LoadSources)
            }

            Message::RecheckDependencies => {
                if let Some(ready) = self.ready_mut() {
                    ready.deps = dependencies::check();
                }
                Task::none()
            }

            Message::InstallDependency(binary) => {
                let Some(dependency) = dependencies::ALL
                    .iter()
                    .find(|dependency| dependency.binary == binary)
                    .cloned()
                else {
                    return Task::none();
                };
                if let Some(ready) = self.ready_mut() {
                    ready.installing_dep = Some(binary.clone());
                }
                cosmic::task::future(async move {
                    let result = dependencies::install(&dependency)
                        .await
                        .map(|()| binary.clone())
                        .map_err(|error| (binary, error.to_string()));
                    Message::DependencyInstalled(Box::new(result))
                })
            }

            Message::DependencyInstalled(result) => {
                if let Some(ready) = self.ready_mut() {
                    ready.installing_dep = None;
                    // Re-checked rather than assumed: an install that reported
                    // success but put nothing on PATH should still read as
                    // missing.
                    ready.deps = dependencies::check();
                    if let Err((name, message)) = *result {
                        ready.status = Some(fl!(
                            "dependencies-install-failed",
                            name = name,
                            message = message
                        ));
                    }
                }
                Task::none()
            }

            Message::ConfigUpdated(config) => {
                self.config = config;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = match &self.state {
            State::Loading => widget::text::body(fl!("scanning"))
                .apply(widget::container)
                .center(Length::Fill)
                .into(),
            State::Unusable(message) => self.view_unusable(message),
            State::Ready(ready) => match self.page() {
                Page::Welcome => self.view_welcome(ready),
                Page::Overview => self.view_overview(ready),
                Page::Steps(category) => self.view_steps(ready, category),
                Page::Run => self.view_run(),
                Page::Releases => self.view_releases(ready),
                Page::Sources => self.view_sources(ready),
                Page::History => self.view_history(ready),
                Page::Dependencies => self.view_dependencies(ready),
                Page::Schedule => self.view_schedule(ready),
                Page::Configuration => self.view_configuration(ready),
            },
        };

        widget::container(content)
            .max_width(MAX_CONTENT_WIDTH)
            .width(Length::Fill)
            .apply(widget::container)
            .center_x(Length::Fill)
            .padding([16, 24])
            .into()
    }
}

// ── Views ───────────────────────────────────────────────────────────────────

impl App {
    /// Shown when topgrade is missing or too old — the one state where nothing
    /// else in the window would mean anything.
    fn view_unusable(&self, message: &str) -> Element<'_, Message> {
        widget::column::with_children(Vec::new())
            .spacing(12)
            .align_x(Alignment::Center)
            .push(widget::icon::from_name("dialog-error-symbolic").size(64))
            .push(widget::text::title3(fl!("topgrade-missing-title")))
            .push(widget::text::body(fl!("topgrade-missing-body")))
            .push(widget::text::caption(message.to_owned()))
            .push(widget::text::caption(fl!(
                "topgrade-missing-hint",
                command = "cargo install topgrade"
            )))
            .apply(widget::container)
            .center(Length::Fill)
            .into()
    }

    fn view_overview<'a>(&'a self, ready: &'a Ready) -> Element<'a, Message> {
        let source = match ready.topgrade.source() {
            Source::System => fl!("topgrade-source-system"),
            Source::Bundled => fl!("topgrade-source-bundled"),
        };

        let mut column = widget::column::with_children(Vec::new())
            .spacing(16)
            .push(widget::text::title2(fl!("overview-heading")))
            .push(
                widget::text::body(fl!(
                    "topgrade-version",
                    version = ready.topgrade.version().to_string()
                ))
                .apply(widget::container),
            )
            .push(widget::text::caption(source));

        column = match &ready.scanning {
            Some((completed, total, last)) => column
                .push(widget::text::body(fl!("scanning")))
                .push(widget::progress_bar::determinate_linear(
                    *completed as f32 / (*total).max(1) as f32,
                ))
                .push(widget::text::caption(fl!(
                    "scanning-progress",
                    completed = completed,
                    total = total,
                    step = last.humanized()
                ))),
            None => column.push(widget::text::body(fl!(
                "overview-subtitle",
                available = ready.capabilities.runnable_count(),
                total = ready.steps.len()
            ))),
        };

        column = column.push(
            widget::row::with_children(Vec::new())
                .spacing(8)
                .push(
                    widget::button::suggested(fl!("run-now"))
                        .on_press(Message::RequestRun { dry_run: false }),
                )
                .push(
                    widget::button::standard(fl!("dry-run"))
                        .on_press(Message::RequestRun { dry_run: true }),
                )
                .push(widget::button::standard(fl!("rescan")).on_press(Message::Rescan)),
        );

        if let Some(status) = &ready.status {
            column = column.push(widget::text::caption(status.clone()));
        }

        // A short summary of each category, so the overview says what would
        // happen without needing every category visited.
        let mut summary = widget::settings::section().title(fl!("steps-heading"));
        for category in Category::ALL {
            let Some(steps) = ready.grouped.get(&category) else {
                continue;
            };
            let runnable = steps
                .iter()
                .filter(|id| {
                    ready
                        .capabilities
                        .get(id)
                        .is_some_and(|r| r.availability.is_runnable())
                })
                .count();
            summary = summary.add(
                widget::settings::item::builder(category.label())
                    .description(fl!(
                        "overview-subtitle",
                        available = runnable,
                        total = steps.len()
                    ))
                    .icon(widget::icon::from_name(category.icon_name()))
                    .control(
                        widget::button::text(fl!("view"))
                            .on_press(Message::SelectPage(Page::Steps(category))),
                    ),
            );
        }

        widget::scrollable(column.push(summary)).into()
    }

    fn view_steps<'a>(&'a self, ready: &'a Ready, category: Category) -> Element<'a, Message> {
        let empty = Vec::new();
        let steps = ready.grouped.get(&category).unwrap_or(&empty);

        // Only sections this topgrade actually has, so a category does not
        // offer settings that the installed release knows nothing about.
        let sections: Vec<&crate::topgrade::schema::Section> =
            crate::topgrade::categories::config_sections(category)
                .iter()
                .filter_map(|name| ready.schema.section(name))
                .collect();

        let mut heading = widget::row::with_children(Vec::new())
            .spacing(8)
            .align_y(Alignment::Center)
            .push(widget::text::title2(category.label()));

        if !sections.is_empty() {
            heading = heading.push(
                widget::tooltip(
                    widget::button::icon(widget::icon::from_name("emblem-system-symbolic"))
                        .on_press(Message::ToggleCategorySettings(category))
                        .padding(6),
                    widget::text(fl!("category-settings")),
                    widget::tooltip::Position::Bottom,
                ),
            );
        }

        let mut column = widget::column::with_children(Vec::new())
            .spacing(12)
            .push(heading)
            .push(
                widget::row::with_children(Vec::new())
                    .spacing(8)
                    .push(
                        widget::button::standard(fl!("enable-all"))
                            .on_press(Message::SetCategoryEnabled(category, true)),
                    )
                    .push(
                        widget::button::standard(fl!("disable-all"))
                            .on_press(Message::SetCategoryEnabled(category, false)),
                    ),
            );

        // The same controls as the configuration page, shown where the steps
        // they affect are, rather than only on a page of their own.
        if ready.category_settings.contains(&category) {
            for section in &sections {
                column = column.push(self.view_section(ready, section));
            }
        }

        let mut section = widget::settings::section();
        let mut shown = 0;

        for id in steps {
            let report = ready.capabilities.get(id);
            let availability = report.map(|report| &report.availability);

            let runnable = availability.is_some_and(Availability::is_runnable);
            // Unavailable steps are hidden unless asked for: on a typical
            // system they are most of the list, and they are the main reason
            // topgrade's own step list is hard to read.
            if !runnable && !self.config.show_unavailable_steps {
                continue;
            }
            shown += 1;

            let description = match availability {
                Some(Availability::Available) => match report.map(|r| r.components.len()) {
                    Some(count) if count > 1 => fl!("step-components", count = count),
                    _ => fl!("step-available"),
                },
                Some(Availability::Unavailable { reason }) => reason.clone(),
                Some(Availability::Deprecated { note }) => note.clone(),
                Some(Availability::Inactive) | None => fl!("step-inactive"),
            };

            // topgrade's own display name is preferred where it gave one; the
            // identifier is only humanized when it did not.
            let title = report
                .and_then(|report| report.components.first())
                .map(|component| component.name.clone())
                .unwrap_or_else(|| id.humanized());

            let item = widget::settings::item::builder(title).description(description);

            section = if runnable {
                let id = id.clone();
                section.add(item.toggler(ready.settings.is_step_enabled(&id), move |value| {
                    Message::ToggleStep(id.clone(), value)
                }))
            } else {
                // Nothing to toggle for a step that cannot run; showing a dead
                // switch would suggest otherwise.
                section.add(item.control(widget::text::caption(fl!("step-unavailable"))))
            };
        }

        if shown == 0 {
            column = column.push(widget::text::body(fl!("steps-none")));
        } else {
            column = column.push(section);
        }

        column = column.push(
            widget::settings::section().add(
                widget::settings::item::builder(fl!("show-unavailable"))
                    .description(fl!("show-unavailable-tooltip"))
                    .toggler(
                        self.config.show_unavailable_steps,
                        Message::ConfigShowUnavailable,
                    ),
            ),
        );

        widget::scrollable(column).into()
    }

    fn view_run(&self) -> Element<'_, Message> {
        let mut column = widget::column::with_children(Vec::new())
            .spacing(12)
            .push(widget::text::title2(fl!("run-heading")));

        let Some(run) = self.run.as_ref() else {
            return column
                .push(widget::text::body(fl!("run-never")))
                .push(
                    widget::row::with_children(Vec::new())
                        .spacing(8)
                        .push(
                            widget::button::suggested(fl!("run-now"))
                                .on_press(Message::RequestRun { dry_run: false }),
                        )
                        .push(
                            widget::button::standard(fl!("dry-run"))
                                .on_press(Message::RequestRun { dry_run: true }),
                        ),
                )
                .into();
        };

        column = match &run.outcome {
            None => {
                let activity = run
                    .current_step
                    .clone()
                    .map(|step| fl!("run-step", step = step))
                    .unwrap_or_else(|| fl!("run-in-progress"));
                column
                    .push(widget::text::body(activity))
                    .push(widget::button::destructive(fl!("cancel-run")).on_press(Message::CancelRun))
            }
            Some(outcome) => {
                let (ok, skipped, failed) = outcome.counts();
                let heading = if outcome.cancelled {
                    fl!("run-cancelled")
                } else if outcome.success && failed == 0 {
                    fl!("run-finished")
                } else {
                    fl!("run-failed")
                };
                column
                    .push(widget::text::title4(heading))
                    .push(widget::text::body(fl!(
                        "run-summary",
                        ok = ok,
                        skipped = skipped,
                        failed = failed
                    )))
                    .push(widget::button::standard(fl!("clear-log")).on_press(Message::ClearLog))
            }
        };

        if run.dry_run {
            column = column.push(widget::text::caption(fl!("run-was-preview")));
        }

        // The summary is the useful part of a finished run, so it is shown as a
        // list rather than left buried in the log.
        if let Some(outcome) = &run.outcome {
            let mut section = widget::settings::section();
            for component in &outcome.components {
                let status = match component.status {
                    Status::Ok => fl!("status-ok"),
                    Status::Skipped => fl!("status-skipped"),
                    Status::Failed => fl!("status-failed"),
                };
                // The status is the control; the description carries topgrade's
                // reason, and is left off when there is none rather than
                // repeating the status beside itself.
                let item = widget::settings::item::builder(component.name.clone());
                let item = match &component.reason {
                    Some(reason) => item.description(reason.clone()),
                    None => item,
                };
                section = section.add(item.control(widget::text::caption(status)));
            }
            column = column.push(section);
        }

        let log = run
            .log
            .iter()
            .fold(widget::column::with_children(Vec::new()).spacing(2), |column, line| {
                column.push(widget::text::monotext(line.clone()))
            });

        column
            .push(
                widget::container(
                    widget::scrollable(log)
                        .id(self.run_log_id.clone())
                        .on_scroll(Message::RunLogScrolled),
                )
                .height(Length::Fill)
                .width(Length::Fill),
            )
            .into()
    }

    /// What the notification setting will actually do, given the schedule.
    ///
    /// "Tell me about upgrades" means two different things depending on whether
    /// the schedule installs them or only looks, and the user should not have to
    /// work out which from a generic label.
    fn notification_wording(&self) -> String {
        if self.config.schedule.automatic {
            fl!("notify-upgrades-installed")
        } else {
            fl!("notify-upgrades-available")
        }
    }

    /// The first screen: the handful of choices worth making before anything
    /// else, rather than a dialog in front of an application nobody has seen.
    fn view_welcome<'a>(&'a self, ready: &'a Ready) -> Element<'a, Message> {
        let missing = ready.deps.iter().filter(|report| report.is_problem()).count();

        let mut column = widget::column::with_children(Vec::new())
            .spacing(12)
            .push(widget::text::title2(fl!("welcome-heading")))
            .push(widget::text::body(fl!("welcome-body")));

        // Anything required that is absent is said first: the rest of these
        // choices are moot if the application cannot do its job.
        if missing > 0 {
            column = column.push(
                widget::settings::section().add(
                    widget::settings::item::builder(fl!(
                        "dependencies-missing-required",
                        count = missing
                    ))
                    .icon(widget::icon::from_name("dialog-error-symbolic"))
                    .control(
                        widget::button::suggested(fl!("nav-dependencies"))
                            .on_press(Message::SelectPage(Page::Dependencies)),
                    ),
                ),
            );
        }

        column = column.push(
            widget::settings::section()
                .title(fl!("welcome-notifications"))
                .add(
                    widget::settings::item::builder(fl!("notify-upgrades"))
                        .description(self.notification_wording())
                        .toggler(self.config.notify_upgrades, Message::ConfigNotifyUpgrades),
                )
                .add(
                    widget::settings::item::builder(fl!("notify-errors"))
                        .description(fl!("notify-errors-description"))
                        .toggler(self.config.notify_errors, Message::ConfigNotifyErrors),
                ),
        );

        let mut upgrading = widget::settings::section()
            .title(fl!("welcome-automatic-heading"))
            .add(
                widget::settings::item::builder(fl!("schedule-enabled"))
                    .toggler(self.config.schedule.enabled, Message::ScheduleEnabled),
            )
            .add(
                widget::settings::item::builder(fl!("schedule-automatic"))
                    .description(fl!("schedule-automatic-description"))
                    .toggler(self.config.schedule.automatic, Message::ScheduleAutomatic),
            );

        // Said plainly and only when it applies: installing without anybody
        // present cannot ask for a password, so it needs rights the rest of
        // this application deliberately does not have.
        if self.config.schedule.automatic {
            upgrading = upgrading.add(widget::text::caption(fl!("welcome-root-warning")));
        }
        column = column.push(upgrading);

        // Offered only where there is something to scan with.
        if ready.deps.iter().any(|report| report.installed && report.dependency.binary == "clamscan")
            || which_clamscan()
        {
            column = column.push(
                widget::settings::section()
                    .title(fl!("welcome-clamav"))
                    .add(
                        widget::settings::item::builder(fl!("clamav-scan"))
                            .description(fl!("clamav-scan-description"))
                            .toggler(self.config.clamav_scan, Message::ConfigClamavScan),
                    )
                    .add(
                        widget::settings::item::builder(fl!("clamav-target")).control(
                            widget::text_input("~", self.config.clamscan_target.clone())
                                .on_input(Message::ConfigClamscanTarget)
                                .width(Length::Fixed(260.0)),
                        ),
                    )
                    .add(
                        widget::settings::item::builder(fl!("clamav-options")).control(
                            widget::text_input(
                                crate::constants::CLAMSCAN_DEFAULT_OPTIONS,
                                self.config.clamscan_options.clone(),
                            )
                            .on_input(Message::ConfigClamscanOptions)
                            .width(Length::Fixed(260.0)),
                        ),
                    ),
            );
        }

        column = column.push(
            widget::settings::section()
                .add(
                    widget::settings::item::builder(fl!("autostart"))
                        .description(fl!("autostart-description"))
                        .toggler(autostart::is_enabled(), Message::ConfigAutostart),
                )
                .add(
                    widget::settings::item::builder(fl!("show-tray-icon"))
                        .toggler(self.config.show_tray_icon, Message::ConfigShowTrayIcon),
                ),
        );

        widget::scrollable(
            column.push(
                widget::button::suggested(fl!("welcome-finish"))
                    .on_press(Message::FinishWelcome),
            ),
        )
        .into()
    }

    /// The tools this application drives, and whether they are here.
    fn view_dependencies<'a>(&'a self, ready: &'a Ready) -> Element<'a, Message> {
        let missing = ready.deps.iter().filter(|report| report.is_problem()).count();
        let can_install = dependencies::Manager::detect().is_some();

        let mut column = widget::column::with_children(Vec::new())
            .spacing(12)
            .push(widget::text::title2(fl!("dependencies-heading")))
            .push(widget::text::body(fl!("dependencies-description")))
            .push(widget::text::body(if missing == 0 {
                fl!("dependencies-all-present")
            } else {
                fl!("dependencies-missing-required", count = missing)
            }));

        if !can_install {
            column = column.push(widget::text::caption(fl!("dependencies-no-manager")));
        }
        if let Some(status) = &ready.status {
            column = column.push(widget::text::caption(status.clone()));
        }

        column = column.push(
            widget::button::standard(fl!("dependencies-recheck"))
                .on_press(Message::RecheckDependencies),
        );

        let mut section = widget::settings::section();
        for report in &ready.deps {
            let requirement = match report.dependency.requirement {
                Requirement::Required => fl!("dependencies-required"),
                Requirement::Optional => fl!("dependencies-optional"),
            };
            let state = if report.installed {
                fl!("dependencies-installed")
            } else {
                fl!("dependencies-missing")
            };

            let mut controls = widget::row::with_children(Vec::new())
                .spacing(8)
                .align_y(Alignment::Center)
                .push(widget::text::caption(format!("{requirement} · {state}")));

            // Offered only for what is actually absent, and only where there is
            // a package manager to do it with.
            if !report.installed && can_install {
                let binary = report.dependency.binary.to_owned();
                let busy = ready.installing_dep.as_deref() == Some(report.dependency.binary);
                controls = controls.push(
                    widget::button::suggested(if busy {
                        fl!("dependencies-installing")
                    } else {
                        fl!("dependencies-install")
                    })
                    .on_press_maybe(
                        ready
                            .installing_dep
                            .is_none()
                            .then_some(Message::InstallDependency(binary)),
                    ),
                );
            }

            // The resolved path is shown for what is present: "which one is it
            // actually using" is the question asked when a tool misbehaves.
            let description = match &report.path {
                Some(path) => format!("{}\n{path}", report.dependency.purpose()),
                None => report.dependency.purpose(),
            };

            section = section.add(
                widget::settings::item::builder(report.dependency.binary.to_owned())
                    .description(description)
                    .icon(widget::icon::from_name(if report.installed {
                        "emblem-ok-symbolic"
                    } else if report.is_problem() {
                        "dialog-error-symbolic"
                    } else {
                        "dialog-information-symbolic"
                    }))
                    .control(controls),
            );
        }

        widget::scrollable(column.push(section)).into()
    }

    /// Projects watched for new releases, or the picker for choosing them.
    fn view_releases<'a>(&'a self, ready: &'a Ready) -> Element<'a, Message> {
        let mut column = widget::column::with_children(Vec::new())
            .spacing(12)
            .push(widget::text::title2(fl!("releases-heading")))
            .push(widget::text::body(fl!("releases-description")));

        if !ready.client.is_usable() {
            // Nothing here can work without one of them; saying so once is
            // better than every row reporting the same failure.
            return column
                .push(widget::text::body(fl!("releases-no-transport")))
                .into();
        }

        // The picker replaces the list while it is open: choosing from 360
        // candidates beside the watched list would leave no room for either.
        if let Some(candidates) = &ready.candidates {
            let selected = candidates.iter().filter(|(_, chosen)| *chosen).count();
            let mut section = widget::settings::section();

            for (index, (candidate, chosen)) in candidates.iter().enumerate() {
                let Some(repo) = &candidate.repo else {
                    continue;
                };
                section = section.add(
                    widget::settings::item::builder(candidate.name.clone())
                        .description(format!(
                            "{} — {}",
                            repo.display(),
                            fl!(
                                "releases-source",
                                source = candidate.source.label().to_owned(),
                                forge = repo.kind.label().to_owned()
                            )
                        ))
                        .toggler(*chosen, move |value| {
                            Message::ToggleCandidate(index, value)
                        }),
                );
            }

            return widget::scrollable(
                column
                    .push(widget::text::body(fl!(
                        "releases-found",
                        count = candidates.len()
                    )))
                    .push(
                        widget::row::with_children(Vec::new())
                            .spacing(8)
                            .push(
                                widget::button::suggested(fl!("releases-add-selected"))
                                    .on_press_maybe(
                                        (selected > 0).then_some(Message::AddSelectedWatches),
                                    ),
                            )
                            .push(
                                widget::button::standard(fl!("releases-cancel-find"))
                                    .on_press(Message::CancelDiscovery),
                            ),
                    )
                    .push(section),
            )
            .into();
        }

        let mut actions = widget::row::with_children(Vec::new()).spacing(8).push(
            widget::button::standard(fl!("releases-find")).on_press_maybe(
                (!ready.discovering).then_some(Message::DiscoverProjects),
            ),
        );
        if !self.config.watches.is_empty() {
            actions = actions.push(
                widget::button::suggested(fl!("releases-check"))
                    .on_press_maybe(ready.checking.is_none().then_some(Message::CheckReleases)),
            );
        }
        column = column.push(actions);

        if ready.discovering {
            column = column.push(widget::text::body(fl!("releases-finding")));
        }
        if let Some((done, total)) = ready.checking {
            column = column
                .push(widget::progress_bar::determinate_linear(
                    done as f32 / total.max(1) as f32,
                ))
                .push(widget::text::caption(fl!(
                    "releases-checking",
                    done = done,
                    total = total
                )));
        }
        if let Some(status) = &ready.status {
            column = column.push(widget::text::caption(status.clone()));
        }

        let interval_index = CheckInterval::ALL
            .iter()
            .position(|interval| *interval == self.config.release_check_interval);
        let channel_index = Channel::ALL
            .iter()
            .position(|channel| *channel == self.config.release_channel);

        // Built here but pushed after the list: these govern the page rather
        // than being the point of it, and a screenful of controls in front of
        // the projects would bury what the user came to look at.
        let mut settings = widget::settings::section()
            .add(
                widget::settings::item::builder(fl!("releases-interval"))
                    .description(match self.config.last_release_check {
                        0 => fl!("releases-never-checked"),
                        last => {
                            let mut text = fl!(
                                "releases-last-checked",
                                when = crate::history::format_timestamp(last)
                            );
                            // Says when the cap next lifts, so "nothing
                            // happened when I opened it" has a visible reason.
                            if let Some(next) = self.config.release_check_interval.next_due(last) {
                                text.push_str(&format!(
                                    " · {}",
                                    fl!(
                                        "releases-next-check",
                                        when = crate::history::format_timestamp(next)
                                    )
                                ));
                            }
                            text
                        }
                    })
                    .control(widget::dropdown(
                        &self.interval_labels,
                        interval_index,
                        Message::ConfigCheckInterval,
                    )),
            )
            .add(
                widget::settings::item::builder(fl!("releases-channel"))
                    .description(fl!("releases-channel-description"))
                    .control(widget::dropdown(
                        &self.channel_labels,
                        channel_index,
                        Message::ConfigChannel,
                    )),
            )
            .add(
                widget::settings::item::builder(fl!("releases-directories"))
                    .description(fl!("releases-directories-description"))
                    .control(
                        widget::row::with_children(Vec::new())
                            .spacing(4)
                            .push(
                                widget::text_input(
                                    fl!("releases-directory-placeholder"),
                                    ready.directory_draft.clone(),
                                )
                                .on_input(Message::DraftDirectory)
                                .on_submit(|_| Message::AddDirectory)
                                .width(Length::Fixed(220.0)),
                            )
                            .push(
                                widget::button::standard(fl!("releases-directory-add"))
                                    .on_press_maybe(
                                        (!ready.directory_draft.trim().is_empty())
                                            .then_some(Message::AddDirectory),
                                    ),
                            ),
                    ),
            );

        for directory in &self.config.appimage_dirs {
            let remove = directory.clone();
            settings = settings.add(
                widget::settings::item::builder(directory.clone()).control(
                    widget::button::icon(widget::icon::from_name("list-remove-symbolic"))
                        .on_press(Message::RemoveDirectory(remove)),
                ),
            );
        }

        // This application's own project is listed first and always, however it
        // was installed. It is synthesized rather than discovered, so building
        // from source — where there is no file to find and nothing in any
        // package database — still gets update notices.
        let own_key = releases::self_key();
        let mut listed: Vec<Watch> = Vec::new();
        if let Some(own) = releases::self_watch() {
            if !self
                .config
                .watches
                .iter()
                .any(|watch| Some(format!("{}/{}", watch.host, watch.path)) == own_key)
            {
                listed.push(own);
            }
        }
        listed.extend(self.config.watches.iter().cloned().map(|mut watch| {
            // The stored version is whatever was running when it was last
            // checked; the running build is the truth. Without this, updating
            // this application would leave it reporting its own old version as
            // out of date until the next check.
            if Some(format!("{}/{}", watch.host, watch.path)) == own_key {
                watch.installed = env!("CARGO_PKG_VERSION").to_owned();
            }
            watch
        }));

        let mut section = widget::settings::section()
            .title(fl!("releases-watched", count = listed.len()));

        for watch in &listed {
            let key = format!("{}/{}", watch.host, watch.path);
            let status = ready.statuses.get(&key);
            let installing = ready.installing.as_deref() == Some(key.as_str());

            let description = match status {
                // Nothing checked this session: show what was seen last time
                // rather than an empty row.
                None if !watch.latest_tag.is_empty() => {
                    let comparison =
                        releases::version::compare(&watch.installed, &watch.latest_tag);
                    let summary = if comparison == releases::version::Ordering::Newer {
                        fl!("releases-update-available", version = watch.latest_tag.clone())
                    } else {
                        fl!("releases-up-to-date")
                    };
                    format!(
                        "{summary} · {}",
                        fl!(
                            "releases-last-checked",
                            when = crate::history::format_timestamp(watch.checked)
                        )
                    )
                }
                None => watch.repo().display(),
                Some(status) => match (&status.error, &status.latest) {
                    (Some(error), _) => fl!("releases-error", message = error.clone()),
                    (None, None) => fl!("releases-no-releases"),
                    (None, Some(release)) => match status.comparison {
                        releases::version::Ordering::Newer => {
                            fl!("releases-update-available", version = release.tag.clone())
                        }
                        // A version that could not be compared is reported as
                        // published rather than claimed to be newer.
                        releases::version::Ordering::Unknown => {
                            fl!("releases-unknown", version = release.tag.clone())
                        }
                        _ => fl!("releases-up-to-date"),
                    },
                },
            };

            let mut controls = widget::row::with_children(Vec::new())
                .spacing(4)
                .align_y(Alignment::Center);

            if let Some(status) = status {
                if let Some(release) = &status.latest {
                    let url = release.web_url.clone();
                    controls = controls.push(
                        widget::button::text(fl!("releases-open"))
                            .on_press(Message::LaunchUrl(url)),
                    );
                }
                if status.is_update() {
                    controls = controls.push(
                        widget::button::suggested(if installing {
                            fl!("releases-installing", name = watch.name.clone())
                        } else {
                            fl!("releases-update")
                        })
                        .on_press_maybe(
                            (ready.installing.is_none())
                                .then(|| Message::InstallRelease(key.clone())),
                        ),
                    );
                }
            }

            if own_key.as_deref() != Some(key.as_str()) {
                controls = controls.push(
                    widget::button::icon(widget::icon::from_name("list-remove-symbolic"))
                        .on_press(Message::RemoveWatch(key.clone())),
                );
            }

            let is_self = own_key.as_deref() == Some(key.as_str());
            let title = if is_self {
                format!("{}  ({})  · {}", watch.name, watch.installed, fl!("releases-self"))
            } else {
                format!("{}  ({})", watch.name, watch.installed)
            };

            section = section.add(
                widget::settings::item::builder(title)
                    .description(description)
                    .control(controls),
            );
        }

        // The watched projects are what the page is for, so they come before
        // the settings that govern them rather than below a screenful of
        // controls.
        widget::scrollable(column.push(section).push(settings)).into()
    }

    /// Where packages come from, and the controls for changing that.
    fn view_sources<'a>(&'a self, ready: &'a Ready) -> Element<'a, Message> {
        let mut column = widget::column::with_children(Vec::new())
            .spacing(12)
            .push(widget::text::title2(fl!("sources-heading")))
            .push(widget::text::body(fl!("sources-description")))
            .push(
                widget::row::with_children(Vec::new()).spacing(8).push(
                    widget::button::standard(fl!("sources-reload"))
                        .on_press(Message::LoadSources),
                ),
            );

        if let Some(status) = &ready.status {
            column = column.push(widget::text::caption(status.clone()));
        }

        if ready.sources.is_empty() {
            column = column.push(widget::text::body(fl!("sources-none")));
        }

        // Grouped by package manager, which is how somebody thinks about them:
        // "my Flatpak remotes" is a question, "all my repositories" rarely is.
        for kind in [repos::Kind::Apt, repos::Kind::Flatpak, repos::Kind::Dnf] {
            let listed: Vec<&Repository> = ready
                .sources
                .iter()
                .filter(|source| source.kind == kind)
                .collect();
            if listed.is_empty() {
                continue;
            }

            let mut section = widget::settings::section().title(kind.label());
            if kind != repos::Kind::Flatpak {
                section = section.add(widget::text::caption(fl!("sources-disable-note")));
            }

            for source in listed {
                let name = source.name.clone();
                let remove_name = source.name.clone();
                let busy = ready.changing_source.as_deref() == Some(source.name.as_str());

                let mut controls = widget::row::with_children(Vec::new())
                    .spacing(8)
                    .align_y(Alignment::Center);

                if busy {
                    controls = controls.push(widget::text::caption(fl!("sources-changing")));
                }

                controls = controls
                    .push(
                        widget::toggler(source.enabled)
                            .on_toggle(move |value| Message::ToggleSource(name.clone(), value)),
                    )
                    .push(
                        widget::button::icon(widget::icon::from_name("list-remove-symbolic"))
                            .on_press(Message::RemoveSource(remove_name)),
                    );

                let mut description = source.detail.clone();
                if source.privileged {
                    description.push_str(&format!("\n{}", fl!("sources-privileged")));
                }

                section = section.add(
                    widget::settings::item::builder(source.name.clone())
                        .description(description)
                        .icon(widget::icon::from_name(if source.enabled {
                            "emblem-ok-symbolic"
                        } else {
                            "window-close-symbolic"
                        }))
                        .control(controls),
                );
            }

            column = column.push(section);
        }

        let (name, url, suite) = &ready.source_draft;
        let can_add = !name.trim().is_empty() && !url.trim().is_empty();

        column = column.push(
            widget::settings::section()
                .title(fl!("sources-add-heading"))
                .add(widget::text::caption(fl!("sources-flatpak-hint")))
                .add(widget::text::caption(fl!("sources-apt-hint")))
                .add(
                    widget::settings::item::builder(fl!("sources-name-placeholder")).control(
                        widget::text_input(fl!("sources-name-placeholder"), name.clone())
                            .on_input(|text| Message::DraftSource(0, text))
                            .width(Length::Fixed(260.0)),
                    ),
                )
                .add(
                    widget::settings::item::builder(fl!("sources-url-placeholder")).control(
                        widget::text_input(fl!("sources-url-placeholder"), url.clone())
                            .on_input(|text| Message::DraftSource(1, text))
                            .width(Length::Fixed(260.0)),
                    ),
                )
                .add(
                    widget::settings::item::builder(fl!("sources-suite-placeholder")).control(
                        widget::text_input(fl!("sources-suite-placeholder"), suite.clone())
                            .on_input(|text| Message::DraftSource(2, text))
                            .width(Length::Fixed(260.0)),
                    ),
                )
                .add(
                    widget::settings::item::builder(String::new()).control(
                        widget::row::with_children(Vec::new())
                            .spacing(8)
                            .push(
                                widget::button::suggested(fl!("sources-add-flatpak"))
                                    .on_press_maybe(
                                        can_add.then_some(Message::AddSource(
                                            repos::Kind::Flatpak,
                                        )),
                                    ),
                            )
                            .push(
                                widget::button::standard(fl!("sources-add-apt")).on_press_maybe(
                                    (can_add && !suite.trim().is_empty())
                                        .then_some(Message::AddSource(repos::Kind::Apt)),
                                ),
                            ),
                    ),
                ),
        );

        widget::scrollable(column).into()
    }

    /// Past runs, or the transcript of one of them.
    fn view_history<'a>(&'a self, ready: &'a Ready) -> Element<'a, Message> {
        // Viewing a transcript replaces the list rather than sitting beside it:
        // a run's output is wide and long, and splitting the width would make
        // both halves worse.
        if let Some((id, text)) = &ready.viewing {
            let lines = text
                .lines()
                .fold(widget::column::with_children(Vec::new()).spacing(2), |column, line| {
                    column.push(widget::text::monotext(line.to_owned()))
                });

            return widget::column::with_children(Vec::new())
                .spacing(12)
                .push(
                    widget::row::with_children(Vec::new())
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .push(
                            widget::button::standard(fl!("history-back"))
                                .on_press(Message::RefreshHistory),
                        )
                        .push(widget::text::title4(id.clone())),
                )
                .push(
                    widget::container(widget::scrollable(lines))
                        .height(Length::Fill)
                        .width(Length::Fill),
                )
                .into();
        }

        let mut column = widget::column::with_children(Vec::new())
            .spacing(12)
            .push(widget::text::title2(fl!("history-heading")));

        if ready.history.is_empty() {
            return column.push(widget::text::body(fl!("history-none"))).into();
        }

        let mut section = widget::settings::section();
        for record in &ready.history {
            let outcome = match record.outcome {
                RunOutcome::Succeeded => fl!("history-outcome-succeeded"),
                RunOutcome::Failed => fl!("history-outcome-failed"),
                RunOutcome::Cancelled => fl!("history-outcome-cancelled"),
            };
            let origin = match record.origin {
                Origin::Manual => fl!("history-origin-manual"),
                Origin::Scheduled => fl!("history-origin-scheduled"),
            };

            let mut detail = fl!(
                "history-detail",
                outcome = outcome,
                origin = origin,
                duration = fl!("history-duration-seconds", seconds = record.duration().to_string())
            );
            detail.push_str(&format!(
                " · {}",
                fl!(
                    "run-summary",
                    ok = record.ok.to_string(),
                    skipped = record.skipped.to_string(),
                    failed = record.failed.to_string()
                )
            ));
            if record.dry_run {
                detail.push_str(&format!(" · {}", fl!("dry-run")));
            }

            let id = record.id.clone();
            let delete_id = record.id.clone();
            section = section.add(
                widget::settings::item::builder(record.started_local())
                    .description(detail)
                    .icon(widget::icon::from_name(match record.outcome {
                        RunOutcome::Failed => "dialog-error-symbolic",
                        RunOutcome::Cancelled => "process-stop-symbolic",
                        RunOutcome::Succeeded => "emblem-ok-symbolic",
                    }))
                    .control(
                        widget::row::with_children(Vec::new())
                            .spacing(4)
                            .push(
                                widget::button::text(fl!("view"))
                                    .on_press(Message::SelectHistory(id)),
                            )
                            .push(
                                widget::button::icon(widget::icon::from_name(
                                    "user-trash-symbolic",
                                ))
                                .on_press(Message::DeleteHistory(delete_id)),
                            ),
                    ),
            );
        }

        column = column.push(section);
        if let Some(status) = &ready.status {
            column = column.push(widget::text::caption(status.clone()));
        }
        widget::scrollable(column).into()
    }

    fn view_schedule<'a>(&'a self, ready: &'a Ready) -> Element<'a, Message> {
        let schedule = self.config.schedule;

        let backend_note = match ready.backend {
            Backend::Systemd => fl!("schedule-backend-systemd"),
            Backend::InApp => fl!("schedule-backend-fallback"),
        };

        let frequency_index = Frequency::ALL
            .iter()
            .position(|frequency| *frequency == schedule.frequency);

        let mut section = widget::settings::section()
            .title(fl!("schedule-heading"))
            .add(
                widget::settings::item::builder(fl!("schedule-enabled"))
                    .description(backend_note)
                    .toggler(schedule.enabled, Message::ScheduleEnabled),
            )
            .add(
                widget::settings::item::builder(fl!("schedule-frequency")).control(
                    widget::dropdown(
                        &self.frequency_labels,
                        frequency_index,
                        Message::ScheduleFrequency,
                    ),
                ),
            );

        // An hourly schedule runs every hour, so only the minute means
        // anything; offering an hour picker would imply otherwise.
        if schedule.frequency != Frequency::Hourly {
            section = section.add(
                widget::settings::item::builder(fl!("schedule-time")).control(
                    widget::row::with_children(Vec::new())
                        .spacing(8)
                        .push(widget::dropdown(
                            &self.hour_labels,
                            Some(schedule.hour.min(23) as usize),
                            Message::ScheduleHour,
                        ))
                        .push(widget::dropdown(
                            &self.minute_labels,
                            Some((schedule.minute.min(59) / 5) as usize),
                            Message::ScheduleMinute,
                        )),
                ),
            );
        } else {
            section = section.add(
                widget::settings::item::builder(fl!("schedule-time")).control(widget::dropdown(
                    &self.minute_labels,
                    Some((schedule.minute.min(59) / 5) as usize),
                    Message::ScheduleMinute,
                )),
            );
        }

        section = section.add(
            widget::settings::item::builder(fl!("schedule-automatic"))
                .description(fl!("schedule-automatic-description"))
                .toggler(schedule.automatic, Message::ScheduleAutomatic),
        );

        let mut column = widget::column::with_children(Vec::new())
            .spacing(12)
            .push(widget::text::title2(fl!("schedule-heading")))
            .push(section)
            .push(widget::button::suggested(fl!("schedule-apply")).on_press(Message::ScheduleApply));

        column = column.push(widget::text::caption(match &ready.next_run {
            Some(when) => fl!("schedule-next-run", when = when.clone()),
            None => fl!("schedule-next-run-unknown"),
        }));

        if let Some(status) = &ready.status {
            column = column.push(widget::text::caption(status.clone()));
        }

        widget::scrollable(column).into()
    }

    fn view_configuration<'a>(&'a self, ready: &'a Ready) -> Element<'a, Message> {
        let mut column = widget::column::with_children(Vec::new())
            .spacing(12)
            .push(widget::text::title2(fl!("configuration-heading")))
            .push(widget::text::caption(fl!(
                "configuration-path",
                path = ready.settings.path().display().to_string()
            )));

        if ready.settings.is_dirty() {
            column = column.push(
                widget::row::with_children(Vec::new())
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(widget::text::body(fl!("configuration-unsaved")))
                    .push(
                        widget::button::suggested(fl!("configuration-save"))
                            .on_press(Message::SaveSettings),
                    ),
            );
        }

        if let Some(status) = &ready.status {
            column = column.push(widget::text::caption(status.clone()));
        }

        for section in &ready.schema.sections {
            column = column.push(self.view_section(ready, section));
        }

        widget::scrollable(column).into()
    }

    /// One configuration section, in the form its contents call for.
    ///
    /// Shared by the configuration page and the per-category settings, so the
    /// same section edited from either place is the same controls backed by the
    /// same document — there is no second implementation to drift.
    fn view_section<'a>(
        &'a self,
        ready: &'a Ready,
        section: &'a crate::topgrade::schema::Section,
    ) -> Element<'a, Message> {
        let mut view = widget::settings::section().title(section.name.clone());
        if !section.doc.is_empty() {
            view = view.add(widget::text::caption(section.doc.clone()));
        }

        if section.free_form {
            return self.view_free_form_section(ready, section, view).into();
        }

        for setting in &section.settings {
            // The step list has its own controls on the category pages. Editing
            // it in two places would let them contradict.
            if (setting.section.as_str(), setting.key.as_str()) == DISABLE_KEY {
                continue;
            }
            view = view.add(self.view_setting(ready, setting));
        }

        view.into()
    }

    /// A section whose entries the user names: the custom-command lists.
    ///
    /// There is no schema to build a form from here, so the section is its own
    /// small editor — every entry gets a field and a way to remove it, and there
    /// is always a blank row at the bottom to add another.
    fn view_free_form_section<'a>(
        &'a self,
        ready: &'a Ready,
        section: &'a crate::topgrade::schema::Section,
        mut view: widget::settings::Section<'a, Message>,
    ) -> widget::settings::Section<'a, Message> {
        let name = section.name.clone();
        view = view.add(widget::text::caption(fl!("custom-commands-description")));

        let entries = ready.settings.free_form_entries(&name);
        if entries.is_empty() {
            view = view.add(widget::text::caption(fl!("custom-commands-none")));
        }

        for (entry, command) in entries {
            let edit_section = name.clone();
            let edit_name = entry.clone();
            let remove_section = name.clone();
            let remove_name = entry.clone();

            view = view.add(
                widget::settings::item::builder(entry)
                    .control(
                        widget::row::with_children(Vec::new())
                            .spacing(4)
                            .align_y(Alignment::Center)
                            .push(
                                widget::text_input(fl!("command-value-placeholder"), command)
                                    .on_input(move |value| {
                                        Message::EditCommand(
                                            edit_section.clone(),
                                            edit_name.clone(),
                                            value,
                                        )
                                    })
                                    .width(Length::Fixed(320.0)),
                            )
                            .push(
                                widget::button::icon(widget::icon::from_name(
                                    "user-trash-symbolic",
                                ))
                                .on_press(Message::RemoveCommand(remove_section, remove_name)),
                            ),
                    ),
            );
        }

        // The row for a new entry. Held as a draft rather than written on every
        // keystroke, so a half-typed name never reaches the file.
        let draft = ready.drafts.get(&name).cloned().unwrap_or_default();
        let name_for_input = name.clone();
        let value_for_input = name.clone();
        let name_for_add = name.clone();
        let can_add = !draft.0.trim().is_empty() && !draft.1.trim().is_empty();

        let mut add = widget::button::suggested(fl!("configuration-add"));
        if can_add {
            add = add.on_press(Message::AddCommand(name_for_add));
        }

        view.add(
            widget::settings::item::builder(fl!("configuration-add")).control(
                widget::row::with_children(Vec::new())
                    .spacing(4)
                    .align_y(Alignment::Center)
                    .push(
                        widget::text_input(fl!("command-name-placeholder"), draft.0)
                            .on_input(move |value| {
                                Message::DraftCommandName(name_for_input.clone(), value)
                            })
                            .width(Length::Fixed(150.0)),
                    )
                    .push(
                        widget::text_input(fl!("command-value-placeholder"), draft.1)
                            .on_input(move |value| {
                                Message::DraftCommandValue(value_for_input.clone(), value)
                            })
                            .width(Length::Fixed(240.0)),
                    )
                    .push(add),
            ),
        )
    }

    /// One control, chosen by the kind of value the setting takes.
    fn view_setting<'a>(&'a self, ready: &'a Ready, setting: &'a Setting) -> Element<'a, Message> {
        let current = ready.settings.get(&setting.section, &setting.key);

        let description = match (&setting.default, &current) {
            (Some(default), None) => fl!("configuration-default", value = default.clone()),
            (None, None) => fl!("configuration-not-set"),
            _ => setting.summary().to_owned(),
        };

        let item = widget::settings::item::builder(setting.label()).description(description);

        let section_name = setting.section.clone();
        let key = setting.key.clone();

        match &setting.kind {
            ValueKind::Bool => {
                // An unset boolean shows topgrade's stated default, so the
                // switch reflects what will actually happen rather than
                // defaulting to off.
                let value = match current {
                    Some(SettingValue::Bool(value)) => value,
                    _ => setting.default.as_deref() == Some("true"),
                };
                item.control(widget::toggler(value).on_toggle(move |value| {
                    Message::EditSetting(
                        section_name.clone(),
                        key.clone(),
                        SettingValue::Bool(value),
                    )
                }))
                .into()
            }

            ValueKind::Enum { options } => {
                let selected = current.as_ref().and_then(|value| match value {
                    SettingValue::Text(text) => options.iter().position(|option| option == text),
                    _ => None,
                });
                let chosen = options.clone();
                item.control(widget::dropdown(
                    // Borrowed from the schema, which outlives the element.
                    options.as_slice(),
                    selected,
                    move |index| {
                        Message::EditSetting(
                            section_name.clone(),
                            key.clone(),
                            // The dropdown reports a position; the value written
                            // is the option that sits there.
                            SettingValue::Text(chosen[index].clone()),
                        )
                    },
                ))
                .into()
            }

            kind => {
                let stored = match &current {
                    Some(SettingValue::Text(text)) => text.clone(),
                    Some(SettingValue::Integer(number)) => number.to_string(),
                    Some(SettingValue::List(values)) => values.join(", "),
                    Some(SettingValue::Bool(value)) => value.to_string(),
                    None => String::new(),
                };
                // A half-typed value lives in `edits` until it parses, so the
                // box does not fight the user by rewriting what they typed.
                let text = ready
                    .edits
                    .get(&format!("{}.{}", setting.section, setting.key))
                    .cloned()
                    .unwrap_or(stored);

                let placeholder = match kind {
                    ValueKind::StepList | ValueKind::StringList => setting.example.clone(),
                    _ => setting.default.clone().unwrap_or_default(),
                };

                item.control(
                    widget::text_input(placeholder, text)
                        .on_input(move |text| {
                            Message::EditText(section_name.clone(), key.clone(), text)
                        })
                        .width(Length::Fixed(280.0)),
                )
                .into()
            }
        }
    }

    fn view_app_settings(&self) -> Element<'_, Message> {
        let theme_index = THEME_OPTIONS
            .iter()
            .position(|theme| *theme == self.config.app_theme);
        let privilege_index = PRIVILEGE_OPTIONS
            .iter()
            .position(|mode| *mode == self.config.privilege_mode);

        let privilege_description = match self.config.privilege_mode {
            PrivilegeMode::AskInWindow => fl!("privilege-pty-description"),
            PrivilegeMode::SystemDialog => fl!("privilege-pkexec-description"),
        };

        widget::settings::view_column(vec![
            widget::settings::section()
                .title(fl!("appearance"))
                .add(
                    widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                        &self.theme_labels,
                        theme_index,
                        |index| Message::ConfigTheme(THEME_OPTIONS[index]),
                    )),
                )
                .into(),
            widget::settings::section()
                .title(fl!("behaviour"))
                .add(
                    widget::settings::item::builder(fl!("privilege-backend"))
                        .description(privilege_description)
                        .control(widget::dropdown(
                            &self.privilege_labels,
                            privilege_index,
                            |index| Message::ConfigPrivilege(PRIVILEGE_OPTIONS[index]),
                        )),
                )
                .add(
                    widget::settings::item::builder(fl!("confirm-before-running")).toggler(
                        self.config.confirm_before_running,
                        Message::ConfigConfirmBeforeRunning,
                    ),
                )
                .add(
                    widget::settings::item::builder(fl!("run-selected-only"))
                        .toggler(self.config.assume_yes, Message::ConfigAssumeYes),
                )
                .add(
                    widget::settings::item::builder(fl!("show-unavailable"))
                        .description(fl!("show-unavailable-tooltip"))
                        .toggler(
                            self.config.show_unavailable_steps,
                            Message::ConfigShowUnavailable,
                        ),
                )
                .add(
                    widget::settings::item::builder(fl!("notify-upgrades"))
                        .description(self.notification_wording())
                        .toggler(self.config.notify_upgrades, Message::ConfigNotifyUpgrades),
                )
                .add(
                    widget::settings::item::builder(fl!("notify-errors"))
                        .description(fl!("notify-errors-description"))
                        .toggler(self.config.notify_errors, Message::ConfigNotifyErrors),
                )
                .add(
                    widget::settings::item::builder(fl!("autostart"))
                        .description(fl!("autostart-description"))
                        // Read from the filesystem rather than from stored
                        // state, so the switch shows what the session will
                        // actually do even if the file was edited by hand.
                        .toggler(autostart::is_enabled(), Message::ConfigAutostart),
                )
                .add(
                    widget::settings::item::builder(fl!("show-tray-icon"))
                        .description(fl!("show-tray-icon-description"))
                        .toggler(self.config.show_tray_icon, Message::ConfigShowTrayIcon),
                )
                .add(
                    widget::settings::item::builder(fl!("welcome-show-again"))
                        .description(fl!("welcome-show-again-description"))
                        .control(
                            widget::button::standard(fl!("nav-welcome"))
                                .on_press(Message::ShowWelcome),
                        ),
                )
                .into(),
        ])
        .into()
    }

    fn view_about(&self) -> Element<'_, Message> {
        widget::column::with_children(Vec::new())
            .spacing(12)
            .align_x(Alignment::Center)
            .push(widget::icon::from_name(crate::constants::APP_ICON).size(72))
            .push(widget::text::title3(fl!("app-title")))
            .push(widget::text::body(env!("CARGO_PKG_VERSION")))
            .push(widget::text::body(fl!("app-description")))
            .push(
                widget::button::link(fl!("repository"))
                    .on_press(Message::LaunchUrl(
                        crate::constants::REPOSITORY_URL.to_owned(),
                    ))
                    .padding(0),
            )
            .push(
                widget::button::link(fl!("support"))
                    .on_press(Message::LaunchUrl(crate::constants::ISSUES_URL.to_owned()))
                    .padding(0),
            )
            .apply(widget::container)
            .center_x(Length::Fill)
            .into()
    }
}

/// Whether the run log should still pin itself to the newest line.
///
/// Split out from the message handler so the reasoning above can be tested:
/// the two cases it separates are easy to state and awkward to check by hand in
/// a window that is scrolling past at speed.
fn should_follow_log(
    following: bool,
    relative_y: f32,
    absolute_y: f32,
    last_absolute_y: f32,
) -> bool {
    if relative_y >= LOG_FOLLOW_THRESHOLD {
        // Sitting at the bottom, however it got there.
        true
    } else if absolute_y + LOG_SCROLL_TOLERANCE < last_absolute_y {
        // Moved backwards through the content, which only a deliberate scroll
        // does — growing the content leaves this position where it was.
        false
    } else {
        following
    }
}

/// Whether a virus scanner is available to offer scanning with.
fn which_clamscan() -> bool {
    std::env::var_os("PATH")
        .map(|path| {
            std::env::split_paths(&path).any(|directory| directory.join("clamscan").is_file())
        })
        .unwrap_or(false)
}

/// Seconds since the Unix epoch.
///
/// A clock set before 1970 reads as zero, which the schedule treats as "never
/// run" — the safe reading, since it declines to start anything.
fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

/// Size used for step icons in list rows.
const _: u16 = ICON_SIZE_ROW;

#[cfg(test)]
mod tests {
    use super::*;

    /// Appending a line makes the content taller while leaving the scroll
    /// position alone, so the position as a fraction of the content falls even
    /// though nobody touched anything. This is the case that breaks a naive
    /// "is the fraction near 1.0?" test, and it happens on every line.
    #[test]
    fn output_arriving_does_not_stop_the_log_following() {
        let still_following = should_follow_log(true, 0.94, 1_000.0, 1_000.0);
        assert!(still_following);
    }

    #[test]
    fn scrolling_up_stops_the_log_following() {
        // The position moved backwards, which growth never does.
        assert!(!should_follow_log(true, 0.60, 400.0, 1_000.0));
    }

    #[test]
    fn scrolling_up_is_honoured_even_while_output_is_arriving() {
        // Content grew *and* the user scrolled in the same frame; the backwards
        // movement is what decides it, so a busy upgrade cannot trap the view.
        assert!(!should_follow_log(true, 0.55, 820.0, 1_000.0));
    }

    #[test]
    fn a_sub_pixel_wobble_is_not_treated_as_scrolling_up() {
        assert!(should_follow_log(true, 0.95, 999.7, 1_000.0));
    }

    #[test]
    fn returning_to_the_bottom_resumes_following() {
        assert!(should_follow_log(false, 1.0, 2_000.0, 1_400.0));
    }

    #[test]
    fn output_arriving_does_not_drag_a_scrolled_up_view_back() {
        // Reading something further up: growth must not re-enable following.
        assert!(!should_follow_log(false, 0.40, 400.0, 400.0));
    }

    #[test]
    fn scrolling_down_without_reaching_the_bottom_keeps_it_off() {
        assert!(!should_follow_log(false, 0.80, 700.0, 400.0));
    }
}
