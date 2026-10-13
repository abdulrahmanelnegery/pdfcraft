#!/usr/bin/env bash
# Build the PdfCraft Enhanced .deb (own app id, icon, settings folder and binary name, so it
# installs next to the regular pdfcraft). Needs cargo and nfpm, or dpkg-deb as the fallback.
#
#   packaging/enhanced/package.sh [--skip-build]
set -euo pipefail
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/enhanced"
APP_ID=dev.pdfcraftenhanced
[ "${1:-}" = "--skip-build" ] || (cd "$ROOT" && cargo build --release --locked -p pdfcraft --features enhanced)
BIN="$CARGO_TARGET_DIR/release/pdfcraft"
WORK="$CARGO_TARGET_DIR/enhanced-package"
STAGE="$WORK/root"
rm -rf "$WORK"
# The real binary sits in its own folder with its OCR models beside it (found at <exe dir>/models),
# so nothing collides with the regular pdfcraft package's /usr/share/pdfcraft/models.
LIB="$STAGE/usr/lib/pdfcraftenhanced"
install -Dm755 "$BIN" "$LIB/pdfcraftenhanced"
strip "$LIB/pdfcraftenhanced" 2>/dev/null || true
mkdir -p "$STAGE/usr/bin"
ln -s ../lib/pdfcraftenhanced/pdfcraftenhanced "$STAGE/usr/bin/pdfcraftenhanced"
install -Dm644 "$HERE/$APP_ID.desktop" "$STAGE/usr/share/applications/$APP_ID.desktop"
mkdir -p "$STAGE/usr/share/icons"
cp -R "$ROOT/assets/app-icon-enhanced/hicolor" "$STAGE/usr/share/icons/"
# OCR models. Without the network the
# package is built without them and OCR is unavailable.
stage_models "$LIB/models" || {
  echo "warning: OCR models not downloaded; building without OCR" >&2
  rm -rf "$LIB/models"
}
mkdir -p "$DIST"
ARCH="$(dpkg --print-architecture)"
OUT="$DIST/pdfcraftenhanced-$VERSION-linux-$ARCH.deb"
if command -v nfpm >/dev/null; then
  export VERSION NFPM_ARCH="$ARCH"
  sed "s|\${STAGE}|$STAGE|g" "$HERE/nfpm.yaml" >"$WORK/nfpm.yaml"
  (cd "$ROOT" && nfpm package -f "$WORK/nfpm.yaml" -p deb -t "$OUT")
else
  mkdir -p "$STAGE/DEBIAN"
  cp "$ROOT/packaging/linux/postinst.sh" "$STAGE/DEBIAN/postinst"
  cp "$ROOT/packaging/linux/postinst.sh" "$STAGE/DEBIAN/postrm"
  chmod 755 "$STAGE/DEBIAN/postinst" "$STAGE/DEBIAN/postrm"
  cat >"$STAGE/DEBIAN/control" <<CTL
Package: pdfcraftenhanced
Version: $VERSION
Section: graphics
Priority: optional
Architecture: $ARCH
Maintainer: PdfCraft Enhanced <noreply@localhost>
Depends: libc6 (>= 2.35), libgcc-s1, libxkbcommon0, libvulkan1 | libegl1
Recommends: cups-client, mesa-vulkan-drivers, xdg-desktop-portal
Description: PdfCraft Enhanced, a PDF reader and editor made for long reading
 Open-source Rust PDF reader and editor (a fork of PdfCraft) with night, soft
 night and sepia reading themes, a per-file reading place (page, position and
 zoom), a distraction-free full-screen mode, icon menus, and Read mode by default.
 Installs next to the regular pdfcraft package.
CTL
  dpkg-deb --build --root-owner-group "$STAGE" "$OUT"
fi
echo "wrote $OUT"
