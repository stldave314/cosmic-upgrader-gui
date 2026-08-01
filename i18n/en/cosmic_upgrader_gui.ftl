app-title = Upgrader
app-description = Configure, schedule and run system-wide upgrades with topgrade.

## Navigation

nav-overview = Overview
nav-schedule = Schedule
nav-configuration = Configuration
nav-run = Run

category-system = System
category-applications = Applications
category-containers = Containers
category-development = Development
category-editors = Editors
category-repositories = Repositories
category-shell = Shell
category-ai-tools = AI Tools
category-cloud = Cloud
category-desktop = Desktop
category-custom = Custom Commands
category-other = Other

## Overview

overview-heading = Upgrade sources
overview-subtitle = { $available } of { $total } steps apply to this system.
topgrade-version = topgrade { $version }
topgrade-source-system = Installed on this system
topgrade-source-bundled = Bundled with this application
scanning = Checking what applies to this system…
scanning-progress = Checked { $completed } of { $total } — { $step }
rescan = Rescan
rescan-tooltip = Check again for newly installed tools

## Steps

steps-heading = Steps
steps-none = No steps in this category.
step-available = Ready
step-unavailable = Unavailable
step-inactive = Not applicable
step-deprecated = Deprecated
step-enabled-tooltip = Include this step when upgrading
step-disabled-tooltip = Skip this step when upgrading
step-components = { $count ->
    [one] 1 component
   *[other] { $count } components
}
enable-all = Enable all
disable-all = Disable all
show-unavailable = Show unavailable steps
show-unavailable-tooltip = Also list steps whose tools are not installed

status-ok = Ready
status-skipped = Skipped
status-failed = Failed

## Running

run-heading = Run
run-now = Run upgrade
dry-run = Preview
dry-run-tooltip = Show what would be done without changing anything
run-in-progress = Upgrading…
run-step = { $step }
run-finished = Finished
run-cancelled = Cancelled
run-failed = Finished with errors
run-never = No upgrade has been run yet.
run-last = Last run { $when }
run-summary = { $ok } succeeded, { $skipped } skipped, { $failed } failed
cancel-run = Stop
clear-log = Clear
copy-log = Copy output
run-selected-only = Run only enabled steps

## Authentication

password-title = Administrator password required
password-body = { $command } needs administrator rights to continue.
password-placeholder = Password
authenticate = Authenticate
authentication-failed = That password was not accepted.

## Schedule

schedule-heading = Scheduled upgrades
schedule-enabled = Check for upgrades on a schedule
schedule-frequency = Frequency
frequency-hourly = Hourly
frequency-daily = Daily
frequency-weekly = Weekly
frequency-monthly = Monthly
schedule-time = Time of day
schedule-automatic = Install upgrades automatically
schedule-automatic-description = When off, a notification reports what is available and nothing is changed.
schedule-next-run = Next run { $when }
schedule-next-run-unknown = Next run time is not known.
schedule-backend-systemd = Runs in the background through a systemd user timer, even when this window is closed.
schedule-backend-fallback = systemd is not available, so scheduled runs only happen while this window is open.
schedule-apply = Apply schedule
schedule-applied = Schedule updated.
schedule-error = The schedule could not be applied: { $message }

## Configuration

configuration-heading = topgrade configuration
configuration-path = Editing { $path }
configuration-default = Default: { $value }
configuration-not-set = Not set
configuration-save = Save changes
configuration-revert = Revert
configuration-reset = Reset to default
configuration-unsaved = You have unsaved changes.
configuration-saved = Configuration saved.
configuration-free-form = These are commands you name yourself. Edit this section in the file directly.
configuration-open-file = Open configuration file
configuration-add = Add
configuration-remove = Remove

## Application settings

settings = Settings
about = About
appearance = Appearance
theme = Theme
theme-system = Match desktop
theme-light = Light
theme-dark = Dark
behaviour = Behaviour
privilege-backend = Administrator rights
privilege-pty = Ask in this window
privilege-pty-description = Runs topgrade in a terminal and prompts here when a password is needed.
privilege-pkexec = System dialog
privilege-pkexec-description = Uses the desktop's own authentication dialog. Prompts once per command.
confirm-before-running = Confirm before starting an upgrade
notify-on-completion = Notify when a scheduled run finishes

