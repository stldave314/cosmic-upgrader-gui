app-title = Bijwerken
app-description = Systeembrede opwaarderingen instellen, plannen en uitvoeren met topgrade.

## Navigation

nav-overview = Overzicht
nav-schedule = Planning
nav-configuration = Configuratie
nav-run = Uitvoeren

category-system = Systeem
category-applications = Toepassingen
category-containers = Containers
category-development = Ontwikkeling
category-editors = Editors
category-repositories = Repository's
category-shell = Shell
category-ai-tools = AI-hulpmiddelen
category-cloud = Cloud
category-desktop = Bureaublad
category-custom = Eigen opdrachten
category-other = Overig

## Overview

overview-heading = Bijwerkbronnen
overview-subtitle = { $available } van de { $total } stappen gelden voor dit systeem.
topgrade-version = topgrade { $version }
topgrade-source-system = Geïnstalleerd op dit systeem
topgrade-source-bundled = Meegeleverd met deze toepassing
scanning = Nagaan wat op dit systeem van toepassing is…
scanning-progress = { $completed } van { $total } nagegaan — { $step }
rescan = Opnieuw nagaan
rescan-tooltip = Opnieuw zoeken naar pas geïnstalleerde hulpmiddelen

## Steps

steps-heading = Stappen
steps-none = Geen stappen in deze categorie.
step-available = Gereed
step-unavailable = Niet beschikbaar
step-inactive = Niet van toepassing
step-deprecated = Verouderd
step-enabled-tooltip = Deze stap meenemen bij het bijwerken
step-disabled-tooltip = Deze stap overslaan bij het bijwerken
step-components = { $count ->
    [one] 1 onderdeel
   *[other] { $count } onderdelen
}
enable-all = Alles inschakelen
disable-all = Alles uitschakelen
show-unavailable = Niet-beschikbare stappen tonen
show-unavailable-tooltip = Ook stappen tonen waarvan de hulpmiddelen niet geïnstalleerd zijn

status-ok = Gereed
status-skipped = Overgeslagen
status-failed = Mislukt

## Running

run-heading = Uitvoeren
run-now = Bijwerken starten
dry-run = Voorbeeld
dry-run-tooltip = Tonen wat er zou gebeuren zonder iets te wijzigen
run-in-progress = Bezig met bijwerken…
run-step = { $step }
run-finished = Voltooid
run-cancelled = Geannuleerd
run-failed = Voltooid met fouten
run-never = Er is nog geen opwaardering uitgevoerd.
run-last = Laatst uitgevoerd { $when }
run-summary = { $ok } geslaagd, { $skipped } overgeslagen, { $failed } mislukt
cancel-run = Stoppen
clear-log = Wissen
copy-log = Uitvoer kopiëren
run-selected-only = Bevestigend antwoorden op pakketbeheerders

## Authentication

password-title = Beheerderswachtwoord vereist
password-body = { $command } heeft beheerdersrechten nodig om door te gaan.
password-placeholder = Wachtwoord
authenticate = Aanmelden
authentication-failed = Dat wachtwoord is niet geaccepteerd.

## Schedule

schedule-heading = Geplande opwaarderingen
schedule-enabled = Volgens een planning op updates controleren
schedule-frequency = Frequentie
frequency-hourly = Elk uur
frequency-daily = Dagelijks
frequency-weekly = Wekelijks
frequency-monthly = Maandelijks
schedule-time = Tijdstip
schedule-automatic = Opwaarderingen automatisch installeren
schedule-automatic-description = Staat dit uit, dan meldt een melding wat beschikbaar is en wordt er niets gewijzigd.
schedule-next-run = Volgende uitvoering { $when }
schedule-next-run-unknown = Het tijdstip van de volgende uitvoering is niet bekend.
schedule-backend-systemd = Draait op de achtergrond via een systemd-gebruikerstimer, ook als dit venster gesloten is.
schedule-backend-fallback = systemd is niet beschikbaar, dus geplande uitvoeringen gebeuren alleen zolang dit venster open is.
schedule-apply = Planning toepassen
schedule-applied = Planning bijgewerkt.
schedule-error = De planning kon niet worden toegepast: { $message }

## Configuration

configuration-heading = topgrade-configuratie
configuration-path = { $path } wordt bewerkt
configuration-default = Standaard: { $value }
configuration-not-set = Niet ingesteld
configuration-save = Wijzigingen opslaan
configuration-revert = Ongedaan maken
configuration-reset = Terugzetten op standaard
configuration-unsaved = Er zijn niet-opgeslagen wijzigingen.
configuration-saved = Configuratie opgeslagen.
configuration-free-form = Dit zijn opdrachten die u zelf een naam geeft. Bewerk dit onderdeel rechtstreeks in het bestand.
configuration-open-file = Configuratiebestand openen
configuration-add = Toevoegen
configuration-remove = Verwijderen

## Application settings

