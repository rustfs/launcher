# GitHub Actions build guide

Automatic jobs in this repository run on the organization's self-hosted runner (`sm-standard-2`). macOS and Windows jobs have no self-hosted runners. They use GitHub-hosted runners only when someone starts the workflow by hand.

## Workflows

### 1. CI (`.github/workflows/ci.yml`)

**Triggers:**
- Push to `main`
- Pull request opened or updated

**Runner:** `sm-standard-2`

**What it checks:**
- Rust formatting (rustfmt)
- Clippy
- Frontend build
- Unit tests

Windows and macOS native tests live in `.github/workflows/ci-native.yml`. The steps are unchanged. That workflow responds only to `workflow_dispatch` and uses `windows-latest` and `macos-latest`.

### 2. Build and Release (`.github/workflows/build.yml`)

**Triggers:**
- Manual dispatch only (`workflow_dispatch`)
- Pushing a tag or publishing a Release does **not** start this workflow

**Runners:**
- `build-check` and `upload-release-assets`: `sm-standard-2`
- macOS Apple Silicon: `macos-latest` (GitHub-hosted; billed only on manual dispatch)
- macOS Intel: `macos-15-intel` (GitHub-hosted; billed only on manual dispatch)
- Windows: `windows-latest` (GitHub-hosted; billed only on manual dispatch)

**Platforms:**
- **macOS Apple Silicon** (aarch64)
- **macOS Intel** (x86_64)
- **Windows** (x86_64)

**Artifacts:**
- macOS: `.app.zip` archive
- Windows: `.msi` package and `.exe` installer

**Names:**
- `rustfs-launcher-{platform}-{arch}-{version}.{ext}` (for example `rustfs-launcher-macos-aarch64-v0.1.0.dmg`)
- `rustfs-launcher-{platform}-{arch}-latest.{ext}` (latest alias)

**Upload destinations:**
- GitHub Release assets
- Cloudflare R2: `s3://${R2_BUCKET}/artifacts/rustfs-launcher/release/`

## How to publish

### Release a new version

1. **Bump the version**

   Edit the version in:
   ```
   src-tauri/Cargo.toml
   src-tauri/tauri.conf.json
   ```

2. **Commit the change**
   ```bash
   git add .
   git commit -m "chore: bump version to v0.1.0"
   git push origin main
   ```

3. **Build and publish manually**

   The upstream sync workflow creates the matching git tag once a day. It does not compile desktop installers.
   Open Actions, choose "Build and Release", or use the GitHub CLI. This uses billed macOS and Windows runners:

   ```bash
   gh workflow run build.yml --ref v0.1.0 -f tag=v0.1.0 -f publish=true
   ```

   `publish` defaults to false. In that mode the workflow uploads a workflow artifact only. It does not publish a Release or upload to R2.

4. **Wait for the build**

   Watch the manual run on the Actions page:
   ```
   https://github.com/YOUR_USERNAME/YOUR_REPO/actions
   ```

5. **After a successful publish**

    When `publish` is true, a successful build:

    - Uploads artifacts to the Release
    - Uploads artifacts to Cloudflare R2 (`s3://${R2_BUCKET}/artifacts/rustfs-launcher/release/`)

### Dispatch a build without publishing

1. Open the Actions page
2. Select the "Build and Release" workflow
3. Click "Run workflow"
4. Choose the branch and run it

## Build artifacts

A finished build produces:

```
rustfs-launcher-macos-aarch64/
  ├── rustfs-launcher-macos-aarch64-v0.1.0.app.zip
  └── rustfs-launcher-macos-aarch64-latest.zip

rustfs-launcher-macos-x86_64/
  ├── rustfs-launcher-macos-x86_64-v0.1.0.app.zip
  └── rustfs-launcher-macos-x86_64-latest.zip

rustfs-launcher-windows-x86_64/
  ├── rustfs-launcher-windows-x86_64-v0.1.0.msi
  ├── rustfs-launcher-windows-x86_64-v0.1.0-setup.exe
  ├── rustfs-launcher-windows-x86_64-latest.msi
  └── rustfs-launcher-windows-x86_64-latest-setup.exe
```

## FAQ

### 1. The build failed

- Read the GitHub Actions log for the error
- Confirm dependencies are configured
- Confirm the RustFS binary download URL is valid

### 2. How do I change build targets?

Edit the `matrix` in `.github/workflows/build.yml`:

```yaml
strategy:
  matrix:
    include:
      - platform: 'sm-standard-2'  # self-hosted Linux; do not use ubuntu-latest
        target: 'x86_64-unknown-linux-gnu'
        # ...
```

### 3. How do I add code signing?

Add these secrets in the repository settings:

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

Then enable the matching signing or upload step in the workflow.

## Dependencies

### Downloaded automatically:
- The RustFS binary (version resolved from `https://version.rustfs.com/latest.json`, file downloaded from GitHub Release assets)

### Actions used:
- `dtolnay/rust-toolchain` - Rust toolchain
- `Swatinem/rust-cache` - Rust build cache
- `actions/setup-node` - Node.js
- `actions/upload-artifact` - artifact upload
- `softprops/action-gh-release` - GitHub Release upload

## Notes

1. **Caching**: Rust and Node.js caches are already configured.
2. **Parallel builds**: the three desktop platforms build at the same time.
3. **Failure isolation**: one platform failing does not cancel the others.
4. **Manual desktop publish**: tag and Release events no longer start GitHub-hosted runners.

## Test workflows locally

### Pre-commit checks

**Run every check before you commit:**

```bash
make pre-commit
```

That runs:
- Formatting (`cargo fmt`)
- Clippy (`cargo clippy`)
- Frontend build (`trunk build`)
- Unit tests (`cargo test`)

**Run one check:**
```bash
make check-fmt      # formatting only
make check-clippy   # Clippy only
make check-frontend # frontend build only
make check-test     # tests only
make fix-fmt        # rewrite formatting
```

See the [local testing guide](TESTING.md).

### Makefile and act

The Makefile wraps local GitHub Actions runs through `act`.

**Install act:**
```bash
make install-act
```

**Common commands:**
```bash
# List targets
make help

# Run the CI workflow locally (quick)
make test-ci

# Run the full CI checks
make test-ci-full

# List workflow jobs
make list-jobs

# Formatting only
make test-fmt

# Clippy only
make test-clippy

# Clear the act cache
make clean
```

**Notes:**
- The first run downloads a Docker image and can take a few minutes
- Docker Desktop must be installed and running
- Local runs use a Linux container, so they can differ slightly from CI
- The build workflow has platform-specific steps that a local run cannot fully reproduce

### Call act directly

```bash
# List CI jobs
act -W .github/workflows/ci.yml -l

# Run one job
act push -W .github/workflows/ci.yml -j check

# Dry run
act push -W .github/workflows/ci.yml -n

# Verbose output
act push -W .github/workflows/ci.yml --verbose
```

## Maintenance

- Automatic jobs no longer spend GitHub-hosted minutes; manual desktop builds still do
- Keep dependency versions current
- Watch that the RustFS binary download URL stays valid
- Fix failed builds when they are reported
- Run `make test-ci` before pushing workflow changes
