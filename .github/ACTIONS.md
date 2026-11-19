# GitHub Actions 自动编译说明

本项目配置了完整的 GitHub Actions CI/CD 流程，支持自动编译和发布。

## 工作流说明

### 1. CI 工作流 (`.github/workflows/ci.yml`)

**触发条件:**
- Push 到 `main` 分支
- 创建或更新 Pull Request

**功能:**
- 代码格式检查 (rustfmt)
- Clippy 代码质量检查
- 前端构建验证
- 单元测试运行

### 2. Upstream Sync 工作流 (`.github/workflows/upstream-sync.yml`)

**触发条件:**
- 每小时定时运行
- 手动触发（可选 `force_build`，版本未变也强制构建）

**功能:**
- 读取 `https://version.rustfs.com/latest.json`，与 `git describe --tags --abbrev=0` 比较（忽略 `v` 前缀）
- 发现新版本时按上游原样打 tag（例如 `1.0.1`）并推送
- 执行 `gh workflow run build.yml --ref <tag>` 触发构建（`GITHUB_TOKEN` 推送的 tag 不会自动触发其他工作流）

### 3. Build and Release 工作流 (`.github/workflows/build.yml`)

**触发条件:**
- 推送匹配 `*.*.*` 的 tag（例如 `1.0.1`）
- 发布 Release（当 Release 状态变为 published 时）
- 手动触发 (workflow_dispatch)，以及 Upstream Sync 在 tag 上的触发

**支持平台:**
- **macOS Apple Silicon** (aarch64)
- **macOS Intel** (x86_64)
- **Windows** (x86_64)

上游该版本没有某个平台的资源时，该平台会被跳过（例如 `1.0.1` 没有 macOS Intel 包）。

**产物类型:**
- macOS: `.dmg`、`.app.zip`，以及 Updater 用的 `.app.tar.gz` 和 `.sig`
- Windows: NSIS 安装程序 `-setup.exe` 和 `.sig`（不生成 `.msi`）
- 汇总: `latest.json`（Updater 清单）和 `SHA256SUMS`

**产物命名格式:**
- `rustfs-launcher-{platform}-{arch}-{version}.{ext}` (例如: `rustfs-launcher-macos-aarch64-1.0.1.dmg`)
- `rustfs-launcher-{platform}-{arch}-latest.{ext}` (最新版本链接)

**产物上传位置:**
- GitHub Release Assets
- Cloudflare R2: `s3://${R2_BUCKET}/artifacts/rustfs-launcher/release/`

## 使用方法

### 发布新版本

正常情况下无需手动操作：上游 `rustfs/rustfs` 发布新版本后，Upstream Sync 会自动打
tag 并触发构建、发布。Launcher 版本号跟随上游 RustFS 版本。

需要手动发布时：

1. **版本号**

   不需要修改 `src-tauri/Cargo.toml` 或 `src-tauri/tauri.conf.json`。构建时会把 tag
   （去掉可选的 `v` 前缀，必须符合 SemVer）写入这两个文件。

