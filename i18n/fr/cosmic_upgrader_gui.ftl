app-title = Mise à niveau
app-description = Configurez, planifiez et lancez les mises à niveau du système avec topgrade.

## Navigation

nav-overview = Vue d'ensemble
nav-schedule = Planification
nav-configuration = Configuration
nav-run = Exécuter

category-system = Système
category-applications = Applications
category-containers = Conteneurs
category-development = Développement
category-editors = Éditeurs
category-repositories = Dépôts
category-shell = Interpréteur de commandes
category-ai-tools = Outils d'IA
category-cloud = Nuage
category-desktop = Bureau
category-custom = Commandes personnalisées
category-other = Autres

## Overview

overview-heading = Sources de mise à niveau
overview-subtitle = { $available } étapes sur { $total } s'appliquent à ce système.
topgrade-version = topgrade { $version }
topgrade-source-system = Installé sur ce système
topgrade-source-bundled = Fourni avec cette application
scanning = Vérification de ce qui s'applique à ce système…
scanning-progress = { $completed } sur { $total } vérifiées — { $step }
rescan = Revérifier
rescan-tooltip = Rechercher à nouveau les outils récemment installés

## Steps

steps-heading = Étapes
steps-none = Aucune étape dans cette catégorie.
step-available = Prêt
step-unavailable = Indisponible
step-inactive = Non applicable
step-deprecated = Obsolète
step-enabled-tooltip = Inclure cette étape lors de la mise à niveau
step-disabled-tooltip = Ignorer cette étape lors de la mise à niveau
step-components = { $count ->
    [one] 1 composant
   *[other] { $count } composants
}
enable-all = Tout activer
disable-all = Tout désactiver
show-unavailable = Afficher les étapes indisponibles
show-unavailable-tooltip = Lister aussi les étapes dont les outils ne sont pas installés

status-ok = Prêt
status-skipped = Ignorée
status-failed = Échec

## Running

run-heading = Exécuter
run-now = Lancer la mise à niveau
dry-run = Aperçu
dry-run-tooltip = Montrer ce qui serait fait sans rien modifier
run-in-progress = Mise à niveau en cours…
run-step = { $step }
run-finished = Terminé
run-cancelled = Annulé
run-failed = Terminé avec des erreurs
run-never = Aucune mise à niveau n'a encore été lancée.
run-last = Dernière exécution { $when }
run-summary = { $ok } réussies, { $skipped } ignorées, { $failed } en échec
cancel-run = Arrêter
clear-log = Effacer
copy-log = Copier la sortie
run-selected-only = Répondre oui aux invites des gestionnaires de paquets

## Authentication

password-title = Mot de passe administrateur requis
password-body = { $command } a besoin des droits d'administrateur pour continuer.
password-placeholder = Mot de passe
authenticate = S'authentifier
authentication-failed = Ce mot de passe n'a pas été accepté.

## Schedule

schedule-heading = Mises à niveau planifiées
schedule-enabled = Rechercher les mises à niveau selon une planification
schedule-frequency = Fréquence
frequency-hourly = Toutes les heures
frequency-daily = Tous les jours
frequency-weekly = Toutes les semaines
frequency-monthly = Tous les mois
schedule-time = Heure de la journée
schedule-automatic = Installer les mises à niveau automatiquement
schedule-automatic-description = Si désactivé, une notification signale ce qui est disponible et rien n'est modifié.
schedule-next-run = Prochaine exécution { $when }
schedule-next-run-unknown = L'heure de la prochaine exécution est inconnue.
schedule-backend-systemd = S'exécute en arrière-plan via une minuterie utilisateur systemd, même lorsque cette fenêtre est fermée.
schedule-backend-fallback = systemd n'est pas disponible ; les exécutions planifiées n'ont donc lieu que lorsque cette fenêtre est ouverte.
schedule-apply = Appliquer la planification
schedule-applied = Planification mise à jour.
schedule-error = La planification n'a pas pu être appliquée : { $message }

## Configuration

