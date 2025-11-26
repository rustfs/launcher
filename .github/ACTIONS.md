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

### 2. Build and Release 工作流 (`.github/workflows/build.yml`)

**触发条件:**
- 推送 tag（例如: `0.1.0`, `v1.0.0`）
- 发布 Release（当 Release 状态变为 published 时）
- 手动触发 (workflow_dispatch)

**支持平台:**
- **macOS Apple Silicon** (aarch64)
- **macOS Intel** (x86_64)
- **Windows** (x86_64)

**产物类型:**
- macOS: `.dmg` 安装包和 `.app.zip` 压缩包
- Windows: `.msi` 安装包和 `.exe` 安装程序

**产物命名格式:**
- `rustfs-launcher-{platform}-{arch}-{version}.{ext}` (例如: `rustfs-launcher-macos-aarch64-v0.1.0.dmg`)
- `rustfs-launcher-{platform}-{arch}-latest.{ext}` (最新版本链接)

**产物上传位置:**
- GitHub Release Assets
- 阿里云 OSS: `oss://rustfs-artifacts/artifacts/rustfs-launcher/release/`

## 使用方法

### 发布新版本

1. **更新版本号**

   编辑以下文件中的版本号:
   ```
   src-tauri/Cargo.toml
   src-tauri/tauri.conf.json
   ```

2. **提交更改**
   ```bash
   git add .
   git commit -m "chore: bump version to v0.1.0"
   git push origin main
   ```

3. **发布方式（选择其一）**

   **方式一：推送 tag（自动创建 Release）**
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

   **方式二：通过 GitHub Web 界面创建 Release**
   - 访问仓库的 "Releases" 页面
   - 点击 "Draft a new release"
   - 创建新的 tag（如 `v0.1.0`）或选择已有 tag
   - 填写 Release 标题和描述
   - 点击 "Publish release"

   **方式三：通过 GitHub CLI**
   ```bash
   gh release create v0.1.0 --title "v0.1.0" --notes "Release notes"
   ```

4. **等待编译完成**

   当 tag 推送或 Release 发布（published）后，GitHub Actions 会自动触发编译。
   访问 GitHub Actions 页面查看编译进度:
   ```
   https://github.com/YOUR_USERNAME/YOUR_REPO/actions
   ```

5. **发布完成**

   编译成功后会自动:
   - 将构建产物上传到 Release Assets
   - 将构建产物上传到阿里云 OSS (`oss://rustfs-artifacts/artifacts/rustfs-launcher/release/`)

### 手动触发编译

1. 访问 GitHub Actions 页面
2. 选择 "Build and Release" 工作流
3. 点击 "Run workflow" 按钮
4. 选择分支并运行

## 编译产物说明

编译完成后，会生成以下文件:

```
rustfs-launcher-macos-aarch64/
  ├── rustfs-launcher-macos-aarch64-v0.1.0.dmg
  ├── rustfs-launcher-macos-aarch64-v0.1.0.app.zip
  ├── rustfs-launcher-macos-aarch64-latest.dmg
  └── rustfs-launcher-macos-aarch64-latest.app.zip

rustfs-launcher-macos-x86_64/
  ├── rustfs-launcher-macos-x86_64-v0.1.0.dmg
  ├── rustfs-launcher-macos-x86_64-v0.1.0.app.zip
  ├── rustfs-launcher-macos-x86_64-latest.dmg
  └── rustfs-launcher-macos-x86_64-latest.app.zip

rustfs-launcher-windows-x86_64/
  ├── rustfs-launcher-windows-x86_64-v0.1.0.msi
  ├── rustfs-launcher-windows-x86_64-v0.1.0-setup.exe
  ├── rustfs-launcher-windows-x86_64-latest.msi
  └── rustfs-launcher-windows-x86_64-latest-setup.exe
```

## 常见问题

### 1. 编译失败怎么办?

- 检查 GitHub Actions 日志，查看具体错误信息
- 确保所有依赖都已正确配置
- 验证 RustFS 二进制文件下载链接是否有效

### 2. 如何修改编译目标?

编辑 `.github/workflows/build.yml` 文件中的 `matrix` 配置:

```yaml
strategy:
  matrix:
    include:
      - platform: 'ubuntu-latest'  # 添加 Linux 支持
        target: 'x86_64-unknown-linux-gnu'
        # ...
```

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

然后在 workflow 中添加签名步骤。

## 依赖项说明

### 自动下载的依赖:
- RustFS 二进制文件 (从 https://dl.rustfs.com 下载)

### GitHub Actions 使用的组件:
- `dtolnay/rust-toolchain` - Rust 工具链
- `Swatinem/rust-cache` - Rust 缓存加速
- `actions/setup-node` - Node.js 环境
- `actions/upload-artifact` - 构建产物上传
- `softprops/action-gh-release` - 自动创建 Release

## 优化建议

1. **缓存优化**: 已配置 Rust 和 Node.js 缓存，加速编译
2. **并行构建**: 三个平台同时编译，节省时间
3. **失败容错**: 单个平台失败不影响其他平台编译
4. **自动化发布**: Tag 推送后自动发布，无需手动操作

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

# 仅测试代码格式化
make test-fmt

# 仅测试 clippy 检查
make test-clippy

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
