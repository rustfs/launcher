# RustFS Launcher

[English](README.md) · [Deutsch](README_DE.md) · [Français](README_FR.md) · [日本語](README_JA.md) · [한국어](README_KO.md) · **简体中文** · [हिन्दी](README_HI.md)

RustFS Launcher 是一个小巧的桌面程序，用来在自己的电脑上运行 [RustFS](https://github.com/rustfs/rustfs)
这个兼容 S3 的对象存储服务。选一个文件夹，按一下按钮，RustFS 就会在所有网卡的 `9000` 端口上监听。本机访问地址是 `http://127.0.0.1:9000`。同一网络里的其他设备，用这台电脑的地址和同一个端口就能连上。初始密钥是 `rustfsadmin` / `rustfsadmin`。

服务端本身已经打包在程序里，不需要再装别的东西，不用开终端，也没有配置文件要改。

![启动之后：监听地址、当前密钥，以及停止](docs/images/launcher-running.png)

## 它能帮你做什么

- 不依赖 Docker，在自己的笔记本上跑一套对象存储，用于开发、测试或作为个人备份目标。
- 把任意 S3 客户端或 SDK 指向这个端点，像用云上的 Bucket 一样使用。本机用 `127.0.0.1`。同一网络里的其他设备，用这台电脑的地址和端口 `9000`。
- 启动之后可以复制监听地址和当前密钥。日志默认隐藏，只有你打开时才会出现。
- 想启动就启动，想停止就停止。服务运行期间，启动器会缩在托盘里继续待命。
- 在程序内部升级启动器（连同内置的 RustFS 一起）。

## 下载

到[发布页](https://github.com/rustfs/launcher/releases)挑选适合自己系统的文件。

| 你的系统 | 文件 |
| --- | --- |
| Windows 10 / 11（64 位） | `rustfs-launcher-windows-x86_64-<version>-setup.exe`（约 50 MB） |
| macOS（Apple 芯片） | `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS（Intel） | `rustfs-launcher-macos-x86_64-<version>.app.zip` |

ARM 版 Windows 也能用，它会以模拟方式运行 x86_64 版本。Linux 目前还没有安装包，可以直接运行
[RustFS 二进制文件](https://github.com/rustfs/rustfs/releases)。

安装包比较大，是因为整个 RustFS 服务端都打包在里面。

## Windows 完整流程

### 1. 下载安装包

打开[发布页](https://github.com/rustfs/launcher/releases)，展开最新版本下的 **Assets**，点击以 `-setup.exe`
结尾的文件。Edge 和 Chrome 有时会对不常见的安装包给出提示，出现时选择「保留」即可。

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

### 5. 选择数据文件夹

![启动之前：数据路径和 Launch RustFS](docs/images/launcher-ready.png)

首屏只有数据路径和 **Launch RustFS**。**Launch RustFS** 一直显示。路径是空的时候，它以 40% 的不透明度呈灰色，不能点。点 **Browse**、自己输入路径，或把文件夹拖进窗口。

主机、端口、控制台和密钥都不在这一屏。标题栏末尾的齿轮打开 **Settings**，一个小面板，用来改主机、API 端口和控制台。它不是第二个启动按钮，也不是单独的一页。

![设置：主机、API 端口和控制台](docs/images/launcher-config.png)

新安装的默认值是：

| 设置 | 默认 |
| --- | --- |
| **Host** | `0.0.0.0`。同一网络里的其他设备可以访问。本机仍然打开 `127.0.0.1`。 |
| **API 端口** | `9000` |
| **Console** | 打开，端口 `9001` |
| **Access key** / **Secret key** | `rustfsadmin` / `rustfsadmin`。启动器不会另造一对密钥。 |

需要改主机、端口或控制台时，启动前在 **Settings** 里改。密钥在服务起来之后，用 **Change administrator** 改。那里的修改下次启动才生效。正在运行的服务仍使用启动时的密钥，直到你再次启动。

页脚是当前版本，然后是 **Upgrade**、**Website**、**Docs** 和 **GitHub**。**Upgrade** 打开更新面板，它不是主按钮。

填过的内容会被记住，下次启动点一下就行。

### 6. 点击 Launch RustFS

不用滚动就能看到路径和 **Launch RustFS**，960×760 的窗口和 720×640 的最小尺寸都是这样。日志一直隐藏，直到服务起来之后你点 **View logs**。**Hide logs** 或 Escape 会关上面板，并把焦点还给这个按钮。启动时的检查和更新提示都不会自动展开日志。

点 **Launch RustFS**。启动过程中按钮上的文字变成 **Launching…**。

启动、停止或升级失败时，屏幕会写出原因和改法。这段说明不会把路径和 **Launch RustFS**（服务起来之后则是 **Stop RustFS**）挤出视野，也不会打开日志面板。最常见的两种情况是数据文件夹不存在，以及端口已被占用。RustFS 还会把更细的记录写到数据目录旁边的 `logs` 文件夹。只有想看逐行输出时，才点 **View logs**。

### 7. 开始使用

启动成功后，路径框会离开这一屏。屏幕上是监听地址，以及服务正在使用的 Access Key 和 Secret Key。每一项都可以复制。**Copy** 是描边按钮。地址和密钥马上就能用，没有额外的确认步骤。

默认主机下，监听地址是 `http://0.0.0.0:9000`。本机把 S3 客户端指向 `http://127.0.0.1:9000`。同一网络里的其他设备，用这台电脑的地址和端口 `9000`。密钥在你改掉之前是 `rustfsadmin` 和 `rustfsadmin`。

**Open Console** 打开网页控制台。**View logs** 打开日志面板。**Change administrator** 用来为下次启动设置另一组 Access Key 和 Secret Key。这一屏上唯一的实心按钮是 **Stop RustFS**，用来停止服务。

把 S3 客户端指向这个端点：

```bash
aws --endpoint-url http://127.0.0.1:9000 s3 mb s3://demo
aws --endpoint-url http://127.0.0.1:9000 s3 cp report.pdf s3://demo/
```

凭证用屏幕上显示的 Access Key 和 Secret Key，区域填 `us-east-1`，寻址方式选 path-style。想换一对密钥，点 **Change administrator**，停止服务后再启动一次。

在控制台里，用同一组 Access Key 和 Secret Key 登录即可。

![RustFS 网页控制台中的存储桶列表](docs/images/rustfs-console.png)

### 8. 停止，或者让它继续跑

点 **Stop RustFS** 会停止服务，并回到路径和 **Launch RustFS**。

关闭窗口并不会退出程序：启动器会缩到通知区域，RustFS 继续提供服务。点击托盘图标可以把窗口叫回来，
右键选择 **Quit** 则会停止服务并退出程序。

### 9. 卸载

在 **设置 → 应用 → 已安装的应用** 里卸载 **RustFS Launcher**，或使用开始菜单文件夹里的卸载程序。
数据文件夹不会被动，不需要了请自行删除。

## macOS

1. 解压 `.app.zip`，把 **RustFS Launcher** 拖进 **应用程序**。
2. 由于发布版本没有经过 Apple 公证，首次打开会被拦下。可以右键点击程序选择 **打开**，或在终端里去掉隔离属性：

   ```bash
   xattr -cr "/Applications/RustFS Launcher.app"
   open "/Applications/RustFS Launcher.app"
   ```

3. 之后的操作和第 4 到第 8 步完全一样。关掉窗口后，点一下 Dock 图标就能重新打开。

## 界面导览

![启动之前：数据路径和 Launch RustFS](docs/images/launcher-ready.png)

启动之前，这一屏是数据路径、**Browse** 和 **Launch RustFS**，加上设置齿轮和页脚。主机、端口、控制台和密钥在 **Settings** 或 **Change administrator** 里。日志在你打开之前不会出现。

**启动之后。** 路径框离开这一屏。监听地址和当前密钥都在屏幕上，各自带 **Copy**。**Open Console**、**View logs** 和 **Change administrator** 在 **Stop RustFS** 上面。

**已经有程序在监听。** 如果主机和端口上有程序在响应，但不是这个启动器拉起来的，路径这一屏会留下来并说明原因。到 **Settings** 里改端口，或者停掉那个程序。不会出现 **Stop RustFS**，因为这个进程不归启动器管。

**升级。** 页脚显示当前版本。**Upgrade** 打开一个面板，里面有 **Check for Updates**，有新版本时还有启动器和 RustFS 的更新。它不是首屏上的主按钮。

**日志。** 一直隐藏，直到你点 **View logs**。启动检查和更新提示不会打开面板。**Hide logs** 或 Escape 会把它关上。*App Logs* 是启动器自己的记录：找了哪些路径、启动了什么、为什么失败。*RustFS Output* 是服务端在控制台上的输出，而较新的 RustFS 会把详细日志写进文件，所以打开之后这一页常常是空的。Auto-scroll 会自动跟随新行，Clear 会清空两个页签。

## 文件都放在哪

| 内容 | 位置 |
| --- | --- |
| 你的对象数据 | 选定的数据文件夹，另外还有一个存放元数据的隐藏目录 `.rustfs.sys` |
| 服务端日志 | 数据文件夹同级的 `logs` 文件夹 |
| 启动器的设置 | 由程序自己保存，没有需要手动编辑的文件 |
| 程序本体 | Windows 在 `%LOCALAPPDATA%\RustFS Launcher`，macOS 在 `/Applications` |
| 手动升级后的 RustFS 二进制 | Windows：`%APPDATA%\com.dandan.rustfs-launcher\binaries`；macOS：`~/Library/Application Support/com.dandan.rustfs-launcher/binaries` |

## 升级

页脚里的 **Upgrade** 负责 **两类互相独立的升级**，都通过 **Check for Updates** 触发（仅 Windows 和 macOS）：

1. **Launcher** — 带签名的整包升级，会替换启动器以及内置的 RustFS，然后重启程序。
2. **RustFS** — 只更新服务端二进制。启动器从上游发布源下载对应平台的压缩包，用 `SHA256SUMS` 校验，再做一次
   `--version` 冒烟测试，最后写入启动器数据目录。不必为了新的 RustFS 去重装启动器。

若 RustFS 正在运行，升级前会征求确认并先停掉服务。升级完成后不会自动再把它拉起来。

签名与发布的细节见 [docs/SELF_UPDATE.md](docs/SELF_UPDATE.md)。

## 遇到问题时

**“Data path does not exist”** —— 先在资源管理器或访达里把文件夹建好，启动器不会替你创建。

**“Port 9000 is already in use”** —— 端口被别的程序（或旧的 RustFS）占用了。在 **Settings** 里换一个 API 端口，或者关掉那个程序。控制台端口同理。

**屏幕说已经有程序在监听** —— 该主机和端口上已经有 RustFS 在响应，但不是这个启动器拉起来的。请到当初启动它的地方停掉，或者在 **Settings** 里换个端口。

**RustFS Output 一直是空的** —— 打开 **View logs** 之后常见的情况。较新的 RustFS 把日志写进数据文件夹旁边的 `logs` 目录。启动器自己的动作在另一个页签 App Logs 里。

**浏览器里出现 `AccessDenied` 之类的 XML** —— 这是 S3 API 在回应浏览器，并不是出了故障。想看网页界面就点 **Open Console**，把监听地址填到 S3 客户端里。

**窗口不见了** —— 关闭窗口只是把它隐藏起来。Windows 上用通知区域的图标，macOS 上点 Dock 图标即可。

**macOS 提示“无法打开”** —— 参见上面 macOS 部分的隔离属性命令。

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
