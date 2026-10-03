#!/usr/bin/env bash
# Fail the macOS package step unless the bundled RustFS binary can be loaded
# without an absolute Homebrew (or other non-system) library path.

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <bundled-rustfs>" >&2
  exit 2
fi

binary=$1
root="$(cd "$(dirname "$0")/.." && pwd)"
macho="$root/scripts/macos_macho.py"

if [[ ! -f "$binary" ]]; then
  echo "Bundled RustFS binary not found: $binary" >&2
  exit 1
fi

python3 "$macho" check "$binary"
need="$(python3 "$macho" needs-lzma "$binary")"
if [[ "$need" == "yes" ]]; then
  dylib="$(dirname "$binary")/liblzma.5.dylib"
  if [[ ! -f "$dylib" ]]; then
    echo "RustFS loads liblzma.5.dylib, but $dylib is not beside the binary." >&2
    python3 "$macho" plan "$binary" >&2
    exit 1
  fi
  python3 "$macho" accept-lzma "$dylib" "$binary"
fi

python3 - "$binary" << 'PY'
import subprocess
import sys

path = sys.argv[1]
try:
    result = subprocess.run(
        [path, "--version"],
        timeout=60,
        check=False,
        capture_output=True,
        text=True,
    )
except subprocess.TimeoutExpired:
    print("Bundled rustfs --version timed out after 60s", file=sys.stderr)
    sys.exit(1)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print(f"Bundled rustfs --version exited {result.returncode}", file=sys.stderr)
    sys.exit(result.returncode or 1)
print("Bundled rustfs --version ok")
PY
