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
- 推送以 `v` 开头的 tag (例如: `v0.1.0`, `v1.0.0`)
- 手动触发 (workflow_dispatch)

**支持平台:**
- **macOS Apple Silicon** (aarch64)
- **macOS Intel** (x86_64)
- **Windows** (x86_64)

**产物类型:**
- macOS: `.dmg` 安装包和 `.app.zip` 压缩包
- Windows: `.msi` 安装包和 `.exe` 安装程序

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

3. **创建并推送 tag**
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

4. **等待编译完成**

   访问 GitHub Actions 页面查看编译进度:
   ```
   https://github.com/YOUR_USERNAME/YOUR_REPO/actions
   ```

5. **发布完成**

   编译成功后会自动创建 GitHub Release，包含所有平台的安装包。

### 手动触发编译

1. 访问 GitHub Actions 页面
2. 选择 "Build and Release" 工作流
3. 点击 "Run workflow" 按钮
4. 选择分支并运行

## 编译产物说明

编译完成后，会生成以下文件:

```
RustFS-Launcher-macOS-aarch64/
  ├── RustFS Launcher.dmg
  └── RustFS-Launcher-macOS-aarch64.app.zip

RustFS-Launcher-macOS-x86_64/
  ├── RustFS Launcher.dmg
  └── RustFS-Launcher-macOS-x86_64.app.zip

RustFS-Launcher-Windows-x86_64/
  ├── RustFS Launcher_x.x.x_x64.msi
  └── RustFS Launcher_x.x.x_x64-setup.exe
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

## 维护注意事项

- 定期检查 GitHub Actions 的使用配额
- 保持依赖版本更新
- 监控 RustFS 二进制文件下载地址的可用性
- 及时处理编译失败的通知
