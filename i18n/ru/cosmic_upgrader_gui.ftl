app-title = Обновление
app-description = Настройка, планирование и запуск общесистемных обновлений с помощью topgrade.

## Navigation

nav-overview = Обзор
nav-schedule = Расписание
nav-configuration = Конфигурация
nav-run = Запуск

category-system = Система
category-applications = Приложения
category-containers = Контейнеры
category-development = Разработка
category-editors = Редакторы
category-repositories = Репозитории
category-shell = Оболочка
category-ai-tools = Инструменты ИИ
category-cloud = Облако
category-desktop = Рабочий стол
category-custom = Свои команды
category-other = Прочее

## Overview

overview-heading = Источники обновлений
overview-subtitle = К этой системе применимо { $available } из { $total } шагов.
topgrade-version = topgrade { $version }
topgrade-source-system = Установлен в этой системе
topgrade-source-bundled = Поставляется с этим приложением
scanning = Проверка того, что применимо к этой системе…
scanning-progress = Проверено { $completed } из { $total } — { $step }
rescan = Проверить снова
rescan-tooltip = Повторно найти недавно установленные инструменты

## Steps

steps-heading = Шаги
steps-none = В этой категории нет шагов.
step-available = Готов
step-unavailable = Недоступен
step-inactive = Неприменим
step-deprecated = Устарел
step-enabled-tooltip = Включать этот шаг при обновлении
step-disabled-tooltip = Пропускать этот шаг при обновлении
step-components = { $count ->
    [one] 1 компонент
    [few] { $count } компонента
    [many] { $count } компонентов
   *[other] { $count } компонента
}
enable-all = Включить все
disable-all = Отключить все
show-unavailable = Показывать недоступные шаги
show-unavailable-tooltip = Показывать также шаги, инструменты которых не установлены

status-ok = Готов
status-skipped = Пропущен
status-failed = Ошибка

## Running

run-heading = Запуск
run-now = Начать обновление
dry-run = Предпросмотр
dry-run-tooltip = Показать, что было бы сделано, ничего не изменяя
run-in-progress = Обновление…
run-step = { $step }
run-finished = Завершено
run-cancelled = Отменено
run-failed = Завершено с ошибками
run-never = Обновление ещё не выполнялось.
run-last = Последний запуск { $when }
run-summary = успешно: { $ok }, пропущено: { $skipped }, с ошибками: { $failed }
cancel-run = Остановить
clear-log = Очистить
copy-log = Скопировать вывод
run-selected-only = Отвечать утвердительно на запросы менеджеров пакетов

## Authentication

password-title = Требуется пароль администратора
password-body = Для продолжения { $command } нужны права администратора.
password-placeholder = Пароль
authenticate = Войти
authentication-failed = Этот пароль не принят.

## Schedule

schedule-heading = Запланированные обновления
schedule-enabled = Проверять обновления по расписанию
schedule-frequency = Периодичность
frequency-hourly = Ежечасно
frequency-daily = Ежедневно
frequency-weekly = Еженедельно
frequency-monthly = Ежемесячно
schedule-time = Время суток
schedule-automatic = Устанавливать обновления автоматически
schedule-automatic-description = Если выключено, уведомление лишь сообщает о доступных обновлениях и ничего не изменяется.
schedule-next-run = Следующий запуск { $when }
schedule-next-run-unknown = Время следующего запуска неизвестно.
schedule-backend-systemd = Выполняется в фоне через пользовательский таймер systemd, даже когда это окно закрыто.
schedule-backend-fallback = systemd недоступен, поэтому запуски по расписанию происходят только пока это окно открыто.
schedule-apply = Применить расписание
schedule-applied = Расписание обновлено.
schedule-error = Не удалось применить расписание: { $message }

## Configuration

