# RustFS Launcher

[English](README.md) · [Deutsch](README_DE.md) · [Français](README_FR.md) · [日本語](README_JA.md) · [한국어](README_KO.md) · **简体中文** · [हिन्दी](README_HI.md)

RustFS Launcher 是一个小巧的桌面程序，用来在自己的电脑上运行 [RustFS](https://github.com/rustfs/rustfs)
这个兼容 S3 的对象存储服务。选一个文件夹，按一下按钮，你就有了一个位于 `http://127.0.0.1:9000` 的 S3 端点。

服务端本身已经打包在程序里，不需要再装别的东西，不用开终端，也没有配置文件要改。

![服务在线时 Data Path 仍在首屏，日志保持隐藏](docs/images/launcher-running.png)

## 它能帮你做什么

- 不依赖 Docker，在自己的笔记本上跑一套对象存储，用于开发、测试或作为个人备份目标。
- 把任意 S3 客户端或 SDK 指向 `127.0.0.1`，像用云上的 Bucket 一样使用。
- 随时了解服务状态：是否在线、占用了哪些端口。日志默认隐藏，只有你打开时才会出现。
- 想启动就启动，想停止就停止。服务运行期间，启动器会缩在托盘里继续待命。
- 在程序内部升级启动器（连同内置的 RustFS 一起）。

## 下载

到[发布页](https://github.com/rustfs/launcher/releases)挑选适合自己系统的文件。当前最新版本是 **1.0.6**。

| 你的系统 | 文件 |
| --- | --- |
| Windows 10 / 11（64 位） | `rustfs-launcher-windows-x86_64-<version>-setup.exe` |
| macOS（Apple 芯片） | `rustfs-launcher-macos-aarch64-<version>.dmg` 和 `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS（Intel，x86_64） | 1.0.6 未发布 |

1.0.6 的实际文件名是 `rustfs-launcher-windows-x86_64-1.0.6-setup.exe`、
`rustfs-launcher-macos-aarch64-1.0.6.dmg` 和 `rustfs-launcher-macos-aarch64-1.0.6.app.zip`。这一版没有
`rustfs-launcher-macos-x86_64-*`，Intel 芯片的 Mac 装不了这个版本。

ARM 版 Windows 也能用，它会以模拟方式运行 x86_64 版本。Linux 没有启动器安装包，请直接运行
[RustFS 二进制文件](https://github.com/rustfs/rustfs/releases)。

安装包比较大，是因为整个 RustFS 服务端都打包在里面。

同一发布页里文件名带 `-latest` 的，是上述带版本号安装包的逐字节副本。
`rustfs-launcher-windows-x86_64-latest-setup.exe` 和 `rustfs-launcher-windows-x86_64-latest.exe` 都是这个
Windows 安装包，只是用了固定文件名。macOS 的 `-latest.dmg` 和 `-latest.app.zip` 也是同样的副本。优先下载带版本号的
`-setup.exe` 或 `.dmg`。

### 核对下载文件

在同一个发布页的 **Assets** 里，和安装包一起下载 `SHA256SUMS`。每一行是 SHA-256 散列值和文件名。

在 Windows 上，用 PowerShell 进入安装包所在的文件夹：

```powershell
Get-FileHash .\rustfs-launcher-windows-x86_64-1.0.6-setup.exe -Algorithm SHA256
```

在 macOS 上，用终端：

```bash
shasum -a 256 rustfs-launcher-macos-aarch64-1.0.6.dmg
```

算出的散列值必须和 `SHA256SUMS` 里该文件名那一行一致，字母大小写无所谓。如果下的是 `.app.zip`，就按那个文件名同样核对。
以后安装更新的版本时，用该版本发布页上的文件名和 `SHA256SUMS`。

## Windows 完整流程

### 1. 下载安装包

打开[发布页](https://github.com/rustfs/launcher/releases)，展开最新版本下的 **Assets**，下载
`rustfs-launcher-windows-x86_64-<version>-setup.exe`（1.0.6 为
`rustfs-launcher-windows-x86_64-1.0.6-setup.exe`）。按上面的方法用 `SHA256SUMS` 核对。Edge 和 Chrome
有时会对不常见的安装包给出提示，出现时选择「保留」即可。

### 2. 通过 SmartScreen

双击安装包。如果出现 **“Windows 已保护你的电脑”**，点击 **更多信息**，再点 **仍要运行**。这个提示是因为安装包
还没有商业代码签名证书，并不代表文件本身有问题。

### 3. 走完安装向导

向导会依次询问：欢迎页、许可协议、安装位置、开始菜单文件夹。

- 默认安装到 `C:\Users\<你的用户名>\AppData\Local\RustFS Launcher`，只为当前用户安装，因此不会要求管理员权限。
- 如果电脑上没有 Microsoft WebView2 运行时，安装程序会自动下载一次，这一步需要联网。
- 最后一页保持勾选 **Run RustFS Launcher**，也可以顺便让它创建桌面快捷方式。

装好之后，可以在开始菜单里找到 **RustFS Launcher**。

### 4. 先建好数据文件夹

RustFS 会把对象保存在你指定的文件夹里，并且要求这个文件夹事先存在。请先在资源管理器里建一个，比如
`D:\RustFS\data`。选一个有足够剩余空间的磁盘上的空文件夹最稳妥。

服务端自己的日志会写到同级的 `logs` 文件夹里，也就是本例中的 `D:\RustFS\logs`。

### 5. 填写表单

![配置面板](docs/images/launcher-config.png)

| 字段 | 含义 |
| --- | --- |
| **Data Path** | 第 4 步建好的文件夹。点 **Browse** 选择，或者把文件夹直接拖到窗口里。这是唯一必填项。 |
| **API Port** | S3 端点的端口。没有别的程序占用时，保持 `9000` 即可。 |
| **Host** | 填 `127.0.0.1` 表示只在本机可用；填 `0.0.0.0` 则同一网络里的其他设备也能访问。 |
| **Console Endpoint** | 想用 RustFS 的网页控制台就打开它，它使用单独的端口，默认 `9001`。 |
| **Access Key** / **Secret Key** | S3 客户端使用的凭证，默认是 `rustfsadmin` / `rustfsadmin`，会和其他表单项一起被记住。主机不是本机回环地址时，启动器会拒绝启动，直到两项都改成非默认值；留空也会回落到这组公开凭证。 |

填过的内容会被记住，下次启动点一下就行。

### 6. 点击 Launch

窗口打开时就能看到 **Data Path** 和 **Launch RustFS**，不用滚动。日志一直隐藏，直到你点 **Show logs**。
**Hide logs** 会再把面板关上。启动时的检查和更新提示都不会自动展开日志。

点 **Launch RustFS**。一两秒后，状态会变成 **Service Online / Managed by Launcher**，表单随之锁定。

启动、停止或升级失败时，首屏会写出原因和改法。日志面板保持关闭。最常见的两种情况是数据文件夹不存在，以及端口已被占用。
RustFS 还会把更细的记录写到数据目录旁边的 `logs` 文件夹。只有想看逐行输出时，才点 **Show logs**。

### 7. 开始使用

服务在线时，启动按钮下方的 **API** 和 **Console** 卡片可以点击。**Console** 打开网页界面；**API** 打开的是 S3 端点本身，
用浏览器访问只会看到一段 XML —— 这个地址是给 S3 客户端用的。

把 S3 客户端指向这个端点：

```bash
aws --endpoint-url http://127.0.0.1:9000 s3 mb s3://demo
aws --endpoint-url http://127.0.0.1:9000 s3 cp report.pdf s3://demo/
```

凭证用你填的 Access Key 和 Secret Key，区域填 `us-east-1`，寻址方式选 path-style。

在控制台里，用同一组 Access Key 和 Secret Key 登录即可。

![RustFS 网页控制台中的存储桶列表](docs/images/rustfs-console.png)

### 8. 停止，或者让它继续跑

点 **Stop RustFS** 会停止服务，表单也随之解锁。

关闭窗口并不会退出程序：启动器会缩到通知区域，RustFS 继续提供服务。点击托盘图标可以把窗口叫回来，
右键选择 **Quit** 则会停止服务并退出程序。

### 9. 卸载

在 **设置 → 应用 → 已安装的应用** 里卸载 **RustFS Launcher**，或使用开始菜单文件夹里的卸载程序。
数据文件夹不会被动，不需要了请自行删除。

## macOS

1. 打开 `.dmg`，把 **RustFS Launcher** 拖进 **应用程序**。如果下的是 `.app.zip`，解压后同样拖进去。
2. 这些构建没有经过 Apple 公证，所以首次打开会被拦下。请右键点击程序，选择 **打开**，再确认。在终端里去掉隔离属性只是可选办法：

   ```bash
   xattr -cr "/Applications/RustFS Launcher.app"
   open "/Applications/RustFS Launcher.app"
   ```

3. 之后的操作和第 4 到第 8 步完全一样。关掉窗口后，点一下 Dock 图标就能重新打开。

## 界面导览

![尚未启动时的界面](docs/images/launcher-ready.png)

首屏是 **Data Path**、**Browse** 和 **Launch RustFS**，标题旁边是服务状态。日志在你打开之前不会出现。

### 语言

点击顶栏的语言按钮（**中** / **EN** / **FR** / **日**）即可切换界面语言。

第一次打开、尚未选择语言时，界面跟随操作系统语言（按主语言标签匹配，如 `zh-CN`、`zh-Hant-TW`、`fr-CA`、
`ja_JP`）：如果属于应用内置的 4 种语言之一就使用它，否则使用英语。

应用内置的 4 种语言是 简体中文（`zh-CN`）、English（`en`）、Français（`fr`）和 日本語（`ja`）。

选定之后立即生效并会被记住，之后操作系统语言不会再覆盖这个选择。

托盘菜单（**Show** 和 **Quit**）、Windows 安装向导、后端返回的错误信息和日志内容仍是英语。macOS 的下载是
磁盘映像，拖进“应用程序”即可，并不是已翻译的安装向导。语言按钮只改变启动器窗口里的语言。

**状态标签。** 第一个说的是服务本身：*Service Online* 表示配置的主机和端口上确实有程序在响应。第二个说的是
这个进程归谁管：

| 标签 | 含义 |
| --- | --- |
| Ready to Launch | 该端口上没有任何东西在运行。 |
| Managed by Launcher | RustFS 由启动器拉起，也可以由它停止。 |
| Detected Externally | 端口有响应，但进程不是这里启动的，比如你在终端里手动跑的 RustFS。表单仍可编辑，方便你改端口。停止按钮不可用，因为这个进程不归启动器管。 |

**概览卡片。** API 和 Console 显示端口号，服务在线后点击即可在浏览器中打开。Mode 只在启动器自己拉起
RustFS 时显示锁定（*Locked*）。检测到外部进程时，表单仍是可编辑（*Editable*）。

**Version & Updates。** 显示启动器版本和内置的 RustFS 版本，需要时可以检查是否有新版本。

**日志。** 启动时是隐藏的。点 **Show logs** 才会打开面板；启动检查和更新提示不会替你打开。*App Logs* 是启动器自己的记录：找了哪些路径、启动了什么、为什么失败。*RustFS Output* 是服务端在
控制台上的输出，而较新的 RustFS 会把详细日志写进文件，所以打开之后这一页常常是空的。Auto-scroll 会自动跟随新行，
Clear 会清空两个页签。

## 文件都放在哪

| 内容 | 位置 |
| --- | --- |
| 你的对象数据 | 选定的数据文件夹，另外还有一个存放元数据的隐藏目录 `.rustfs.sys` |
| 服务端日志 | 数据文件夹同级的 `logs` 文件夹 |
| 启动器的设置 | 由程序自己保存，没有需要手动编辑的文件 |
| 程序本体 | Windows 在 `%LOCALAPPDATA%\RustFS Launcher`，macOS 在 `/Applications` |
| 手动升级后的 RustFS 二进制 | Windows：`%APPDATA%\com.dandan.rustfs-launcher\binaries`；macOS：`~/Library/Application Support/com.dandan.rustfs-launcher/binaries` |

## 升级

Version & Updates 卡片负责 **两类互相独立的升级**，都通过 **Check for Updates** 触发（仅 Windows 和 macOS）：

1. **Launcher** — 带签名的整包升级，会替换启动器以及内置的 RustFS，然后重启程序。
2. **RustFS** — 只更新服务端二进制。启动器从上游发布源下载对应平台的压缩包，用 `SHA256SUMS` 校验，再做一次
   `--version` 冒烟测试，最后写入启动器数据目录。不必为了新的 RustFS 去重装启动器。

若 RustFS 正在运行，升级前会征求确认并先停掉服务。升级完成后不会自动再把它拉起来。

签名与发布的细节见 [docs/SELF_UPDATE.md](docs/SELF_UPDATE.md)。

## 遇到问题时

**“Data path does not exist”** —— 先在资源管理器或访达里把文件夹建好，启动器不会替你创建。

**“Port 9000 is already in use”** —— 端口被别的程序（或旧的 RustFS）占用了。换一个 API 端口，或者关掉那个
程序。控制台端口同理。

**标签显示 Detected Externally** —— 该主机和端口上已经有 RustFS 在响应，但不是这个启动器拉起来的。请到当初
启动它的地方停掉，或者给启动器换个端口。

**RustFS Output 一直是空的** —— 这是你打开日志之后常见的情况。较新的 RustFS 把日志写进数据文件夹旁边的 `logs` 目录。启动器
自己的动作在另一个页签 App Logs 里。

**浏览器里出现 `AccessDenied` 之类的 XML** —— 这是 S3 API 在回应浏览器，并不是出了故障。想看网页界面就点
**Console** 卡片，而 API 地址请填到 S3 客户端里。

**窗口不见了** —— 关闭窗口只是把它隐藏起来。Windows 上用通知区域的图标，macOS 上点 Dock 图标即可。

**macOS 提示“无法打开”** —— 右键点击程序，选择 **打开**，再确认。上面 macOS 部分里的 `xattr` 命令只是可选办法。

## 自己编译

需要 [Rust](https://rustup.rs/)、[Node.js](https://nodejs.org/) 和 [Trunk](https://trunkrs.dev/)
（`cargo install trunk`）。

```bash
./build.sh          # Windows 上用 build.bat，会下载对应平台的 RustFS 二进制文件
cargo tauri dev     # 带热重载运行
cargo tauri build   # 打包安装程序
```

提交 PR 前先跑一次 `make pre-commit`，它会执行格式检查、Clippy、前端构建和测试。更多约定见
[AGENTS.md](AGENTS.md)，发布流程见 [.github/ACTIONS.md](.github/ACTIONS.md)。编辑器推荐 VS Code，配合
[Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) 和
[rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) 插件。

## 许可证

Apache-2.0，详见 [LICENSE](LICENSE)。
