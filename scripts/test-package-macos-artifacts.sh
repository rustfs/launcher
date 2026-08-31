#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fixture_dir="$(mktemp -d "${TMPDIR:-/tmp}/launcher-macos-artifacts.XXXXXX")"
trap 'rm -rf "$fixture_dir"' EXIT

app_dir="$fixture_dir/target/release/bundle/macos/RustFS Launcher.app"
updater_file="$fixture_dir/target/release/bundle/macos/RustFS Launcher.app.tar.gz"
dmg_file="$fixture_dir/target/release/bundle/dmg/RustFS Launcher.dmg"
artifact_dir="$fixture_dir/artifacts"
stub_dir="$fixture_dir/bin"

mkdir -p "$app_dir" "$(dirname "$dmg_file")" "$artifact_dir" "$stub_dir" "$fixture_dir/unrelated"
printf 'app' > "$app_dir/Contents.txt"
printf 'updater' > "$updater_file"
printf 'signature' > "${updater_file}.sig"
printf 'dmg' > "$dmg_file"

printf '%s\n' \
  '#!/usr/bin/env bash' \
  'set -euo pipefail' \
  'output="${@: -1}"' \
  'printf "zip" > "$output"' \
  > "$stub_dir/ditto"
chmod +x "$stub_dir/ditto"

(
  cd "$fixture_dir/unrelated"
  DITTO_BIN="$stub_dir/ditto" \
    bash "$repo_root/scripts/package-macos-artifacts.sh" \
      "$app_dir" \
      "$updater_file" \
      "$dmg_file" \
      "$artifact_dir" \
      "rustfs-launcher-macos-aarch64-1.2.3-rc.1" \
      "rustfs-launcher-macos-aarch64-latest" \
      true
)

expected=(
  rustfs-launcher-macos-aarch64-1.2.3-rc.1.app.tar.gz
  rustfs-launcher-macos-aarch64-1.2.3-rc.1.app.tar.gz.sig
  rustfs-launcher-macos-aarch64-1.2.3-rc.1.app.zip
  rustfs-launcher-macos-aarch64-1.2.3-rc.1.dmg
  rustfs-launcher-macos-aarch64-latest.app.tar.gz
  rustfs-launcher-macos-aarch64-latest.app.tar.gz.sig
  rustfs-launcher-macos-aarch64-latest.app.zip
  rustfs-launcher-macos-aarch64-latest.dmg
)

for name in "${expected[@]}"; do
  test -f "$artifact_dir/$name"
done

dev_artifact_dir="$fixture_dir/dev-artifacts"
DITTO_BIN="$stub_dir/ditto" \
  bash "$repo_root/scripts/package-macos-artifacts.sh" \
    "$app_dir" \
    "" \
    "" \
    "$dev_artifact_dir" \
    "rustfs-launcher-macos-aarch64-dev" \
    "rustfs-launcher-macos-aarch64-latest" \
    false

test -f "$dev_artifact_dir/rustfs-launcher-macos-aarch64-dev.app.zip"
test -f "$dev_artifact_dir/rustfs-launcher-macos-aarch64-latest.app.zip"
test ! -e "$dev_artifact_dir/rustfs-launcher-macos-aarch64-dev.app.tar.gz"

unsigned_updater="$fixture_dir/unsigned.app.tar.gz"
printf 'unsigned' > "$unsigned_updater"
if DITTO_BIN="$stub_dir/ditto" \
  bash "$repo_root/scripts/package-macos-artifacts.sh" \
    "$app_dir" \
    "$unsigned_updater" \
    "" \
    "$fixture_dir/unsigned-artifacts" \
    "rustfs-launcher-macos-aarch64-unsigned" \
    "rustfs-launcher-macos-aarch64-latest" \
    true; then
  echo "expected an unsigned updater to be rejected" >&2
  exit 1
fi

echo "package-macos-artifacts test passed"
