# WORK IN PROGRESS

Keep this file short and actionable. Update it at the end of each session.

## Status (2025-12-08)
- **Input Debouncing**: Implemented `debounce_ms` property for `TextInput` in `ui.rs` and applied it to `MathTool` and `TextViewer` inputs.
- **Symbolic CAS**: Implemented symbolic differentiation in the Math Tool. Users can now compute derivatives (e.g., `deriv(x^2, x)` -> `2*x`) with simplification support.
- **Refine PDF Placement Overlay**: Updated PDF signature tool to respect page aspect ratio for accurate marker placement.
- **Offload Blocking File I/O**: Moved PDF loading, File Info, and Text View operations to a background worker thread to prevent UI freezes.
- **WebView Text Search**: Implemented search bar in Text Viewer. `render_text_viewer_screen` now exposes `find_query` in the JSON payload, allowing the Kotlin renderer to trigger `webView.findAllAsync()`. Added navigation controls (Next/Prev/Clear) and unit tests.
- **Sensor Smoothing**: Implemented Low-Pass Filters (alpha=0.2) for Compass (angular), Barometer, and Magnetometer to reduce jitter. Logic in `sensor_utils.rs` with unit tests; state persists across updates.
- **Math Expression Evaluator**: Implemented a parser and evaluator for mathematical expressions (`features/math_tool.rs`). Supports arithmetic, powers, and basic functions (`sin`, `cos`, `sqrt`, `log`). UI includes history tracking.
- **16KB Alignment (Android 15)**: Updated build config to enforce 16KB page alignment for native libraries.
- **Image Tools (Hybrid)**: Implemented Image Converter and Resizer. Rust manages UI/State; Kotlin handles image processing (Bitmap/Compress). Features: Format conversion (WebP/PNG/JPEG), Resizing (Scale/Quality), and Target Size capping. *Tests added: Rendering logic and State serialization verified.*
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
- **Refactoring Complete**: `lib.rs` size reduced by extracting UI rendering logic to feature modules. Codebase is more modular.

## Technical Debt & Issues (High Priority)
1. **JSON Overhead**: Full UI tree serialized on every update causes GC churn.
   - *Action*: Implement partial updates/diffing or separate data channels.
2. **UI Scalability**: `LinearLayout` usage for lists risks OOM.
   - *Action*: Implement a JSON-backed `RecyclerView` adapter.

## Roadmap (Future Features)
- **Symbolic Integration**: Extend math tool to support basic integration.

## Immediate Focus
- Harden input UX: avoid spamming Rust on every character.
- On-device QA for text viewer (large logs), TalkBack, theme toggles.
- Ensure Back buttons stay wired for all nested flows.
- Robolectric coverage: add tests for `CodeView` and `PdfSignPlacement`.
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
- Hash verify: on-device QA.
- DSL polish: keep grouping readable.

## Notes
- Prism assets are MIT; license stored in `assets/prism/PRISM_LICENSE.txt`.
- Keep APK slim (< 5 MB).