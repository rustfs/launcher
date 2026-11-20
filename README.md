# RustFS Launcher

**English** · [Deutsch](README_DE.md) · [Français](README_FR.md) · [日本語](README_JA.md) · [한국어](README_KO.md) · [简体中文](README_ZH.md) · [हिन्दी](README_HI.md)

RustFS Launcher is a small desktop app that runs [RustFS](https://github.com/rustfs/rustfs), an S3-compatible
object storage server, on your own computer. You pick a folder, press one button, and RustFS listens on
every network interface at port `9000`. On this computer the endpoint is `http://127.0.0.1:9000`. Other
devices on your network can reach it at this computer's address, same port. The keys start as `rustfsadmin`
/ `rustfsadmin`.

The server itself is bundled inside the app. There is nothing else to install, no terminal to open, and no
configuration file to edit.

![After start: the listening address, the current keys, and Stop](docs/images/launcher-running.png)

## What you can do with it

- Run object storage on your laptop for development, testing, or a personal backup target — without Docker.
- Point any S3 client or SDK at the endpoint and use it like a bucket in the cloud. On this computer that is `127.0.0.1`. On another device on the same network, use this computer's address and port `9000`.
- Open the console from the listening row, and copy the current keys after start. Logs stay hidden until you open them.
- Start and stop RustFS whenever you want. The launcher keeps running in the tray while the server works.
- Update the launcher (and the RustFS build inside it) from the app itself.

## Download

Grab the newest file for your machine from the [releases page](https://github.com/rustfs/launcher/releases).

| Your system | File |
| --- | --- |
| Windows 10 / 11, 64-bit | `rustfs-launcher-windows-x86_64-<version>-setup.exe` (about 50 MB) |
| macOS, Apple Silicon | `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS, Intel | `rustfs-launcher-macos-x86_64-<version>.app.zip` |

Windows on ARM works as well — it runs the x86_64 build through emulation. There is no Linux package yet; on
Linux, run the [RustFS binary](https://github.com/rustfs/rustfs/releases) directly.

The downloads are large because a full RustFS server is packed into the installer.

## Windows, step by step

### 1. Download the installer

Open the [releases page](https://github.com/rustfs/launcher/releases), expand **Assets** on the newest release
and click the file ending in `-setup.exe`. Edge and Chrome sometimes flag installers they rarely see; choose
**Keep** if the browser asks.

### 2. Get past SmartScreen

Double-click the file. If you see **Windows protected your PC**, click **More info**, then **Run anyway**. The
warning shows up because the installer is not signed with a commercial code-signing certificate yet — it is not
a verdict about the file.

### 3. Walk through the installer

The wizard asks the usual questions: welcome, license, install location, Start menu folder.

- The default location is `C:\Users\<you>\AppData\Local\RustFS Launcher`. It installs for your user only, so
  Windows will not ask for administrator rights.
- If your PC is missing the Microsoft WebView2 runtime, the installer downloads it once. This step needs an
  internet connection.
- On the last page, leave **Run RustFS Launcher** ticked. You can also let it create a desktop shortcut.

Afterwards the app lives in the Start menu under **RustFS Launcher**.

### 4. Create a folder for your data

RustFS stores objects in a folder you choose, and it expects that folder to exist already. Create something
like `D:\RustFS\data` in Explorer first. An empty folder on a drive with free space is the safest choice.

The server keeps its own log files in a `logs` folder next to the one you picked — `D:\RustFS\logs` in this
example.

### 5. Choose the data folder

![Before start: the data path and Launch RustFS](docs/images/launcher-ready.png)

The first screen is the data path and **Launch RustFS**. **Launch RustFS** is always visible. It stays faded,
at 40% opacity, until the path is filled in. Click **Browse**, type a path, or drop a folder on the window.

Host, port, the console, and the keys are not on this screen. The gear at the end of the header opens
**Settings**, a small square panel for the host, the API port, and the console. It is not a second launch
button, and it is not a separate page.

![Settings: host, API port, and console](docs/images/launcher-config.png)

A new install uses these defaults:

| Setting | Default |
| --- | --- |
| **Host** | `0.0.0.0`. The endpoint is reachable from other devices on your network. This computer still opens `127.0.0.1`. |
| **API port** | `9000` |
| **Console** | On, at port `9001` |
| **Access key** / **Secret key** | `rustfsadmin` / `rustfsadmin`. The launcher does not invent a new pair. |

Change the host, ports, or console in **Settings** before you launch if you need to. Change the keys after
the server is up, with **Change administrator**. A change there is used the next time you launch. The running
server keeps the keys it started with until then.

The footer shows the current version, then **Upgrade**, **Website**, **Docs**, and **GitHub**. **Upgrade**
opens the update panel. It is not the main button.

Your entries are remembered, so the next start is a single click.

### 6. Press Launch RustFS

You do not have to scroll to reach the path or **Launch RustFS**, including in a 960×760 window and at the
720×640 minimum. Logs stay hidden until you press **View logs** after the server is running. **Hide logs**,
or the Escape key, closes the panel and returns focus to that button. A startup check or an update message
does not open it.

Click **Launch RustFS**. The label changes to **Launching…** while the start is in progress.

If start, stop, or an update fails, the screen states the reason and how to fix it. That note does not push
the path and **Launch RustFS** (or **Stop RustFS**, once the server is up) off the screen, and it does not
open the log panel. The most common start failures are a data folder that does not exist and a port that is
already taken. RustFS also writes detail in the `logs` folder next to the data directory. Press **View logs**
when you want the line-by-line output.

### 7. Use the storage

After a successful start the path field leaves. If the console is on, the launcher opens it in the default
browser once. Later status checks do not open it again. The screen shows the listening address and the access
key and secret key the server is using. **Open in browser**, beside the listening address, opens that same
console. It does not copy the address and it does not open the S3 port. The access key and secret key each
have **Copy**. Those buttons are outlines. You can use the address and the keys immediately. There is no extra
confirmation step.

With the default host the listening address is `http://0.0.0.0:9000`. On this computer, point an S3 client
at `http://127.0.0.1:9000`. From another device on the same network, use this computer's address and port
`9000`. The keys are `rustfsadmin` and `rustfsadmin` until you change them.

**Open Console** opens the web console. **View logs** opens the log panel. **Change administrator** is where
you set a different access key and secret key for the next launch. **Stop RustFS** is the only filled button
on this screen, and it stops the server.

Point an S3 client at the endpoint:

```bash
aws --endpoint-url http://127.0.0.1:9000 s3 mb s3://demo
aws --endpoint-url http://127.0.0.1:9000 s3 cp report.pdf s3://demo/
```

Use the access key and secret key shown on the screen, `us-east-1` as the region, and path-style addressing.
To use a different pair, press **Change administrator**, stop the server, and launch again.

In the console, sign in with that same access key and secret key.

![Buckets in the RustFS web console](docs/images/rustfs-console.png)

### 8. Stop the server, or leave it running

**Stop RustFS** shuts the server down and brings back the path and **Launch RustFS**.

Closing the window does not quit anything — the launcher hides in the notification area and RustFS keeps
serving. Click the tray icon to bring the window back, or right-click it and choose **Quit**, which stops the
server and closes the app.

### 9. Uninstall

Remove **RustFS Launcher** from **Settings → Apps → Installed apps**, or use the uninstaller in its Start menu
folder. Your data folder is left alone; delete it yourself if you no longer need the objects in it.

## macOS

1. Unzip the `.app.zip` and drag **RustFS Launcher** into **Applications**.
2. The first launch is blocked, because the release builds are not notarized by Apple. Either right-click the
   app and choose **Open**, or clear the quarantine flag from a terminal:

   ```bash
   xattr -cr "/Applications/RustFS Launcher.app"
   open "/Applications/RustFS Launcher.app"
   ```

3. From there, everything works as described in steps 4 to 8 above. Clicking the Dock icon brings the window
   back after you close it.

## A tour of the window

![Before start: the data path and Launch RustFS](docs/images/launcher-ready.png)

Before start, the screen is the data path, **Browse**, and **Launch RustFS**, plus the settings gear and the
footer. Host, port, console, and keys stay in **Settings** or **Change administrator**. Logs are not shown
until you open them.

**After start.** The path field leaves. The listening address stays on screen, with **Open in browser** beside
it. The access key and secret key each have **Copy**. **Open Console**, **View logs**, and **Change
administrator** sit above **Stop RustFS**. **Open Console** stays available so you can open the console again.

**Something else is already listening.** If a program answers on the host and port but this launcher did not
start it, the path screen stays up and says so. Change the port in **Settings**, or stop that program. **Stop
RustFS** is not offered, because this launcher does not own that process.

**Upgrade.** The footer shows the current version. **Upgrade** opens a panel with **Check for Updates** and,
when one exists, the launcher and RustFS updates. It is not a button on the first screen.

**Logs.** Hidden until you press **View logs**. Startup checks and update messages do not open the panel.
**Hide logs** or Escape closes it. *App Logs* is the launcher talking: what it looked for, what it started,
why something failed. *RustFS Output* shows whatever the server prints on its console — recent RustFS builds
write their detailed logs to files instead, so this tab is often quiet after you open it. Auto-scroll follows
new lines; Clear empties both tabs.

## Where things end up

| What | Where |
| --- | --- |
| Your objects | The data folder you chose, plus a hidden `.rustfs.sys` folder for metadata |
| Server logs | A `logs` folder next to the data folder |
| Launcher settings | Stored by the app itself; no file for you to edit |
| The app | `%LOCALAPPDATA%\RustFS Launcher` on Windows, `/Applications` on macOS |
| Updated RustFS binary | App data: `%APPDATA%\com.dandan.rustfs-launcher\binaries` on Windows, `~/Library/Application Support/com.dandan.rustfs-launcher/binaries` on macOS |

## Updating

**Upgrade** in the footer covers **two independent updates**, both started by **Check for Updates**
(Windows and macOS only):

1. **Launcher** — a signed whole-app update. This replaces the launcher and the bundled RustFS, then restarts
   the app.
2. **RustFS** — a verified binary-only update from the upstream RustFS release feed. The archive is checked
   against `SHA256SUMS`, smoke-tested, and installed into the launcher data directory. You do not need a new
   launcher installer just to pick up a newer RustFS.

If RustFS is running, the launcher asks before stopping it. RustFS is not started again automatically after
either update.

The signing and release mechanics are documented in [docs/SELF_UPDATE.md](docs/SELF_UPDATE.md).

## When something goes wrong

**"Data path does not exist"** — create the folder in Explorer or Finder first; the launcher does not create it
for you.

**"Port 9000 is already in use"** — another program (or an older RustFS) holds the port. Pick a different API
port in **Settings**, or stop the other program. The same applies to the console port.

**The screen says something is already listening** — a RustFS is already answering on that host and port, but
this launcher did not start it. Stop that process where you started it, or change the port in **Settings**.

**RustFS Output stays empty** — after you open **View logs**, that tab is often quiet. Recent RustFS builds
write their logs into the `logs` folder next to your data directory. The other tab, App Logs, is what the
launcher itself did.

**The browser shows XML like `AccessDenied`** — that is the S3 API answering a browser, and nothing is broken.
Use **Open Console** for a web interface, and give the listening address to an S3 client instead.

**The window disappeared** — closing the window only hides it. Use the tray icon on Windows, or the Dock icon on
macOS.

**macOS says the app "cannot be opened"** — see the quarantine command in the macOS section above.

## Building it yourself

You need [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/), and [Trunk](https://trunkrs.dev/)
(`cargo install trunk`).

```bash
./build.sh          # build.bat on Windows — downloads the RustFS binary for your platform
cargo tauri dev     # run it with hot reload
cargo tauri build   # produce installers
```

Run `make pre-commit` before sending a pull request; it runs formatting, Clippy, the frontend build, and the
tests. More detail lives in [AGENTS.md](AGENTS.md), the release workflows in
[.github/ACTIONS.md](.github/ACTIONS.md). For editing, VS Code with the
[Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) and
[rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extensions is a
good setup.

## License

Apache-2.0. See [LICENSE](LICENSE).
