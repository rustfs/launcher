#!/usr/bin/env bash
# Verify a downloaded RustFS archive against a sha256sum-style manifest.
# Usage: verify-rustfs-checksum.sh <manifest> <file> [asset-name]
# asset-name defaults to the file's basename. Pass it when the local file
# was saved under a different name than the upstream release asset.

set -euo pipefail

verify_rustfs_checksum() {
  local manifest="$1"
  local file="$2"
  local asset_name="${3:-$(basename "$file")}"
  local expected
  expected="$(
    awk -v name="$asset_name" '
      NF >= 2 {
        digest = tolower($1)
        listed = $2
        sub(/^\*/, "", listed)
        if (listed == name && length(digest) == 64) {
          print digest
          exit
        }
      }
    ' "$manifest"
  )"
  if [[ -z "$expected" ]]; then
    echo "No SHA256 digest for ${asset_name} in ${manifest}" >&2
    return 1
  fi

  local actual
  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$file" | awk '{print tolower($1)}')"
  else
    actual="$(shasum -a 256 "$file" | awk '{print tolower($1)}')"
  fi
  if [[ "$actual" != "$expected" ]]; then
    echo "Checksum mismatch for ${asset_name}: expected ${expected}, got ${actual}" >&2
    return 1
  fi
  echo "Verified ${asset_name}"
}

if [[ "${1:-}" == "--self-test" ]]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  printf 'rustfs' > "$tmp/rustfs-windows-x86_64-v1.0.1.zip"
  if command -v sha256sum >/dev/null 2>&1; then
    digest="$(sha256sum "$tmp/rustfs-windows-x86_64-v1.0.1.zip" | awk '{print $1}')"
  else
    digest="$(shasum -a 256 "$tmp/rustfs-windows-x86_64-v1.0.1.zip" | awk '{print $1}')"
  fi
  printf '%s *%s\n' "$digest" "rustfs-windows-x86_64-v1.0.1.zip" > "$tmp/SHA256SUMS"
  cp "$tmp/rustfs-windows-x86_64-v1.0.1.zip" "$tmp/local-name.zip"
  verify_rustfs_checksum "$tmp/SHA256SUMS" "$tmp/local-name.zip" "rustfs-windows-x86_64-v1.0.1.zip" >/dev/null
  printf 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  other.zip\n' > "$tmp/missing"
  if verify_rustfs_checksum "$tmp/missing" "$tmp/local-name.zip" "rustfs-windows-x86_64-v1.0.1.zip" >/dev/null 2>&1; then
    echo "expected a missing digest to fail" >&2
    exit 1
  fi
  echo "verify-rustfs-checksum self-test passed"
  exit 0
fi

verify_rustfs_checksum "${1:?manifest}" "${2:?file}" "${3:-}"
