app-title = Aktualisierer
app-description = Systemweite Aktualisierungen mit topgrade einrichten, planen und ausführen.

## Navigation

nav-overview = Übersicht
nav-schedule = Zeitplan
nav-configuration = Konfiguration
nav-run = Ausführen

category-system = System
category-applications = Anwendungen
category-containers = Container
category-development = Entwicklung
category-editors = Editoren
category-repositories = Repositorien
category-shell = Shell
category-ai-tools = KI-Werkzeuge
category-cloud = Cloud
category-desktop = Schreibtisch
category-custom = Eigene Befehle
category-other = Sonstige

## Overview

overview-heading = Aktualisierungsquellen
overview-subtitle = { $available } von { $total } Schritten gelten für dieses System.
topgrade-version = topgrade { $version }
topgrade-source-system = Auf diesem System installiert
topgrade-source-bundled = Mit dieser Anwendung geliefert
scanning = Prüfe, was für dieses System gilt …
scanning-progress = { $completed } von { $total } geprüft — { $step }
rescan = Erneut prüfen
rescan-tooltip = Erneut nach neu installierten Werkzeugen suchen

## Steps

steps-heading = Schritte
steps-none = Keine Schritte in dieser Kategorie.
step-available = Bereit
step-unavailable = Nicht verfügbar
step-inactive = Nicht zutreffend
step-deprecated = Veraltet
step-enabled-tooltip = Diesen Schritt beim Aktualisieren einbeziehen
step-disabled-tooltip = Diesen Schritt beim Aktualisieren überspringen
step-components = { $count ->
    [one] 1 Komponente
   *[other] { $count } Komponenten
}
enable-all = Alle aktivieren
disable-all = Alle deaktivieren
show-unavailable = Nicht verfügbare Schritte anzeigen
show-unavailable-tooltip = Auch Schritte auflisten, deren Werkzeuge nicht installiert sind

status-ok = Bereit
status-skipped = Übersprungen
status-failed = Fehlgeschlagen

## Running

run-heading = Ausführen
run-now = Aktualisierung starten
dry-run = Vorschau
dry-run-tooltip = Zeigen, was getan würde, ohne etwas zu ändern
run-in-progress = Aktualisiere …
run-step = { $step }
run-finished = Abgeschlossen
run-cancelled = Abgebrochen
run-failed = Mit Fehlern beendet
run-never = Es wurde noch keine Aktualisierung ausgeführt.
run-last = Zuletzt ausgeführt { $when }
run-summary = { $ok } erfolgreich, { $skipped } übersprungen, { $failed } fehlgeschlagen
cancel-run = Anhalten
clear-log = Leeren
copy-log = Ausgabe kopieren
run-selected-only = Nachfragen der Paketverwaltungen automatisch bejahen

## Authentication

password-title = Administratorkennwort erforderlich
password-body = { $command } benötigt Administratorrechte, um fortzufahren.
password-placeholder = Kennwort
authenticate = Authentifizieren
authentication-failed = Dieses Kennwort wurde nicht akzeptiert.

## Schedule

schedule-heading = Geplante Aktualisierungen
schedule-enabled = Regelmäßig nach Aktualisierungen suchen
schedule-frequency = Häufigkeit
frequency-hourly = Stündlich
frequency-daily = Täglich
frequency-weekly = Wöchentlich
frequency-monthly = Monatlich
schedule-time = Uhrzeit
schedule-automatic = Aktualisierungen automatisch installieren
schedule-automatic-description = Ist dies aus, meldet eine Benachrichtigung nur, was verfügbar ist; nichts wird geändert.
schedule-next-run = Nächste Ausführung { $when }
schedule-next-run-unknown = Der nächste Ausführungszeitpunkt ist nicht bekannt.
schedule-backend-systemd = Läuft im Hintergrund über einen systemd-Benutzertimer, auch wenn dieses Fenster geschlossen ist.
schedule-backend-fallback = systemd ist nicht verfügbar, daher laufen geplante Ausführungen nur, solange dieses Fenster offen ist.
schedule-apply = Zeitplan anwenden
schedule-applied = Zeitplan aktualisiert.
schedule-error = Der Zeitplan konnte nicht angewendet werden: { $message }

## Configuration

configuration-heading = topgrade-Konfiguration
configuration-path = { $path } wird bearbeitet
configuration-default = Vorgabe: { $value }
configuration-not-set = Nicht gesetzt
configuration-save = Änderungen speichern
configuration-revert = Verwerfen
configuration-reset = Auf Vorgabe zurücksetzen
configuration-unsaved = Es gibt ungespeicherte Änderungen.
configuration-saved = Konfiguration gespeichert.
configuration-free-form = Dies sind selbst benannte Befehle. Bearbeiten Sie diesen Abschnitt direkt in der Datei.
configuration-open-file = Konfigurationsdatei öffnen
configuration-add = Hinzufügen
configuration-remove = Entfernen

