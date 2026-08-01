app-title = Aktualizator
app-description = Konfiguruj, planuj i uruchamiaj aktualizacje całego systemu za pomocą topgrade.

## Navigation

nav-overview = Przegląd
nav-schedule = Harmonogram
nav-configuration = Konfiguracja
nav-run = Uruchom

category-system = System
category-applications = Aplikacje
category-containers = Kontenery
category-development = Programowanie
category-editors = Edytory
category-repositories = Repozytoria
category-shell = Powłoka
category-ai-tools = Narzędzia SI
category-cloud = Chmura
category-desktop = Pulpit
category-custom = Własne polecenia
category-other = Inne

## Overview

overview-heading = Źródła aktualizacji
overview-subtitle = { $available } z { $total } kroków dotyczy tego systemu.
topgrade-version = topgrade { $version }
topgrade-source-system = Zainstalowany w tym systemie
topgrade-source-bundled = Dołączony do tej aplikacji
scanning = Sprawdzanie, co dotyczy tego systemu…
scanning-progress = Sprawdzono { $completed } z { $total } — { $step }
rescan = Sprawdź ponownie
rescan-tooltip = Poszukaj ponownie nowo zainstalowanych narzędzi

## Steps

steps-heading = Kroki
steps-none = Brak kroków w tej kategorii.
step-available = Gotowy
step-unavailable = Niedostępny
step-inactive = Nie dotyczy
step-deprecated = Przestarzały
step-enabled-tooltip = Uwzględnij ten krok podczas aktualizacji
step-disabled-tooltip = Pomiń ten krok podczas aktualizacji
step-components = { $count ->
    [one] 1 składnik
    [few] { $count } składniki
    [many] { $count } składników
   *[other] { $count } składnika
}
enable-all = Włącz wszystkie
disable-all = Wyłącz wszystkie
show-unavailable = Pokaż niedostępne kroki
show-unavailable-tooltip = Wypisz także kroki, których narzędzia nie są zainstalowane

status-ok = Gotowy
status-skipped = Pominięty
status-failed = Niepowodzenie

## Running

run-heading = Uruchom
run-now = Rozpocznij aktualizację
dry-run = Podgląd
dry-run-tooltip = Pokaż, co zostałoby zrobione, bez wprowadzania zmian
run-in-progress = Aktualizowanie…
run-step = { $step }
run-finished = Ukończono
run-cancelled = Anulowano
run-failed = Ukończono z błędami
run-never = Nie wykonano jeszcze żadnej aktualizacji.
run-last = Ostatnie uruchomienie { $when }
run-summary = { $ok } powiodło się, { $skipped } pominięto, { $failed } nie powiodło się
cancel-run = Zatrzymaj
clear-log = Wyczyść
copy-log = Skopiuj wyjście
run-selected-only = Odpowiadaj twierdząco na pytania menedżerów pakietów

## Authentication

password-title = Wymagane hasło administratora
password-body = { $command } wymaga uprawnień administratora, aby kontynuować.
password-placeholder = Hasło
authenticate = Uwierzytelnij
authentication-failed = To hasło nie zostało przyjęte.

## Schedule

schedule-heading = Zaplanowane aktualizacje
schedule-enabled = Sprawdzaj aktualizacje według harmonogramu
schedule-frequency = Częstotliwość
frequency-hourly = Co godzinę
frequency-daily = Codziennie
frequency-weekly = Co tydzień
frequency-monthly = Co miesiąc
schedule-time = Pora dnia
schedule-automatic = Instaluj aktualizacje automatycznie
schedule-automatic-description = Gdy wyłączone, powiadomienie informuje tylko o dostępnych aktualizacjach i nic nie jest zmieniane.
schedule-next-run = Następne uruchomienie { $when }
schedule-next-run-unknown = Czas następnego uruchomienia nie jest znany.
schedule-backend-systemd = Działa w tle przez licznik czasu użytkownika systemd, nawet gdy to okno jest zamknięte.
schedule-backend-fallback = systemd jest niedostępny, więc zaplanowane uruchomienia następują tylko wtedy, gdy to okno jest otwarte.
schedule-apply = Zastosuj harmonogram
schedule-applied = Harmonogram zaktualizowany.
schedule-error = Nie udało się zastosować harmonogramu: { $message }

## Configuration

configuration-heading = Konfiguracja topgrade
configuration-path = Edytowanie { $path }
configuration-default = Domyślnie: { $value }
configuration-not-set = Nieustawione
configuration-save = Zapisz zmiany
configuration-revert = Przywróć
configuration-reset = Przywróć wartość domyślną
configuration-unsaved = Są niezapisane zmiany.
configuration-saved = Konfiguracja zapisana.
configuration-free-form = To polecenia, które nazywasz samodzielnie. Edytuj tę sekcję bezpośrednio w pliku.
configuration-open-file = Otwórz plik konfiguracyjny
configuration-add = Dodaj
configuration-remove = Usuń

## Application settings

settings = Ustawienia
about = O programie
appearance = Wygląd
theme = Motyw
theme-system = Jak pulpit
theme-light = Jasny
theme-dark = Ciemny
behaviour = Zachowanie
privilege-backend = Uprawnienia administratora
privilege-pty = Pytaj w tym oknie
privilege-pty-description = Uruchamia topgrade w terminalu i pyta tutaj, gdy potrzebne jest hasło.
privilege-pkexec = Okno systemowe
privilege-pkexec-description = Używa okna uwierzytelniania pulpitu. Pyta raz na polecenie.
confirm-before-running = Potwierdzaj przed rozpoczęciem aktualizacji
notify-on-completion = Powiadamiaj o zakończeniu zaplanowanego uruchomienia

