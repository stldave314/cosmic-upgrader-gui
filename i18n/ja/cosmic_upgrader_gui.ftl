app-title = アップグレーダー
app-description = topgrade を使ってシステム全体の更新を設定・予約・実行します。

## Navigation

nav-overview = 概要
nav-schedule = スケジュール
nav-configuration = 設定
nav-run = 実行

category-system = システム
category-applications = アプリケーション
category-containers = コンテナ
category-development = 開発
category-editors = エディター
category-repositories = リポジトリ
category-shell = シェル
category-ai-tools = AI ツール
category-cloud = クラウド
category-desktop = デスクトップ
category-custom = カスタムコマンド
category-other = その他

## Overview

overview-heading = 更新元
overview-subtitle = { $total } 個中 { $available } 個のステップがこのシステムに該当します。
topgrade-version = topgrade { $version }
topgrade-source-system = このシステムにインストール済み
topgrade-source-bundled = このアプリケーションに同梱
scanning = このシステムに該当する項目を確認しています…
scanning-progress = { $total } 個中 { $completed } 個を確認 — { $step }
rescan = 再確認
rescan-tooltip = 新しくインストールされたツールを再度探します

## Steps

steps-heading = ステップ
steps-none = このカテゴリーにステップはありません。
step-available = 準備完了
step-unavailable = 利用できません
step-inactive = 該当しません
step-deprecated = 非推奨
step-enabled-tooltip = 更新時にこのステップを含めます
step-disabled-tooltip = 更新時にこのステップを省略します
step-components = { $count ->
   *[other] { $count } 個のコンポーネント
}
enable-all = すべて有効化
disable-all = すべて無効化
show-unavailable = 利用できないステップを表示
show-unavailable-tooltip = ツールが未インストールのステップも一覧に表示します

status-ok = 準備完了
status-skipped = スキップ
status-failed = 失敗

## Running

run-heading = 実行
run-now = 更新を実行
dry-run = プレビュー
dry-run-tooltip = 何も変更せずに実行内容を表示します
run-in-progress = 更新しています…
run-step = { $step }
run-finished = 完了しました
run-cancelled = 中止しました
run-failed = エラーが発生して終了しました
run-never = まだ更新は実行されていません。
run-last = 最終実行 { $when }
run-summary = 成功 { $ok } 件、スキップ { $skipped } 件、失敗 { $failed } 件
cancel-run = 停止
clear-log = クリア
copy-log = 出力をコピー
run-selected-only = パッケージマネージャーの確認に自動で「はい」と答える

## Authentication

password-title = 管理者パスワードが必要です
password-body = 続行するには { $command } に管理者権限が必要です。
password-placeholder = パスワード
authenticate = 認証
authentication-failed = そのパスワードは受け付けられませんでした。

## Schedule

schedule-heading = 予約された更新
schedule-enabled = スケジュールに従って更新を確認する
schedule-frequency = 頻度
frequency-hourly = 毎時
frequency-daily = 毎日
frequency-weekly = 毎週
frequency-monthly = 毎月
schedule-time = 時刻
schedule-automatic = 更新を自動的にインストールする
schedule-automatic-description = 無効の場合、利用可能な更新を通知するだけで、何も変更しません。
schedule-next-run = 次回の実行 { $when }
schedule-next-run-unknown = 次回の実行時刻は不明です。
schedule-backend-systemd = systemd のユーザータイマーによって、このウィンドウを閉じていてもバックグラウンドで実行されます。
schedule-backend-fallback = systemd が利用できないため、予約実行はこのウィンドウが開いている間だけ行われます。
schedule-apply = スケジュールを適用
schedule-applied = スケジュールを更新しました。
schedule-error = スケジュールを適用できませんでした: { $message }

## Configuration

configuration-heading = topgrade の設定
configuration-path = { $path } を編集しています
configuration-default = 既定値: { $value }
configuration-not-set = 未設定
configuration-save = 変更を保存
configuration-revert = 元に戻す
configuration-reset = 既定値に戻す
configuration-unsaved = 保存されていない変更があります。
configuration-saved = 設定を保存しました。
configuration-free-form = これは自分で名前を付けるコマンドです。この項目はファイルを直接編集してください。
configuration-open-file = 設定ファイルを開く
configuration-add = 追加
configuration-remove = 削除

## Application settings

settings = 設定
about = このアプリについて
appearance = 外観
theme = テーマ
theme-system = デスクトップに合わせる
theme-light = ライト
theme-dark = ダーク
behaviour = 動作
privilege-backend = 管理者権限
privilege-pty = このウィンドウで尋ねる
privilege-pty-description = topgrade を端末で実行し、パスワードが必要になったらここで尋ねます。
privilege-pkexec = システムのダイアログ
privilege-pkexec-description = デスクトップの認証ダイアログを使います。コマンドごとに 1 回尋ねます。
confirm-before-running = 更新を開始する前に確認する
notify-on-completion = 予約実行の終了時に通知する

## Errors and empty states