settings = Instellingen
about = Over
appearance = Vormgeving
theme = Thema
theme-system = Zoals het bureaublad
theme-light = Licht
theme-dark = Donker
behaviour = Gedrag
privilege-backend = Beheerdersrechten
privilege-pty = In dit venster vragen
privilege-pty-description = Voert topgrade uit in een terminal en vraagt hier wanneer een wachtwoord nodig is.
privilege-pkexec = Systeemvenster
privilege-pkexec-description = Gebruikt het aanmeldvenster van het bureaublad. Vraagt eenmaal per opdracht.
confirm-before-running = Bevestigen voordat een opwaardering start
notify-on-completion = Melden wanneer een geplande uitvoering klaar is

## Errors and empty states

topgrade-missing-title = topgrade is niet geïnstalleerd
topgrade-missing-body = Deze toepassing stuurt topgrade aan, dat niet op dit systeem is gevonden.
topgrade-missing-hint = Installeer het met uw pakketbeheerder, of met: { $command }
topgrade-too-old-title = topgrade is te oud
topgrade-too-old-body = topgrade { $found } is gevonden, maar { $required } of nieuwer is nodig.
error-title = Er is iets misgegaan
retry = Opnieuw proberen

## Common

cancel = Annuleren
close = Sluiten
save = Opslaan
ok = OK
toggle-sidebar = Zijbalk tonen/verbergen
git-description = Git-beschrijving
repository = Repository
support = Ondersteuning

## History, first run, custom commands and status area

autostart = Starten met de sessie
autostart-description = Voegt een item toe in ~/.config/autostart
category-settings = Instellingen voor deze categorie
category-settings-none = Deze categorie heeft geen eigen topgrade-instellingen.
command-name-placeholder = Naam
command-value-placeholder = Uit te voeren opdracht
custom-commands-description = Opdrachten die u zelf een naam geeft. topgrade voert ze uit als een eigen stap.
custom-commands-none = Nog geen eigen opdrachten.
first-run-accept = Doorgaan
first-run-autostart = Starten met de sessie
first-run-autostart-description = Start geminimaliseerd bij aanmelden, zodat geplande controles kunnen draaien.
first-run-body = Deze opties veranderen hoe de toepassing zich buiten haar eigen venster gedraagt. U kunt ze later wijzigen bij Instellingen.
first-run-title = Een paar keuzes
first-run-tray = Een pictogram in het statusgebied tonen
first-run-tray-description = Hiermee kunt u het venster verbergen en terughalen, en bijwerken starten zonder het te openen.
hide-to-tray = Verbergen in het statusgebied
history-back = Terug naar de lijst
history-delete = Verwijderen
history-detail = { $outcome } · { $origin } · { $duration }
history-duration-seconds = { $seconds } s
history-heading = Eerdere uitvoeringen
history-none = Er is nog geen uitvoering vastgelegd.
history-origin-manual = Hier gestart
history-origin-scheduled = Gepland
history-outcome-cancelled = Geannuleerd
history-outcome-failed = Mislukt
history-outcome-succeeded = Geslaagd
history-transcript-unavailable = Het logboek van deze uitvoering kon niet worden gelezen.
view = Bekijken
keep-run-logs = Te bewaren uitvoeringen
minimize-to-tray = Verbergen in het statusgebied in plaats van afsluiten
minimize-to-tray-description = Voegt een knop Verbergen toe. De sluitknop van het venster sluit nog steeds af.
nav-history = Geschiedenis
notify-failed-steps = Mislukt: { $steps }
notify-title-failed = Bijwerken voltooid met fouten
notify-title-succeeded = Bijwerken voltooid
show-tray-icon = Een pictogram in het statusgebied tonen
tray-hide = Venster verbergen
tray-quit = Afsluiten
tray-show = Venster tonen
tray-unavailable = Op dit bureaublad is geen statusgebied gevonden, dus wordt er geen pictogram getoond.

## Releases

nav-releases = Uitgaven
releases-add-selected = Selectie volgen
releases-cancel-find = Annuleren
releases-check = Zoeken naar updates
releases-checking = { $done } van { $total } gecontroleerd…
releases-description = Software die is geïnstalleerd via de uitgavepagina van een project heeft geen pakketbeheerder erachter, dus topgrade kan het niet bijwerken. Deze worden bij het project zelf gecontroleerd.
releases-error = Controleren mislukt: { $message }
releases-find = Projecten zoeken
releases-finding = Geïnstalleerde pakketten doorzoeken…
releases-found = { $count } projecten op dit systeem gevonden. Kies welke u wilt volgen.
releases-heading = Projectuitgaven
releases-installed = { $name } bijgewerkt naar { $version }
releases-install-failed = { $name } kon niet worden bijgewerkt: { $message }
releases-installing = { $name } wordt geïnstalleerd…
releases-no-asset = Geen bestand in deze uitgave past bij dit systeem; gebruik de uitgavepagina.
releases-none = Er worden nog geen projecten gevolgd.
releases-no-releases = Geen uitgaven
releases-no-transport = Noch curl noch gh is geïnstalleerd, dus uitgaven kunnen niet worden gecontroleerd.
releases-open = Uitgavepagina
releases-remove = Niet meer volgen
releases-source = { $source } · { $forge }
releases-unidentified = Project onbekend — { $version } geïnstalleerd
releases-unknown = { $version } uitgebracht
releases-update = Bijwerken
releases-update-available = { $version } beschikbaar
releases-up-to-date = Actueel
releases-watched = { $count } gevolgd
run-was-preview = Dit was een voorbeeld — er is niets op het systeem gewijzigd.