## Application settings

settings = Einstellungen
about = Über
appearance = Erscheinungsbild
theme = Erscheinungsbild
theme-system = Wie der Schreibtisch
theme-light = Hell
theme-dark = Dunkel
behaviour = Verhalten
privilege-backend = Administratorrechte
privilege-pty = In diesem Fenster fragen
privilege-pty-description = Führt topgrade in einem Terminal aus und fragt hier, wenn ein Kennwort benötigt wird.
privilege-pkexec = Systemdialog
privilege-pkexec-description = Nutzt den Authentifizierungsdialog des Schreibtischs. Fragt einmal pro Befehl.
confirm-before-running = Vor dem Start einer Aktualisierung nachfragen
notify-on-completion = Benachrichtigen, wenn eine geplante Ausführung endet

## Errors and empty states

topgrade-missing-title = topgrade ist nicht installiert
topgrade-missing-body = Diese Anwendung steuert topgrade, das auf diesem System nicht gefunden wurde.
topgrade-missing-hint = Installieren Sie es über Ihre Paketverwaltung oder mit: { $command }
topgrade-too-old-title = topgrade ist zu alt
topgrade-too-old-body = topgrade { $found } wurde gefunden, benötigt wird aber { $required } oder neuer.
error-title = Etwas ist schiefgelaufen
retry = Erneut versuchen

## Common

cancel = Abbrechen
close = Schließen
save = Speichern
ok = OK
toggle-sidebar = Seitenleiste umschalten
git-description = Git-Beschreibung
repository = Repositorium
support = Unterstützung

## History, first run, custom commands and status area

autostart = Mit der Sitzung starten
autostart-description = Fügt einen Eintrag in ~/.config/autostart hinzu
category-settings = Einstellungen für diese Kategorie
category-settings-none = Diese Kategorie hat keine eigenen topgrade-Einstellungen.
command-name-placeholder = Name
command-value-placeholder = Auszuführender Befehl
custom-commands-description = Selbst benannte Befehle. topgrade führt sie als eigenen Schritt aus.
custom-commands-none = Noch keine eigenen Befehle.
first-run-accept = Weiter
first-run-autostart = Mit der Sitzung starten
first-run-autostart-description = Startet minimiert bei der Anmeldung, damit geplante Prüfungen laufen können.
first-run-body = Diese Einstellungen ändern, wie sich die Anwendung außerhalb ihres eigenen Fensters verhält. Sie können sie später in den Einstellungen ändern.
first-run-title = Ein paar Entscheidungen
first-run-tray = Symbol im Systembereich anzeigen
first-run-tray-description = Ermöglicht es, das Fenster auszublenden und zurückzuholen und eine Aktualisierung zu starten, ohne es zu öffnen.
history-back = Zurück zur Liste
history-delete = Löschen
history-detail = { $outcome } · { $origin } · { $duration }
history-duration-seconds = { $seconds } s
history-heading = Frühere Ausführungen
history-none = Es wurden noch keine Ausführungen aufgezeichnet.
history-origin-manual = Hier gestartet
history-origin-scheduled = Geplant
history-outcome-cancelled = Abgebrochen
history-outcome-failed = Fehlgeschlagen
history-outcome-succeeded = Erfolgreich
history-transcript-unavailable = Das Protokoll dieser Ausführung konnte nicht gelesen werden.
view = Ansehen
keep-run-logs = Aufzubewahrende Ausführungen
nav-history = Verlauf
notify-failed-steps = Fehlgeschlagen: { $steps }
notify-title-failed = Aktualisierung mit Fehlern beendet
notify-title-succeeded = Aktualisierung abgeschlossen
show-tray-icon = Symbol im Systembereich anzeigen
tray-quit = Beenden
tray-show = Fenster anzeigen
tray-unavailable = Auf dieser Arbeitsumgebung wurde kein Systembereich gefunden, daher wird kein Symbol angezeigt.

## Releases

