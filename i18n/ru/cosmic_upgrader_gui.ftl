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
hide-to-tray = Скрыть в область состояния
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
minimize-to-tray = Скрывать в область состояния вместо выхода
minimize-to-tray-description = Добавляет кнопку «Скрыть». Кнопка закрытия окна по-прежнему завершает работу.
nav-history = Журнал
notify-failed-steps = С ошибками: { $steps }
notify-title-failed = Обновление завершено с ошибками
notify-title-succeeded = Обновление завершено
show-tray-icon = Показывать значок в области состояния
tray-hide = Скрыть окно
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
