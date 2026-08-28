# RustFS Launcher

<p align="center">
  <a href="./README.md">English</a> |
  简体中文 |
  <a href="./README_DE.md">Deutsch</a> |
  <a href="./README_FR.md">Français</a> |
  <a href="./README_JA.md">日本語</a> |
  <a href="./README_KO.md">한국어</a> |
  <a href="./README_HI.md">हिन्दी</a>
</p>

RustFS Launcher 是一个桌面小程序，用来在自己电脑上跑 [RustFS](https://github.com/rustfs/rustfs)。选一个目录，点 Launch，本机就会起来一套 S3 兼容的 API。网页控制台是可选的。

安装包里已经带了 RustFS 本体，不用再单独下。

界面目前是英文。下面这些说明有多语言版本。

![启动前的 Launcher 窗口](docs/images/launcher-ready.png)

## 下载

安装包在 [Releases](https://github.com/rustfs/launcher/releases) 页面。

| 电脑 | 下这个文件 |
| --- | --- |
| Windows 10/11，64 位 | `rustfs-launcher-windows-x86_64-<version>-setup.exe` |
| macOS，Apple Silicon | `rustfs-launcher-macos-aarch64-<version>.app.zip` |
| macOS，Intel | `rustfs-launcher-macos-x86_64-<version>.app.zip` |

不要下 **Source code**。那是源码包，不是安装程序。

如果最新的 tag 下面只有源码 zip，往下翻，找到 Assets 里真正带 `setup.exe` 或 `.app.zip` 的那次发布。

ARM 版 Windows 走 x86_64 安装包，靠系统模拟运行。Linux 安装包目前还没有。

## Windows：下载、安装、启动

大多数人走这条。

### 1. 下安装包

1. 打开 [github.com/rustfs/launcher/releases](https://github.com/rustfs/launcher/releases)。
2. 选一次带安装文件的发布。
3. 在 **Assets** 里下载 `rustfs-launcher-windows-x86_64-…-setup.exe`。

### 2. 装上

双击这个 `.exe`。

Windows 可能会弹出 **Windows 已保护你的电脑**。安装包暂时没有 Authenticode 签名，SmartScreen 会多问一句。确认文件来自这个仓库的 Releases 之后，点 **更多信息**，再点 **仍要运行**。

按提示点完 NSIS 安装向导。开始菜单里会出现 **RustFS Launcher**。

### 3. 先建数据目录

Launcher 不会替你建这个目录。在资源管理器里新建一个空文件夹，比如 `D:\rustfs\data`。别把别的文件塞进去。

PowerShell：

```powershell
New-Item -ItemType Directory -Force -Path D:\rustfs\data
```

### 4. 填表单

从开始菜单打开 **RustFS Launcher**。

- **Data Path** — 点 **Browse**，或者把文件夹拖进窗口。这项必填，而且目录必须已经存在。
- **API Port** — 一般用 `9000`，端口被占了再换。
- **Host** — 保持 `127.0.0.1`，只让本机访问。
- **Console Endpoint** — 默认关着。要网页控制台就打开。端口常用 `9001`。API 和 Console 不能用同一个端口。
- **Access Key / Secret Key** — 预填的是 `rustfsadmin` / `rustfsadmin`。这台电脑还有别人用的话，改掉。

然后点 **Launch RustFS**。

### 5. 起来之后

状态会变成 **Service Online**，表单锁住，按钮变成 **Stop RustFS**。

![启动成功后的 Launcher](docs/images/launcher-running.png)

点 **API** 卡片打开 `http://127.0.0.1:9000`，点 **Console** 打开 `http://127.0.0.1:9001`。控制台用你刚才填的那对密钥登录。

S3 客户端用 path-style，连 9000 端口。

服务端日志不在数据目录里面，而在它旁边。数据路径是 `D:\rustfs\data` 的话，日志在 `D:\rustfs\logs`。

## 窗口里都有什么

左边是配置，右边是日志。

**App Logs** 是 Launcher 自己打的。**RustFS Output** 是服务进程的输出。启动失败先看 **RustFS Output**。

左边的更新区域可以从 GitHub 检查新版本。那两行字目前还是中文（`版本与更新`、`检查更新`）。更新会换掉整个应用，包括内置的 RustFS。数据目录不会动。如果 RustFS 正在跑，安装前会先问你。

关掉窗口并不是退出。程序躲到托盘，RustFS 继续跑。托盘图标右键 → **Quit** 才会停服务并退出。**Show**，或者左键点图标，窗口会回来。再开一次应用，只会把已经在跑的窗口拉到前台。

## macOS

按芯片下对应的 zip，解压后把 `RustFS Launcher.app` 拖进「应用程序」。

GitHub 构建没有公证，第一次打开可能被拦：

```bash
xattr -cr "/Applications/RustFS Launcher.app"
open "/Applications/RustFS Launcher.app"
```

或者按住 Control 点应用，选 **打开**。

## 出问题时

**安装包被拦截。** 确认是从这个仓库下的。文件属性里，确认来源后可以点 **解除锁定**。

**Data path is required / does not exist。** 先建好文件夹，再 Browse 一次。

**端口被占用。** 换一个端口，或者把占着 9000 / 9001 的程序停掉。

**控制台打不开。** 得打开 Console Endpoint。控制台端口是 9001，不是 7001。

**Detected Externally。** 这个 API 端口上已经有进程在听，但不是这个 Launcher 拉起来的。停掉那个进程，或者换端口。**Stop RustFS** 只能停它自己管的进程。

**一点 Launch 就退出。** 两个日志页签都看一下。常见原因：目录不存在、没写权限、端口冲突。

## 这是单机节点

Launcher 把 RustFS 当桌面进程跑，不是 Windows 服务。试用、开发够用。生产集群请看 Linux 安装：[docs.rustfs.com/zh/installation](https://docs.rustfs.com/zh/installation)。

更细的 Windows 步骤：[在 Windows 上安装 RustFS](https://docs.rustfs.com/zh/installation/windows)。

## 从源码编译

见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

[Apache License 2.0](LICENSE)。
