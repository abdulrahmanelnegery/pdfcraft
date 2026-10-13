# PdfCraft Enhanced

A PDF reader and editor built on [PdfCraft](https://github.com/storytold/pdfcraft), an open-source,
Rust-native PDF application (MIT OR Apache-2.0). This is an independent fork with its own app id
(`dev.pdfcraftenhanced`), icon (gold border) and settings folder, so it installs next to the regular
PdfCraft. It is not affiliated with the ArtCraft Team and carries none of its branding.

## What is different

- **Night mode** (View menu): page rasters are drawn with lightness flipped and hues kept, so a
  book reads light-on-dark. Display only; the document never changes. The choice is remembered.
- **Reading position**: each file reopens at the page you left it on, like any PDF reader. Positions
  are kept locally for the last 500 files.

Everything else is upstream PdfCraft. The original README is kept in `docs/upstream-README.md`.

## Build and install (Debian/Ubuntu)

```
packaging/enhanced/package.sh      # needs cargo; uses nfpm if present, else dpkg-deb
sudo apt install ./dist/release/pdfcraftenhanced-*.deb
```

Run it as `pdfcraftenhanced`. Settings live in `~/.local/share/pdfcraftenhanced`.

## Licence

MIT OR Apache-2.0, as upstream: see `LICENSE-MIT`, `LICENSE-APACHE` and `NOTICE`.
