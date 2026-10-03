#!/usr/bin/env bash
# Portable checks for the macOS load-command guard. Building liblzma itself
# only happens on Darwin, inside prepare-macos-rustfs.sh.

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
macho="$root/scripts/macos_macho.py"
prepare="$root/scripts/prepare-macos-rustfs.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

python3 "$macho" --self-test

python3 "$macho" write-fixture "$tmp/clean" /usr/lib/libSystem.B.dylib \
  /System/Library/Frameworks/CoreFoundation.framework/CoreFoundation
bash "$prepare" "$tmp/clean"

python3 "$macho" write-fixture "$tmp/openssl" \
  /usr/local/opt/openssl@3/lib/libssl.3.dylib
set +e
openssl_out="$(bash "$prepare" "$tmp/openssl" 2>&1)"
openssl_code=$?
set -e
if [[ "$openssl_code" -eq 0 ]]; then
  echo "expected an OpenSSL load command to fail the guard" >&2
  exit 1
fi
if ! printf '%s\n' "$openssl_out" | grep -F "/usr/local/opt/openssl@3/lib/libssl.3.dylib" >/dev/null; then
  echo "guard error did not quote the OpenSSL load command:" >&2
  printf '%s\n' "$openssl_out" >&2
  exit 1
fi

python3 "$macho" write-fixture "$tmp/star" "/opt/homebrew/*/libfoo.1.dylib"
set +e
star_out="$(bash "$prepare" "$tmp/star" 2>&1)"
star_code=$?
set -e
if [[ "$star_code" -eq 0 ]]; then
  echo "expected a literal asterisk install name to fail the guard" >&2
  exit 1
fi
if ! printf '%s\n' "$star_out" | grep -F "/opt/homebrew/*/libfoo.1.dylib" >/dev/null; then
  echo "guard error did not quote the asterisk load command:" >&2
  printf '%s\n' "$star_out" >&2
  exit 1
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
  python3 "$macho" write-fixture "$tmp/lzma" \
    /opt/homebrew/opt/xz/lib/liblzma.5.dylib \
    /usr/lib/libSystem.B.dylib
  set +e
  lzma_out="$(bash "$prepare" "$tmp/lzma" 2>&1)"
  lzma_code=$?
  set -e
  if [[ "$lzma_code" -eq 0 ]]; then
    echo "expected non-macOS packaging to reject an unvendored liblzma dependency" >&2
    exit 1
  fi
  if ! printf '%s\n' "$lzma_out" | grep -F "/opt/homebrew/opt/xz/lib/liblzma.5.dylib" >/dev/null; then
    echo "guard error did not quote the Homebrew liblzma load command:" >&2
    printf '%s\n' "$lzma_out" >&2
    exit 1
  fi
fi

echo "prepare-macos-rustfs guard test passed"
