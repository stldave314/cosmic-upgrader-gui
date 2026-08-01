// SPDX-License-Identifier: GPL-3.0

//! An icon in the panel's status area.
//!
//! COSMIC's status area speaks the freedesktop StatusNotifierItem protocol —
//! `cosmic-applet-status-area` owns `org.kde.StatusNotifierWatcher` — so an
//! ordinary D-Bus item appears in it. `ksni` implements that protocol; this
//! module is the small amount on top which says what the item is and turns
//! clicks into messages the application already understands.
//!
//! ## Why there is no "hide to the status area"
//!
//! Because it cannot be made to work here, and half of it working is worse than
//! none of it. Three routes were tried against COSMIC:
//!
//! * `window::set_mode(Hidden)` is accepted and does nothing. Driving it through
//!   this very menu over D-Bus and comparing the screen before and after shows
//!   an unchanged display.
//! * `window::minimize(id, true)` does work — but nothing can undo it. Wayland's
//!   xdg-shell has `set_minimized` and deliberately no inverse: a client cannot
//!   un-minimize itself, so the icon could put the window away and never bring
//!   it back.
//! * Closing the window ends the application, since libcosmic's
//!   `exit_on_main_window_closed` is `pub(crate)` with no setter.
//!
//! So the item is not offered. What the status area *can* do it does: raise the
//! window, start an upgrade without opening it, and quit.

use tokio::sync::mpsc;

use crate::constants::{APP_ICON_SYMBOLIC, APP_ID};
use crate::debug::UI;
use crate::debug_log;
use crate::fl;

/// What the user asked for from the status area.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    /// Bring the window back.
    Show,
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
        APP_ICON_SYMBOLIC.to_owned()
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
            icon_name: APP_ICON_SYMBOLIC.to_owned(),
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
