#!/usr/bin/env bash
# Make a downloaded macOS RustFS binary runnable without Homebrew.
#
# RustFS 1.0.1 links liblzma with the absolute install name
# /opt/homebrew/opt/xz/lib/liblzma.5.dylib (compatibility version 14.0.0).
# This builds xz 5.8.4, whose liblzma advertises compatibility version 14,
# rewrites that load command to @loader_path/liblzma.5.dylib, and places the
# library next to the binary. Any other non-relocatable load command fails
# the build instead of being patched.

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <rustfs-binary>" >&2
  exit 2
fi

binary=$1
root="$(cd "$(dirname "$0")/.." && pwd)"
macho="$root/scripts/macos_macho.py"

if [[ ! -f "$binary" ]]; then
  echo "RustFS binary not found: $binary" >&2
  exit 1
fi

plan_json="$(python3 "$macho" plan "$binary")"
status="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["status"])' "$plan_json")"

case "$status" in
  not-macho|clean)
    exit 0
    ;;
  error|blocked)
    python3 "$macho" check "$binary" >&2 || true
    echo "Refusing to bundle $binary" >&2
    exit 1
    ;;
  vendor-lzma)
    ;;
  *)
    echo "Unexpected Mach-O plan status: $status" >&2
    exit 1
    ;;
esac

if [[ "$(uname -s)" != "Darwin" ]]; then
  python3 "$macho" check "$binary" >&2 || true
  echo "Vendoring liblzma for $binary requires macOS. Refusing to bundle a binary that still has a non-relocatable liblzma load command." >&2
  exit 1
fi

XZ_VERSION="5.8.4"
XZ_SHA256="0014c7886930454fe8bd4228665b51af55eeae560ea135c9c4cd33f55b2591d9"
dest_dylib="$(dirname "$binary")/liblzma.5.dylib"
workdir="$(mktemp -d)"
cleanup() {
  rm -rf "$workdir"
}
trap cleanup EXIT

echo "Building xz ${XZ_VERSION} to vendor liblzma next to $(basename "$binary")"
curl -fsSL --retry 3 --retry-delay 5 \
  -o "$workdir/xz.tar.gz" \
  "https://github.com/tukaani-project/xz/releases/download/v${XZ_VERSION}/xz-${XZ_VERSION}.tar.gz"
python3 - "$workdir/xz.tar.gz" "$XZ_SHA256" << 'PY'
import hashlib, pathlib, sys
digest = hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest()
expected = sys.argv[2]
if digest != expected:
    raise SystemExit(f"xz tarball checksum mismatch: expected {expected}, got {digest}")
print(f"Verified xz tarball {digest}")
PY
tar -xzf "$workdir/xz.tar.gz" -C "$workdir"

export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"
configure_dir="$(echo "$workdir"/xz-"$XZ_VERSION")"
(
  cd "$configure_dir"
  ./configure \
    --disable-static \
    --enable-shared \
    --disable-nls \
    --disable-xz \
    --disable-xzdec \
    --disable-lzmadec \
    --disable-lzmainfo \
    --disable-scripts \
    --disable-doc \
    --prefix="$workdir/prefix"
  make -j"$(sysctl -n hw.ncpu)"
)

built=""
while IFS= read -r candidate; do
  if python3 "$macho" is-macho "$candidate"; then
    built=$candidate
    break
  fi
done < <(find "$configure_dir" -type f -name 'liblzma.5.dylib' -print)
if [[ -z "$built" ]]; then
  echo "xz ${XZ_VERSION} did not produce a Mach-O liblzma.5.dylib" >&2
  exit 1
fi

cp "$built" "$dest_dylib"
chmod 755 "$dest_dylib"
python3 "$macho" rewrite-id "$dest_dylib" "@loader_path/liblzma.5.dylib"

python3 - "$macho" "$plan_json" "$binary" << 'PY'
import json, subprocess, sys
macho, plan_json, binary = sys.argv[1:]
plan = json.loads(plan_json)
seen = set()
for item in plan["liblzma"]:
    path = item["path"]
    if path == "@loader_path/liblzma.5.dylib" or path in seen:
        continue
    seen.add(path)
    subprocess.run([sys.executable, macho, "rewrite", binary, path, "@loader_path/liblzma.5.dylib"], check=True)
PY

if ! python3 "$macho" accept-lzma "$dest_dylib" "$binary"; then
  echo "Built liblzma cannot satisfy $binary" >&2
  exit 1
fi
python3 "$macho" check "$binary"

if command -v codesign >/dev/null 2>&1; then
  codesign --force --sign - "$dest_dylib"
  codesign --force --sign - "$binary"
fi

echo "Vendored @loader_path/liblzma.5.dylib beside $(basename "$binary")"
