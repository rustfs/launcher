# Contributing to RustFS Launcher

User-facing install and usage notes live in [README.md](README.md). This file is for people building or changing the app.

## Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)
- [Trunk](https://trunkrs.dev/) — `cargo install trunk`

## Project layout

- `src/` — Leptos UI (`main.rs`, `app.rs`, `src/logs.css`)
- `src-tauri/src/` — Tauri backend (`commands.rs`, `process.rs`, `state.rs`)
- `src-tauri/binaries/` — platform RustFS binaries fetched by the build scripts; keep this untracked
- `public/` — static assets for Trunk
- `Trunk.toml`, `src-tauri/tauri.conf.json` — web client and desktop shell config

## Building

Download the RustFS binary for your machine first:

### macOS / Linux

```bash
./build.sh
cargo tauri dev      # development
cargo tauri build    # production bundle
```

### Windows

```cmd
build.bat
cargo tauri dev
cargo tauri build
```

The script only fetches the binary for the current platform:

- macOS Apple Silicon: `rustfs-macos-aarch64`
- macOS Intel: `rustfs-macos-x86_64`
- Windows x86_64: `rustfs-windows-x86_64.exe`
- Windows ARM64: uses the x86_64 binary via emulation until upstream ships a native build

Browser-only UI work:

```bash
trunk serve --port 1421
```

## Checks before a push

```bash
make pre-commit
```

That runs `cargo fmt`, Clippy, `trunk build`, and `cargo test`. Individual targets:

```bash
make check-fmt
make check-clippy
make check-frontend
make check-test
make fix-fmt
make check-upstream
```

Backend tests:

```bash
cargo test -p rustfs-launcher
```

Code that shells out to the RustFS binary should keep smoke coverage with a stub path (see `src-tauri/src/process.rs`).

## Style

Idiomatic Rust, 4-space indent, `snake_case` modules, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants. Run `cargo fmt`. Group Leptos components per route as `pub fn component_name() -> impl IntoView`. Keep CSS in `styles.css` or `src/logs.css`, kebab-case class names.

## Upstream sync and releases

Hourly, `.github/workflows/upstream-sync.yml` reads `https://version.rustfs.com/latest.json`. A new rustfs/rustfs tag can create a launcher tag and kick `.github/workflows/build.yml`, which builds Windows and macOS installers and publishes them on the GitHub release.

Manual trigger is available on that workflow (`force_build`). Signing and the in-app updater are documented in [docs/SELF_UPDATE.md](docs/SELF_UPDATE.md). Workflow internals: [.github/ACTIONS.md](.github/ACTIONS.md). Local Act testing: [.github/TESTING.md](.github/TESTING.md).

## Commits and PRs

Conventional Commits (`feat:`, `fix:`, `chore:`), subject ≤ 72 characters. Split backend and UI when it helps review. Include what you ran (or a screenshot for UI work) and wait for CI.

Recommended editor setup: [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
