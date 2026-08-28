# RustFS Launcher

<p align="center">
  English |
  <a href="./README_ZH.md">简体中文</a> |
  <a href="./README_DE.md">Deutsch</a> |
  <a href="./README_FR.md">Français</a> |
  <a href="./README_JA.md">日本語</a> |
  <a href="./README_KO.md">한국어</a> |
  <a href="./README_HI.md">हिन्दी</a>
</p>

RustFS Launcher is a small desktop app for running [RustFS](https://github.com/rustfs/rustfs) on your own computer. You pick a folder, press Launch, and an S3-compatible API comes up on this machine. The web console is optional.

The installer already contains the RustFS binary. You do not download that separately.

The window itself is in English. These pages are just the documentation.

![Launcher window before start](docs/images/launcher-ready.png)

## Download

Builds live on the [Releases](https://github.com/rustfs/launcher/releases) page.

| Computer | File |
| --- | --- |
| Windows 10/11, 64-bit | `rustfs-launcher-windows-x86_64-<version>-setup.exe` |
| macOS, Apple Silicon | `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS, Intel | `rustfs-launcher-macos-x86_64-<version>.app.zip` |

Do not grab **Source code**. That zip is the Git tree, not the app.

If the newest tag only lists source archives, scroll until you find a release that actually has a `setup.exe` or `.app.zip` under Assets.

Windows on ARM uses the x86_64 installer through emulation. There is no Linux package yet.

## Windows: download, install, start

This is the path most people want.

### 1. Get the installer

1. Open [github.com/rustfs/launcher/releases](https://github.com/rustfs/launcher/releases).
2. Open a release that has installer files.
3. Under **Assets**, download `rustfs-launcher-windows-x86_64-…-setup.exe`.

### 2. Run it

Double-click the `.exe`.

Windows may say **Windows protected your PC**. The installer is not Authenticode-signed yet, so SmartScreen is being cautious. If the file came from this repo’s Releases page, click **More info**, then **Run anyway**.

Click through the NSIS installer. It puts **RustFS Launcher** in the Start menu.

### 3. Make a data folder first

Launcher will not create this for you. In Explorer, add an empty folder, for example `D:\rustfs\data`. Do not dump other files in there.

In PowerShell:

```powershell
New-Item -ItemType Directory -Force -Path D:\rustfs\data
```

### 4. Fill the form

Open **RustFS Launcher** from the Start menu.

- **Data Path** — click **Browse**, or drag the folder onto the window. The path is required, and it has to exist.
- **API Port** — `9000` unless something else already owns that port.
- **Host** — leave `127.0.0.1` so only this computer can connect.
- **Console Endpoint** — off by default. Turn it on if you want the web UI. Port `9001` is the usual value. API and console ports cannot be the same.
- **Access Key / Secret Key** — the app pre-fills `rustfsadmin` / `rustfsadmin`. Change them if anyone else uses this machine.

Then click **Launch RustFS**.

### 5. When it is up

The status pill turns **Service Online**, the form locks, and the button becomes **Stop RustFS**.

![Launcher after a successful start](docs/images/launcher-running.png)

Click the **API** card to open `http://127.0.0.1:9000`, or **Console** for `http://127.0.0.1:9001`. Sign into the console with the same keys you set.

S3 clients should use path-style addressing against port 9000.

Server logs land next to the data folder, not inside it. A path of `D:\rustfs\data` means log files under `D:\rustfs\logs`.

## Around the window

Left side is setup. Right side is logs.

**App Logs** is the launcher talking. **RustFS Output** is the server process. If something fails, look at **RustFS Output** first.

The update block can check GitHub for a newer launcher. Those two labels are still in Chinese (`版本与更新`, `检查更新`). An update replaces the whole app, including the bundled RustFS. Your data folder is left alone. If RustFS is running, the app asks before it stops it.

Closing the window does not quit. It hides to the tray, and RustFS keeps running. Right-click the tray icon → **Quit** to stop the server and exit. **Show**, or a left-click on the icon, brings the window back. Starting the app a second time just focuses the one that is already open.

## macOS

Download the zip that matches your chip, unzip it, move `RustFS Launcher.app` into Applications.

GitHub builds are not notarized, so the first open may fail:

```bash
xattr -cr "/Applications/RustFS Launcher.app"
open "/Applications/RustFS Launcher.app"
```

Or Control-click the app and choose **Open**.

## If it misbehaves

**Installer blocked.** Confirm you downloaded it from this repo. In the file’s Properties dialog, **Unblock** is fine after that.

**Data path is required / does not exist.** Create the folder first, then Browse again.

**Port already in use.** Pick another number, or stop whatever is sitting on 9000 / 9001.

**Console does not open.** Console Endpoint has to be on. The console is port 9001, not 7001.

**Detected Externally.** Something is already listening on that API port, but this launcher did not start it. Stop that process, or change the port. **Stop RustFS** only works for a process this app started.

**RustFS dies right after launch.** Read both log tabs. Usual causes: missing folder, no write permission, port clash.

## This is a local node

Launcher runs a single-node RustFS as a desktop process, not a Windows service. That is enough for trying it out and for development. Production clusters belong on Linux: [docs.rustfs.com/en/installation](https://docs.rustfs.com/en/installation).

A longer Windows walkthrough is at [Install RustFS on Windows](https://docs.rustfs.com/en/installation/windows).

## Building from source

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[Apache License 2.0](LICENSE).
