# WORK IN PROGRESS

Keep this file short and actionable. Update it at the end of each session.

## Status (2025-12-09)
- **GZIP Save As**: Implemented "Save As" flow for GZIP compression. Standardized worker output to "Result saved to: ..." and updated Android layer to dynamically guess MIME types (including .gz) and launch system file picker.
- **File Encryption (The Vault)**: Implemented secure file encryption/decryption using `age` crate. Added `VaultState`, `WorkerJob::Vault`, and updated UI protocol to support `password_mask` for secure input.
- **Batch Processing**: Implemented batch processing for Images (Resize/Convert) and PDFs (Merge). Added `allow_multiple_files` to `Button`, updated Android `OpenMultipleDocuments` handling, and added batch queues (`VirtualList`) in Rust state.
- **Logical Engine**: Implemented a lightweight RDF/logic module (`features/logic.rs`). Supports adding triples, importing from CSV, and querying with wildcards. Added `LogicState` and corresponding UI.
- **Archive Filtering**: Implemented search/filtering in Archive Viewer. Users can now filter the file list by name using a debounced text input.

## Status (2025-12-08)
- **Input Debouncing**: Implemented `debounce_ms` property for `TextInput` in `ui.rs` and applied it to `MathTool`, `TextViewer`, and `ArchiveViewer`.
- **Symbolic CAS**: Implemented symbolic differentiation in the Math Tool.
- **Refine PDF Placement Overlay**: Updated PDF signature tool to respect page aspect ratio.
- **Offload Blocking File I/O**: Moved heavy I/O to background worker.
- **WebView Text Search**: Implemented search bar in Text Viewer.
- **Sensor Smoothing**: Implemented Low-Pass Filters for sensors.
- **Math Expression Evaluator**: Implemented parser/evaluator for math expressions.
- **16KB Alignment**: Enforced 16KB alignment for Android 15.
- **Image Tools**: Implemented Image Converter and Resizer (Hybrid Rust/Kotlin).
- **Camera Scanning**: Implemented QR scanning with `rxing`.
- **QR Data Transfer**: Implemented file transfer protocol (Sender/Receiver).
- **Presets**: Implemented settings persistence.
- **System Panels**: Implemented system info dashboard.
- **Compression**: Implemented GZIP/ZIP tools.
- **PDF Tools**: Implemented Split, Merge, Sign, Reorder.
- **File Inspector**: Implemented binary/text inspection.

## Technical Debt & Issues (High Priority)
1. **JSON Overhead**: Full UI tree serialized on every update causes GC churn.
   - *Action*: Implement partial updates/diffing or separate data channels.
2. **UI Scalability**: `LinearLayout` usage for lists risks OOM.
   - *Action*: Implement a JSON-backed `RecyclerView` adapter.
3. **CSV Parsing**: Naive split(',') in Logic Engine doesn't handle quoted fields.

## Roadmap (Future Features)
- **Symbolic Integration**: Extend math tool to support basic integration.
- **Search/Filtering**: Extend filtering to other lists (e.g., dependency list).

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
