#!/usr/bin/env bash
# Pick the upstream RustFS zip for a launcher matrix slug.
# Usage: select-rustfs-asset.sh <slug> <tag> <asset-list>
# Prints the chosen file name, or exits 1 when none match.

set -euo pipefail

select_rustfs_asset() {
  local slug="$1"
  local tag="$2"
  local asset_list="$3"
  local version="${tag#v}"
  local candidates=(
    "${slug}-latest.zip"
    "${slug}-v${version}.zip"
    "${slug}-${tag}.zip"
    "${slug}-${version}.zip"
  )
  local name
  for name in "${candidates[@]}"; do
    if printf '%s\n' "$asset_list" | grep -Fxq "$name"; then
      printf '%s\n' "$name"
      return 0
    fi
  done
  return 1
}

if [[ "${1:-}" == "--self-test" ]]; then
  fixture=$'rustfs-macos-aarch64-latest.zip\nrustfs-macos-aarch64-v1.0.0-rc.4.zip\nrustfs-windows-x86_64-latest.zip\nrustfs-windows-x86_64-v1.0.0-rc.4.zip'
  test "$(select_rustfs_asset rustfs-macos-aarch64 1.0.0-rc.4 "$fixture")" = "rustfs-macos-aarch64-latest.zip"
  test "$(select_rustfs_asset rustfs-windows-x86_64 v1.0.0-rc.4 "$fixture")" = "rustfs-windows-x86_64-latest.zip"
  if select_rustfs_asset rustfs-macos-x86_64 1.0.0-rc.4 "$fixture"; then
    echo "expected Intel macOS to be missing" >&2
    exit 1
  fi
  only_versioned=$'rustfs-macos-aarch64-v1.0.0-rc.4.zip'
  test "$(select_rustfs_asset rustfs-macos-aarch64 1.0.0-rc.4 "$only_versioned")" = "rustfs-macos-aarch64-v1.0.0-rc.4.zip"
  echo "select-rustfs-asset self-test passed"
  exit 0
fi

select_rustfs_asset "${1:?slug}" "${2:?tag}" "${3:?asset list}"
