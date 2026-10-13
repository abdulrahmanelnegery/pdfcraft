# PdfCraft Enhanced

<img src="assets/app-icon-enhanced/hicolor/256x256/apps/dev.pdfcraftenhanced.png" alt="PdfCraft Enhanced icon" width="96" align="right">

A PDF reader and editor made for long reading. It is built on [PdfCraft](https://github.com/storytold/pdfcraft), an open-source, Rust-native PDF application, and adds a better reading experience on top of it: page themes, a reading place for every file, a calm full-screen mode, and a phone-friendly Android app.

It is an independent fork, not affiliated with the ArtCraft Team, and it carries none of their branding. It installs **next to** the regular PdfCraft (own app id `dev.pdfcraftenhanced`, own icon, own settings), so you can compare the two.

## Download

Get the file for your system from the [latest release](https://github.com/abdulrahmanelnegery/pdfcraft/releases/latest). Next to every file is a checksum file with the same name plus `.sha256`.

| System | File |
|---|---|
| Android phone or tablet (64-bit, almost every phone since 2016) | `PdfCraft-Enhanced_<version>_Android_arm64-v8a.apk` |
| Android, older 32-bit devices | `PdfCraft-Enhanced_<version>_Android_armeabi-v7a.apk` |
| Android emulator, Chromebook | `PdfCraft-Enhanced_<version>_Android_x86_64.apk` |
| Windows 10/11, Intel or AMD | `PdfCraft-Enhanced_<version>_Windows_x64.msi` |
| Windows, 32-bit | `PdfCraft-Enhanced_<version>_Windows_x86.msi` |
| Windows on ARM | `PdfCraft-Enhanced_<version>_Windows_arm64.msi` |
| macOS 11+, Apple silicon | `PdfCraft-Enhanced_<version>_macOS_Apple-Silicon.dmg` |
| macOS, Intel | `PdfCraft-Enhanced_<version>_macOS_Intel.dmg` |
| Debian, Ubuntu, Mint, Pop!_OS (64-bit) | `PdfCraft-Enhanced_<version>_Linux_amd64.deb` |

Check a download with `sha256sum -c <file>.sha256` (Linux, macOS) or `Get-FileHash <file>` (Windows).

### Install

- **Android**: open the APK and allow installs from your file manager when Android asks. The app is signed with this project's own key, not by Google Play.
- **Windows**: run the `.msi`. It is not code-signed yet, so SmartScreen may show "Windows protected your PC": choose *More info*, then *Run anyway*.
- **macOS**: open the `.dmg` and drag the app to Applications. It is not notarized by Apple yet, so the first launch is blocked: open *System Settings > Privacy & Security*, find the message about PdfCraft Enhanced and choose *Open Anyway* (or run `xattr -dr com.apple.quarantine "/Applications/PdfCraft Enhanced.app"` once).
- **Linux**: `sudo apt install ./PdfCraft-Enhanced_<version>_Linux_amd64.deb`, then run `pdfcraftenhanced`. To update, quit the app first, then install again with `--reinstall`. Settings live in `~/.local/share/pdfcraftenhanced`.

## What is new compared with PdfCraft

- **Reading themes**: Normal page, Night, Soft night and Sepia (Menu > View > Reading theme, or the palette button on the right rail). Night flips lightness and keeps hue, so images stay recognisable. Remembered.
- **Reading place**: every file reopens at the page, the spot in the page and the zoom you left it at. New files open at fit-width.
- **Page display remembered**: single page, two-up and continuous scrolling (continuous by default).
- **Full-screen reading**: the quick tools and the page controls float on the left in their own islands, with one toggle that hides and shows them. Esc leaves.
- **Read mode by default**, with no side panel opened on its own. Icons on every menu entry and roomier menu rows.
- **Android**: a compact layout for phones (one title bar, a switcher for open books), finger-drag scrolling, the system document picker, opening PDFs from other apps ("Open with"), and Arabic and Chinese file names through the system fonts.
- Fixes taken from the original project, such as refusing to save a file that cannot be reopened.

Everything else is PdfCraft: read, comment, fill and sign forms, organize, combine, split, edit, redact, protect and export PDFs, plus the `pdfcraft-cli` runner. The original README is kept in [`docs/upstream-README.md`](docs/upstream-README.md).

Not done yet: the Android app has no system clipboard or share sheet, and the Windows and macOS packages are unsigned.

## Build it yourself

You need Rust (1.95 or newer, [rustup](https://rustup.rs)). The fork's identity is the cargo feature `enhanced`.

```
cargo run -p pdfcraft --features enhanced            # run it
packaging/enhanced/package.sh                        # Debian package (dpkg-deb or nfpm)
```

- **Windows and macOS packages** are built by the GitHub Actions workflow *Enhanced installers* (`.github/workflows/enhanced-release.yml`): an MSI for x64, x86 and arm64, a disk image for Apple silicon and Intel. With Apple Developer ID secrets in the repository the macOS build is signed and notarized; without them it is ad-hoc signed.
- **Android APKs**: `KEYSTORE=<release.jks> KEYSTORE_PASSWORD=<pw> apps/pdfcraft-android/build-apks.sh [arm64-v8a armeabi-v7a x86_64]` (needs the Android SDK and NDK, a JDK and `cargo install cargo-apk`).
- **Release file names and checksums**: `packaging/enhanced/release-names.sh <raw dir> <out dir>`.

## Contributing

The features that are not specific to this edition are offered to the original project as pull requests; this repository keeps its own `main`, which is protected (changes go through pull requests, no force pushes). Issues are welcome.

## Licence

MIT OR Apache-2.0, as the original: see [`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE) and [`NOTICE`](NOTICE). The lion icon is the original project's artwork under the same licence, with a gold border added here.
