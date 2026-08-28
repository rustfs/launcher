# RustFS Launcher

[English](README.md) · [Deutsch](README_DE.md) · [Français](README_FR.md) · [日本語](README_JA.md) · [한국어](README_KO.md) · **简体中文** · [हिन्दी](README_HI.md)

RustFS Launcher 是一个小巧的桌面程序，用来在自己的电脑上运行 [RustFS](https://github.com/rustfs/rustfs)
这个兼容 S3 的对象存储服务。选一个文件夹，按一下按钮，你就有了一个位于 `http://127.0.0.1:9000` 的 S3 端点。

服务端本身已经打包在程序里，不需要再装别的东西，不用开终端，也没有配置文件要改。

![RustFS 正在运行时的启动器界面](docs/images/launcher-running.png)

## 它能帮你做什么

- 不依赖 Docker，在自己的笔记本上跑一套对象存储，用于开发、测试或作为个人备份目标。
- 把任意 S3 客户端或 SDK 指向 `127.0.0.1`，像用云上的 Bucket 一样使用。
- 随时了解服务状态：是否在线、占用了哪些端口、实时日志。
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

### 5. 填写表单

![配置面板](docs/images/launcher-config.png)

| 字段 | 含义 |
| --- | --- |
| **Data Path** | 第 4 步建好的文件夹。点 **Browse** 选择，或者把文件夹直接拖到窗口里。这是唯一必填项。 |
| **API Port** | S3 端点的端口。没有别的程序占用时，保持 `9000` 即可。 |
| **Host** | 填 `127.0.0.1` 表示只在本机可用；填 `0.0.0.0` 则同一网络里的其他设备也能访问。 |
| **Console Endpoint** | 想用 RustFS 的网页控制台就打开它，它使用单独的端口，默认 `9001`。 |
| **Access Key** / **Secret Key** | S3 客户端使用的凭证，默认是 `rustfsadmin` / `rustfsadmin`。一旦服务对外可访问，请务必修改。 |

填过的内容会被记住，下次启动点一下就行。

### 6. 点击 Launch

点 **Launch RustFS**。一两秒后，顶部会变成 **Service Online / Managed by Launcher**，表单随之锁定，
**App Logs** 里会列出启动器做了什么：用了哪个可执行文件、传了哪些参数、拿到的进程号是多少。

启动失败时，原因同样在这个日志面板里。最常见的两种情况是数据文件夹不存在，以及端口已被占用。

### 7. 开始使用

服务在线时，顶部的 **API** 和 **Console** 卡片可以点击。**Console** 打开网页界面；**API** 打开的是 S3 端点本身，
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

1. 解压 `.app.zip`，把 **RustFS Launcher** 拖进 **应用程序**。
2. 由于发布版本没有经过 Apple 公证，首次打开会被拦下。可以右键点击程序选择 **打开**，或在终端里去掉隔离属性：

   ```bash
   xattr -cr "/Applications/RustFS Launcher.app"
   open "/Applications/RustFS Launcher.app"
   ```

3. 之后的操作和第 4 到第 8 步完全一样。关掉窗口后，点一下 Dock 图标就能重新打开。

## 界面导览

![尚未启动时的界面](docs/images/launcher-ready.png)

**状态标签。** 第一个说的是服务本身：*Service Online* 表示配置的主机和端口上确实有程序在响应。第二个说的是
这个进程归谁管：

| 标签 | 含义 |
| --- | --- |
| Ready to Launch | 该端口上没有任何东西在运行。 |
| Managed by Launcher | RustFS 由启动器拉起，也可以由它停止。 |
| Detected Externally | 端口有响应，但进程不是这里启动的，比如你在终端里手动跑的 RustFS。这种情况下停止按钮不可用。 |

**概览卡片。** API 和 Console 显示端口号，服务在线后点击即可在浏览器中打开。Mode 表示表单当前是可编辑
（*Editable*）还是因服务运行而锁定（*Locked*）。

**Version & Updates。** 显示启动器版本和内置的 RustFS 版本，需要时可以检查是否有新版本。

**日志。** *App Logs* 是启动器自己的记录：找了哪些路径、启动了什么、为什么失败。*RustFS Output* 是服务端在
控制台上的输出，而较新的 RustFS 会把详细日志写进文件，所以这一页常常是空的。Auto-scroll 会自动跟随新行，
Clear 会清空两个页签。

## 文件都放在哪

| 内容 | 位置 |
| --- | --- |
| 你的对象数据 | 选定的数据文件夹，另外还有一个存放元数据的隐藏目录 `.rustfs.sys` |
| 服务端日志 | 数据文件夹同级的 `logs` 文件夹 |
| 启动器的设置 | 由程序自己保存，没有需要手动编辑的文件 |
| 程序本体 | Windows 在 `%LOCALAPPDATA%\RustFS Launcher`，macOS 在 `/Applications` |

## 升级

在 Version & Updates 卡片里点 **Check for Updates**。如果有新版本，启动器会下载、校验签名并安装。若 RustFS
正在运行，它会先征求你的同意，停掉服务后再重启自己。升级会整体替换程序，包括内置的 RustFS 服务端。

签名与发布的细节见 [docs/SELF_UPDATE.md](docs/SELF_UPDATE.md)。

## 遇到问题时

**“Data path does not exist”** —— 先在资源管理器或访达里把文件夹建好，启动器不会替你创建。

**“Port 9000 is already in use”** —— 端口被别的程序（或旧的 RustFS）占用了。换一个 API 端口，或者关掉那个
程序。控制台端口同理。

**标签显示 Detected Externally** —— 该主机和端口上已经有 RustFS 在响应，但不是这个启动器拉起来的。请到当初
启动它的地方停掉，或者给启动器换个端口。

**RustFS Output 一直是空的** —— 较新的 RustFS 就是这样，它把日志写进数据文件夹旁边的 `logs` 目录。想看启动器
自己的动作，请切到 App Logs。

**浏览器里出现 `AccessDenied` 之类的 XML** —— 这是 S3 API 在回应浏览器，并不是出了故障。想看网页界面就点
**Console** 卡片，而 API 地址请填到 S3 客户端里。

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
