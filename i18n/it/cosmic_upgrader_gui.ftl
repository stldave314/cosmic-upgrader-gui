app-title = Aggiornamenti
app-description = Configura, pianifica ed esegui aggiornamenti di sistema con topgrade.

## Navigation

nav-overview = Panoramica
nav-schedule = Pianificazione
nav-configuration = Configurazione
nav-run = Esegui

category-system = Sistema
category-applications = Applicazioni
category-containers = Container
category-development = Sviluppo
category-editors = Editor
category-repositories = Repository
category-shell = Shell
category-ai-tools = Strumenti di IA
category-cloud = Cloud
category-desktop = Scrivania
category-custom = Comandi personalizzati
category-other = Altro

## Overview

overview-heading = Sorgenti di aggiornamento
overview-subtitle = { $available } passaggi su { $total } si applicano a questo sistema.
topgrade-version = topgrade { $version }
topgrade-source-system = Installato su questo sistema
topgrade-source-bundled = Incluso con questa applicazione
scanning = Verifica di ciò che si applica a questo sistema…
scanning-progress = Verificati { $completed } su { $total } — { $step }
rescan = Verifica di nuovo
rescan-tooltip = Cerca di nuovo gli strumenti installati di recente

## Steps

steps-heading = Passaggi
steps-none = Nessun passaggio in questa categoria.
step-available = Pronto
step-unavailable = Non disponibile
step-inactive = Non applicabile
step-deprecated = Deprecato
step-enabled-tooltip = Includi questo passaggio durante l'aggiornamento
step-disabled-tooltip = Salta questo passaggio durante l'aggiornamento
step-components = { $count ->
    [one] 1 componente
   *[other] { $count } componenti
}
enable-all = Attiva tutto
disable-all = Disattiva tutto
show-unavailable = Mostra i passaggi non disponibili
show-unavailable-tooltip = Elenca anche i passaggi i cui strumenti non sono installati

status-ok = Pronto
status-skipped = Saltato
status-failed = Non riuscito

## Running

run-heading = Esegui
run-now = Avvia l'aggiornamento
dry-run = Anteprima
dry-run-tooltip = Mostra cosa verrebbe fatto senza modificare nulla
run-in-progress = Aggiornamento in corso…
run-step = { $step }
run-finished = Completato
run-cancelled = Annullato
run-failed = Completato con errori
run-never = Nessun aggiornamento è ancora stato eseguito.
run-last = Ultima esecuzione { $when }
run-summary = { $ok } riusciti, { $skipped } saltati, { $failed } non riusciti
cancel-run = Ferma
clear-log = Pulisci
copy-log = Copia l'output
run-selected-only = Rispondi affermativamente ai gestori di pacchetti

## Authentication

password-title = È richiesta la password di amministratore
password-body = { $command } richiede i permessi di amministratore per continuare.
password-placeholder = Password
authenticate = Autentica
authentication-failed = Questa password non è stata accettata.

## Schedule

schedule-heading = Aggiornamenti pianificati
schedule-enabled = Cerca aggiornamenti secondo una pianificazione
schedule-frequency = Frequenza
frequency-hourly = Ogni ora
frequency-daily = Ogni giorno
frequency-weekly = Ogni settimana
frequency-monthly = Ogni mese
schedule-time = Ora del giorno
schedule-automatic = Installa gli aggiornamenti automaticamente
schedule-automatic-description = Se disattivato, una notifica segnala cosa è disponibile e nulla viene modificato.
schedule-next-run = Prossima esecuzione { $when }
schedule-next-run-unknown = L'orario della prossima esecuzione non è noto.
schedule-backend-systemd = Viene eseguito in background tramite un timer utente di systemd, anche a finestra chiusa.
schedule-backend-fallback = systemd non è disponibile, quindi le esecuzioni pianificate avvengono solo mentre questa finestra è aperta.
schedule-apply = Applica la pianificazione
schedule-applied = Pianificazione aggiornata.
schedule-error = Non è stato possibile applicare la pianificazione: { $message }

