# ARCHITECTURE (BRIEF)

This app follows a Rust-core / Kotlin-renderer split with backend-driven UI over JSON.

## Stack & Responsibilities
- **Rust core**: owns `AppState`, navigation stack, business logic (hashes, PDF ops, archives, text processing), and renders screens as JSON (typed builders). JNI entry catches panics.
- **Kotlin renderer**: parses JSON and builds native Views (no Compose/fragments). Widgets: Column/Grid/Text/Button/TextInput/Checkbox/Progress/ShaderToy/ImageBase64/ColorSwatch/PdfPagePicker/SignaturePad/DepsList/CodeView. Renderer validates required fields and falls back to an inline error screen on schema issues.
- **Async**: Kotlin calls Rust on background threads for blocking work; UI updates on main thread. Loading overlay used for “loading_only” calls.

## Navigation
- `Vec<Screen>` stack in Rust; Home is root. Hardware Back calls `back` action; inline Back buttons shown when depth > 1 (QR, text tools, archive viewer, sensor logger, color tools, Kotlin image flows, text viewer).
- `snapshot`/`restore_state` serialize/rehydrate `AppState` for Activity recreation.

## Files & FDs
- Kotlin SAF picker opens `ParcelFileDescriptor` and passes detached FDs/paths to Rust. Rust avoids panics across JNI and reports errors as UI JSON.

## UI/UX Highlights
- **Renderer diffing**: Kotlin assigns stable IDs (explicit `id`, `bind_key`, or action) and reuses Views instead of nuking the tree, reducing jank and keeping input focus/keyboard stable across updates.
- **Text viewer**: WebView + bundled Prism (MIT) assets in `assets/prism/`; syntax highlighting for JSON/Markdown/TOML/YAML/Rust, theme toggle, wrap, line numbers, internal scroll. Archive text-like entries are tappable and open directly. Large-log UX gaps remain (pagination/search, see TODOs).
- **PDF tools**: PdfRenderer thumbnails via `PdfPagePicker`; lopdf handles extract/delete/merge/title/signature with px→pt scaling from SignaturePad dimensions/DPI; Y flipped using MediaBox height. Signature placement now supports a visual tap overlay (`PdfSignPlacement`) that emits normalized coords and writes onto the selected page (no extra pages). Memory mapping (memmap2) prevents loading entire PDFs into heap. Outputs prefer the source directory (or Downloads/content cache) with `_modified_YYMMDDhhmm.pdf` suffix and logcat breadcrumbs for debugging.
- **Text viewer binary guardrails**: Detects binary/unsupported files; shows a 4KB hex preview with “Load anyway” instead of forcing a full read. Normal text still capped at 256KB with syntax highlighting.
- **Kotlin image flow**: Conversion on Kotlin side; Rust drives screens, output dir selection, results. Future: add resize/quality controls for mail-friendly outputs.
- **Archive viewer**: ZIP listing (capped, truncated flag); text entries are buttons that load into the text viewer.
- **Color/Text tools/QR/Sensor logger**: Pure-Rust logic with native UI; QR encoded via `qrcode` and shown as base64 image.
- **Accessibility**: `content_description` propagated on widgets; Back buttons consistent; renderer guardrails prevent crashes on malformed payloads.

## Assets & Licensing
- Prism assets (core + minimal languages + line numbers) are bundled into a single `prism-bundle.min.js`; MIT license lives in `app/src/main/assets/prism/PRISM_LICENSE.txt`. Keep asset set small to honor APK budget (<5 MB target).

## Build & Testing
- Rust: `cargo test` (panic-catching JNI, typed UI builders). Android: Gradle builds arm64-only, shrink/obfuscate enabled; deps metadata generated to assets for About screen. Robolectric tests validate renderer JSON schema (TextInput/Checkbox/Progress/Grid/PdfPagePicker/DepsList) and should be extended for `CodeView`.

## Pending
- Harden schema validation end-to-end; add renderer tests for `CodeView`/Prism payloads.
- On-device QA: text viewer (large files, binary/UTF-8 errors, TalkBack), sensor logger permissions/CSV, size audits via `scripts/size_report.sh`.
- UX gaps to address: input diffing vs. binding churn (avoid keyboard loss), PDF signature positioning UX (grid/preview overlay), sensor logging survival via Foreground Service, text viewer pagination/search, output “Save As” flows, back-stack safety prompts, image resize/quality controls, DSL grouping widget.

## Known Risks (short)
- Global Rust `STATE` mutex can block long ops; consider queue/timeout; watch for poison on panic.
- FD lifetime: detached FDs rely on manual close; migrate to `OwnedFd`/RAII.
- PDF signing coords: normalized mapping needs tests/clamping; may misplace on DPI mismatch.
- Text viewer reads 256 KiB into memory; binary/large files need streaming/hex fallback.
- Schema drift: no versioned JSON schema between Rust/Kotlin; renderer guards cover most cases but not all.
