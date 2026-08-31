#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 7 ]]; then
  echo "usage: $0 <app-dir> <updater-file> <dmg-file> <artifact-dir> <artifact-prefix> <latest-prefix> <is-release>" >&2
  exit 2
fi

app_dir="$1"
updater_file="$2"
dmg_file="$3"
artifact_dir="$4"
artifact_prefix="$5"
latest_prefix="$6"
is_release="$7"
ditto_bin="${DITTO_BIN:-ditto}"

if [[ ! -d "$app_dir" ]]; then
  echo "macOS app bundle not found: $app_dir" >&2
  exit 1
fi

mkdir -p "$artifact_dir"

if [[ -n "$dmg_file" ]]; then
  if [[ ! -f "$dmg_file" ]]; then
    echo "macOS DMG not found: $dmg_file" >&2
    exit 1
  fi
  cp "$dmg_file" "$artifact_dir/${artifact_prefix}.dmg"
  echo "Created: ${artifact_prefix}.dmg"
fi

app_parent="$(dirname "$app_dir")"
app_name="$(basename "$app_dir")"
(
  cd "$app_parent"
  "$ditto_bin" -c -k --keepParent "$app_name" "$artifact_dir/${artifact_prefix}.app.zip"
)
echo "Created: ${artifact_prefix}.app.zip"

if [[ "$is_release" == "true" ]]; then
  if [[ -z "$updater_file" || ! -f "$updater_file" || ! -f "${updater_file}.sig" ]]; then
    echo "macOS updater artifact or signature is missing: ${updater_file:-unset}" >&2
    exit 1
  fi

  updater_name="${artifact_prefix}.app.tar.gz"
  cp "$updater_file" "$artifact_dir/$updater_name"
  cp "${updater_file}.sig" "$artifact_dir/${updater_name}.sig"
  echo "Created updater: $updater_name"
fi

for file in "$artifact_dir/${artifact_prefix}"*; do
  [[ -f "$file" ]] || continue
  file_name="$(basename "$file")"
  suffix="${file_name#"$artifact_prefix"}"
  cp "$file" "$artifact_dir/${latest_prefix}${suffix}"
  echo "Created latest: ${latest_prefix}${suffix}"
done

if ! find "$artifact_dir" -maxdepth 1 -type f -print -quit | grep -q .; then
  echo "no macOS artifacts were generated" >&2
  exit 1
fi