topgrade-missing-title = topgrade がインストールされていません
topgrade-missing-body = このアプリケーションは topgrade を利用しますが、このシステムでは見つかりませんでした。
topgrade-missing-hint = パッケージマネージャー、または次のコマンドでインストールしてください: { $command }
topgrade-too-old-title = topgrade のバージョンが古すぎます
topgrade-too-old-body = topgrade { $found } が見つかりましたが、{ $required } 以降が必要です。
error-title = 問題が発生しました
retry = 再試行

## Common

cancel = キャンセル
close = 閉じる
save = 保存
ok = OK
toggle-sidebar = サイドバーの表示を切り替え
git-description = Git の説明
repository = リポジトリ
support = サポート

## History, first run, custom commands and status area

autostart = セッションと一緒に起動する
autostart-description = ~/.config/autostart にエントリーを追加します
category-settings = このカテゴリーの設定
category-settings-none = このカテゴリーに固有の topgrade 設定はありません。
command-name-placeholder = 名前
command-value-placeholder = 実行するコマンド
custom-commands-description = 自分で名前を付けるコマンドです。topgrade は独立したステップとして実行します。
custom-commands-none = カスタムコマンドはまだありません。
first-run-accept = 続ける
first-run-autostart = セッションと一緒に起動する
first-run-autostart-description = 予約された確認が実行できるよう、ログイン時に最小化して起動します。
first-run-body = これらの設定は、アプリケーションがウィンドウの外でどう振る舞うかを変えます。あとから設定で変更できます。
first-run-title = いくつかの選択
first-run-tray = ステータス領域にアイコンを表示する
first-run-tray-description = ウィンドウを隠して呼び戻したり、開かずに更新を開始したりできます。
history-back = 一覧に戻る
history-delete = 削除
history-detail = { $outcome } · { $origin } · { $duration }
history-duration-seconds = { $seconds } 秒
history-heading = 過去の実行
history-none = まだ実行は記録されていません。
history-origin-manual = ここから開始
history-origin-scheduled = 予約
history-outcome-cancelled = 中止
history-outcome-failed = 失敗
history-outcome-succeeded = 成功
history-transcript-unavailable = この実行のログを読み取れませんでした。
view = 表示
keep-run-logs = 保持する実行数
nav-history = 履歴
notify-failed-steps = 失敗: { $steps }
notify-title-failed = 更新がエラーで終了しました
notify-title-succeeded = 更新が完了しました
show-tray-icon = ステータス領域にアイコンを表示する
tray-quit = 終了
tray-show = ウィンドウを表示
tray-unavailable = このデスクトップにステータス領域が見つからないため、アイコンは表示されません。

## Releases

nav-releases = リリース
releases-add-selected = 選択を監視する
releases-cancel-find = キャンセル
releases-check = 更新を確認
releases-checking = { $total } 件中 { $done } 件を確認中…
releases-description = プロジェクトのリリースページからインストールしたソフトウェアにはパッケージマネージャーがないため、topgrade では更新できません。これらはプロジェクト自体に問い合わせて確認します。
releases-error = 確認できませんでした: { $message }
releases-find = プロジェクトを探す
releases-finding = インストール済みパッケージを調べています…
releases-found = このシステムで { $count } 件のプロジェクトが見つかりました。監視するものを選んでください。
releases-heading = プロジェクトのリリース
releases-installed = { $name } を { $version } に更新しました
releases-install-failed = { $name } を更新できませんでした: { $message }
releases-installing = { $name } をインストールしています…
releases-no-asset = このリリースにはこのシステムに合うファイルがありません。リリースページをご利用ください。
releases-none = まだ監視しているプロジェクトはありません。
releases-no-releases = リリースなし
releases-no-transport = curl も gh もインストールされていないため、リリースを確認できません。
releases-open = リリースページ
releases-remove = 監視をやめる
releases-source = { $source } · { $forge }
releases-unidentified = プロジェクト不明 — { $version } がインストール済み
releases-unknown = { $version } が公開されています
releases-update = 更新
releases-update-available = { $version } が利用可能
releases-up-to-date = 最新です
releases-watched = { $count } 件を監視中
run-was-preview = これはプレビューです。システムには何も変更されていません。

interval-daily = 毎日
interval-manual = 要求されたときのみ
interval-six-hourly = 6 時間ごと
interval-weekly = 毎週
releases-interval = 自動的に確認する
releases-last-checked = 最終確認 { $when }
releases-never-checked = 未確認
releases-next-check = 次回 { $when }

## Dependencies, release channel and directories