## Errors and empty states

topgrade-missing-title = topgrade is not installed
topgrade-missing-body = This application drives topgrade, which could not be found on this system.
topgrade-missing-hint = Install it with your package manager, or with: { $command }
topgrade-too-old-title = topgrade is too old
topgrade-too-old-body = topgrade { $found } was found, but { $required } or newer is needed.
error-title = Something went wrong
retry = Try again

## Common

cancel = Cancel
close = Close
save = Save
ok = OK
toggle-sidebar = Toggle sidebar
git-description = Git description
repository = Repository
support = Support

## History

nav-history = History
history-heading = Past runs
history-none = No runs have been recorded yet.
view = View
history-delete = Delete
history-back = Back to list
history-origin-manual = Started here
history-origin-scheduled = Scheduled
history-outcome-succeeded = Succeeded
history-outcome-failed = Failed
history-outcome-cancelled = Cancelled
history-detail = { $outcome } · { $origin } · { $duration }
history-duration-seconds = { $seconds }s
history-transcript-unavailable = The transcript for this run could not be read.
keep-run-logs = Runs to keep

## Notifications

notify-title-succeeded = Upgrade finished
notify-title-failed = Upgrade finished with errors
notify-failed-steps = Failed: { $steps }

## First run

first-run-title = A couple of choices
first-run-body = These change how the application behaves outside its own window. You can change them later in Settings.
first-run-autostart = Start with the desktop session
first-run-autostart-description = Launches minimized at login so scheduled checks can run.
first-run-tray = Show an icon in the status area
first-run-tray-description = Lets you hide the window and bring it back, and start an upgrade without opening it.
first-run-accept = Continue
autostart = Start with the desktop session
autostart-description = Adds an entry to ~/.config/autostart
show-tray-icon = Show an icon in the status area

## Category settings and custom commands

category-settings = Settings for this category
category-settings-none = This category has no topgrade settings of its own.
command-name-placeholder = Name
command-value-placeholder = Command to run
custom-commands-none = No custom commands yet.
custom-commands-description = Commands you name yourself. topgrade runs them as their own step.

## Status area

tray-show = Show window
tray-quit = Quit
tray-unavailable = No status area was found on this desktop, so no icon is shown.

## Releases

nav-releases = Releases
releases-heading = Project releases
releases-description = Software installed from a project's releases page has no package manager behind it, so topgrade cannot update it. These are checked against the project itself.
releases-none = No projects are being watched yet.
releases-find = Find projects
releases-finding = Looking through installed packages…
releases-found = { $count } projects found on this system. Choose which to watch.
releases-check = Check for updates
releases-checking = Checking { $done } of { $total }…
releases-watched = { $count } watched
releases-update-available = { $version } available
releases-up-to-date = Up to date
releases-unknown = { $version } published
releases-no-releases = No releases published
releases-error = Could not check: { $message }
releases-update = Update
releases-open = Release page
releases-remove = Stop watching
releases-add-selected = Watch selected
releases-cancel-find = Cancel
releases-installing = Installing { $name }…
releases-installed = { $name } updated to { $version }
releases-install-failed = { $name } could not be updated: { $message }
releases-no-asset = No file on this release matches this system; use the release page.
releases-no-transport = Neither curl nor gh is installed, so releases cannot be checked.
releases-unidentified = No project known — { $version } installed
releases-source = { $source } · { $forge }
releases-interval = Check automatically
releases-last-checked = Last checked { $when }
releases-never-checked = Not checked yet
interval-manual = Only when asked
interval-six-hourly = Every 6 hours
interval-daily = Daily
interval-weekly = Weekly
releases-next-check = next { $when }
run-was-preview = This was a preview — nothing on the system was changed.

## Dependencies