nav-releases = Veröffentlichungen
releases-add-selected = Auswahl beobachten
releases-cancel-find = Abbrechen
releases-check = Nach Aktualisierungen suchen
releases-checking = { $done } von { $total } geprüft …
releases-description = Software, die von der Veröffentlichungsseite eines Projekts installiert wurde, hat keine Paketverwaltung hinter sich, daher kann topgrade sie nicht aktualisieren. Diese werden beim Projekt selbst geprüft.
releases-error = Prüfung nicht möglich: { $message }
releases-find = Projekte suchen
releases-finding = Installierte Pakete werden durchsucht …
releases-found = { $count } Projekte auf diesem System gefunden. Wählen Sie aus, welche beobachtet werden sollen.
releases-heading = Projektveröffentlichungen
releases-installed = { $name } auf { $version } aktualisiert
releases-install-failed = { $name } konnte nicht aktualisiert werden: { $message }
releases-installing = { $name } wird installiert …
releases-no-asset = Keine Datei dieser Veröffentlichung passt zu diesem System; nutzen Sie die Veröffentlichungsseite.
releases-none = Es werden noch keine Projekte beobachtet.
releases-no-releases = Keine Veröffentlichungen
releases-no-transport = Weder curl noch gh ist installiert, daher können Veröffentlichungen nicht geprüft werden.
releases-open = Veröffentlichungsseite
releases-remove = Nicht mehr beobachten
releases-source = { $source } · { $forge }
releases-unidentified = Kein Projekt bekannt — { $version } installiert
releases-unknown = { $version } veröffentlicht
releases-update = Aktualisieren
releases-update-available = { $version } verfügbar
releases-up-to-date = Aktuell
releases-watched = { $count } beobachtet
run-was-preview = Dies war eine Vorschau — am System wurde nichts geändert.

interval-daily = Täglich
interval-manual = Nur auf Anfrage
interval-six-hourly = Alle 6 Stunden
interval-weekly = Wöchentlich
releases-interval = Automatisch prüfen
releases-last-checked = Zuletzt geprüft { $when }
releases-never-checked = Noch nicht geprüft
releases-next-check = nächste { $when }

## Dependencies, release channel and directories

channel-pre-release = Betas und Veröffentlichungskandidaten einbeziehen
channel-stable = Nur stabile
dep-authentication-dismissed = Die Authentifizierung wurde abgebrochen.
dep-curl = Holt Veröffentlichungsinformationen von Projekt-Hosts und lädt Aktualisierungen herunter.
dependencies-all-present = Alles, was diese Anwendung benötigt, ist installiert.
dependencies-description = Diese Anwendung arbeitet, indem sie andere Programme steuert. Ein fehlendes Programm wird zu einer Funktion, die stillschweigend nichts tut, daher sind sie hier mit ihrem Zweck aufgeführt.
dependencies-heading = Benötigte Werkzeuge
dependencies-install = Installieren
dependencies-installed = Installiert
dependencies-install-failed = { $name } konnte nicht installiert werden: { $message }
dependencies-installing = Wird installiert …
dependencies-missing = Nicht installiert
dependencies-no-manager = Es wurde keine unterstützte Paketverwaltung gefunden, daher kann von hier aus nichts installiert werden.
dependencies-optional = Optional
dependencies-recheck = Erneut prüfen
dependencies-required = Erforderlich
dep-gh = Übermittelt Ihre GitHub-Anmeldedaten und erhöht das Limit für Veröffentlichungsprüfungen von 60 auf 5000 Anfragen pro Stunde.
dep-notify-send = Meldet das Ergebnis einer geplanten Ausführung, die niemand beobachtet hat.
dep-pkexec = Fragt über den Dialog der Arbeitsumgebung nach Administratorrechten, für Systemaktualisierungen und Paketinstallationen.
dep-systemctl = Führt den Aktualisierungszeitplan als systemd-Benutzertimer, damit er auch bei geschlossenem Fenster läuft.
dep-topgrade = Führt die Aktualisierungen selbst durch. Ohne dieses Programm hat diese Anwendung nichts zu steuern.
dep-xdg-open = Öffnet Veröffentlichungsseiten und Links in Ihrem Browser.
nav-dependencies = Abhängigkeiten
releases-channel = Anzubietende Veröffentlichungen
releases-directories = Verzeichnisse für heruntergeladene Anwendungen
releases-directories-description = Wird nach AppImages und anderen heruntergeladenen Programmen durchsucht. Relative Pfade beziehen sich auf Ihr persönliches Verzeichnis.
releases-directory-add = Verzeichnis hinzufügen
releases-directory-placeholder = Applications
releases-self = Diese Anwendung
dependencies-missing-required = { $count ->
    [one] 1 benötigtes Werkzeug fehlt.
   *[other] { $count } benötigte Werkzeuge fehlen.
}
releases-channel-description = Ob Veröffentlichungskandidaten und Betas als Aktualisierungen gelten.

## Welcome, notifications and virus scanning