channel-pre-release = ベータ版とリリース候補も含める
channel-stable = 安定版のみ
dep-authentication-dismissed = 認証は取り消されました。
dep-curl = プロジェクトのホストからリリース情報を取得し、更新をダウンロードします。
dependencies-all-present = このアプリケーションに必要なものはすべてインストールされています。
dependencies-description = このアプリケーションは他のプログラムを動かすことで機能します。欠けているとその機能が黙って何もしなくなるため、用途とともにここに一覧します。
dependencies-heading = 必要なツール
dependencies-install = インストール
dependencies-installed = インストール済み
dependencies-install-failed = { $name } をインストールできませんでした: { $message }
dependencies-installing = インストールしています…
dependencies-missing = 未インストール
dependencies-no-manager = 対応するパッケージマネージャーが見つからないため、ここからはインストールできません。
dependencies-optional = 任意
dependencies-recheck = 再確認
dependencies-required = 必須
dep-gh = GitHub の資格情報を利用し、リリース確認の上限を毎時 60 件から 5000 件に引き上げます。
dep-notify-send = 誰も見ていない予約実行の結果を通知します。
dep-pkexec = システム更新やパッケージのインストールのために、デスクトップのダイアログで管理者権限を求めます。
dep-systemctl = 更新のスケジュールを systemd のユーザータイマーとして保持し、ウィンドウを閉じていても実行します。
dep-topgrade = 更新そのものを実行します。これがないとこのアプリケーションには動かす対象がありません。
dep-xdg-open = リリースページやリンクをブラウザーで開きます。
nav-dependencies = 依存関係
releases-channel = 提示するリリース
releases-directories = ダウンロードしたアプリケーションの場所
releases-directories-description = AppImage やその他のダウンロード済みプログラムを探します。相対パスはホームディレクトリーからです。
releases-directory-add = ディレクトリーを追加
releases-directory-placeholder = Applications
releases-self = このアプリケーション
dependencies-missing-required = { $count ->
   *[other] 必須のツールが { $count } 個不足しています。
}
releases-channel-description = リリース候補やベータ版を更新として扱うかどうか。

## Welcome, notifications and virus scanning

clamav-clean = スキャン完了: { $scanned } 個のファイルを確認し、何も見つかりませんでした。
clamav-failed = スキャンを実行できませんでした: { $message }
clamav-infected = スキャン完了: 感染ファイルが { $infected } 個見つかりました。
clamav-options = スキャンのオプション
clamav-scan = ウイルス定義の更新後にスキャンする
clamav-scan-description = ClamAV がインストールされています。topgrade が定義を最新に保ち、定義が変わり次第これが新しい定義でスキャンします。
clamav-scanning = ウイルス定義が変わりました — スキャンしています…
clamav-target = スキャンの対象
nav-welcome = ようこそ
notify-errors = 更新に失敗したら知らせる
notify-errors-description = 他の通知を切っていても失敗は通知されます。これも切った場合を除きます。
notify-title-available = 更新が利用できます
notify-title-installed = 更新をインストールしました
notify-upgrades = 更新について知らせる
notify-upgrades-available = インストールできるものをお知らせします。
notify-upgrades-installed = インストールされたものをお知らせします。
welcome-automatic-heading = 更新のインストール
welcome-body = いま決めておくとよい項目です。あとから設定でも変更でき、どれも取り消せます。
welcome-clamav = ウイルススキャン
welcome-finish = 完了
welcome-heading = 更新の設定
welcome-notifications = 通知
welcome-root-warning = 無人でのインストールには管理者権限が必要なため、予約実行は root で動作するシステムサービスとして設定されます。このアプリケーションの他の部分は root では動作しません。

## Package sources

nav-sources = パッケージの取得元
sources-add-apt = APT ソースを追加
sources-add-flatpak = Flatpak リモートを追加
sources-add-heading = 取得元を追加
sources-apt-hint = APT ソースは /etc/apt/sources.list.d に書き込まれ、管理者権限が必要です。
sources-changing = 適用しています…
sources-description = パッケージマネージャーが取得するリポジトリーです。topgrade は導入済みのものを更新し、これらは何が入手できるかを決めます。
sources-disable-note = APT と dnf の取得元は削除ではなく無効化されるため、変更は手作業で元に戻せます。
sources-disabled = 無効
sources-enabled = 有効
sources-flatpak-hint = Flatpak リモートはあなた専用に追加され、パスワードは不要です。.flatpakrepo の URL を指定してください。
sources-heading = パッケージの取得元
sources-name-placeholder = 名前
sources-none = パッケージの取得元が見つかりませんでした。
sources-privileged = 変更には管理者権限が必要です。
sources-reload = 再読み込み
sources-remove = 削除
sources-suite-placeholder = スイート (例: stable)
sources-url-placeholder = URL
show-tray-icon-description = ウィンドウを前面に出し、開かずに更新を開始し、終了します。ウィンドウを隠すことはできません — Wayland にはそれを元に戻す手段がありません。
welcome-show-again = 設定画面
welcome-show-again-description = 最初の画面とその選択に戻ります。

releases-restart-failed = 新しいバージョンを起動できませんでした: { $message }
releases-restart-unknown-path = このアプリケーション自身の場所が分かりません
releases-restarting = 更新しました — 新しいバージョンで再起動しています…

releases-found-dismiss = これらは不要
releases-found-new-why = ファイルからインストールされているため、どのパッケージマネージャーにも新しいものがありません。リリースを監視しますか？
releases-found-watch = これらを監視
upgrade-releases-with-run = リリースページから導入したプログラムも更新する
upgrade-releases-with-run-description = topgrade はそれらを追跡するツールで導入した場合にのみ更新します。手作業で入れたパッケージはどれにも該当しないため、更新時にここで適用します。
releases-found-new = { $count ->
   *[other] ほかに更新するもののない導入済みプログラムが { $count } 個
}
