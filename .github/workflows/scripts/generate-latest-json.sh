#!/usr/bin/env bash
set -euo pipefail

# Generates a Tauri updater `latest.json` manifest from downloaded release artifacts.
# Usage: generate-latest-json.sh <tag> <release-dir> <output-file>
#   e.g. generate-latest-json.sh v0.1.3 release release/latest.json

TAG="${1:?missing tag (e.g. v0.1.3)}"
DIR="${2:?missing release dir}"
OUT="${3:?missing output file}"

REPO="holdxen/SVNexus"
VERSION="${TAG#v}"
PUB_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
BASE_URL="https://github.com/${REPO}/releases/download/${TAG}"

entries='{}'

find_artifact() {
  find "$DIR" -type f -name "$1" ! -name '*.sig' -print -quit
}

add_platform() {
  local key="$1"
  shift
  local file="" pattern
  for pattern in "$@"; do
    file="$(find_artifact "$pattern")"
    if [ -n "$file" ]; then
      break
    fi
  done
  if [ -z "$file" ]; then
    echo "::error::No updater artifact for ${key} (patterns: $*)"
    echo "Files present under ${DIR}:"
    find "$DIR" -type f | sort | sed 's/^/  /'
    exit 1
  fi
  if [ ! -f "${file}.sig" ]; then
    echo "::error::Missing signature file: ${file}.sig"
    find "$DIR" -type f | sort | sed 's/^/  /'
    exit 1
  fi
  local name url sig
  name="$(basename "$file")"
  url="${BASE_URL}/${name}"
  sig="$(tr -d '\n\r' < "${file}.sig")"
  entries="$(jq -c --arg key "$key" --arg url "$url" --arg sig "$sig" \
    '. + {($key): {url: $url, signature: $sig}}' <<<"$entries")"
  echo "  ${key} -> ${name}"
}

# CLI 2.11.4 + createUpdaterArtifacts:true 的实际产物：
#   macOS   -> .app.tar.gz（唯一走 updater_bundle 打包的平台）
#   Linux   -> 裸 .AppImage + .sig（插件按内容嗅探，gz/裸文件都接受）
#   Windows -> 裸 -setup.exe / .msi + .sig（同上，zip/exe/msi 内容嗅探）
# 每个平台优先列压缩包形态，命中不了再回退裸安装包。
echo "Generating latest.json for ${TAG}"
add_platform "darwin-aarch64"  "*.app.tar.gz"
add_platform "linux-x86_64"    "*amd64*.AppImage.tar.gz" "*amd64*.AppImage"
add_platform "linux-aarch64"   "*aarch64*.AppImage.tar.gz" "*aarch64*.AppImage" "*arm64*.AppImage"
add_platform "windows-x86_64"  "*x64*.nsis.zip" "*x64*-setup.exe" "*x64*.msi.zip" "*x64*.msi"
add_platform "windows-aarch64" "*arm64*.nsis.zip" "*arm64*-setup.exe" "*arm64*.msi.zip" "*arm64*.msi"

jq -n \
  --arg version "$VERSION" \
  --arg pub_date "$PUB_DATE" \
  --argjson platforms "$entries" \
  '{version: $version, pub_date: $pub_date, platforms: $platforms}' > "$OUT"

echo "Wrote ${OUT}:"
cat "$OUT"