## Configuration

configuration-heading = Configurazione di topgrade
configuration-path = Modifica di { $path }
configuration-default = Predefinito: { $value }
configuration-not-set = Non impostato
configuration-save = Salva le modifiche
configuration-revert = Annulla
configuration-reset = Ripristina il valore predefinito
configuration-unsaved = Ci sono modifiche non salvate.
configuration-saved = Configurazione salvata.
configuration-free-form = Sono comandi a cui dai un nome tu. Modifica questa sezione direttamente nel file.
configuration-open-file = Apri il file di configurazione
configuration-add = Aggiungi
configuration-remove = Rimuovi

## Application settings

settings = Impostazioni
about = Informazioni
appearance = Aspetto
theme = Tema
theme-system = Come la scrivania
theme-light = Chiaro
theme-dark = Scuro
behaviour = Comportamento
privilege-backend = Permessi di amministratore
privilege-pty = Chiedi in questa finestra
privilege-pty-description = Esegue topgrade in un terminale e chiede qui quando serve una password.
privilege-pkexec = Finestra di sistema
privilege-pkexec-description = Usa la finestra di autenticazione della scrivania. Chiede una volta per comando.
confirm-before-running = Chiedi conferma prima di avviare un aggiornamento
notify-on-completion = Notifica al termine di un'esecuzione pianificata

## Errors and empty states

topgrade-missing-title = topgrade non è installato
topgrade-missing-body = Questa applicazione pilota topgrade, che non è stato trovato su questo sistema.
topgrade-missing-hint = Installalo con il tuo gestore di pacchetti, oppure con: { $command }
topgrade-too-old-title = topgrade è troppo vecchio
topgrade-too-old-body = È stato trovato topgrade { $found }, ma serve { $required } o successivo.
error-title = Qualcosa è andato storto
retry = Riprova

## Common

cancel = Annulla
close = Chiudi
save = Salva
ok = OK
toggle-sidebar = Mostra/nascondi la barra laterale
git-description = Descrizione Git
repository = Repository
support = Supporto

## History, first run, custom commands and status area

autostart = Avvia con la sessione
autostart-description = Aggiunge una voce in ~/.config/autostart
category-settings = Impostazioni di questa categoria
category-settings-none = Questa categoria non ha impostazioni proprie di topgrade.
command-name-placeholder = Nome
command-value-placeholder = Comando da eseguire
custom-commands-description = Comandi a cui dai un nome tu. topgrade li esegue come passaggio a sé.
custom-commands-none = Nessun comando personalizzato per ora.
first-run-accept = Continua
first-run-autostart = Avvia con la sessione
first-run-autostart-description = Si avvia ridotto a icona all'accesso, così le verifiche pianificate possono essere eseguite.
first-run-body = Queste opzioni cambiano il comportamento dell'applicazione al di fuori della sua finestra. Puoi modificarle più tardi nelle Impostazioni.
first-run-title = Un paio di scelte
first-run-tray = Mostra un'icona nell'area di stato
first-run-tray-description = Permette di nascondere la finestra e richiamarla, e di avviare un aggiornamento senza aprirla.
history-back = Torna all'elenco
history-delete = Elimina
history-detail = { $outcome } · { $origin } · { $duration }
history-duration-seconds = { $seconds } s
history-heading = Esecuzioni precedenti
history-none = Non è ancora stata registrata alcuna esecuzione.
history-origin-manual = Avviata qui
history-origin-scheduled = Pianificata
history-outcome-cancelled = Annullata
history-outcome-failed = Non riuscita
history-outcome-succeeded = Riuscita
history-transcript-unavailable = Non è stato possibile leggere il registro di questa esecuzione.
view = Visualizza
keep-run-logs = Esecuzioni da conservare
nav-history = Cronologia
notify-failed-steps = Non riusciti: { $steps }
notify-title-failed = Aggiornamento terminato con errori
notify-title-succeeded = Aggiornamento terminato
show-tray-icon = Mostra un'icona nell'area di stato
tray-quit = Esci
tray-show = Mostra la finestra
tray-unavailable = In questa scrivania non è stata trovata un'area di stato, quindi non viene mostrata alcuna icona.

