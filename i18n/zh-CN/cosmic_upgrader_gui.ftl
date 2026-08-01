app-title = 升级器
app-description = 使用 topgrade 配置、计划并执行全系统升级。

## Navigation

nav-overview = 概览
nav-schedule = 计划
nav-configuration = 配置
nav-run = 运行

category-system = 系统
category-applications = 应用程序
category-containers = 容器
category-development = 开发
category-editors = 编辑器
category-repositories = 仓库
category-shell = Shell
category-ai-tools = AI 工具
category-cloud = 云
category-desktop = 桌面
category-custom = 自定义命令
category-other = 其他

## Overview

overview-heading = 升级来源
overview-subtitle = { $total } 个步骤中有 { $available } 个适用于本系统。
topgrade-version = topgrade { $version }
topgrade-source-system = 已安装在本系统
topgrade-source-bundled = 随本应用程序提供
scanning = 正在检查哪些内容适用于本系统…
scanning-progress = 已检查 { $total } 个中的 { $completed } 个 — { $step }
rescan = 重新检查
rescan-tooltip = 重新查找新安装的工具

## Steps

steps-heading = 步骤
steps-none = 此类别中没有步骤。
step-available = 就绪
step-unavailable = 不可用
step-inactive = 不适用
step-deprecated = 已弃用
step-enabled-tooltip = 升级时包含此步骤
step-disabled-tooltip = 升级时跳过此步骤
step-components = { $count ->
   *[other] { $count } 个组件
}
enable-all = 全部启用
disable-all = 全部禁用
show-unavailable = 显示不可用的步骤
show-unavailable-tooltip = 同时列出工具尚未安装的步骤

status-ok = 就绪
status-skipped = 已跳过
status-failed = 失败

## Running

run-heading = 运行
run-now = 开始升级
dry-run = 预览
dry-run-tooltip = 显示将要执行的操作，但不做任何更改
run-in-progress = 正在升级…
run-step = { $step }
run-finished = 已完成
run-cancelled = 已取消
run-failed = 完成，但有错误
run-never = 尚未执行过升级。
run-last = 上次运行 { $when }
run-summary = 成功 { $ok } 项，跳过 { $skipped } 项，失败 { $failed } 项
cancel-run = 停止
clear-log = 清除
copy-log = 复制输出
run-selected-only = 自动确认包管理器的提示

## Authentication

password-title = 需要管理员密码
password-body = { $command } 需要管理员权限才能继续。
password-placeholder = 密码
authenticate = 验证
authentication-failed = 该密码未被接受。

## Schedule

schedule-heading = 计划的升级
schedule-enabled = 按计划检查升级
schedule-frequency = 频率
frequency-hourly = 每小时
frequency-daily = 每天
frequency-weekly = 每周
frequency-monthly = 每月
schedule-time = 时间
schedule-automatic = 自动安装升级
schedule-automatic-description = 关闭时，仅通过通知报告可用的升级，不做任何更改。
schedule-next-run = 下次运行 { $when }
schedule-next-run-unknown = 下次运行时间未知。
schedule-backend-systemd = 通过 systemd 用户定时器在后台运行，即使此窗口已关闭。
schedule-backend-fallback = systemd 不可用，因此计划运行仅在此窗口打开时进行。
schedule-apply = 应用计划
schedule-applied = 计划已更新。
schedule-error = 无法应用计划：{ $message }

## Configuration

configuration-heading = topgrade 配置
configuration-path = 正在编辑 { $path }
configuration-default = 默认值：{ $value }
configuration-not-set = 未设置
configuration-save = 保存更改
configuration-revert = 还原
configuration-reset = 重置为默认值
configuration-unsaved = 有未保存的更改。
configuration-saved = 配置已保存。
configuration-free-form = 这些是由您自己命名的命令。请直接在文件中编辑此节。
configuration-open-file = 打开配置文件
configuration-add = 添加
configuration-remove = 移除

## Application settings

settings = 设置
about = 关于
appearance = 外观
theme = 主题
theme-system = 跟随桌面
theme-light = 浅色
theme-dark = 深色
behaviour = 行为
privilege-backend = 管理员权限
privilege-pty = 在此窗口中询问
privilege-pty-description = 在终端中运行 topgrade，需要密码时在此处询问。
privilege-pkexec = 系统对话框
privilege-pkexec-description = 使用桌面自带的验证对话框。每条命令询问一次。
confirm-before-running = 开始升级前先确认
notify-on-completion = 计划运行结束时通知