configuration-heading = Configuration de topgrade
configuration-path = Modification de { $path }
configuration-default = Par défaut : { $value }
configuration-not-set = Non défini
configuration-save = Enregistrer les modifications
configuration-revert = Rétablir
configuration-reset = Réinitialiser à la valeur par défaut
configuration-unsaved = Des modifications ne sont pas enregistrées.
configuration-saved = Configuration enregistrée.
configuration-free-form = Ce sont des commandes que vous nommez vous-même. Modifiez cette section directement dans le fichier.
configuration-open-file = Ouvrir le fichier de configuration
configuration-add = Ajouter
configuration-remove = Retirer

## Application settings

settings = Paramètres
about = À propos
appearance = Apparence
theme = Thème
theme-system = Comme le bureau
theme-light = Clair
theme-dark = Sombre
behaviour = Comportement
privilege-backend = Droits d'administrateur
privilege-pty = Demander dans cette fenêtre
privilege-pty-description = Lance topgrade dans un terminal et demande ici lorsqu'un mot de passe est nécessaire.
privilege-pkexec = Boîte de dialogue du système
privilege-pkexec-description = Utilise la boîte de dialogue d'authentification du bureau. Demande une fois par commande.
confirm-before-running = Confirmer avant de lancer une mise à niveau
notify-on-completion = Notifier à la fin d'une exécution planifiée

## Errors and empty states

topgrade-missing-title = topgrade n'est pas installé
topgrade-missing-body = Cette application pilote topgrade, qui est introuvable sur ce système.
topgrade-missing-hint = Installez-le avec votre gestionnaire de paquets, ou avec : { $command }
topgrade-too-old-title = topgrade est trop ancien
topgrade-too-old-body = topgrade { $found } a été trouvé, mais { $required } ou plus récent est nécessaire.
error-title = Une erreur est survenue
retry = Réessayer

## Common

cancel = Annuler
close = Fermer
save = Enregistrer
ok = OK
toggle-sidebar = Afficher/masquer la barre latérale
git-description = Description Git
repository = Dépôt
support = Assistance

## History, first run, custom commands and status area

autostart = Démarrer avec la session
autostart-description = Ajoute une entrée dans ~/.config/autostart
category-settings = Paramètres de cette catégorie
category-settings-none = Cette catégorie n'a pas de paramètres topgrade propres.
command-name-placeholder = Nom
command-value-placeholder = Commande à exécuter
custom-commands-description = Des commandes que vous nommez vous-même. topgrade les exécute comme une étape à part.
custom-commands-none = Aucune commande personnalisée pour l'instant.
first-run-accept = Continuer
first-run-autostart = Démarrer avec la session
first-run-autostart-description = Se lance réduit à l'ouverture de session pour que les vérifications planifiées puissent avoir lieu.
first-run-body = Ces options changent le comportement de l'application en dehors de sa propre fenêtre. Vous pourrez les modifier plus tard dans les Paramètres.
first-run-title = Deux ou trois choix
first-run-tray = Afficher une icône dans la zone d'état
first-run-tray-description = Permet de masquer la fenêtre et de la rappeler, et de lancer une mise à niveau sans l'ouvrir.
history-back = Retour à la liste
history-delete = Supprimer
history-detail = { $outcome } · { $origin } · { $duration }
history-duration-seconds = { $seconds } s
history-heading = Exécutions passées
history-none = Aucune exécution n'a encore été enregistrée.
history-origin-manual = Lancée ici
history-origin-scheduled = Planifiée
history-outcome-cancelled = Annulée
history-outcome-failed = Échec
history-outcome-succeeded = Réussie
history-transcript-unavailable = Le journal de cette exécution n'a pas pu être lu.
view = Afficher
keep-run-logs = Exécutions à conserver
nav-history = Historique
notify-failed-steps = En échec : { $steps }
notify-title-failed = Mise à niveau terminée avec des erreurs
notify-title-succeeded = Mise à niveau terminée
show-tray-icon = Afficher une icône dans la zone d'état
tray-quit = Quitter
tray-show = Afficher la fenêtre
tray-unavailable = Aucune zone d'état n'a été trouvée sur ce bureau, aucune icône n'est donc affichée.

## Releases

