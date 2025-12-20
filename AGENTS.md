# Repository Guidelines

Use this guide when contributing to RustFS Launcher; it highlights the project layout, tooling, and expectations.

## Project Structure & Module Organization
- `src/`: Leptos client entrypoint (`main.rs`), UI composition (`app.rs`), and contextual styles (`logs.css`).
- `src-tauri/src/`: Tauri backend modules: `commands.rs` for invokable actions, `process.rs` for RustFS binary orchestration, `state.rs` for shared application state, plus error types.
- `src-tauri/binaries/`: Platform-specific RustFS executables fetched by the build scripts; keep this directory untracked in Git.
- `public/`: Static assets bundled by Trunk (`leptos.svg`, `tauri.svg`).
- `Trunk.toml` and `tauri.conf.json`: runtime configuration for the web client and desktop shell.

## Build, Test, and Development Commands
- `./build.sh` (macOS/Linux) or `build.bat` (Windows): download the matching RustFS binary into `src-tauri/binaries/`.
- `cargo tauri dev`: launch the desktop shell with hot reload across the Rust backend and Leptos UI.
- `trunk serve --port 1421`: run the Leptos client in a browser-only workflow using the `Trunk.toml` settings.
- `cargo tauri build`: produce distributable desktop bundles.
- `cargo fmt --all` and `cargo clippy --workspace --all-targets`: enforce formatting and linting before pushing.
- `./scripts/check-upstream-version.sh`: check for new versions from upstream rustfs/rustfs repository.

## Automated Build & Release
The repository includes automated workflows to keep in sync with upstream rustfs/rustfs releases:

### Upstream Version Sync (`.github/workflows/upstream-sync.yml`)
- **Scheduled Check**: Runs daily at UTC 6:00 (Beijing 14:00) to check for new rustfs/rustfs releases.
- **Automatic Trigger**: When a new upstream version is detected, it automatically creates a corresponding git tag and triggers the build workflow.
- **Manual Trigger**: Can be manually triggered via GitHub Actions with an optional `force_build` parameter.
- **Version Tracking**: Compares upstream release tags with local repository tags to detect updates.

### Build Workflow (`.github/workflows/build.yml`)
- Triggered automatically by new tags created by the upstream sync workflow.
- Builds platform-specific installers for Windows (macOS support can be uncommented).
- Downloads the latest RustFS binaries from `dl.rustfs.com` during the build process.
- Produces distributable packages (DMG, MSI, AppImage, etc.) as GitHub release artifacts.

This automation ensures that whenever rustfs/rustfs publishes a new version, this launcher repository will automatically build and release updated installers within 24 hours.

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