configuration-heading = Конфигурация topgrade
configuration-path = Редактируется { $path }
configuration-default = По умолчанию: { $value }
configuration-not-set = Не задано
configuration-save = Сохранить изменения
configuration-revert = Отменить
configuration-reset = Сбросить к значению по умолчанию
configuration-unsaved = Есть несохранённые изменения.
configuration-saved = Конфигурация сохранена.
configuration-free-form = Это команды, которым вы сами даёте имена. Правьте этот раздел прямо в файле.
configuration-open-file = Открыть файл конфигурации
configuration-add = Добавить
configuration-remove = Удалить

## Application settings

settings = Параметры
about = О программе
appearance = Внешний вид
theme = Тема
theme-system = Как на рабочем столе
theme-light = Светлая
theme-dark = Тёмная
behaviour = Поведение
privilege-backend = Права администратора
privilege-pty = Спрашивать в этом окне
privilege-pty-description = Запускает topgrade в терминале и спрашивает здесь, когда нужен пароль.
privilege-pkexec = Системное окно
privilege-pkexec-description = Использует окно проверки подлинности рабочего стола. Спрашивает один раз на команду.
confirm-before-running = Подтверждать перед началом обновления
notify-on-completion = Уведомлять о завершении запуска по расписанию

## Errors and empty states

topgrade-missing-title = topgrade не установлен
topgrade-missing-body = Это приложение управляет topgrade, который не найден в этой системе.
topgrade-missing-hint = Установите его через менеджер пакетов или командой: { $command }
topgrade-too-old-title = topgrade слишком старый
topgrade-too-old-body = Найден topgrade { $found }, но нужен { $required } или новее.
error-title = Что-то пошло не так
retry = Повторить

## Common

cancel = Отмена
close = Закрыть
save = Сохранить
ok = ОК
toggle-sidebar = Переключить боковую панель
git-description = Описание Git
repository = Репозиторий
support = Поддержка

## History, first run, custom commands and status area

autostart = Запускать вместе с сеансом
autostart-description = Добавляет запись в ~/.config/autostart
category-settings = Параметры этой категории
category-settings-none = У этой категории нет собственных параметров topgrade.
command-name-placeholder = Название
command-value-placeholder = Команда для запуска
custom-commands-description = Команды, которым вы сами даёте имена. topgrade выполняет их как отдельный шаг.
custom-commands-none = Своих команд пока нет.
first-run-accept = Далее
first-run-autostart = Запускать вместе с сеансом
first-run-autostart-description = Запускается свёрнутым при входе, чтобы работали проверки по расписанию.
first-run-body = Эти параметры меняют поведение приложения за пределами его собственного окна. Их можно изменить позже в параметрах.
first-run-title = Несколько решений
first-run-tray = Показывать значок в области состояния
first-run-tray-description = Позволяет скрыть окно и вернуть его, а также начать обновление, не открывая его.
history-back = Назад к списку
history-delete = Удалить
history-detail = { $outcome } · { $origin } · { $duration }
history-duration-seconds = { $seconds } с
history-heading = Прошлые запуски
history-none = Пока не записано ни одного запуска.
history-origin-manual = Запущено здесь
history-origin-scheduled = По расписанию
history-outcome-cancelled = Отменено
history-outcome-failed = Ошибка
history-outcome-succeeded = Успешно
history-transcript-unavailable = Не удалось прочитать журнал этого запуска.
view = Смотреть
keep-run-logs = Сколько запусков хранить
nav-history = Журнал
notify-failed-steps = С ошибками: { $steps }
notify-title-failed = Обновление завершено с ошибками
notify-title-succeeded = Обновление завершено
show-tray-icon = Показывать значок в области состояния
tray-quit = Выйти
tray-show = Показать окно
tray-unavailable = На этом рабочем столе область состояния не найдена, поэтому значок не показывается.

## Releases

