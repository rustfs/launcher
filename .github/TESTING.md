# Local GitHub Actions testing

This repository uses a Makefile and [act](https://github.com/nektos/act) to run GitHub Actions workflows locally.

## Quick start

### 1. Install dependencies

**macOS:**
```bash
# Install Docker Desktop if needed
# https://www.docker.com/products/docker-desktop

# Install act through the Makefile
make install-act
```

**Install act yourself:**
```bash
brew install act
```

### 2. Check the install

```bash
# Confirm Docker is running
docker ps

# Confirm the act version
act --version
```

## Usage

### Pre-commit check (recommended)

**Run this before you commit:**

```bash
make pre-commit
```

It runs the CI checks in order:
1. **Formatting** - `cargo fmt --all --check`
2. **Clippy** - `cargo clippy --all-targets --all-features -- -D warnings`
3. **Frontend build** - `trunk build`
4. **Unit tests** - `cargo test --all-features`

When they pass you will see:
```
==========================================
✅ All pre-commit checks passed!
==========================================
Your code is ready to commit and push.
```

### Run one check

```bash
# Formatting only
make check-fmt

# Clippy only
make check-clippy

# Frontend build only
make check-frontend

# Tests only
make check-test

# Rewrite formatting
make fix-fmt
```

### List targets

```bash
make help
```

Example output:
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

### Common test commands

#### CI workflow

```bash
# Quick run for day-to-day work
make test-ci

# Full run, including dependencies
make test-ci-full

# Verbose output
make test-ci-verbose
```

#### One check

```bash
# Formatting only
make test-fmt

# Clippy only
make test-clippy

# Local tests
make test-local
```

#### Workflow info

```bash
# List jobs
make list-jobs

# Preview the plan without running it
make dry-run-ci
```

#### Clear the cache

```bash
# Remove the act cache and temporary files
make clean
```

## Workflows

### CI (`ci.yml`)

Triggers:
- Push to `main`
- Pull request

Steps:
- Rust formatting (`cargo fmt`)
- Clippy (`cargo clippy`)
- Frontend build
- Unit tests

Runner: `sm-standard-2` (mapped to an Ubuntu image for local `act` runs).

Local command:
```bash
make test-ci
```

### Native CI (`ci-native.yml`)

Manual only (`workflow_dispatch`). It runs the same Windows and macOS native tests on `windows-latest` and `macos-latest`. Local `act` cannot reproduce those hosts.

### Build (`build.yml`)

Triggers:
- Manual dispatch only (`workflow_dispatch`)
- Tag pushes and Release events do not start it

Platforms:
- macOS (Apple Silicon) on `macos-latest`
- macOS (Intel) on `macos-15-intel`
- Windows (x86_64) on `windows-latest`

Linux helper jobs use `sm-standard-2`.

Local command:
```bash
make test-build
```

The build workflow has platform-specific steps. A local run cannot fully reproduce every platform.

## Configuration

### .actrc

`.actrc` in the repository root sets act defaults:
- `sm-standard-2` and `ubuntu-latest` use the `catthehacker/ubuntu:act-latest` image
- Container architecture: `linux/amd64`
- Containers are reused so later runs are faster

Create `.actrc.local` for a personal override (it is gitignored).

## FAQ

### 1. Docker daemon error

**Error:**
```
Cannot connect to the Docker daemon
```

**Fix:**
- Start Docker Desktop
- Run `docker ps` to confirm Docker is up

### 2. The first run is slow

**Cause:** the first run downloads a Docker image (about 1-2 GB).

**Fix:**
- Wait for the download
- Later runs are faster because the container is reused

### 3. Permission denied

**Error:**
```
Permission denied
```

**Fix:**
```bash
# Make the Makefile executable
chmod +x Makefile

# Or use sudo (not recommended)
sudo make test-ci
```

### 4. act is out of date

**Fix:**
```bash
# Upgrade act
brew upgrade act

# Or reinstall it
make install-act
```

### 5. Disk space

**Fix:**
```bash
# Clear the act cache
make clean

# Remove unused Docker images
docker system prune -a
```

## Practices

1. **Run this before every commit**
   ```bash
   make pre-commit
   ```
   This is the check that matches CI.

2. **Check while you work**
   ```bash
   # Formatting after an edit
   make check-fmt

   # Rewrite formatting
   make fix-fmt

   # Clippy
   make check-clippy
   ```

3. **After you edit a workflow**
   ```bash
   make dry-run-ci  # preview the plan
   make test-ci     # run it
   ```

4. **Clear the cache periodically**
   ```bash
   make clean
   ```

5. **Suggested flow**
   ```bash
   # 1. Edit
   vim src-tauri/src/main.rs

   # 2. Fix formatting
   make fix-fmt

   # 3. Run every check
   make pre-commit

   # 4. Commit when it passes
   git add .
   git commit -m "feat: add new feature"
   git push
   ```

## Advanced

### Run one job

```bash
make test-ci-job
# Enter a job name when prompted, for example: check
```

### Use another Docker image

Edit `.actrc.local`:
```
-P sm-standard-2=catthehacker/ubuntu:full-latest
```

### Pass environment variables

```bash
act push -W .github/workflows/ci.yml --env RUST_LOG=debug
```

### Debug a workflow

```bash
# Verbose Make target
make test-ci-verbose

# Or call act directly
act push -W .github/workflows/ci.yml --verbose
```

## References

- [act documentation](https://github.com/nektos/act)
- [GitHub Actions documentation](https://docs.github.com/en/actions)
- [Project Actions guide](.github/ACTIONS.md)

## Contributing

Open an issue or pull request if you have a suggestion.
