# GitHub Actions 本地测试指南

本项目使用 Makefile 和 [act](https://github.com/nektos/act) 工具来实现 GitHub Actions 工作流的本地测试。

## 快速开始

### 1. 安装依赖

**macOS:**
```bash
# 安装 Docker Desktop (如未安装)
# 下载地址: https://www.docker.com/products/docker-desktop

# 使用 Makefile 自动安装 act
make install-act
```

**手动安装 act:**
```bash
brew install act
```

### 2. 验证安装

```bash
# 检查 Docker 是否运行
docker ps

# 检查 act 版本
act --version
```

## 使用方法

### 提交前检查 (推荐)

**最重要的命令 - 在提交代码前运行:**

```bash
make pre-commit
```

这个命令会依次运行所有 CI 检查项,确保你的代码能通过 GitHub Actions CI:
1. **代码格式检查** - `cargo fmt --all --check`
2. **Clippy 代码质量检查** - `cargo clippy --all-targets --all-features -- -D warnings`
3. **前端构建** - `trunk build`
4. **单元测试** - `cargo test --all-features`

如果所有检查都通过,你会看到:
```
==========================================
✅ All pre-commit checks passed!
==========================================
Your code is ready to commit and push.
```

### 单独运行检查

如果你只想运行特定的检查:

```bash
# 仅检查代码格式
make check-fmt

# 仅运行 Clippy
make check-clippy

# 仅构建前端
make check-frontend

# 仅运行测试
make check-test

# 自动修复代码格式
make fix-fmt
```

### 查看所有可用命令

```bash
make help
```

输出示例:
```
RustFS Launcher - GitHub Actions Local Testing

Available targets:
  make help          - Show this help message
  make install-act   - Install act tool for local GitHub Actions testing
  make test-ci       - Run CI workflow locally (quick, Ubuntu only)
  make test-ci-full  - Run CI workflow with full checks
  make test-build    - Test build workflow locally (single platform)
  make list-jobs     - List all available jobs in workflows
  make clean         - Clean act cache and temporary files
```

### 常用测试命令

#### 测试 CI 工作流

```bash
# 快速测试 (推荐用于日常开发)
make test-ci

# 完整测试 (包含所有依赖)
make test-ci-full

# 详细输出模式
make test-ci-verbose
```

#### 测试特定检查

```bash
# 仅测试代码格式化
make test-fmt

# 仅测试 clippy 检查
make test-clippy

# 运行本地测试
make test-local
```

#### 查看工作流信息

```bash
# 列出所有任务
make list-jobs

# 预览执行计划 (不实际运行)
make dry-run-ci
```

#### 清理缓存

```bash
# 清理 act 缓存和临时文件
make clean
```

## 工作流说明

### CI 工作流 (ci.yml)

触发条件:
- Push 到 main 分支
- Pull Request

包含步骤:
- Rust 代码格式检查 (`cargo fmt`)
- Clippy 静态分析 (`cargo clippy`)
- 前端构建验证
- 单元测试运行

本地测试命令:
```bash
make test-ci
```

### Build 工作流 (build.yml)

触发条件:
- 推送 tag (v*)
- 手动触发

支持平台:
- macOS (Apple Silicon)
- macOS (Intel)
- Windows (x86_64)

本地测试命令:
```bash
make test-build
```

⚠️ **注意:** build 工作流包含平台特定步骤,本地测试无法完全模拟所有平台的构建过程。

## 配置文件

### .actrc

项目根目录的 `.actrc` 文件配置了 act 的默认行为:
- 使用 `catthehacker/ubuntu:act-latest` 镜像
- 容器架构: `linux/amd64`
- 启用容器重用以加快后续运行速度

如需自定义配置,可创建 `.actrc.local` 文件(已在 .gitignore 中)。

## 常见问题

### 1. Docker daemon 错误

**错误信息:**
```
Cannot connect to the Docker daemon
```

**解决方法:**
- 确保 Docker Desktop 已启动
- 运行 `docker ps` 验证 Docker 是否正常工作

### 2. 首次运行很慢

**原因:** 第一次运行需要下载 Docker 镜像(约 1-2GB)

**解决方法:**
- 耐心等待下载完成
- 后续运行会快得多(容器会被重用)

### 3. 权限错误

**错误信息:**
```
Permission denied
```

**解决方法:**
```bash
# 确保 Makefile 可执行
chmod +x Makefile

# 或使用 sudo (不推荐)
sudo make test-ci
```

### 4. act 版本过旧

**解决方法:**
```bash
# 更新 act
brew upgrade act

# 或重新安装
make install-act
```

### 5. 磁盘空间不足

**解决方法:**
```bash
# 清理 act 缓存
make clean

# 清理 Docker 镜像
docker system prune -a
```

## 最佳实践

1. **提交代码前务必运行**
   ```bash
   make pre-commit
   ```
   这是最重要的步骤! 确保你的代码能通过所有 CI 检查。

2. **开发过程中频繁检查**
   ```bash
   # 写完代码后快速验证格式
   make check-fmt

   # 修复格式问题
   make fix-fmt

   # 验证代码质量
   make check-clippy
   ```

3. **修改工作流后验证**
   ```bash
   make dry-run-ci  # 预览执行计划
   make test-ci     # 实际运行测试
   ```

4. **定期清理缓存**
   ```bash
   make clean
   ```

5. **推荐的工作流程**
   ```bash
   # 1. 编写代码
   vim src-tauri/src/main.rs

   # 2. 自动修复格式
   make fix-fmt

   # 3. 运行所有检查
   make pre-commit

   # 4. 如果通过,提交代码
   git add .
   git commit -m "feat: add new feature"
   git push
   ```

## 高级用法

### 运行特定任务

```bash
make test-ci-job
# 然后按提示输入任务名称,例如: check
```

### 使用不同的 Docker 镜像

编辑 `.actrc.local`:
```
-P sm-standard-2=catthehacker/ubuntu:full-latest
```

### 传递环境变量

```bash
act push -W .github/workflows/ci.yml --env RUST_LOG=debug
```

### 调试工作流

```bash
# 使用详细输出
make test-ci-verbose

# 或直接使用 act
act push -W .github/workflows/ci.yml --verbose
```

## 参考资料

- [act 官方文档](https://github.com/nektos/act)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [项目 Actions 使用说明](.github/ACTIONS.md)

## 贡献

如有改进建议,欢迎提交 Issue 或 Pull Request。