## Releases

nav-releases = Rilasci
releases-add-selected = Osserva la selezione
releases-cancel-find = Annulla
releases-check = Cerca aggiornamenti
releases-checking = { $done } di { $total } verificati…
releases-description = Il software installato dalla pagina dei rilasci di un progetto non ha un gestore di pacchetti alle spalle, quindi topgrade non può aggiornarlo. Questi vengono verificati presso il progetto stesso.
releases-error = Impossibile verificare: { $message }
releases-find = Cerca progetti
releases-finding = Esame dei pacchetti installati…
releases-found = Trovati { $count } progetti su questo sistema. Scegli quali osservare.
releases-heading = Rilasci dei progetti
releases-installed = { $name } aggiornato a { $version }
releases-install-failed = Impossibile aggiornare { $name }: { $message }
releases-installing = Installazione di { $name }…
releases-no-asset = Nessun file di questo rilascio corrisponde a questo sistema; usa la pagina del rilascio.
releases-none = Non è ancora osservato alcun progetto.
releases-no-releases = Nessun rilascio
releases-no-transport = Non sono installati né curl né gh, quindi i rilasci non possono essere verificati.
releases-open = Pagina del rilascio
releases-remove = Smetti di osservare
releases-source = { $source } · { $forge }
releases-unidentified = Progetto sconosciuto — { $version } installato
releases-unknown = { $version } pubblicato
releases-update = Aggiorna
releases-update-available = { $version } disponibile
releases-up-to-date = Aggiornato
releases-watched = { $count } osservati
run-was-preview = Questa era un'anteprima: non è stato modificato nulla nel sistema.

interval-daily = Ogni giorno
interval-manual = Solo su richiesta
interval-six-hourly = Ogni 6 ore
interval-weekly = Ogni settimana
releases-interval = Verifica automatica
releases-last-checked = Ultima verifica { $when }
releases-never-checked = Non ancora verificato
releases-next-check = prossima { $when }

## Dependencies, release channel and directories

channel-pre-release = Includi beta e candidate al rilascio
channel-stable = Solo stabili
dep-authentication-dismissed = L'autenticazione è stata annullata.
dep-curl = Recupera le informazioni sui rilasci dagli host dei progetti e scarica gli aggiornamenti.
dependencies-all-present = Tutto ciò che serve a questa applicazione è installato.
dependencies-description = Questa applicazione funziona pilotando altri programmi. Uno mancante diventa una funzione che non fa nulla in silenzio, perciò sono elencati qui con il loro scopo.
dependencies-heading = Strumenti necessari
dependencies-install = Installa
dependencies-installed = Installato
dependencies-install-failed = Impossibile installare { $name }: { $message }
dependencies-installing = Installazione…
dependencies-missing = Non installato
dependencies-no-manager = Non è stato trovato alcun gestore di pacchetti supportato, quindi non si può installare da qui.
dependencies-optional = Facoltativo
dependencies-recheck = Verifica di nuovo
dependencies-required = Necessario
dep-gh = Fornisce le tue credenziali GitHub, portando il limite delle verifiche da 60 a 5000 richieste all'ora.
dep-notify-send = Riferisce l'esito di un'esecuzione pianificata che nessuno stava guardando.
dep-pkexec = Chiede i permessi di amministratore tramite la finestra della scrivania, per aggiornamenti di sistema e installazioni.
dep-systemctl = Mantiene la pianificazione come timer utente di systemd, così viene eseguita anche a finestra chiusa.
dep-topgrade = Esegue gli aggiornamenti veri e propri. Senza di esso questa applicazione non ha nulla da pilotare.
dep-xdg-open = Apre le pagine dei rilasci e i collegamenti nel browser.
nav-dependencies = Dipendenze
releases-channel = Rilasci da proporre
releases-directories = Cartelle delle applicazioni scaricate
releases-directories-description = Vi si cercano AppImage e altri programmi scaricati. I percorsi relativi partono dalla tua cartella personale.
releases-directory-add = Aggiungi cartella
releases-directory-placeholder = Applications
releases-self = Questa applicazione
dependencies-missing-required = { $count ->
    [one] Manca 1 strumento necessario.
   *[other] Mancano { $count } strumenti necessari.
}
releases-channel-description = Se le candidate al rilascio e le beta contano come aggiornamenti.