nav-releases = Publications
releases-add-selected = Suivre la sélection
releases-cancel-find = Annuler
releases-check = Rechercher des mises à jour
releases-checking = { $done } sur { $total } vérifiés…
releases-description = Les logiciels installés depuis la page des publications d'un projet n'ont pas de gestionnaire de paquets derrière eux, donc topgrade ne peut pas les mettre à jour. Ceux-ci sont vérifiés auprès du projet lui-même.
releases-error = Vérification impossible : { $message }
releases-find = Rechercher des projets
releases-finding = Examen des paquets installés…
releases-found = { $count } projets trouvés sur ce système. Choisissez ceux à suivre.
releases-heading = Publications des projets
releases-installed = { $name } mis à jour vers { $version }
releases-install-failed = { $name } n'a pas pu être mis à jour : { $message }
releases-installing = Installation de { $name }…
releases-no-asset = Aucun fichier de cette publication ne correspond à ce système ; utilisez la page de la publication.
releases-none = Aucun projet n'est encore suivi.
releases-no-releases = Aucune publication
releases-no-transport = Ni curl ni gh n'est installé, les publications ne peuvent donc pas être vérifiées.
releases-open = Page de la publication
releases-remove = Ne plus suivre
releases-source = { $source } · { $forge }
releases-unidentified = Projet inconnu — { $version } installé
releases-unknown = { $version } publiée
releases-update = Mettre à jour
releases-update-available = { $version } disponible
releases-up-to-date = À jour
releases-watched = { $count } suivis
run-was-preview = Ceci était un aperçu — rien n'a été modifié sur le système.

interval-daily = Tous les jours
interval-manual = Uniquement à la demande
interval-six-hourly = Toutes les 6 heures
interval-weekly = Toutes les semaines
releases-interval = Vérifier automatiquement
releases-last-checked = Dernière vérification { $when }
releases-never-checked = Pas encore vérifié
releases-next-check = prochaine { $when }

## Dependencies, release channel and directories

channel-pre-release = Inclure les bêtas et les préversions
channel-stable = Stables uniquement
dep-authentication-dismissed = L'authentification a été annulée.
dep-curl = Récupère les informations de publication auprès des hôtes de projets et télécharge les mises à jour.
dependencies-all-present = Tout ce dont cette application a besoin est installé.
dependencies-description = Cette application fonctionne en pilotant d'autres programmes. Un programme manquant devient une fonction qui ne fait rien en silence, ils sont donc listés ici avec leur rôle.
dependencies-heading = Outils nécessaires
dependencies-install = Installer
dependencies-installed = Installé
dependencies-install-failed = { $name } n'a pas pu être installé : { $message }
dependencies-installing = Installation…
dependencies-missing = Non installé
dependencies-no-manager = Aucun gestionnaire de paquets pris en charge n'a été trouvé, l'installation depuis ici est impossible.
dependencies-optional = Facultatif
dependencies-recheck = Vérifier à nouveau
dependencies-required = Requis
dep-gh = Transmet vos identifiants GitHub, faisant passer la limite de vérification de 60 à 5000 requêtes par heure.
dep-notify-send = Signale le résultat d'une exécution planifiée que personne ne regardait.
dep-pkexec = Demande les droits d'administrateur via la boîte de dialogue du bureau, pour les mises à niveau système et l'installation de paquets.
dep-systemctl = Tient la planification sous forme de minuterie utilisateur systemd, pour qu'elle s'exécute fenêtre fermée.
dep-topgrade = Effectue les mises à niveau elles-mêmes. Sans lui, cette application n'a rien à piloter.
dep-xdg-open = Ouvre les pages de publication et les liens dans votre navigateur.
nav-dependencies = Dépendances
releases-channel = Publications à proposer
releases-directories = Dossiers des applications téléchargées
releases-directories-description = Ces dossiers sont parcourus à la recherche d'AppImages et d'autres programmes téléchargés. Les chemins relatifs partent de votre dossier personnel.
releases-directory-add = Ajouter un dossier
releases-directory-placeholder = Applications
releases-self = Cette application
dependencies-missing-required = { $count ->
    [one] 1 outil requis est manquant.
   *[other] { $count } outils requis sont manquants.
}
releases-channel-description = Si les préversions et les bêtas comptent comme des mises à jour.

## Welcome, notifications and virus scanning

