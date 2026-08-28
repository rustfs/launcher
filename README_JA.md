# RustFS Launcher

<p align="center">
  <a href="./README.md">English</a> |
  <a href="./README_ZH.md">简体中文</a> |
  <a href="./README_DE.md">Deutsch</a> |
  <a href="./README_FR.md">Français</a> |
  日本語 |
  <a href="./README_KO.md">한국어</a> |
  <a href="./README_HI.md">हिन्दी</a>
</p>

RustFS Launcher は、自分のパソコンで [RustFS](https://github.com/rustfs/rustfs) を動かすための小さなデスクトップアプリです。フォルダを選んで Launch を押せば、このマシン上に S3 互換 API が立ち上がります。Web コンソールは任意です。

RustFS 本体はインストーラに入っています。別途ダウンロードする必要はありません。

画面の文言は英語です。この説明だけ多言語にしています。

![起動前の Launcher](docs/images/launcher-ready.png)

## ダウンロード

ビルドは [Releases](https://github.com/rustfs/launcher/releases) にあります。

| 環境 | ファイル |
| --- | --- |
| Windows 10/11、64 ビット | `rustfs-launcher-windows-x86_64-<version>-setup.exe` |
| macOS、Apple Silicon | `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS、Intel | `rustfs-launcher-macos-x86_64-<version>.app.zip` |

**Source code** はアプリではありません。Git のソースです。

最新タグの Assets がソース zip だけなら、下にスクロールして `setup.exe` か `.app.zip` があるリリースを選んでください。

ARM 版 Windows は x86_64 インストーラをエミュレーションで動かします。Linux 用パッケージはまだありません。

## Windows：ダウンロード、インストール、起動

いちばん多い手順です。

### 1. インストーラを取る

1. [github.com/rustfs/launcher/releases](https://github.com/rustfs/launcher/releases) を開く。
2. インストーラが入っているリリースを選ぶ。
3. **Assets** から `rustfs-launcher-windows-x86_64-…-setup.exe` をダウンロードする。

### 2. 入れる

`.exe` をダブルクリックします。

**Windows によって PC が保護されました** と出ることがあります。Authenticode 署名がまだ無いので、SmartScreen が止めます。このリポジトリの Releases から取ったファイルなら、**詳細情報** → **実行** で進めてください。

NSIS の画面を進めると、スタートメニューに **RustFS Launcher** が入ります。

### 3. 先にデータ用フォルダを作る

Launcher はフォルダを作りません。エクスプローラーで空のフォルダを作ってください。例: `D:\rustfs\data`。余計なファイルは置かないこと。

PowerShell:

```powershell
New-Item -ItemType Directory -Force -Path D:\rustfs\data
```

### 4. フォームを埋める

スタートメニューから **RustFS Launcher** を開きます。

- **Data Path** — **Browse** するか、フォルダをウィンドウへドロップ。必須で、すでに存在する必要があります。
- **API Port** — 空いていれば `9000`。
- **Host** — `127.0.0.1` のままなら、このマシンからしか繋がりません。
- **Console Endpoint** — 初期値はオフ。Web UI が要るならオン。ポートはだいたい `9001`。API と同じ番号にはできません。
- **Access Key / Secret Key** — 最初は `rustfsadmin` / `rustfsadmin` が入っています。他の人も使うマシンなら変えてください。

**Launch RustFS** を押します。

### 5. 起動したあと

表示が **Service Online** になり、フォームはロック、ボタンは **Stop RustFS** に変わります。

![起動に成功した Launcher](docs/images/launcher-running.png)

**API** カードで `http://127.0.0.1:9000`、**Console** で `http://127.0.0.1:9001` が開きます。コンソールは、いま入れたキーでログインします。

S3 クライアントは path-style、ポート 9000 です。

サーバーログはデータフォルダの中ではなく、隣に出ます。`D:\rustfs\data` なら `D:\rustfs\logs` です。

## 画面の見方

左が設定、右がログです。

**App Logs** は Launcher 側。**RustFS Output** はサーバープロセスです。失敗したら、まず **RustFS Output** を見てください。

左の更新欄から GitHub の新バージョンを確認できます。ラベルはまだ中国語です（`版本与更新`、`检查更新`）。更新はアプリ一式（同梱の RustFS 含む）を入れ替えます。データフォルダは触りません。RustFS が動いているときは、止める前に確認します。

ウィンドウを閉じても終了しません。トレイに隠れ、RustFS は動き続けます。トレイアイコンを右クリックして **Quit** でサーバー停止と終了です。**Show**、または左クリックでウィンドウが戻ります。もう一度起動しても、既に開いているウィンドウが前面に出るだけです。

## macOS

チップに合う zip を落として展開し、`RustFS Launcher.app` をアプリケーションへ移します。

GitHub ビルドは公証されていません。初回は弾かれることがあります。

```bash
xattr -cr "/Applications/RustFS Launcher.app"
open "/Applications/RustFS Launcher.app"
```

またはアプリを Control クリックして **開く**。

## うまくいかないとき

**インストーラが止まる。** このリポジトリから取ったか確認。プロパティの **ブロックの解除** は、確認後なら問題ありません。

**Data path is required / does not exist。** 先にフォルダを作って、もう一度 Browse。

**ポート使用中。** 番号を変えるか、9000 / 9001 を使っているプロセスを止める。

**コンソールが開かない。** Console Endpoint をオンにする。ポートは 9001 で、7001 ではありません。

**Detected Externally。** その API ポートで何かが待っているが、この Launcher が起動したものではない、という意味です。そのプロセスを止めるか、ポートを変えてください。**Stop RustFS** が止められるのは、このアプリが起動したプロセスだけです。

**Launch 直後に落ちる。** 両方のログを見る。よくあるのは、フォルダがない、書き込めない、ポート衝突です。

## これはローカルノードです

Launcher は RustFS をデスクトッププロセスとして動かします。Windows サービスではありません。試す・開発する用途向けです。本番クラスタは Linux 側です: [docs.rustfs.com/en/installation](https://docs.rustfs.com/en/installation)。

Windows の長い手順: [Install RustFS on Windows](https://docs.rustfs.com/en/installation/windows)。

## ソースからビルド

[CONTRIBUTING.md](CONTRIBUTING.md) を見てください。

## ライセンス

[Apache License 2.0](LICENSE)。
