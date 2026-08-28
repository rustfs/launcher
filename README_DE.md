# RustFS Launcher

<p align="center">
  <a href="./README.md">English</a> |
  <a href="./README_ZH.md">简体中文</a> |
  Deutsch |
  <a href="./README_FR.md">Français</a> |
  <a href="./README_JA.md">日本語</a> |
  <a href="./README_KO.md">한국어</a> |
  <a href="./README_HI.md">हिन्दी</a>
</p>

RustFS Launcher ist eine kleine Desktop-App, mit der du [RustFS](https://github.com/rustfs/rustfs) lokal startest. Ordner wählen, Launch drücken, fertig: auf diesem Rechner läuft eine S3-kompatible API. Die Web-Konsole ist optional.

Das RustFS-Binary steckt schon im Installer. Extra herunterladen musst du es nicht.

Die Oberfläche ist auf Englisch. Diese Seiten gibt es in mehreren Sprachen.

![Launcher-Fenster vor dem Start](docs/images/launcher-ready.png)

## Download

Die Builds liegen unter [Releases](https://github.com/rustfs/launcher/releases).

| Rechner | Datei |
| --- | --- |
| Windows 10/11, 64-Bit | `rustfs-launcher-windows-x86_64-<version>-setup.exe` |
| macOS, Apple Silicon | `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS, Intel | `rustfs-launcher-macos-x86_64-<version>.app.zip` |

Nicht **Source code** nehmen. Das ist der Git-Baum, nicht die App.

Stehen unter dem neuesten Tag nur Quell-Zips, weiter scrollen, bis unter Assets wirklich eine `setup.exe` oder `.app.zip` liegt.

Windows on ARM nimmt den x86_64-Installer und emuliert. Ein Linux-Paket gibt es noch nicht.

## Windows: herunterladen, installieren, starten

Der übliche Weg.

### 1. Installer holen

1. [github.com/rustfs/launcher/releases](https://github.com/rustfs/launcher/releases) öffnen.
2. Ein Release mit Installer-Dateien wählen.
3. Unter **Assets** `rustfs-launcher-windows-x86_64-…-setup.exe` laden.

### 2. Installieren

Die `.exe` doppelklicken.

Windows kann **Windows hat den PC geschützt** anzeigen. Der Installer ist noch nicht Authenticode-signiert, SmartScreen hakelt deshalb. Kommt die Datei von der Releases-Seite dieses Repos: **Weitere Informationen**, dann **Trotzdem ausführen**.

Den NSIS-Assistenten durchklicken. Danach steht **RustFS Launcher** im Startmenü.

### 3. Zuerst den Datenordner anlegen

Launcher legt den Ordner nicht selbst an. Im Explorer einen leeren Ordner erstellen, z. B. `D:\rustfs\data`. Keine fremden Dateien dort ablegen.

In PowerShell:

```powershell
New-Item -ItemType Directory -Force -Path D:\rustfs\data
```

### 4. Formular ausfüllen

**RustFS Launcher** über das Startmenü öffnen.

- **Data Path** — **Browse** klicken oder den Ordner ins Fenster ziehen. Pflichtfeld, und der Ordner muss schon existieren.
- **API Port** — `9000`, außer der Port ist schon belegt.
- **Host** — `127.0.0.1` lassen, dann kommt nur dieser Rechner drauf.
- **Console Endpoint** — standardmäßig aus. Einschalten, wenn du die Web-UI willst. Port meist `9001`. API- und Console-Port dürfen nicht gleich sein.
- **Access Key / Secret Key** — vorausgefüllt `rustfsadmin` / `rustfsadmin`. Ändern, wenn noch jemand anderes an den Rechner kommt.

Dann **Launch RustFS**.

### 5. Wenn es läuft

Die Statusanzeige wird **Service Online**, das Formular sperrt sich, der Button heißt **Stop RustFS**.

![Launcher nach erfolgreichem Start](docs/images/launcher-running.png)

**API**-Karte öffnet `http://127.0.0.1:9000`, **Console** öffnet `http://127.0.0.1:9001`. An der Konsole mit denselben Keys anmelden.

S3-Clients: path-style, Port 9000.

Server-Logs liegen neben dem Datenordner, nicht darin. Bei `D:\rustfs\data` also unter `D:\rustfs\logs`.

## Was im Fenster passiert

Links die Einstellungen, rechts die Logs.

**App Logs** kommt vom Launcher. **RustFS Output** vom Serverprozess. Bei Fehlern zuerst **RustFS Output** ansehen.

Der Update-Block kann auf GitHub nach einer neuen Launcher-Version schauen. Die beiden Beschriftungen sind noch auf Chinesisch (`版本与更新`, `检查更新`). Ein Update ersetzt die ganze App inklusive gebündeltem RustFS. Der Datenordner bleibt. Läuft RustFS, fragt die App vorher nach.

Das Fenster schließen beendet nichts. Die App liegt im Tray, RustFS läuft weiter. Rechtsklick auf das Tray-Icon → **Quit** hält den Server an und beendet. **Show** oder Linksklick holt das Fenster zurück. Ein zweiter Start fokussiert nur das schon laufende Fenster.

## macOS

Zip passend zum Chip laden, entpacken, `RustFS Launcher.app` nach Programme legen.

Die GitHub-Builds sind nicht notarisiert. Der erste Start kann scheitern:

```bash
xattr -cr "/Applications/RustFS Launcher.app"
open "/Applications/RustFS Launcher.app"
```

Oder Control-Klick auf die App, dann **Öffnen**.

## Wenn etwas hakt

**Installer blockiert.** Datei wirklich von diesem Repo? Unter Eigenschaften danach **Zulassen** / Unblock ist in Ordnung.

**Data path is required / does not exist.** Ordner zuerst anlegen, dann nochmal Browse.

**Port belegt.** Anderen Port nehmen oder was auf 9000 / 9001 beenden.

**Konsole geht nicht auf.** Console Endpoint muss an sein. Port ist 9001, nicht 7001.

**Detected Externally.** Auf dem API-Port lauscht schon etwas, aber nicht dieser Launcher. Prozess beenden oder Port wechseln. **Stop RustFS** gilt nur für Prozesse, die diese App gestartet hat.

**RustFS stirbt direkt nach Launch.** Beide Log-Reiter lesen. Meist: Ordner fehlt, keine Schreibrechte, Portkollision.

## Das ist ein lokaler Knoten

Launcher startet RustFS als Desktop-Prozess, nicht als Windows-Dienst. Zum Ausprobieren und Entwickeln reicht das. Produktionscluster gehören auf Linux: [docs.rustfs.com/en/installation](https://docs.rustfs.com/en/installation).

Längere Windows-Anleitung: [Install RustFS on Windows](https://docs.rustfs.com/en/installation/windows).

## Aus dem Quellcode bauen

Siehe [CONTRIBUTING.md](CONTRIBUTING.md).

## Lizenz

[Apache License 2.0](LICENSE).
