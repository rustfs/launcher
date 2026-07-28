# 自升级发布说明

RustFS Launcher 使用 Tauri 2 Updater 进行整包升级。更新包同时包含 Launcher
和对应平台的 RustFS 二进制文件。

## 版本约定

- Git tag 是 Launcher 版本，必须符合 SemVer，例如 `v0.2.0`。
- 发布构建会去除 tag 的 `v` 前缀并写入 Tauri 和 Cargo 包版本。
- 构建时解析到的上游 RustFS 版本通过 `RUSTFS_VERSION` 编译进 Launcher。
- 手动触发构建时可用 `rustfs_tag` 指定内置版本；未指定时使用 RustFS 最新版。

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

推送版本 tag 后，构建工作流会：

1. 校验并注入 Launcher 版本。
2. 下载该发布对应的 RustFS 二进制文件。
3. 构建 macOS Apple Silicon、macOS Intel 和 Windows x86_64 安装包。
4. 生成 Tauri Updater 产物及 `.sig` 签名。
5. 汇总全部平台并生成 `latest.json`。
6. 将安装包、签名和更新清单上传到同一个 GitHub Release。

客户端使用以下固定地址检查更新：

```text
https://github.com/rustfs/launcher/releases/latest/download/latest.json
```

必须在所有平台产物生成成功后再发布 `latest.json`，避免客户端获取到不完整
的版本。

## 用户侧行为

- 用户点击“检查更新”后，Launcher 请求并验证更新清单。
- 如果由 Launcher 管理的 RustFS 正在运行，安装前会请求用户确认。
- 用户确认后 RustFS 被停止，更新包下载并通过签名校验，然后安装。
- 安装成功后 Launcher 自动重启；RustFS 不会自动恢复运行。

用户的数据目录和保存在 WebView 本地存储中的 Launcher 配置不属于应用安装
包，正常覆盖升级不会主动删除它们。

## 发布前检查

```bash
cargo fmt --all -- --check
cargo check -p rustfs-launcher-ui --target wasm32-unknown-unknown
cargo clippy -p rustfs-launcher --all-targets --all-features -- -D warnings
cargo test -p rustfs-launcher --all-features
```

正式对外发布前，还应配置 Apple Developer ID 签名与公证，以及 Windows
Authenticode 代码签名。