## Welcome, notifications and virus scanning

clamav-clean = Scansione terminata: { $scanned } file controllati, nulla trovato.
clamav-failed = Non è stato possibile eseguire la scansione: { $message }
clamav-infected = Scansione terminata: { $infected } file infetti.
clamav-options = Opzioni di scansione
clamav-scan = Analizza dopo l'aggiornamento del database virus
clamav-scan-description = ClamAV è installato. topgrade ne tiene aggiornato il database; questo esegue una scansione con il nuovo database appena cambia.
clamav-scanning = Il database virus è cambiato — scansione in corso…
clamav-target = Cosa analizzare
nav-welcome = Benvenuto
notify-errors = Avvisami quando un aggiornamento fallisce
notify-errors-description = I fallimenti vengono segnalati anche con le altre notifiche disattivate, a meno che non si disattivi anche questa.
notify-title-available = Sono disponibili aggiornamenti
notify-title-installed = Aggiornamenti installati
notify-upgrades = Informami sugli aggiornamenti
notify-upgrades-available = Ti verrà detto cosa è disponibile da installare.
notify-upgrades-installed = Ti verrà detto cosa è stato installato.
welcome-automatic-heading = Installazione degli aggiornamenti
welcome-body = Alcune scelte che vale la pena fare ora. Sono tutte nelle Impostazioni in seguito, e nulla è definitivo.
welcome-clamav = Scansione antivirus
welcome-finish = Fatto
welcome-heading = Configura gli aggiornamenti
welcome-notifications = Notifiche
welcome-root-warning = L'installazione non presidiata richiede i permessi di amministratore, quindi l'esecuzione pianificata viene installata come servizio di sistema eseguito come root. Nient'altro in questa applicazione viene eseguito come root.

## Package sources

nav-sources = Sorgenti dei pacchetti
sources-add-apt = Aggiungi sorgente APT
sources-add-flatpak = Aggiungi remote Flatpak
sources-add-heading = Aggiungi una sorgente
sources-apt-hint = Una sorgente APT viene scritta in /etc/apt/sources.list.d e richiede i permessi di amministratore.
sources-changing = Applicazione…
sources-description = I repository da cui attingono i gestori di pacchetti. topgrade aggiorna ciò che è installato; questi decidono cosa è disponibile.
sources-disable-note = Le sorgenti APT e dnf vengono disattivate anziché eliminate, così una modifica si può annullare a mano.
sources-disabled = Disattivata
sources-enabled = Attivata
sources-flatpak-hint = Un remote Flatpak viene aggiunto solo per te e non richiede password. Indica un URL .flatpakrepo.
sources-heading = Da dove arrivano i pacchetti
sources-name-placeholder = Nome
sources-none = Nessuna sorgente di pacchetti trovata.
sources-privileged = Modificarla richiede i permessi di amministratore.
sources-reload = Ricarica
sources-remove = Rimuovi
sources-suite-placeholder = Suite (es. stable)
sources-url-placeholder = URL
show-tray-icon-description = Porta in primo piano la finestra, avvia un aggiornamento senza aprirla ed esce. Non può nascondere la finestra: Wayland non offre modo di annullarlo.
welcome-show-again = Schermata di configurazione
welcome-show-again-description = Torna alla prima schermata e alle sue scelte.