nav-releases = Выпуски
releases-add-selected = Отслеживать выбранные
releases-cancel-find = Отмена
releases-check = Проверить обновления
releases-checking = Проверено { $done } из { $total }…
releases-description = За программами, установленными со страницы выпусков проекта, не стоит менеджер пакетов, поэтому topgrade не может их обновить. Они проверяются у самого проекта.
releases-error = Не удалось проверить: { $message }
releases-find = Найти проекты
releases-finding = Просмотр установленных пакетов…
releases-found = В этой системе найдено проектов: { $count }. Выберите, какие отслеживать.
releases-heading = Выпуски проектов
releases-installed = { $name } обновлён до { $version }
releases-install-failed = Не удалось обновить { $name }: { $message }
releases-installing = Установка { $name }…
releases-no-asset = Ни один файл этого выпуска не подходит для этой системы; воспользуйтесь страницей выпуска.
releases-none = Пока не отслеживается ни один проект.
releases-no-releases = Выпусков нет
releases-no-transport = Не установлены ни curl, ни gh, поэтому выпуски проверить нельзя.
releases-open = Страница выпуска
releases-remove = Не отслеживать
releases-source = { $source } · { $forge }
releases-unidentified = Проект неизвестен — установлена { $version }
releases-unknown = Опубликована { $version }
releases-update = Обновить
releases-update-available = Доступна { $version }
releases-up-to-date = Актуально
releases-watched = Отслеживается: { $count }
run-was-preview = Это был предпросмотр — в системе ничего не изменено.

interval-daily = Ежедневно
interval-manual = Только по запросу
interval-six-hourly = Каждые 6 часов
interval-weekly = Еженедельно
releases-interval = Проверять автоматически
releases-last-checked = Последняя проверка { $when }
releases-never-checked = Ещё не проверялось
releases-next-check = следующая { $when }

## Dependencies, release channel and directories

channel-pre-release = Включать бета-версии и предварительные выпуски
channel-stable = Только стабильные
dep-authentication-dismissed = Аутентификация отменена.
dep-curl = Получает сведения о выпусках с серверов проектов и загружает обновления.
dependencies-all-present = Всё необходимое для этого приложения установлено.
dependencies-description = Это приложение работает, управляя другими программами. Отсутствующая программа превращается в функцию, которая молча ничего не делает, поэтому они перечислены здесь с указанием назначения.
dependencies-heading = Необходимые инструменты
dependencies-install = Установить
dependencies-installed = Установлено
dependencies-install-failed = Не удалось установить { $name }: { $message }
dependencies-installing = Установка…
dependencies-missing = Не установлено
dependencies-no-manager = Поддерживаемый менеджер пакетов не найден, поэтому отсюда ничего установить нельзя.
dependencies-optional = Необязательно
dependencies-recheck = Проверить снова
dependencies-required = Обязательно
dep-gh = Передаёт ваши учётные данные GitHub, повышая предел проверок с 60 до 5000 запросов в час.
dep-notify-send = Сообщает результат запуска по расписанию, за которым никто не следил.
dep-pkexec = Запрашивает права администратора через окно рабочего стола — для обновлений системы и установки пакетов.
dep-systemctl = Хранит расписание как пользовательский таймер systemd, чтобы оно работало при закрытом окне.
dep-topgrade = Выполняет сами обновления. Без него этому приложению нечем управлять.
dep-xdg-open = Открывает страницы выпусков и ссылки в браузере.
nav-dependencies = Зависимости
releases-channel = Какие выпуски предлагать
releases-directories = Каталоги загруженных приложений
releases-directories-description = В них ищутся AppImage и другие загруженные программы. Относительные пути отсчитываются от домашнего каталога.
releases-directory-add = Добавить каталог
releases-directory-placeholder = Applications
releases-self = Это приложение
dependencies-missing-required = { $count ->
    [one] Отсутствует 1 необходимый инструмент.
    [few] Отсутствует { $count } необходимых инструмента.
    [many] Отсутствует { $count } необходимых инструментов.
   *[other] Отсутствует { $count } необходимого инструмента.
}
releases-channel-description = Считать ли предварительные выпуски и бета-версии обновлениями.

## Welcome, notifications and virus scanning