nav-dependencies = Dependencies
dependencies-heading = Required tools
dependencies-description = This application works by driving other programs. A missing one turns into a feature that quietly does nothing, so they are listed here with what each is for.
dependencies-required = Required
dependencies-optional = Optional
dependencies-installed = Installed
dependencies-missing = Not installed
dependencies-install = Install
dependencies-installing = Installing…
dependencies-recheck = Check again
dependencies-all-present = Everything this application needs is installed.
dependencies-missing-required = { $count ->
    [one] 1 required tool is missing.
   *[other] { $count } required tools are missing.
}
dependencies-no-manager = No supported package manager was found, so these cannot be installed from here.
dependencies-install-failed = { $name } could not be installed: { $message }
dep-authentication-dismissed = Authentication was dismissed.
dep-topgrade = Performs the upgrades themselves. Without it this application has nothing to drive.
dep-curl = Fetches release information from project hosts, and downloads updates.
dep-gh = Carries your GitHub credentials, raising the release-check limit from 60 requests an hour to 5000.
dep-pkexec = Asks for administrator rights through the desktop's own dialog, for system upgrades and package installs.
dep-notify-send = Reports the result of a scheduled run, which nobody was watching.
dep-systemctl = Keeps the upgrade schedule as a systemd user timer, so it runs when this window is closed.
dep-xdg-open = Opens release pages and links in your browser.

## Release channel and directories

releases-channel = Releases to offer
channel-stable = Stable only
channel-pre-release = Include betas and release candidates
releases-self = This application
releases-directories = Where downloaded applications are kept
releases-directories-description = Searched for AppImages and other downloaded programs. Relative paths are taken from your home directory.
releases-directory-add = Add directory
releases-directory-placeholder = Applications
releases-channel-description = Whether release candidates and betas count as updates.
notify-title-installed = Upgrades installed
notify-title-available = Upgrades are available

## Welcome

nav-welcome = Welcome
welcome-heading = Set up upgrades
welcome-body = A few choices worth making now. All of them are in Settings afterwards, and nothing here is permanent.
welcome-finish = Done
welcome-notifications = Being told about upgrades
welcome-automatic-heading = Installing upgrades
welcome-root-warning = Unattended installation needs administrator rights, so the scheduled run is installed as a system service running as root. Nothing else in this application runs as root.
welcome-clamav = Virus scanning
clamav-scan = Scan after the virus database updates
clamav-scan-description = ClamAV is installed. topgrade keeps its database current; this scans with the new database once it changes.
clamav-options = Scan options
clamav-target = What to scan
notify-upgrades = Tell me about upgrades
notify-upgrades-installed = You will be told what was installed.
notify-upgrades-available = You will be told what is available to install.
notify-errors = Tell me when an upgrade fails
notify-errors-description = Failures are reported even when other notifications are off, unless this is turned off too.
clamav-scanning = Virus database changed — scanning…
clamav-clean = Scan finished: { $scanned } files checked, nothing found.
clamav-infected = Scan finished: { $infected } infected file(s) found.
clamav-failed = The scan could not run: { $message }

## Package sources

nav-sources = Package sources
sources-heading = Where packages come from
sources-description = The repositories your package managers pull from. topgrade upgrades what is installed; these decide what is available in the first place.
sources-none = No package sources were found.
sources-reload = Reload
sources-enabled = Enabled
sources-disabled = Disabled
sources-privileged = Changing this needs administrator rights.
sources-remove = Remove
sources-disable-note = APT and dnf sources are turned off rather than deleted, so a change can be undone by hand.
sources-add-heading = Add a source
sources-add-apt = Add APT source
sources-add-flatpak = Add Flatpak remote
sources-name-placeholder = Name
sources-url-placeholder = URL
sources-suite-placeholder = Suite (e.g. stable)
sources-flatpak-hint = A Flatpak remote is added for you alone and needs no password. Point it at a .flatpakrepo URL.
sources-apt-hint = An APT source is written to /etc/apt/sources.list.d and needs administrator rights.
sources-changing = Applying…
show-tray-icon-description = Raises the window, starts an upgrade without opening it, and quits. It cannot hide the window — Wayland gives no way to undo that.
welcome-show-again = Set-up screen
welcome-show-again-description = Go back to the first screen and its choices.
releases-restarting = Updated — restarting into the new version…
releases-restart-failed = The new version could not be started: { $message }
releases-restart-unknown-path = this application's own location is not known
