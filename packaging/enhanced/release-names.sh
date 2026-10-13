#!/usr/bin/env bash
# Give the release packages their public names and make a checksum file for each, named after it
# (<package>.sha256, with the same name inside, so `sha256sum -c <package>.sha256` works).
#
#   packaging/enhanced/release-names.sh <dir with the raw packages> <out dir>
#
# Raw names (what the build produces) -> public names:
#   pdfcraftenhanced-<v>-windows-<arch>.msi      -> PdfCraft-Enhanced_<v>_Windows_<arch>.msi
#   pdfcraftenhanced-<v>-macos-arm64.dmg         -> PdfCraft-Enhanced_<v>_macOS_Apple-Silicon.dmg
#   pdfcraftenhanced-<v>-macos-x64.dmg           -> PdfCraft-Enhanced_<v>_macOS_Intel.dmg
#   pdfcraftenhanced-<v>-android-<abi>.apk       -> PdfCraft-Enhanced_<v>_Android_<abi>.apk
#   pdfcraftenhanced-<v>-linux-amd64.deb         -> PdfCraft-Enhanced_<v>_Linux_amd64.deb
set -euo pipefail
src="${1:?raw packages dir}"; out="${2:?output dir}"
mkdir -p "$out"
for f in "$src"/pdfcraftenhanced-*; do
  base="$(basename "$f")"
  case "$base" in *.sha256) continue ;; esac
  if [[ "$base" =~ ^pdfcraftenhanced-([0-9][^-]*(-[0-9]+)?)-(windows|macos|android|linux)-(.+)\.(msi|dmg|apk|deb)$ ]]; then
    v="${BASH_REMATCH[1]}"; os="${BASH_REMATCH[3]}"; arch="${BASH_REMATCH[4]}"; ext="${BASH_REMATCH[5]}"
  else
    echo "skipping $base (not a package name)" >&2; continue
  fi
  case "$os:$arch" in
    windows:*) label="Windows_$arch" ;;
    macos:arm64) label="macOS_Apple-Silicon" ;;
    macos:x64) label="macOS_Intel" ;;
    android:*) label="Android_$arch" ;;
    linux:*) label="Linux_$arch" ;;
    *) echo "skipping $base" >&2; continue ;;
  esac
  name="PdfCraft-Enhanced_${v}_${label}.${ext}"
  cp -- "$f" "$out/$name"
  (cd "$out" && sha256sum -- "$name" > "$name.sha256")
  echo "$name"
done
