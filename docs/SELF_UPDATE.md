# 自升级发布说明

RustFS Launcher 支持两条独立的升级路径，均面向 **Windows 和 macOS**：

- **Launcher 整包升级**：Tauri 2 Updater，更新包同时包含 Launcher 和对应平台的 RustFS 二进制。
- **RustFS 二进制升级**：用户可在启动器内手动检查并安装上游 `rustfs/rustfs` 的新版本，无需重装启动器。

Linux 不是发布目标。启动器在 Linux 上会拒绝解析平台二进制，并提示仅支持 Windows 与 macOS。

## 版本约定

- Launcher 版本跟随上游 RustFS：`upstream-sync` 工作流发现上游新版本后，按上游
  `latest.json` 报告的原样打 tag（目前不带 `v`，例如 `1.0.1`），因此 Launcher
  `1.0.1` 默认内置 RustFS `1.0.1`。
- tag 去掉可选的 `v` 前缀后必须符合 SemVer，发布构建会把它写入 Tauri 和 Cargo
  包版本；仓库中的 `0.1.0` 只是占位值，不需要手动修改。
- 构建时解析到的上游 RustFS 版本通过 `RUSTFS_VERSION` 编译进 Launcher。
- 手动触发构建时可用 `rustfs_tag` 指定内置版本；未指定时使用 RustFS 最新版。
- 只修改 Launcher 本身时没有独立的版本号空间。`upstream-sync` 用
  `git describe --tags --abbrev=0` 与上游版本做字符串比较，最新 tag 与上游不一致
  就会触发构建，所以单独发版前要先确认不会被同步工作流反复触发。

## 更新签名

Updater 签名与 macOS/Windows 的系统代码签名相互独立，两者都需要配置。

首次配置 Updater 签名：

```bash
cargo tauri signer generate -w ~/.tauri/rustfs-launcher.key
```

将生成的公钥内容配置到 `src-tauri/tauri.conf.json` 的
`plugins.updater.pubkey`。私钥不得提交到仓库，应备份到安全位置，并配置以下
GitHub Actions Secrets：

- `TAURI_SIGNING_PRIVATE_KEY`：私钥文件内容
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`：私钥密码；无密码时留空

丢失私钥后，已经安装的客户端将无法验证后续版本，因此必须保留离线备份。

## 发布过程

正常情况下发布全自动完成：`upstream-sync` 每小时检查一次上游，发现新版本后推送
tag，再执行 `gh workflow run build.yml --ref <tag>`。之所以显式触发，是因为用
`GITHUB_TOKEN` 推送的 tag 不会触发其他工作流。从上游发版到 Launcher 安装包发布，
通常不超过两小时（最多等一小时检查，再加约 40 分钟构建）。

在 tag 上运行的构建（自动同步、手动推送 `*.*.*` tag、发布 GitHub Release）会：

1. 校验并注入 Launcher 版本。
2. 下载内置的 RustFS 二进制文件（默认上游最新版），并用 `SHA256SUMS` 校验。
3. 构建 macOS Apple Silicon、macOS Intel 和 Windows x86_64 安装包；上游该版本没有
   对应平台资源时跳过该平台（例如 `1.0.1` 没有 macOS Intel 包）。
4. 生成 Tauri Updater 产物及 `.sig` 签名。
5. 汇总已构建的平台并生成 `latest.json`。
6. 将安装包、签名和更新清单上传到同一个 GitHub Release，并同步到 Cloudflare R2。

在分支上手动运行并填写 `tag` 时默认只演练，勾选 `publish` 才会发布；不填 `tag`
则是 `dev-<sha>` 开发构建，不生成 Updater 产物。

客户端使用以下固定地址检查更新：

```text
https://github.com/rustfs/launcher/releases/latest/download/latest.json
```

必须在所有平台产物生成成功后再发布 `latest.json`，避免客户端获取到不完整
的版本。

## 用户侧行为

启动时会静默检查一次更新；用户也可随时点击“Check for Updates”。检查会同时请求：

- Launcher 更新清单（签名校验）
- 上游 `https://version.rustfs.com/latest.json`（RustFS 版本）

### Launcher 整包

- 如果由 Launcher 管理的 RustFS 正在运行，安装前会请求用户确认。
- 用户确认后 RustFS 被停止，更新包下载并通过签名校验，然后安装。
- 安装成功后 Launcher 自动重启；RustFS 不会自动恢复运行。

### RustFS 二进制

- 仅下载当前平台的上游 zip（macOS aarch64 / macOS x86_64 / Windows x86_64）。
- 检查更新时会读取同一 Release 的 `SHA256SUMS`。若最新版本没有该平台的资源，界面显示原因而不是“可更新”。
- 用同一 Release 中的 `SHA256SUMS` 校验压缩包。
- 解压后对二进制执行 `--version`；失败则中止，不覆盖现有文件。
- 安装到启动器数据目录下的 `binaries/`，并写入 `installed.json`。
- 若启动器随后通过整包升级内置了更新的 RustFS，内置版本优先。

用户的数据目录和保存在 WebView 本地存储中的 Launcher 配置不属于应用安装
包，正常覆盖升级不会主动删除它们。

## 发布前检查

```bash
cargo fmt --all -- --check
cargo check -p rustfs-launcher-ui --target wasm32-unknown-unknown
cargo clippy -p rustfs-launcher --all-targets --all-features -- -D warnings
cargo test -p rustfs-launcher --all-features
cargo test -p rustfs-launcher-ui
```

正式对外发布前，还应配置 Apple Developer ID 签名与公证，以及 Windows
Authenticode 代码签名。