clamav-clean = Проверка завершена: проверено файлов — { $scanned }, ничего не найдено.
clamav-failed = Не удалось выполнить проверку: { $message }
clamav-infected = Проверка завершена: заражённых файлов — { $infected }.
clamav-options = Параметры проверки
clamav-scan = Проверять после обновления базы вирусов
clamav-scan-description = ClamAV установлен. topgrade поддерживает его базу в актуальном состоянии; это выполняет проверку новой базой, как только она изменится.
clamav-scanning = База вирусов изменилась — идёт проверка…
clamav-target = Что проверять
nav-welcome = Добро пожаловать
notify-errors = Сообщать о неудачных обновлениях
notify-errors-description = Об ошибках сообщается даже при выключенных прочих уведомлениях, если только не выключить и это.
notify-title-available = Доступны обновления
notify-title-installed = Обновления установлены
notify-upgrades = Сообщать об обновлениях
notify-upgrades-available = Вам сообщат, что доступно для установки.
notify-upgrades-installed = Вам сообщат, что было установлено.
welcome-automatic-heading = Установка обновлений
welcome-body = Несколько решений, которые стоит принять сейчас. Все они потом есть в параметрах, и ничего необратимого здесь нет.
welcome-clamav = Проверка на вирусы
welcome-finish = Готово
welcome-heading = Настройка обновлений
welcome-notifications = Уведомления
welcome-root-warning = Установка без присмотра требует прав администратора, поэтому запуск по расписанию устанавливается как системная служба, работающая от root. Ничто другое в этом приложении от root не работает.

## Package sources

nav-sources = Источники пакетов
sources-add-apt = Добавить источник APT
sources-add-flatpak = Добавить репозиторий Flatpak
sources-add-heading = Добавить источник
sources-apt-hint = Источник APT записывается в /etc/apt/sources.list.d и требует прав администратора.
sources-changing = Применение…
sources-description = Репозитории, из которых берут ваши менеджеры пакетов. topgrade обновляет установленное; эти определяют, что вообще доступно.
sources-disable-note = Источники APT и dnf выключаются, а не удаляются, чтобы изменение можно было отменить вручную.
sources-disabled = Выключен
sources-enabled = Включён
sources-flatpak-hint = Репозиторий Flatpak добавляется только для вас и не требует пароля. Укажите адрес .flatpakrepo.
sources-heading = Откуда берутся пакеты
sources-name-placeholder = Название
sources-none = Источники пакетов не найдены.
sources-privileged = Изменение требует прав администратора.
sources-reload = Обновить
sources-remove = Удалить
sources-suite-placeholder = Выпуск (например, stable)
sources-url-placeholder = Адрес
show-tray-icon-description = Поднимает окно, запускает обновление, не открывая его, и завершает работу. Скрыть окно он не может — в Wayland нет способа это отменить.
welcome-show-again = Экран настройки
welcome-show-again-description = Вернуться к первому экрану и его вопросам.

releases-restart-failed = Не удалось запустить новую версию: { $message }
releases-restart-unknown-path = расположение самого приложения неизвестно
releases-restarting = Обновлено — перезапуск в новую версию…

releases-found-dismiss = Не эти
releases-found-new-why = Установлены из файла, поэтому ни один менеджер пакетов не предложит новее. Следить за их выпусками?
releases-found-watch = Следить за этими
upgrade-releases-with-run = Обновлять и программы, установленные со страниц выпусков
upgrade-releases-with-run-description = topgrade обновляет их, только если они установлены средством, которое их отслеживает. Установленный вручную пакет не покрывает ни одно, поэтому обновление применяет их здесь.
releases-found-new = { $count ->
    [one] 1 установленная программа, которую больше ничто не обновит
    [few] { $count } установленные программы, которые больше ничто не обновит
    [many] { $count } установленных программ, которые больше ничто не обновит
   *[other] { $count } установленной программы, которую больше ничто не обновит
}
