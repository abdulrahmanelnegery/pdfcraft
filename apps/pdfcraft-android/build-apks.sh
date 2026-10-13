#!/usr/bin/env bash
# Build the signed APKs of PdfCraft Enhanced, one per CPU architecture (ABI):
#   dist/android/pdfcraftenhanced-<version>-android-<abi>.apk
#
#   KEYSTORE=<release.jks> KEYSTORE_PASSWORD=<pw> apps/pdfcraft-android/build-apks.sh [abi ...]
#   (default ABIs: arm64-v8a armeabi-v7a x86_64)
#
# cargo-apk builds the native library and the APK around it; the activity is a Java class
# (java/dev/pdfcraftenhanced/MainActivity.java: the system document picker), so this script compiles
# it with javac and d8, adds the dex to the APK, aligns and signs it.
#
# Needs: the Android SDK (platform 34, build-tools) and NDK 27 under $ANDROID_HOME, a JDK, cargo-apk
# and the Rust Android targets.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
: "${KEYSTORE:?set KEYSTORE to the release keystore}" "${KEYSTORE_PASSWORD:?set KEYSTORE_PASSWORD}"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}" ANDROID_SDK_ROOT="${ANDROID_HOME}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$(ls -d "$ANDROID_HOME"/ndk/* | sort | tail -1)}"
BT="${BUILD_TOOLS:-$(ls -d "$ANDROID_HOME"/build-tools/* | sort -V | tail -1)}"
JAR="$ANDROID_HOME/platforms/android-34/android.jar"
TC="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
VERSION="$(awk '/^\[workspace.package\]/{p=1;next} p&&$1=="version"{gsub(/"/,"",$3);print $3;exit}' "$ROOT/Cargo.toml")"
OUT="$ROOT/dist/android"; WORK="$ROOT/target/android-dex"
mkdir -p "$OUT"; rm -rf "$WORK"; mkdir -p "$WORK/classes"

# The activity's dex, once for every ABI.
javac -source 8 -target 8 -Xlint:-options -cp "$JAR" -d "$WORK/classes" "$HERE"/java/dev/pdfcraftenhanced/*.java
"$BT/d8" --release --min-api 26 --lib "$JAR" --output "$WORK" "$WORK"/classes/dev/pdfcraftenhanced/*.class

abis=("$@"); [ ${#abis[@]} -gt 0 ] || abis=(arm64-v8a armeabi-v7a x86_64)
for abi in "${abis[@]}"; do
  case "$abi" in
    arm64-v8a) triple=aarch64-linux-android; cc=aarch64-linux-android26-clang ;;
    armeabi-v7a) triple=armv7-linux-androideabi; cc=armv7a-linux-androideabi26-clang ;;
    x86_64) triple=x86_64-linux-android; cc=x86_64-linux-android26-clang ;;
    *) echo "unknown abi $abi" >&2; exit 2 ;;
  esac
  echo "== $abi"
  envvar="${triple//-/_}"
  export "CC_$envvar=$TC/$cc" "AR_$envvar=$TC/llvm-ar"
  (cd "$HERE" && CARGO_APK_RELEASE_KEYSTORE="$KEYSTORE" CARGO_APK_RELEASE_KEYSTORE_PASSWORD="$KEYSTORE_PASSWORD" cargo apk build --release --target "$triple")
  raw="$ROOT/target/release/apk/PdfCraftEnhanced-unaligned.apk"
  work="$WORK/$abi.apk"; cp "$raw" "$work"
  python3 - "$work" "$WORK/classes.dex" <<'PY'
import sys, zipfile
apk, dex = sys.argv[1:3]
with zipfile.ZipFile(apk, "a", zipfile.ZIP_DEFLATED) as z:
    z.write(dex, "classes.dex")
PY
  "$BT/zipalign" -f -p 4 "$work" "$WORK/$abi-aligned.apk"
  final="$OUT/pdfcraftenhanced-$VERSION-android-$abi.apk"
  "$BT/apksigner" sign --v4-signing-enabled false --ks "$KEYSTORE" --ks-pass "pass:$KEYSTORE_PASSWORD" --out "$final" "$WORK/$abi-aligned.apk"
  "$BT/apksigner" verify "$final"
  (cd "$OUT" && sha256sum -- "$(basename "$final")" > "$(basename "$final").sha256")
  echo "built $final"
done