clamav-clean = Analyse terminée : { $scanned } fichiers vérifiés, rien trouvé.
clamav-failed = L'analyse n'a pas pu être lancée : { $message }
clamav-infected = Analyse terminée : { $infected } fichier(s) infecté(s).
clamav-options = Options d'analyse
clamav-scan = Analyser après la mise à jour de la base antivirale
clamav-scan-description = ClamAV est installé. topgrade tient sa base à jour ; ceci analyse avec la nouvelle base dès qu'elle change.
clamav-scanning = La base antivirale a changé — analyse en cours…
clamav-target = Ce qui est analysé
nav-welcome = Bienvenue
notify-errors = M'avertir en cas d'échec d'une mise à niveau
notify-errors-description = Les échecs sont signalés même si les autres notifications sont désactivées, sauf si ceci l'est aussi.
notify-title-available = Des mises à niveau sont disponibles
notify-title-installed = Mises à niveau installées
notify-upgrades = M'informer des mises à niveau
notify-upgrades-available = On vous dira ce qui est disponible à installer.
notify-upgrades-installed = On vous dira ce qui a été installé.
welcome-automatic-heading = Installation des mises à niveau
welcome-body = Quelques choix à faire maintenant. Ils sont tous dans les Paramètres ensuite, et rien n'est définitif.
welcome-clamav = Analyse antivirus
welcome-finish = Terminé
welcome-heading = Configurer les mises à niveau
welcome-notifications = Notifications
welcome-root-warning = Une installation sans surveillance nécessite les droits d'administrateur, l'exécution planifiée est donc installée comme service système s'exécutant en root. Rien d'autre dans cette application ne s'exécute en root.

## Package sources

nav-sources = Sources de paquets
sources-add-apt = Ajouter une source APT
sources-add-flatpak = Ajouter un dépôt Flatpak
sources-add-heading = Ajouter une source
sources-apt-hint = Une source APT est écrite dans /etc/apt/sources.list.d et nécessite les droits d'administrateur.
sources-changing = Application…
sources-description = Les dépôts d'où vos gestionnaires de paquets tirent. topgrade met à jour ce qui est installé ; ceux-ci décident de ce qui est disponible.
sources-disable-note = Les sources APT et dnf sont désactivées plutôt que supprimées, pour qu'un changement puisse être défait à la main.
sources-disabled = Désactivée
sources-enabled = Activée
sources-flatpak-hint = Un dépôt Flatpak est ajouté pour vous seul et ne nécessite pas de mot de passe. Indiquez une URL .flatpakrepo.
sources-heading = D'où viennent les paquets
sources-name-placeholder = Nom
sources-none = Aucune source de paquets trouvée.
sources-privileged = Modifier ceci nécessite les droits d'administrateur.
sources-reload = Recharger
sources-remove = Retirer
sources-suite-placeholder = Suite (par ex. stable)
sources-url-placeholder = URL
show-tray-icon-description = Met la fenêtre au premier plan, lance une mise à niveau sans l'ouvrir, et quitte. Il ne peut pas masquer la fenêtre : Wayland n'offre aucun moyen de l'annuler.
welcome-show-again = Écran de configuration
welcome-show-again-description = Revenir au premier écran et à ses choix.

releases-restart-failed = La nouvelle version n'a pas pu être lancée : { $message }
releases-restart-unknown-path = l'emplacement de cette application est inconnu
releases-restarting = Mis à jour — redémarrage dans la nouvelle version…

releases-found-dismiss = Pas ceux-ci
releases-found-new-why = Installés depuis un fichier, aucun gestionnaire de paquets n'en a donc de plus récent. Suivre leurs publications ?
releases-found-watch = Suivre ceux-ci
upgrade-releases-with-run = Mettre aussi à jour les programmes installés depuis des pages de publication
upgrade-releases-with-run-description = topgrade ne les met à jour que s'ils ont été installés via un outil qui les suit. Un paquet installé à la main n'est couvert par aucun, donc une mise à niveau les applique ici.
releases-found-new = { $count ->
    [one] 1 programme installé que rien d'autre ne mettra à jour
   *[other] { $count } programmes installés que rien d'autre ne mettra à jour
}