## Errors and empty states

topgrade-missing-title = 未安装 topgrade
topgrade-missing-body = 本应用程序需要 topgrade，但在本系统上未找到。
topgrade-missing-hint = 请使用您的包管理器安装，或使用：{ $command }
topgrade-too-old-title = topgrade 版本过旧
topgrade-too-old-body = 找到了 topgrade { $found }，但需要 { $required } 或更高版本。
error-title = 出现了问题
retry = 重试

## Common

cancel = 取消
close = 关闭
save = 保存
ok = 确定
toggle-sidebar = 切换侧边栏
git-description = Git 描述
repository = 仓库
support = 支持

## History, first run, custom commands and status area

autostart = 随会话启动
autostart-description = 在 ~/.config/autostart 中添加一个条目
category-settings = 此类别的设置
category-settings-none = 此类别没有自己的 topgrade 设置。
command-name-placeholder = 名称
command-value-placeholder = 要运行的命令
custom-commands-description = 由您自己命名的命令。topgrade 会将它们作为独立的步骤运行。
custom-commands-none = 还没有自定义命令。
first-run-accept = 继续
first-run-autostart = 随会话启动
first-run-autostart-description = 登录时以最小化方式启动，以便计划的检查可以运行。
first-run-body = 这些选项会改变应用程序在自己窗口之外的行为。您以后可以在设置中更改它们。
first-run-title = 几项选择
first-run-tray = 在状态区域显示图标
first-run-tray-description = 可以隐藏窗口并将其恢复，也可以不打开窗口就开始升级。
hide-to-tray = 隐藏到状态区域
history-back = 返回列表
history-delete = 删除
history-detail = { $outcome } · { $origin } · { $duration }
history-duration-seconds = { $seconds } 秒
history-heading = 过往运行
history-none = 尚未记录任何运行。
history-origin-manual = 在此启动
history-origin-scheduled = 按计划
history-outcome-cancelled = 已取消
history-outcome-failed = 失败
history-outcome-succeeded = 成功
history-transcript-unavailable = 无法读取此次运行的日志。
view = 查看
keep-run-logs = 保留的运行数
minimize-to-tray = 隐藏到状态区域而不是退出
minimize-to-tray-description = 添加一个“隐藏”按钮。窗口的关闭按钮仍会退出。
nav-history = 历史
notify-failed-steps = 失败：{ $steps }
notify-title-failed = 升级完成，但有错误
notify-title-succeeded = 升级完成
show-tray-icon = 在状态区域显示图标
tray-hide = 隐藏窗口
tray-quit = 退出
tray-show = 显示窗口
tray-unavailable = 在此桌面上未找到状态区域，因此不显示图标。

## Releases

nav-releases = 发布
releases-add-selected = 监视所选项
releases-cancel-find = 取消
releases-check = 检查更新
releases-checking = 已检查 { $total } 个中的 { $done } 个…
releases-description = 从项目发布页面安装的软件背后没有包管理器，因此 topgrade 无法更新它们。这些会直接向项目本身查询。
releases-error = 无法检查：{ $message }
releases-find = 查找项目
releases-finding = 正在检查已安装的软件包…
releases-found = 在此系统上找到 { $count } 个项目。请选择要监视的项目。
releases-heading = 项目发布
releases-installed = { $name } 已更新到 { $version }
releases-install-failed = 无法更新 { $name }：{ $message }
releases-installing = 正在安装 { $name }…
releases-no-asset = 此发布中没有与本系统匹配的文件；请使用发布页面。
releases-none = 尚未监视任何项目。
releases-no-releases = 没有发布
releases-no-transport = 未安装 curl 或 gh，因此无法检查发布。
releases-open = 发布页面
releases-remove = 停止监视
releases-source = { $source } · { $forge }
releases-unidentified = 项目未知 — 已安装 { $version }
releases-unknown = 已发布 { $version }
releases-update = 更新
releases-update-available = 有 { $version } 可用
releases-up-to-date = 已是最新
releases-watched = 已监视 { $count } 个
run-was-preview = 这是预览 — 系统未做任何更改。

interval-daily = 每天
interval-manual = 仅在请求时
interval-six-hourly = 每 6 小时
interval-weekly = 每周
releases-interval = 自动检查
releases-last-checked = 上次检查 { $when }
releases-never-checked = 尚未检查
releases-next-check = 下次 { $when }

