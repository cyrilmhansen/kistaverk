# WORK IN PROGRESS

Keep this file short and actionable. Update it at the end of each session.

## Status (2025-12-03)
- **Camera Scanning (Rust-Driven)**: Implemented robust QR code scanning for Receiver using `rxing` pure-Rust decoder. JNI bridge passes camera frames (Y-plane) to Rust, and CameraX manages the camera lifecycle and frame acquisition on the Android side.
- **QR Data Transfer (Receiver)**: Implemented protocol logic for parsing and reassembling split files (`QRTX` header). Added manual entry UI for verification. Logic verified via unit tests.
- **QR Data Transfer (Sender)**: Implemented "QR Slideshow" to broadcast files. Features chunking (512B), adjustable speed, and play/pause controls. Protocol: `QRTX|i/n|base64`. Verified via Rust unit tests.
- **Presets**: Implemented a system to save/load favorite tool settings. Added persistence logic in `presets.rs`, `PresetManager` UI, and integrated with Dithering and Pixel Art tools. Added tests for persistence cycle.
- **System Panels**: Implemented a dashboard for system information (Storage, Network, Battery, Device). Rust defines the data structures and UI; Kotlin feeds the data via JNI. Includes unit tests for binding parsing.
- **GZIP Compression**: Implemented GZIP compression and decompression for single files (`gzip_compress`, `gzip_decompress` in `compression.rs`). Includes roundtrip unit tests.
- **ZIP Creation**: Implemented ZIP creation (`create_archive` in `archive.rs`). Supports compressing both directories (recursively) and single files. UI updated to include "Compress to ZIP" action. Includes unit tests for directory compression.
- **PDF Reordering**: Implemented page reordering logic (`reorder_pages` in `pdf.rs`) and UI integration. Users can now specify a new page order (e.g., "2, 1, 3") to restructure PDF documents. Includes unit tests for logic verification.
- **ZIP Extraction**: Implemented full ZIP extraction ("Extract All" and single file "Extract") with directory traversal protection (Zip Slip). Added unit tests for path sanitization.
- **File Inspector**: Upgraded "File Info" to "File Inspector". Now includes a 512-byte hex dump preview and UTF-8 text detection check.
- **Refactoring Complete**: `lib.rs` size reduced by extracting UI rendering logic to feature modules (`misc_screens.rs`, `file_info.rs`, `hashes.rs`, `text_viewer.rs`, `sensor_logger.rs`) and shared helpers to `ui.rs`. Codebase is more modular and easier to maintain.
- App state/navigation: Rust owns a `Vec<Screen>` stack; hardware Back pops safely. Inline Back buttons present when depth > 1 (QR, text tools, archive, sensor logger, color, Kotlin image).
- Renderer now does basic diffing: stable node IDs (explicit `id`, `bind_key`, or action) allow view reuse and keyboard/focus stability instead of full `removeAllViews`.
- Text viewer: Prism-backed WebView (MIT assets bundled into `prism-bundle.min.js`), language guessing, wrap toggle, theme toggle, line numbers. Archive text entries open directly in viewer. Binary sniff with hex preview; chunked, windowed loads (128 KB) with next/prev and byte-offset jump.
- Tests: `cargo test` green. JNI guarded by `catch_unwind`; renderer validation in Kotlin prevents malformed payload crashes. Rust PDF loader uses `memmap2` to avoid loading whole files into heap.
- PDFs: Signature overlay uses `PdfSignPlacement` tap targets with normalized coords; signatures append to existing page content (no new pages). Temp outputs now prefer source directory (or Downloads/cache for SAF content) with `_modified_YYMMDDhhmm.pdf` suffix. SignaturePad disables parent scroll during draw.
- Save As: UI now offers “Save as…” for PDF/image outputs via ACTION_CREATE_DOCUMENT; Android copies the existing file into a user-chosen location.
- Text viewer binary guardrails: sniff first 4KB for binary/unsupported content; show hex preview and “Load anyway” instead of loading full file. Text loads still capped at 256KB.
 - Compass: single GLSL/GLSurfaceView implementation; driven by Kotlin sensors with throttled sync back to Rust; Rust state stores last angle/error for snapshots.
 - Barometer/Magnetometer: GLSL widgets fed by sensors with throttled Rust sync; error surfaced when sensors unavailable.
- DSL: Section/Card grouping widgets added to the DSL and renderer; menu now uses them for quick access and category grouping.

## Technical Debt & Issues (High Priority)
1. **Global Mutex Contention**: `STATE` mutex in `lib.rs` blocks all JNI calls during long-running ops.
   - *Action*: Refactor heavy tasks to spawn threads and update state via channels/local locks.
2. **JSON Overhead**: Full UI tree serialized on every update causes GC churn.
   - *Action*: Implement partial updates/diffing or separate data channels.
3. **Blocking I/O**: JNI calls block the thread.
   - *Action*: Move file I/O to a dedicated blocking thread pool.
4. **UI Scalability**: `LinearLayout` usage for lists risks OOM.
   - *Action*: Implement a JSON-backed `RecyclerView` adapter.

## Roadmap (Future Features)
- **Math/CAS**: Numeric solver and optional symbolic CAS.

## Immediate Focus
- Harden input UX: avoid spamming Rust on every character.
- On-device QA for text viewer (large logs), TalkBack, theme toggles.
- Ensure Back buttons stay wired for all nested flows.
- Robolectric coverage: add tests for `CodeView` and `PdfSignPlacement`.
- PDF UX: refine placement overlay.
- Rust core: mitigate long-held STATE mutex.
- Text viewer: polish chunked loads/paging.
- DSL grouping: add renderer tests.
- Compass/Barometer/Magnetometer: smoothing/filtering.

## Near-Term
- Schema hardening: typed builders, validation.
- Snapshot/restore: wire into CI.
- Packaging: verify arm64-only, clean ABIs.
- Sensor logger: QA permissions, GPS, CSV.
- Text viewer: WebView search bar.
- PDF UX: 3x3 placement grid.
- Outputs: “Save As” flow.
- Image tools: resize/quality controls.
- Hash verify: on-device QA.
- DSL polish: keep grouping readable.

## Notes
- Prism assets are MIT; license stored in `assets/prism/PRISM_LICENSE.txt`.
- Keep APK slim (<5 MB).***
