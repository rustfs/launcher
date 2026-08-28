# RustFS Launcher

<p align="center">
  <a href="./README.md">English</a> |
  <a href="./README_ZH.md">简体中文</a> |
  <a href="./README_DE.md">Deutsch</a> |
  Français |
  <a href="./README_JA.md">日本語</a> |
  <a href="./README_KO.md">한국어</a> |
  <a href="./README_HI.md">हिन्दी</a>
</p>

RustFS Launcher est une petite appli bureau pour faire tourner [RustFS](https://github.com/rustfs/rustfs) sur votre machine. Vous choisissez un dossier, vous cliquez sur Launch, et une API compatible S3 démarre en local. La console web est optionnelle.

Le binaire RustFS est déjà dans l’installateur. Pas besoin de le télécharger à part.

L’interface est en anglais. Ces pages existent en plusieurs langues.

![Fenêtre du launcher avant le démarrage](docs/images/launcher-ready.png)

## Téléchargement

Les builds sont sur la page [Releases](https://github.com/rustfs/launcher/releases).

| Machine | Fichier |
| --- | --- |
| Windows 10/11, 64 bits | `rustfs-launcher-windows-x86_64-<version>-setup.exe` |
| macOS, Apple Silicon | `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS, Intel | `rustfs-launcher-macos-x86_64-<version>.app.zip` |

Ne prenez pas **Source code**. C’est l’arbre Git, pas l’appli.

Si le tag le plus récent n’a que des zip de sources, descendez jusqu’à une release qui contient vraiment un `setup.exe` ou un `.app.zip` dans Assets.

Sur Windows ARM, c’est l’installateur x86_64, via l’émulation. Il n’y a pas encore de paquet Linux.

## Windows : télécharger, installer, démarrer

C’est le chemin le plus courant.

### 1. Récupérer l’installateur

1. Ouvrez [github.com/rustfs/launcher/releases](https://github.com/rustfs/launcher/releases).
2. Choisissez une release qui contient des installateurs.
3. Dans **Assets**, téléchargez `rustfs-launcher-windows-x86_64-…-setup.exe`.

### 2. L’installer

Double-cliquez sur le `.exe`.

Windows peut afficher **Windows a protégé votre PC**. L’installateur n’est pas encore signé Authenticode, SmartScreen râle. Si le fichier vient bien de la page Releases de ce dépôt : **Informations complémentaires**, puis **Exécuter quand même**.

Suivez l’assistant NSIS. **RustFS Launcher** apparaît ensuite dans le menu Démarrer.

### 3. Créer le dossier de données d’abord

Le launcher ne le crée pas. Dans l’Explorateur, faites un dossier vide, par exemple `D:\rustfs\data`. N’y mettez pas d’autres fichiers.

En PowerShell :

```powershell
New-Item -ItemType Directory -Force -Path D:\rustfs\data
```

### 4. Remplir le formulaire

Ouvrez **RustFS Launcher** depuis le menu Démarrer.

- **Data Path** — **Browse**, ou glissez le dossier dans la fenêtre. Champ obligatoire, et le dossier doit déjà exister.
- **API Port** — `9000`, sauf si le port est déjà pris.
- **Host** — laissez `127.0.0.1` pour n’accepter que cette machine.
- **Console Endpoint** — désactivé par défaut. Activez-le pour la console web. Port habituel : `9001`. API et console ne peuvent pas partager le même port.
- **Access Key / Secret Key** — prérempli avec `rustfsadmin` / `rustfsadmin`. Changez-les si d’autres personnes utilisent cet ordinateur.

Puis **Launch RustFS**.

### 5. Une fois démarré

Le pastille passe à **Service Online**, le formulaire se verrouille, le bouton devient **Stop RustFS**.

![Launcher après un démarrage réussi](docs/images/launcher-running.png)

La carte **API** ouvre `http://127.0.0.1:9000`, **Console** ouvre `http://127.0.0.1:9001`. Connectez-vous à la console avec les mêmes clés.

Les clients S3 doivent utiliser le path-style, port 9000.

Les journaux serveur sont à côté du dossier de données, pas dedans. Pour `D:\rustfs\data`, c’est `D:\rustfs\logs`.

## Autour de la fenêtre

À gauche, la config. À droite, les logs.

**App Logs** vient du launcher. **RustFS Output** vient du processus serveur. En cas d’échec, commencez par **RustFS Output**.

Le bloc de mise à jour peut interroger GitHub pour une nouvelle version. Les deux libellés sont encore en chinois (`版本与更新`, `检查更新`). Une mise à jour remplace toute l’appli, y compris RustFS. Le dossier de données n’est pas touché. Si RustFS tourne, l’appli demande avant de l’arrêter.

Fermer la fenêtre ne quitte pas. L’appli se range dans la barre d’état, RustFS continue. Clic droit sur l’icône → **Quit** pour arrêter le serveur et sortir. **Show**, ou un clic gauche, ramène la fenêtre. Relancer l’appli ne fait que remettre au premier plan celle qui tourne déjà.

## macOS

Téléchargez le zip correspondant à votre puce, décompressez, mettez `RustFS Launcher.app` dans Applications.

Les builds GitHub ne sont pas notariés. Le premier lancement peut échouer :

```bash
xattr -cr "/Applications/RustFS Launcher.app"
open "/Applications/RustFS Launcher.app"
```

Ou Control-clic sur l’appli, puis **Ouvrir**.

## Si ça coince

**Installateur bloqué.** Vérifiez qu’il vient de ce dépôt. Dans les propriétés du fichier, **Débloquer** est OK ensuite.

**Data path is required / does not exist.** Créez le dossier, puis Browse à nouveau.

**Port déjà utilisé.** Changez de port, ou arrêtez ce qui occupe 9000 / 9001.

**La console ne s’ouvre pas.** Console Endpoint doit être activé. Le port est 9001, pas 7001.

**Detected Externally.** Quelque chose écoute déjà sur ce port API, mais ce n’est pas ce launcher. Arrêtez ce processus, ou changez de port. **Stop RustFS** ne concerne que le processus démarré par cette appli.

**RustFS s’arrête juste après Launch.** Lisez les deux onglets de logs. Causes fréquentes : dossier manquant, pas d’écriture, conflit de port.

## C’est un nœud local

Le launcher lance RustFS comme processus bureau, pas comme service Windows. Ça suffit pour essayer et pour développer. Les clusters de prod vont sur Linux : [docs.rustfs.com/en/installation](https://docs.rustfs.com/en/installation).

Guide Windows plus long : [Install RustFS on Windows](https://docs.rustfs.com/en/installation/windows).

## Compiler depuis les sources

Voir [CONTRIBUTING.md](CONTRIBUTING.md).

## Licence

[Apache License 2.0](LICENSE).