2. **发布方式（选择其一）**

   **方式一：推送 tag**
   ```bash
   git tag 1.0.1
   git push origin 1.0.1
   ```

   **方式二：通过 GitHub Web 界面或 CLI 发布 Release**
   ```bash
   gh release create 1.0.1 --title "1.0.1" --notes "Release notes"
   ```

   注意：只改 Launcher 时没有独立的版本号空间，tag 与上游版本不一致会被 Upstream Sync
   视为“有新版本”，详见 [docs/SELF_UPDATE.md](../docs/SELF_UPDATE.md#版本约定)。

3. **等待编译完成**

   在 Actions 页面查看进度：https://github.com/rustfs/launcher/actions

4. **发布完成**

    编译成功后会自动:

    - 将构建产物上传到 Release Assets
    - 将构建产物上传到 Cloudflare R2 (`s3://${R2_BUCKET}/artifacts/rustfs-launcher/release/`)

### 手动触发编译

1. 访问 GitHub Actions 页面
2. 选择 "Build and Release" 工作流
3. 点击 "Run workflow" 按钮，选择分支或 tag 并运行

可选输入:
- `tag`: 以发布模式构建该版本。在分支上运行时默认只演练，不上传
- `publish`: 配合 `tag` 使用，勾选后才上传到 Release 和 R2
- `rustfs_tag`: 指定内置的 RustFS 版本，默认上游最新版

在 tag 上运行（Use workflow from 选择 tag）会直接按正式发布处理并上传。
不填 `tag` 在分支上运行时为 `dev-<sha>` 开发构建，不生成 Updater 产物。

## 编译产物说明

以 `1.0.1` 为例，Release 中包含:

```
rustfs-launcher-macos-aarch64-1.0.1.dmg
rustfs-launcher-macos-aarch64-1.0.1.app.zip
rustfs-launcher-macos-aarch64-1.0.1.app.tar.gz(.sig)
rustfs-launcher-macos-aarch64-latest.{dmg,app.zip,app.tar.gz(.sig)}
rustfs-launcher-windows-x86_64-1.0.1-setup.exe(.sig)
rustfs-launcher-windows-x86_64-latest-setup.exe(.sig)
rustfs-launcher-windows-x86_64-latest.exe
latest.json
SHA256SUMS
```

上游提供 macOS Intel 资源时，还会有对应的 `rustfs-launcher-macos-x86_64-*` 文件。

## 常见问题

### 1. 编译失败怎么办?

- 检查 GitHub Actions 日志，查看具体错误信息
- 确保所有依赖都已正确配置
- 验证 RustFS 二进制文件下载链接是否有效

### 2. 如何修改编译目标?

编辑 `.github/workflows/build.yml` 中 `build` job 的 `matrix.include`。启动器只支持
macOS 和 Windows（`src-tauri/src/platform.rs` 会拒绝其他平台），新增平台需要同时修改
`platform.rs`、`build.sh`/`build.bat` 和 `scripts/select-rustfs-asset.sh`。

### 3. 如何添加代码签名?

在 GitHub Repository Settings 中添加以下 Secrets:

**macOS:**
- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD`

**Windows:**
- `WINDOWS_CERTIFICATE`
- `WINDOWS_CERTIFICATE_PASSWORD`

**Cloudflare R2:**
- `R2_ACCESS_KEY_ID`
- `R2_SECRET_ACCESS_KEY`
- `R2_ENDPOINT`
- `R2_BUCKET`

然后在 workflow 中启用对应的签名或上传步骤。

## 依赖项说明

### 自动下载的依赖:
- RustFS 二进制文件 (通过 `https://version.rustfs.com/latest.json` 解析版本，并从 GitHub Release Assets 下载)

### GitHub Actions 使用的组件:
- `dtolnay/rust-toolchain` - Rust 工具链
- `Swatinem/rust-cache` - Rust 缓存加速
- `actions/setup-node` - Node.js 环境
- `actions/upload-artifact` - 构建产物上传
- `softprops/action-gh-release` - 自动创建 Release

## 优化建议

1. **缓存优化**: 已配置 Rust 缓存，加速编译
2. **并行构建**: 三个平台同时编译，节省时间
3. **失败容错**: 单个平台失败不影响其他平台编译
4. **自动化发布**: 上游发版后由 Upstream Sync 自动打 tag 并发布，无需手动操作

## 本地测试工作流

### 快速开始 - Pre-commit 检查

**推荐: 在提交代码前运行所有检查**

```bash
make pre-commit
```

这个命令会自动运行所有 CI 检查项:
- ✅ 代码格式检查 (`cargo fmt`)
- ✅ Clippy 静态分析 (`cargo clippy`)
- ✅ 前端构建 (`trunk build`)
- ✅ 单元测试 (`cargo test`)

**单独运行检查:**
```bash
make check-fmt      # 仅检查格式
make check-clippy   # 仅运行 Clippy
make check-frontend # 仅构建前端
make check-test     # 仅运行测试
make fix-fmt        # 自动修复格式
```

详细使用说明请参考: [本地测试指南](TESTING.md)

### 使用 Makefile 和 act 工具

项目提供了 Makefile 来简化本地测试 GitHub Actions 工作流的流程。

**安装 act 工具:**
```bash
make install-act
```

**常用命令:**
```bash
# 查看所有可用命令
make help

# 本地运行 CI 工作流 (快速测试)
make test-ci

# 运行完整的 CI 检查
make test-ci-full

# 列出所有工作流任务
make list-jobs

# 仅检查代码格式化
make check-fmt

# 仅运行 clippy 检查
make check-clippy

# 清理 act 缓存
make clean
```

**注意事项:**
- 首次运行会下载 Docker 镜像,可能需要几分钟
- 需要确保 Docker Desktop 已安装并运行
- 本地测试使用 Linux 容器,行为可能与实际 CI 环境略有差异
- build 工作流包含平台特定步骤,本地测试可能无法完全模拟

### 手动使用 act

如果需要更细粒度的控制:

```bash
# 查看 CI 工作流的所有任务
act -W .github/workflows/ci.yml -l

# 运行特定任务
act push -W .github/workflows/ci.yml -j check

# 查看将要执行的命令 (dry run)
act push -W .github/workflows/ci.yml -n

# 使用详细输出
act push -W .github/workflows/ci.yml --verbose
```

## 维护注意事项

- 定期检查 GitHub Actions 的使用配额
- 保持依赖版本更新
- 监控 RustFS 二进制文件下载地址的可用性
- 及时处理编译失败的通知
- 推送前使用 `make test-ci` 本地验证工作流
