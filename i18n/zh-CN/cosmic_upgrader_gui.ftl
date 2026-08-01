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