clamav-clean = Prüfung beendet: { $scanned } Dateien geprüft, nichts gefunden.
clamav-failed = Die Prüfung konnte nicht ausgeführt werden: { $message }
clamav-infected = Prüfung beendet: { $infected } infizierte Datei(en) gefunden.
clamav-options = Prüfoptionen
clamav-scan = Nach Aktualisierung der Virendatenbank prüfen
clamav-scan-description = ClamAV ist installiert. topgrade hält die Datenbank aktuell; dies prüft mit der neuen Datenbank, sobald sie sich ändert.
clamav-scanning = Virendatenbank geändert — Prüfung läuft …
clamav-target = Was geprüft wird
nav-welcome = Willkommen
notify-errors = Bei fehlgeschlagener Aktualisierung benachrichtigen
notify-errors-description = Fehlschläge werden auch dann gemeldet, wenn andere Benachrichtigungen aus sind — es sei denn, dies wird ebenfalls ausgeschaltet.
notify-title-available = Aktualisierungen sind verfügbar
notify-title-installed = Aktualisierungen installiert
notify-upgrades = Über Aktualisierungen informieren
notify-upgrades-available = Sie werden erfahren, was installiert werden kann.
notify-upgrades-installed = Sie werden erfahren, was installiert wurde.
welcome-automatic-heading = Aktualisierungen installieren
welcome-body = Ein paar Entscheidungen, die jetzt sinnvoll sind. Alle finden sich danach in den Einstellungen, und nichts davon ist endgültig.
welcome-clamav = Virenprüfung
welcome-finish = Fertig
welcome-heading = Aktualisierungen einrichten
welcome-notifications = Benachrichtigungen
welcome-root-warning = Unbeaufsichtigtes Installieren erfordert Administratorrechte, daher wird die geplante Ausführung als Systemdienst mit Root-Rechten eingerichtet. Nichts anderes in dieser Anwendung läuft als Root.

## Package sources

nav-sources = Paketquellen
sources-add-apt = APT-Quelle hinzufügen
sources-add-flatpak = Flatpak-Remote hinzufügen
sources-add-heading = Quelle hinzufügen
sources-apt-hint = Eine APT-Quelle wird nach /etc/apt/sources.list.d geschrieben und erfordert Administratorrechte.
sources-changing = Wird angewendet …
sources-description = Die Repositorien, aus denen Ihre Paketverwaltungen beziehen. topgrade aktualisiert, was installiert ist; diese bestimmen, was überhaupt verfügbar ist.
sources-disable-note = APT- und dnf-Quellen werden abgeschaltet statt gelöscht, damit eine Änderung von Hand rückgängig gemacht werden kann.
sources-disabled = Deaktiviert
sources-enabled = Aktiviert
sources-flatpak-hint = Ein Flatpak-Remote wird nur für Sie hinzugefügt und benötigt kein Kennwort. Verweisen Sie auf eine .flatpakrepo-URL.
sources-heading = Woher Pakete kommen
sources-name-placeholder = Name
sources-none = Es wurden keine Paketquellen gefunden.
sources-privileged = Diese Änderung erfordert Administratorrechte.
sources-reload = Neu laden
sources-remove = Entfernen
sources-suite-placeholder = Suite (z. B. stable)
sources-url-placeholder = URL
show-tray-icon-description = Holt das Fenster nach vorn, startet eine Aktualisierung ohne es zu öffnen und beendet die Anwendung. Es kann das Fenster nicht ausblenden — Wayland bietet keine Möglichkeit, das rückgängig zu machen.
welcome-show-again = Einrichtungsbildschirm
welcome-show-again-description = Zurück zum ersten Bildschirm und seinen Entscheidungen.

releases-restart-failed = Die neue Version konnte nicht gestartet werden: { $message }
releases-restart-unknown-path = der eigene Speicherort dieser Anwendung ist nicht bekannt
releases-restarting = Aktualisiert — Neustart in die neue Version …

releases-found-dismiss = Diese nicht
releases-found-new-why = Aus einer Datei installiert, also hat keine Paketverwaltung eine neuere. Auf Veröffentlichungen beobachten?
releases-found-watch = Diese beobachten
upgrade-releases-with-run = Auch von Veröffentlichungsseiten installierte Programme aktualisieren
upgrade-releases-with-run-description = topgrade aktualisiert diese nur, wenn sie über ein Werkzeug installiert wurden, das sie verfolgt. Ein von Hand installiertes Paket deckt keines ab, daher übernimmt das eine Aktualisierung hier.
releases-found-new = { $count ->
    [one] 1 installiertes Programm, das sonst nichts aktualisiert
   *[other] { $count } installierte Programme, die sonst nichts aktualisiert
}