## Dependencies, release channel and directories

channel-pre-release = 包含测试版和候选发布版
channel-stable = 仅稳定版
dep-authentication-dismissed = 身份验证已取消。
dep-curl = 从项目主机获取发布信息并下载更新。
dependencies-all-present = 此应用程序所需的一切均已安装。
dependencies-description = 此应用程序通过驱动其他程序来工作。缺少某个程序会让相应功能悄无声息地失效，因此在此列出它们及其用途。
dependencies-heading = 所需工具
dependencies-install = 安装
dependencies-installed = 已安装
dependencies-install-failed = 无法安装 { $name }：{ $message }
dependencies-installing = 正在安装…
dependencies-missing = 未安装
dependencies-no-manager = 未找到受支持的包管理器，因此无法从此处安装。
dependencies-optional = 可选
dependencies-recheck = 重新检查
dependencies-required = 必需
dep-gh = 使用您的 GitHub 凭据，将发布检查上限从每小时 60 次提升到 5000 次。
dep-notify-send = 报告无人查看的计划运行的结果。
dep-pkexec = 通过桌面自带的对话框请求管理员权限，用于系统升级和软件包安装。
dep-systemctl = 以 systemd 用户定时器的形式保持升级计划，使其在此窗口关闭时也能运行。
dep-topgrade = 执行升级本身。没有它，此应用程序就无事可做。
dep-xdg-open = 在浏览器中打开发布页面和链接。
nav-dependencies = 依赖项
releases-channel = 要提供的发布
releases-directories = 已下载应用程序的目录
releases-directories-description = 会在其中查找 AppImage 和其他已下载的程序。相对路径从您的主目录开始。
releases-directory-add = 添加目录
releases-directory-placeholder = Applications
releases-self = 此应用程序
dependencies-missing-required = { $count ->
   *[other] 缺少 { $count } 个必需的工具。
}
releases-channel-description = 候选发布版和测试版是否算作更新。

## Welcome, notifications and virus scanning

clamav-clean = 扫描完成：已检查 { $scanned } 个文件，未发现问题。
clamav-failed = 无法执行扫描：{ $message }
clamav-infected = 扫描完成：发现 { $infected } 个受感染文件。
clamav-options = 扫描选项
clamav-scan = 病毒库更新后进行扫描
clamav-scan-description = 已安装 ClamAV。topgrade 会保持其病毒库为最新；病毒库一旦更新，此项就用新库进行扫描。
clamav-scanning = 病毒库已更新 — 正在扫描…
clamav-target = 扫描范围
nav-welcome = 欢迎
notify-errors = 升级失败时通知我
notify-errors-description = 即使关闭了其他通知，失败仍会被通知，除非此项也被关闭。
notify-title-available = 有可用的升级
notify-title-installed = 升级已安装
notify-upgrades = 通知我升级情况
notify-upgrades-available = 将告知您有哪些可供安装。
notify-upgrades-installed = 将告知您安装了什么。
welcome-automatic-heading = 安装升级
welcome-body = 现在值得做的几项选择。之后都可在设置中找到，且没有任何一项是不可更改的。
welcome-clamav = 病毒扫描
welcome-finish = 完成
welcome-heading = 设置升级
welcome-notifications = 通知
welcome-root-warning = 无人值守安装需要管理员权限，因此计划运行将作为以 root 身份运行的系统服务安装。此应用程序的其他部分都不会以 root 运行。

## Package sources

nav-sources = 软件包来源
sources-add-apt = 添加 APT 源
sources-add-flatpak = 添加 Flatpak 远程
sources-add-heading = 添加来源
sources-apt-hint = APT 源会写入 /etc/apt/sources.list.d，需要管理员权限。
sources-changing = 正在应用…
sources-description = 您的包管理器拉取的仓库。topgrade 升级已安装的内容；这些决定有什么可用。
sources-disable-note = APT 和 dnf 来源是被停用而非删除的，因此更改可以手工撤销。
sources-disabled = 已停用
sources-enabled = 已启用
sources-flatpak-hint = Flatpak 远程仅为您添加，无需密码。请指向 .flatpakrepo 网址。
sources-heading = 软件包来自哪里
sources-name-placeholder = 名称
sources-none = 未找到软件包来源。
sources-privileged = 更改此项需要管理员权限。
sources-reload = 重新加载
sources-remove = 移除
sources-suite-placeholder = 套件（例如 stable）
sources-url-placeholder = 网址
