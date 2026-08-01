// SPDX-License-Identifier: GPL-3.0

//! An icon in the panel's status area.
//!
//! COSMIC's status area speaks the freedesktop StatusNotifierItem protocol —
//! `cosmic-applet-status-area` owns `org.kde.StatusNotifierWatcher` — so an
//! ordinary D-Bus item appears in it. `ksni` implements that protocol; this
//! module is the small amount on top which says what the item is and turns
//! clicks into messages the application already understands.
//!
//! ## What hiding means here
//!
//! The window is *hidden*, not closed: `window::set_mode(Hidden)` leaves it
//! alive and simply stops showing it. That matters, because in this revision of
//! libcosmic an application cannot outlive its main window — `exit_on_close` and
//! `exit_on_main_window_closed` are both `pub(crate)` with no setter, so there
//! is no supported way to keep the event loop running once the window has
//! actually closed.
//!
//! The consequence is worth being plain about: hiding to the status area works
//! from the application's own control and from this menu, but the window
//! manager's close button still quits, because libcosmic closes the window
//! before the application is consulted. Turning that into "close to the status
//! area" needs a public setter upstream.

use tokio::sync::mpsc;

use crate::constants::{APP_ICON, APP_ID};
use crate::debug::UI;
use crate::debug_log;
use crate::fl;

/// What the user asked for from the status area.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    /// Bring the window back.
    Show,
    /// Put the window away, leaving the icon.
    Hide,
    /// Start an upgrade without opening the window first.
    Run,
    Quit,
}

/// The item itself.
///
/// Holds only a channel: everything it can do is something the application does,
/// and duplicating any of that state here would give it a second copy able to
/// disagree with the window.
struct Item {
    commands: mpsc::UnboundedSender<Command>,
    /// Shown in the menu so the item reports what the application is doing
    /// without the window being open.
    running: bool,
}

impl Item {
    fn send(&self, command: Command) {
        // A closed channel means the application has gone; the item is about to
        // be shut down with it, so there is nothing useful to report.
        let _ = self.commands.send(command);
    }
}

impl ksni::Tray for Item {
    fn id(&self) -> String {
        APP_ID.to_owned()
    }

    fn title(&self) -> String {
        fl!("app-title")
    }

    fn icon_name(&self) -> String {
        APP_ICON.to_owned()
    }

    fn category(&self) -> ksni::Category {
        // Not `ApplicationStatus`: this is a system maintenance tool, and the
        // category is what decides where some panels place it.
        ksni::Category::SystemServices
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: fl!("app-title"),
            description: if self.running {
                fl!("run-in-progress")
            } else {
                fl!("app-description")
            },
            icon_name: APP_ICON.to_owned(),
            icon_pixmap: Vec::new(),
        }
    }

    /// A left click shows the window, which is what a click on a tray icon
    /// almost always means.
    fn activate(&mut self, _x: i32, _y: i32) {
        self.send(Command::Show);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::{MenuItem, StandardItem};

        vec![
            MenuItem::Standard(StandardItem {
                label: fl!("tray-show"),
                icon_name: "window-new-symbolic".to_owned(),
                activate: Box::new(|item: &mut Self| item.send(Command::Show)),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: fl!("tray-hide"),
                icon_name: "window-close-symbolic".to_owned(),
                activate: Box::new(|item: &mut Self| item.send(Command::Hide)),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: fl!("run-now"),
                icon_name: "system-software-update-symbolic".to_owned(),
                // Disabled rather than hidden while a run is in progress, so the
                // menu does not change shape under the pointer.
                enabled: !self.running,
                activate: Box::new(|item: &mut Self| item.send(Command::Run)),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: fl!("tray-quit"),
                icon_name: "application-exit-symbolic".to_owned(),
                activate: Box::new(|item: &mut Self| item.send(Command::Quit)),
                ..Default::default()
            }),
        ]
    }
}

/// A running status-area item.
pub struct Tray {
    handle: ksni::Handle<Item>,
}

impl Tray {
    /// Register an item and return a handle to it, plus the stream of commands
    /// it will produce.
    ///
    /// Returns `None` when there is no status-area implementation listening —
    /// which is an ordinary state, not an error: plenty of desktops have no
    /// tray, and the application is perfectly usable without one.
    pub async fn start() -> Option<(Self, mpsc::UnboundedReceiver<Command>)> {
        use ksni::TrayMethods;

        let (commands, receiver) = mpsc::unbounded_channel();
        let item = Item {
            commands,
            running: false,
        };

        match item.spawn().await {
            Ok(handle) => {
                debug_log!(UI, "status area item registered");
                Some((Self { handle }, receiver))
            }
            Err(error) => {
                debug_log!(UI, "no status area available: {error}");
                None
            }
        }
    }

    /// Tell the item whether an upgrade is under way, so its menu and tooltip
    /// match what the application is doing.
    pub async fn set_running(&self, running: bool) {
        self.handle
            .update(|item: &mut Item| item.running = running)
            .await;
    }

    /// Remove the item from the status area.
    pub async fn shutdown(&self) {
        debug_log!(UI, "removing status area item");
        self.handle.shutdown().await;
    }
}