## Errors and empty states

topgrade-missing-title = topgrade nie jest zainstalowany
topgrade-missing-body = Ta aplikacja steruje programem topgrade, którego nie znaleziono w tym systemie.
topgrade-missing-hint = Zainstaluj go menedżerem pakietów lub poleceniem: { $command }
topgrade-too-old-title = topgrade jest zbyt stary
topgrade-too-old-body = Znaleziono topgrade { $found }, ale wymagany jest { $required } lub nowszy.
error-title = Coś poszło nie tak
retry = Spróbuj ponownie

## Common

cancel = Anuluj
close = Zamknij
save = Zapisz
ok = OK
toggle-sidebar = Przełącz panel boczny
git-description = Opis Git
repository = Repozytorium
support = Wsparcie

## History, first run, custom commands and status area

autostart = Uruchamiaj z sesją
autostart-description = Dodaje wpis w ~/.config/autostart
category-settings = Ustawienia tej kategorii
category-settings-none = Ta kategoria nie ma własnych ustawień topgrade.
command-name-placeholder = Nazwa
command-value-placeholder = Polecenie do wykonania
custom-commands-description = Polecenia, które nazywasz samodzielnie. topgrade uruchamia je jako osobny krok.
custom-commands-none = Nie ma jeszcze własnych poleceń.
first-run-accept = Dalej
first-run-autostart = Uruchamiaj z sesją
first-run-autostart-description = Uruchamia się zminimalizowany przy logowaniu, aby zaplanowane sprawdzenia mogły działać.
first-run-body = Te opcje zmieniają zachowanie aplikacji poza jej własnym oknem. Możesz je zmienić później w Ustawieniach.
first-run-title = Kilka wyborów
first-run-tray = Pokaż ikonę w obszarze stanu
first-run-tray-description = Pozwala ukryć okno i przywrócić je oraz rozpocząć aktualizację bez otwierania go.
hide-to-tray = Ukryj w obszarze stanu
history-back = Powrót do listy
history-delete = Usuń
history-detail = { $outcome } · { $origin } · { $duration }
history-duration-seconds = { $seconds } s
history-heading = Poprzednie uruchomienia
history-none = Nie zapisano jeszcze żadnego uruchomienia.
history-origin-manual = Uruchomione tutaj
history-origin-scheduled = Zaplanowane
history-outcome-cancelled = Anulowane
history-outcome-failed = Niepowodzenie
history-outcome-succeeded = Powodzenie
history-transcript-unavailable = Nie udało się odczytać zapisu tego uruchomienia.
view = Pokaż
keep-run-logs = Uruchomienia do zachowania
minimize-to-tray = Ukryj w obszarze stanu zamiast kończyć
minimize-to-tray-description = Dodaje przycisk Ukryj. Przycisk zamknięcia okna nadal kończy działanie.
nav-history = Historia
notify-failed-steps = Nie powiodły się: { $steps }
notify-title-failed = Aktualizacja zakończona z błędami
notify-title-succeeded = Aktualizacja zakończona
show-tray-icon = Pokaż ikonę w obszarze stanu
tray-hide = Ukryj okno
tray-quit = Zakończ
tray-show = Pokaż okno
tray-unavailable = Na tym pulpicie nie znaleziono obszaru stanu, więc ikona nie jest pokazywana.

## Releases

nav-releases = Wydania
releases-add-selected = Obserwuj zaznaczone
releases-cancel-find = Anuluj
releases-check = Sprawdź aktualizacje
releases-checking = Sprawdzono { $done } z { $total }…
releases-description = Oprogramowanie zainstalowane ze strony wydań projektu nie ma za sobą menedżera pakietów, więc topgrade nie może go zaktualizować. Te są sprawdzane bezpośrednio u projektu.
releases-error = Nie udało się sprawdzić: { $message }
releases-find = Znajdź projekty
releases-finding = Przeglądanie zainstalowanych pakietów…
releases-found = Znaleziono { $count } projektów w tym systemie. Wybierz, które obserwować.
releases-heading = Wydania projektów
releases-installed = Zaktualizowano { $name } do { $version }
releases-install-failed = Nie udało się zaktualizować { $name }: { $message }
releases-installing = Instalowanie { $name }…
releases-no-asset = Żaden plik tego wydania nie pasuje do tego systemu; użyj strony wydania.
releases-none = Nie obserwujesz jeszcze żadnych projektów.
releases-no-releases = Brak wydań
releases-no-transport = Nie zainstalowano ani curl, ani gh, więc nie można sprawdzić wydań.
releases-open = Strona wydania
releases-remove = Przestań obserwować
releases-source = { $source } · { $forge }
releases-unidentified = Nieznany projekt — zainstalowano { $version }
releases-unknown = Opublikowano { $version }
releases-update = Aktualizuj
releases-update-available = Dostępne { $version }
releases-up-to-date = Aktualne
releases-watched = Obserwowanych: { $count }
run-was-preview = To był podgląd — nic w systemie nie zostało zmienione.

interval-daily = Codziennie
interval-manual = Tylko na żądanie
interval-six-hourly = Co 6 godzin
interval-weekly = Co tydzień
releases-interval = Sprawdzaj automatycznie
releases-last-checked = Ostatnio sprawdzono { $when }
releases-never-checked = Jeszcze nie sprawdzono
releases-next-check = następne { $when }
