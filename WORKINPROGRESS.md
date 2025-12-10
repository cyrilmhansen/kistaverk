# WORK IN PROGRESS

Keep this file short and actionable. Update it at the end of each session.

## Status (2025-12-11)
- **Cron/Task Scheduler**: Implemented core logic, UI, and state (`features/scheduler.rs`). Pending unit tests.
- **Embedded Scripting (Rhai)**: Implemented "Scripting Lab" (`features/scripting.rs`). Users can write and execute Rhai scripts with a custom UI (editor + console).
- **SQL Query Lab**: Implemented `features/sql_engine.rs` using SQLite. Supports importing CSV/JSON files as tables and running SQL queries.
- **Dependency List**: Moved dependency list rendering to Rust (`features/dependencies.rs`), enabling instant search/filtering of open-source licenses.
- **Preset Filtering**: Added real-time filtering to the Preset Manager.

## Status (2025-12-09)
- **JWT Decoder**: Implemented JWT Decoder in `features/jwt.rs`.
- **GZIP Save As**: Implemented "Save As" flow for GZIP compression.
- **File Encryption (The Vault)**: Implemented secure file encryption/decryption using `age` crate.
- **Batch Processing**: Implemented batch processing for Images (Resize/Convert) and PDFs (Merge).
- **Logical Engine**: Implemented a lightweight RDF/logic module (`features/logic.rs`).
- **Archive Filtering**: Implemented search/filtering in Archive Viewer.

## Technical Debt & Issues (High Priority)
1. **Missing Tests**: Scheduler feature (`features/scheduler.rs`) lacks unit tests.
   - *Action*: Implement `mod tests` for scheduler logic and runtime.
2. **JSON Overhead**: Full UI tree serialized on every update causes GC churn.
   - *Action*: Implement partial updates/diffing or separate data channels.
3. **UI Scalability**: `LinearLayout` usage for lists risks OOM.
   - *Action*: Implement a JSON-backed `RecyclerView` adapter.
4. **CSV Parsing**: Naive split(',') in Logic Engine doesn't handle quoted fields.

## Roadmap (Future Features)
- **Symbolic Integration**: Extend math tool to support basic integration.
- **Search/Filtering**: Extend filtering to other lists (e.g., dependency list).

## Immediate Focus
- **Testing**: Add unit tests for Scheduler.
- Harden input UX: avoid spamming Rust on every character.
- On-device QA for text viewer (large logs), TalkBack, theme toggles.
- Ensure Back buttons stay wired for all nested flows.
- Robolectric coverage: add tests for `CodeView` and `PdfSignPlacement`.

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
