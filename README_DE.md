# RustFS Launcher

[English](README.md) · **Deutsch** · [Français](README_FR.md) · [日本語](README_JA.md) · [한국어](README_KO.md) · [简体中文](README_ZH.md) · [हिन्दी](README_HI.md)

RustFS Launcher ist eine kleine Desktop-Anwendung, die [RustFS](https://github.com/rustfs/rustfs) – einen
S3-kompatiblen Objektspeicher – auf dem eigenen Rechner startet. Sie wählen einen Ordner aus, drücken einen
Knopf, und RustFS lauscht auf allen Netzwerkschnittstellen an Port `9000`. Auf diesem Rechner ist der Endpunkt
`http://127.0.0.1:9000`. Andere Geräte im selben Netz erreichen ihn über die Adresse dieses Rechners und denselben
Port. Die Schlüssel beginnen mit `rustfsadmin` / `rustfsadmin`.

Der Server steckt bereits in der Anwendung. Es gibt nichts weiter zu installieren, kein Terminal zu öffnen und
keine Konfigurationsdatei zu bearbeiten.

![Nach dem Start: die lauschende Adresse, die aktuellen Schlüssel und Stopp](docs/images/launcher-running.png)

## Wofür Sie ihn benutzen können

- Objektspeicher auf dem eigenen Laptop betreiben – für Entwicklung, Tests oder als privates Backup-Ziel, ganz
  ohne Docker.
- Einen beliebigen S3-Client oder ein SDK auf den Endpunkt richten und wie einen Bucket in der Cloud benutzen.
  Auf diesem Rechner ist das `127.0.0.1`. Ein anderes Gerät im selben Netz nimmt die Adresse dieses Rechners und
  Port `9000`.
- Nach dem Start die lauschende Adresse und die aktuellen Schlüssel kopieren. Die Protokolle bleiben verborgen,
  bis Sie sie öffnen.
- RustFS jederzeit starten und stoppen. Der Launcher bleibt im Infobereich, während der Server arbeitet.
- Den Launcher – und das darin enthaltene RustFS – aus der Anwendung heraus aktualisieren.

## Herunterladen

Die passende Datei finden Sie auf der [Release-Seite](https://github.com/rustfs/launcher/releases).

| System | Datei |
| --- | --- |
| Windows 10 / 11, 64 Bit | `rustfs-launcher-windows-x86_64-<version>-setup.exe` (ca. 50 MB) |
| macOS, Apple Silicon | `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS, Intel | `rustfs-launcher-macos-x86_64-<version>.app.zip` |

Windows on ARM funktioniert ebenfalls – dort läuft der x86_64-Build per Emulation. Ein Linux-Paket gibt es noch
nicht; unter Linux starten Sie die [RustFS-Binärdatei](https://github.com/rustfs/rustfs/releases) direkt.

Die Downloads sind groß, weil ein vollständiger RustFS-Server mit im Installationsprogramm steckt.

## Windows, Schritt für Schritt

### 1. Installationsprogramm herunterladen

Öffnen Sie die [Release-Seite](https://github.com/rustfs/launcher/releases), klappen Sie beim neuesten Release
**Assets** auf und laden Sie die Datei herunter, die auf `-setup.exe` endet. Edge und Chrome warnen manchmal bei
selten heruntergeladenen Installationsprogrammen; wählen Sie dann **Beibehalten**.

### 2. An SmartScreen vorbei

Doppelklicken Sie die Datei. Erscheint **Der Computer wurde durch Windows geschützt**, klicken Sie auf
**Weitere Informationen** und dann auf **Trotzdem ausführen**. Die Warnung kommt daher, dass das
Installationsprogramm noch nicht mit einem kommerziellen Zertifikat signiert ist – sie ist kein Urteil über die
Datei selbst.

### 3. Durch den Assistenten klicken

Der Assistent stellt die üblichen Fragen: Willkommen, Lizenz, Installationsort, Startmenü-Ordner.

- Voreingestellt ist `C:\Users\<Name>\AppData\Local\RustFS Launcher`. Die Installation erfolgt nur für Ihr
  Benutzerkonto, deshalb fragt Windows nicht nach Administratorrechten.
- Fehlt auf dem PC die Microsoft-WebView2-Laufzeit, lädt das Installationsprogramm sie einmalig nach. Dafür wird
  eine Internetverbindung gebraucht.
- Lassen Sie auf der letzten Seite **Run RustFS Launcher** angehakt. Eine Desktop-Verknüpfung können Sie dort
  ebenfalls anlegen lassen.

Danach liegt die Anwendung im Startmenü unter **RustFS Launcher**.

### 4. Einen Ordner für die Daten anlegen

RustFS legt Objekte in einem Ordner Ihrer Wahl ab und erwartet, dass es diesen Ordner bereits gibt. Legen Sie
also zuerst im Explorer etwas wie `D:\RustFS\data` an. Am sichersten ist ein leerer Ordner auf einem Laufwerk
mit freiem Platz.

Seine eigenen Protokolldateien schreibt der Server in einen `logs`-Ordner neben dem gewählten Ordner – in
diesem Beispiel also `D:\RustFS\logs`.

### 5. Den Datenordner wählen

![Vor dem Start: Datenpfad und Launch RustFS](docs/images/launcher-ready.png)

Der erste Bildschirm ist der Datenpfad und **Launch RustFS**. **Launch RustFS** ist immer sichtbar. Solange der
Pfad leer ist, bleibt die Schaltfläche auf 40 % Deckkraft und lässt sich nicht drücken. **Browse** wählen, den
Pfad eintippen oder einen Ordner auf das Fenster ziehen.

Host, Port, Konsole und Schlüssel stehen nicht auf diesem Bildschirm. Das Zahnrad am Ende der Titelleiste öffnet
**Settings**, ein kleines quadratisches Feld für Host, API-Port und Konsole. Es ist keine zweite Startschaltfläche
und keine eigene Seite.

![Einstellungen: Host, API-Port und Konsole](docs/images/launcher-config.png)

Eine neue Installation verwendet diese Vorgaben:

| Einstellung | Vorgabe |
| --- | --- |
| **Host** | `0.0.0.0`. Andere Geräte im Netz erreichen den Endpunkt. Dieser Rechner öffnet weiterhin `127.0.0.1`. |
| **API-Port** | `9000` |
| **Console** | Ein, Port `9001` |
| **Access key** / **Secret key** | `rustfsadmin` / `rustfsadmin`. Der Launcher erfindet kein eigenes Paar. |

Host, Ports oder Konsole ändern Sie vor dem Start in **Settings**. Die Schlüssel ändern Sie, nachdem der Server
läuft, mit **Change administrator**. Diese Änderung gilt beim nächsten Start. Der laufende Server behält die
Schlüssel, mit denen er gestartet wurde, bis dahin.

Die Fußzeile zeigt die aktuelle Version, danach **Upgrade**, **Website**, **Docs** und **GitHub**. **Upgrade**
öffnet das Update-Feld. Es ist nicht die Hauptschaltfläche.

Die Eingaben werden gemerkt, der nächste Start ist also ein einziger Klick.

### 6. Launch RustFS drücken

Zum Pfad und zu **Launch RustFS** müssen Sie nicht scrollen, auch nicht in einem Fenster von 960×760 und bei
der Mindestgröße 720×640. Die Protokolle bleiben verborgen, bis Sie nach dem Start **View logs** drücken.
**Hide logs** oder die Escape-Taste schließt den Bereich und setzt den Fokus auf diese Schaltfläche zurück.
Eine Startprüfung oder eine Update-Meldung öffnet ihn nicht.

Klicken Sie auf **Launch RustFS**. Während des Starts steht auf der Schaltfläche **Launching…**.

Schlägt Start, Stopp oder ein Update fehl, nennt der Bildschirm den Grund und wie Sie ihn beheben. Der Hinweis
schiebt den Pfad und **Launch RustFS** (oder **Stop RustFS**, sobald der Server läuft) nicht aus dem Blick, und
er öffnet den Protokollbereich nicht. Am häufigsten fehlt der Datenordner oder der Port ist schon belegt.
RustFS schreibt außerdem Details in den Ordner `logs` neben dem Datenordner. **View logs** zeigt die Zeilen nur,
wenn Sie sie wollen.

### 7. Den Speicher benutzen

Nach einem erfolgreichen Start verschwindet das Pfadfeld. Ist die Konsole eingeschaltet, öffnet der Launcher
sie einmal im Standardbrowser. Spätere Statusprüfungen öffnen sie nicht erneut. Der Bildschirm zeigt die
lauschende Adresse sowie den Access Key und den Secret Key, die der Server verwendet. **Open in browser** neben
der Adresse öffnet dieselbe Konsole. Es kopiert die Adresse nicht und öffnet nicht den S3-Port. Access Key und
Secret Key haben jeweils **Copy**. Das sind Schaltflächen mit Rahmen. Adresse und Schlüssel können Sie sofort
verwenden. Es gibt keinen zusätzlichen Bestätigungsschritt.

Mit dem vorgegebenen Host lautet die lauschende Adresse `http://0.0.0.0:9000`. Auf diesem Rechner richten Sie
einen S3-Client auf `http://127.0.0.1:9000`. Von einem anderen Gerät im selben Netz aus nehmen Sie die Adresse
dieses Rechners und Port `9000`. Die Schlüssel bleiben `rustfsadmin` und `rustfsadmin`, bis Sie sie ändern.

**Open Console** öffnet die Weboberfläche. **View logs** öffnet die Protokolle. **Change administrator** setzt
einen anderen Access Key und Secret Key für den nächsten Start. **Stop RustFS** ist auf diesem Bildschirm die
einzige gefüllte Schaltfläche und stoppt den Server.

Richten Sie einen S3-Client auf den Endpunkt:

```bash
aws --endpoint-url http://127.0.0.1:9000 s3 mb s3://demo
aws --endpoint-url http://127.0.0.1:9000 s3 cp bericht.pdf s3://demo/
```

Als Zugangsdaten dienen der Access Key und der Secret Key auf dem Bildschirm, als Region `us-east-1`, und die
Adressierung erfolgt im Path-Style. Für ein anderes Paar **Change administrator** drücken, den Server stoppen
und erneut starten.

In der Konsole melden Sie sich mit demselben Access Key und Secret Key an.

![Buckets in der RustFS-Weboberfläche](docs/images/rustfs-console.png)

### 8. Stoppen oder weiterlaufen lassen

**Stop RustFS** fährt den Server herunter und bringt den Pfad und **Launch RustFS** zurück.

Das Schließen des Fensters beendet nichts: Der Launcher versteckt sich im Infobereich, RustFS läuft weiter. Ein
Klick auf das Symbol holt das Fenster zurück; über die rechte Maustaste und **Quit** stoppen Sie den Server und
beenden die Anwendung.

### 9. Deinstallieren

Entfernen Sie **RustFS Launcher** über **Einstellungen → Apps → Installierte Apps** oder über den Eintrag im
Startmenü-Ordner. Ihr Datenordner bleibt unangetastet; löschen Sie ihn selbst, wenn Sie die Objekte darin nicht
mehr brauchen.

## macOS

1. Entpacken Sie die `.app.zip` und ziehen Sie **RustFS Launcher** nach **Programme**.
2. Der erste Start wird blockiert, weil die Release-Builds nicht von Apple notarisiert sind. Klicken Sie die App
   mit der rechten Maustaste an und wählen Sie **Öffnen**, oder entfernen Sie das Quarantäne-Attribut im
   Terminal:

   ```bash
   xattr -cr "/Applications/RustFS Launcher.app"
   open "/Applications/RustFS Launcher.app"
   ```

3. Danach gilt alles aus den Schritten 4 bis 8. Ein Klick auf das Dock-Symbol holt das Fenster zurück, nachdem
   Sie es geschlossen haben.

## Ein Rundgang durch das Fenster

![Vor dem Start: Datenpfad und Launch RustFS](docs/images/launcher-ready.png)

Vor dem Start zeigt der Bildschirm den Datenpfad, **Browse** und **Launch RustFS**, dazu das Zahnrad und die
Fußzeile. Host, Port, Konsole und Schlüssel bleiben in **Settings** oder **Change administrator**. Die
Protokolle sind nicht zu sehen, bis Sie sie öffnen.

**Nach dem Start.** Das Pfadfeld verschwindet. Die lauschende Adresse bleibt auf dem Bildschirm, daneben **Open
in browser**. Access Key und Secret Key haben jeweils **Copy**. **Open Console**, **View logs** und **Change
administrator** liegen über **Stop RustFS**. **Open Console** öffnet die Konsole erneut.

**Es antwortet schon etwas.** Antwortet ein Programm auf Host und Port, das dieser Launcher nicht gestartet hat,
bleibt der Pfadbildschirm und sagt das. Ändern Sie den Port in **Settings** oder beenden Sie das Programm.
**Stop RustFS** wird nicht angeboten, weil dieser Launcher den Prozess nicht besitzt.

**Upgrade.** Die Fußzeile zeigt die aktuelle Version. **Upgrade** öffnet ein Feld mit **Check for Updates** und,
falls vorhanden, den Updates für Launcher und RustFS. Es ist keine Schaltfläche auf dem ersten Bildschirm.

**Protokolle.** Verborgen, bis Sie **View logs** drücken. Startprüfungen und Update-Meldungen öffnen den Bereich
nicht. **Hide logs** oder Escape schließt ihn. *App Logs* ist der Launcher selbst: wonach er gesucht hat, was er
gestartet hat, warum etwas schiefging. *RustFS Output* zeigt, was der Server auf der Konsole ausgibt – neuere
RustFS-Builds schreiben ihre ausführlichen Protokolle in Dateien, deshalb bleibt dieser Reiter nach dem Öffnen
oft leer. Auto-scroll folgt neuen Zeilen, Clear leert beide Reiter.

## Wo was landet

| Was | Wo |
| --- | --- |
| Ihre Objekte | Im gewählten Datenordner, dazu ein verstecktes `.rustfs.sys` für Metadaten |
| Server-Protokolle | In einem `logs`-Ordner neben dem Datenordner |
| Launcher-Einstellungen | Speichert die Anwendung selbst; es gibt keine Datei zum Bearbeiten |
| Die Anwendung | `%LOCALAPPDATA%\RustFS Launcher` unter Windows, `/Programme` unter macOS |

## Aktualisieren

**Upgrade** in der Fußzeile öffnet **Check for Updates**. Gibt es ein neueres Release, lädt der
Launcher es herunter, prüft die Signatur und installiert es. Läuft gerade RustFS, fragt der Launcher nach und
stoppt den Server, bevor er sich selbst neu startet. Ein Update ersetzt die komplette Anwendung samt
mitgeliefertem RustFS-Server.

Signatur- und Release-Details stehen in [docs/SELF_UPDATE.md](docs/SELF_UPDATE.md).

## Wenn etwas nicht klappt

**„Data path does not exist“** – legen Sie den Ordner zuerst im Explorer oder Finder an; der Launcher erstellt
ihn nicht.

**„Port 9000 is already in use“** – ein anderes Programm (oder ein älteres RustFS) belegt den Port. Wählen Sie
in **Settings** einen anderen API-Port oder beenden Sie das andere Programm. Für den Konsolen-Port gilt dasselbe.

**Der Bildschirm sagt, dass schon etwas lauscht** – auf diesem Host und Port antwortet bereits ein RustFS, das
dieser Launcher nicht gestartet hat. Beenden Sie diesen Prozess dort, wo Sie ihn gestartet haben, oder ändern
Sie den Port in **Settings**.

**RustFS Output bleibt leer** – nachdem Sie **View logs** geöffnet haben, ist dieser Reiter oft still. Neuere
RustFS-Builds schreiben in den `logs`-Ordner neben Ihrem Datenordner. Was der Launcher selbst tut, steht im
anderen Reiter, App Logs.

**Der Browser zeigt XML wie `AccessDenied`** – das ist die S3-API, die einem Browser antwortet; es ist nichts
kaputt. Für eine Weboberfläche nehmen Sie **Open Console**, die lauschende Adresse gehört in einen S3-Client.

**Das Fenster ist verschwunden** – Schließen versteckt es nur. Benutzen Sie das Symbol im Infobereich unter
Windows oder das Dock-Symbol unter macOS.

**macOS meldet, die App „kann nicht geöffnet werden“** – siehe den Quarantäne-Befehl im macOS-Abschnitt.

## Selbst bauen

Sie brauchen [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/) und [Trunk](https://trunkrs.dev/)
(`cargo install trunk`).

```bash
./build.sh          # unter Windows build.bat – lädt die passende RustFS-Binärdatei
cargo tauri dev     # mit Hot Reload starten
cargo tauri build   # Installationspakete erzeugen
```

Führen Sie vor einem Pull Request `make pre-commit` aus; das prüft Formatierung, Clippy, den Frontend-Build und
die Tests. Mehr Details stehen in [AGENTS.md](AGENTS.md), die Release-Workflows in
[.github/ACTIONS.md](.github/ACTIONS.md). Zum Entwickeln eignet sich VS Code mit den Erweiterungen
[Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) und
[rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Lizenz

Apache-2.0, siehe [LICENSE](LICENSE).
