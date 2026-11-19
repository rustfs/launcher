# Repository Guidelines

Use this guide when contributing to RustFS Launcher; it highlights the project layout, tooling, and expectations.

## Project Structure & Module Organization
- `src/`: Leptos client entrypoint (`main.rs`), UI composition (`app.rs`), form/log/toast components (`components/`), DTOs mirrored from the backend (`types.rs`), pure UI logic (`helpers.rs`), and contextual styles (`logs.css`).
- `src-tauri/src/`: Tauri backend modules: `commands.rs` for invokable actions, `process.rs` for RustFS binary orchestration, `state.rs` for shared application state, `rustfs_update.rs` and `binaries.rs` for the in-app RustFS update and installed-binary bookkeeping, `platform.rs`/`version.rs`/`network.rs`/`config.rs` helpers, plus error types. Only macOS and Windows are supported targets.
- `src-tauri/binaries/`: Platform-specific RustFS executables fetched by the build scripts; keep this directory untracked in Git.
- `public/`: Static assets bundled by Trunk (`leptos.svg`, `tauri.svg`).
- `Trunk.toml` and `tauri.conf.json`: runtime configuration for the web client and desktop shell.

## Build, Test, and Development Commands
- `./build.sh` (macOS) or `build.bat` (Windows): download the matching RustFS binary into `src-tauri/binaries/`.
- `cargo tauri dev`: launch the desktop shell with hot reload across the Rust backend and Leptos UI.
- `trunk serve --port 1421`: run the Leptos client in a browser-only workflow using the `Trunk.toml` settings.
- `cargo tauri build`: produce distributable desktop bundles.
- `cargo fmt --all` and `cargo clippy --workspace --all-targets`: enforce formatting and linting before pushing.
- `./scripts/check-upstream-version.sh`: check for new versions from upstream rustfs/rustfs repository.

## Automated Build & Release
Launcher releases track upstream rustfs/rustfs: by default the launcher version is the bundled RustFS version (tag `1.0.1` ships RustFS `1.0.1`). See `docs/SELF_UPDATE.md` for signing and updater details.

### Upstream Version Sync (`.github/workflows/upstream-sync.yml`)
- **Scheduled Check**: Runs hourly against `https://version.rustfs.com/latest.json`.
- **Version Tracking**: Compares the upstream tag with `git describe --tags --abbrev=0`; a leading `v` is ignored on both sides.
- **Automatic Trigger**: On a new version it pushes a tag named exactly as upstream reports it (currently without a `v`, e.g. `1.0.1`), then runs `gh workflow run build.yml --ref <tag>`. The explicit dispatch is required because tags pushed with `GITHUB_TOKEN` do not trigger other workflows.
- **Manual Trigger**: `workflow_dispatch` with an optional `force_build` input rebuilds even when versions match.

### Build Workflow (`.github/workflows/build.yml`)
- **Triggers**: tag push matching `*.*.*`, a published GitHub Release, or `workflow_dispatch`.
- **Publishing**: Any run on a tag ref (including the dispatch from upstream sync) publishes to GitHub Releases and Cloudflare R2. A dispatch from a branch with the `tag` input is a dry run unless `publish` is set; without `tag` it is a `dev-<sha>` build with updater artifacts disabled.
- **Version**: The tag (minus any `v`) must be SemVer; CI writes it into `tauri.conf.json` and `src-tauri/Cargo.toml`, so the `0.1.0` in the repo is a placeholder. The bundled RustFS defaults to the latest upstream release (override with `rustfs_tag`) and is compiled in as `RUSTFS_VERSION`.
- **Platforms**: macOS Apple Silicon, macOS Intel, and Windows x86_64. A platform is skipped when the upstream release has no asset for it (e.g. `1.0.1` has no macOS Intel build).
- **Outputs**: macOS `.dmg`, `.app.zip`, and updater `.app.tar.gz` + `.sig`; Windows NSIS `-setup.exe` + `.sig`; plus `latest.json` (updater manifest) and `SHA256SUMS`. No MSI, AppImage, or Linux bundles are produced.

A new upstream release is normally published as launcher installers within a couple of hours: up to one hour until the next check, plus roughly 40 minutes of build time.

## Coding Style & Naming Conventions
Use idiomatic Rust formatting (4-space indentation, `snake_case` modules/functions, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants) and guard changes with `cargo fmt`.
Group Leptos components per route, exposing them via `pub fn component_name() -> impl IntoView`.
Keep CSS in `styles.css` or `src/logs.css`, favoring kebab-case class names and scoped selectors.

## Testing Guidelines
Run `cargo test -p rustfs-launcher` to exercise backend modules; add unit tests adjacent to the code under `#[cfg(test)]`.
For logic that shells out to the RustFS binary, add smoke coverage that injects a stub path (see `src-tauri/src/process.rs`).
Document manual UI checks—launcher start-up, binary download, command invocation—in PR descriptions until automated UI tests exist.

## Commit & Pull Request Guidelines
Follow Conventional Commits (`feat:`, `fix:`, `chore:`) as seen in history; keep the subject ≤72 characters and scope focused.
Split backend and UI updates when practical to aid review.
PRs should include an intent summary, the commands/tests run (or screenshots for UI tweaks), and links to relevant issues.
Request review before merging and wait for CI; resolve any fmt/clippy/test failures locally first.

## Communication
与用户交流时必须全程使用中文。