interval-daily = Dagelijks
interval-manual = Alleen op verzoek
interval-six-hourly = Elke 6 uur
interval-weekly = Wekelijks
releases-interval = Automatisch controleren
releases-last-checked = Laatst gecontroleerd { $when }
releases-never-checked = Nog niet gecontroleerd
releases-next-check = volgende { $when }

## Dependencies, release channel and directories

channel-pre-release = Bèta's en release-kandidaten meenemen
channel-stable = Alleen stabiele
dep-authentication-dismissed = De aanmelding is geannuleerd.
dep-curl = Haalt uitgave-informatie op bij projecthosts en downloadt updates.
dependencies-all-present = Alles wat deze toepassing nodig heeft, is geïnstalleerd.
dependencies-description = Deze toepassing werkt door andere programma's aan te sturen. Een ontbrekend programma wordt een functie die stilletjes niets doet, daarom staan ze hier met hun doel.
dependencies-heading = Benodigde hulpmiddelen
dependencies-install = Installeren
dependencies-installed = Geïnstalleerd
dependencies-install-failed = { $name } kon niet worden geïnstalleerd: { $message }
dependencies-installing = Bezig met installeren…
dependencies-missing = Niet geïnstalleerd
dependencies-no-manager = Er is geen ondersteunde pakketbeheerder gevonden, dus hiervandaan kan niets worden geïnstalleerd.
dependencies-optional = Optioneel
dependencies-recheck = Opnieuw controleren
dependencies-required = Vereist
dep-gh = Gebruikt uw GitHub-gegevens, waardoor de limiet van 60 naar 5000 verzoeken per uur gaat.
dep-notify-send = Meldt het resultaat van een geplande uitvoering waar niemand naar keek.
dep-pkexec = Vraagt beheerdersrechten via het venster van het bureaublad, voor systeemupgrades en pakketinstallaties.
dep-systemctl = Houdt de planning als systemd-gebruikerstimer, zodat die draait als dit venster gesloten is.
dep-topgrade = Voert de opwaarderingen zelf uit. Zonder dit heeft deze toepassing niets aan te sturen.
dep-xdg-open = Opent uitgavepagina's en koppelingen in uw browser.
nav-dependencies = Afhankelijkheden
releases-channel = Uitgaven om aan te bieden
releases-directories = Mappen met gedownloade toepassingen
releases-directories-description = Hierin wordt gezocht naar AppImages en andere gedownloade programma's. Relatieve paden gaan uit van uw persoonlijke map.
releases-directory-add = Map toevoegen
releases-directory-placeholder = Applications
releases-self = Deze toepassing
dependencies-missing-required = { $count ->
    [one] Er ontbreekt 1 vereist hulpmiddel.
   *[other] Er ontbreken { $count } vereiste hulpmiddelen.
}
releases-channel-description = Of release-kandidaten en bèta's als updates tellen.

## Welcome, notifications and virus scanning

clamav-clean = Scan klaar: { $scanned } bestanden gecontroleerd, niets gevonden.
clamav-failed = De scan kon niet worden uitgevoerd: { $message }
clamav-infected = Scan klaar: { $infected } geïnfecteerd(e) bestand(en) gevonden.
clamav-options = Scanopties
clamav-scan = Scannen nadat de virusdatabase is bijgewerkt
clamav-scan-description = ClamAV is geïnstalleerd. topgrade houdt de database actueel; dit scant met de nieuwe database zodra die verandert.
clamav-scanning = Virusdatabase gewijzigd — bezig met scannen…
clamav-target = Wat er wordt gescand
nav-welcome = Welkom
notify-errors = Waarschuw me als een opwaardering mislukt
notify-errors-description = Mislukkingen worden gemeld ook als andere meldingen uitstaan, tenzij dit ook wordt uitgezet.
notify-title-available = Er zijn opwaarderingen beschikbaar
notify-title-installed = Opwaarderingen geïnstalleerd
notify-upgrades = Vertel me over opwaarderingen
notify-upgrades-available = U hoort wat er beschikbaar is om te installeren.
notify-upgrades-installed = U hoort wat er is geïnstalleerd.
welcome-automatic-heading = Opwaarderingen installeren
welcome-body = Een paar keuzes die nu de moeite waard zijn. Ze staan daarna allemaal bij Instellingen, en niets hiervan is blijvend.
welcome-clamav = Virusscan
welcome-finish = Klaar
welcome-heading = Opwaarderingen instellen
welcome-notifications = Meldingen
welcome-root-warning = Onbeheerd installeren vereist beheerdersrechten, dus de geplande uitvoering wordt geïnstalleerd als systeemdienst die als root draait. Niets anders in deze toepassing draait als root.
