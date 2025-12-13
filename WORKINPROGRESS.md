# WORK IN PROGRESS

Keep this file short and actionable. Update it at the end of each session.

## Status (2025-12-12)
- **Regex Tester Enhancements**: Added Global Search and Common Patterns (`features/regex_tester.rs`).
- **Unit Converter**: Implemented core logic, UI, state, and unit tests (`features/unit_converter.rs`).
- **Symbolic Integration**: Merged extensions for `exp`, `tan`, `atan` in `math_tool.rs`.
- **Cron/Task Scheduler**: Implemented core logic, UI, state, and unit tests (`features/scheduler.rs`).
- **Embedded Scripting (Rhai)**: Implemented "Scripting Lab" (`features/scripting.rs`).
- **SQL Query Lab**: Implemented `features/sql_engine.rs`.
- **Dependency List**: Implemented searching and filtering (`features/dependencies.rs`).

## Status (2025-12-11)
- **Preset Filtering**: Added real-time filtering to the Preset Manager.

## Technical Debt & Issues (High Priority)
1. **JSON Overhead**: Full UI tree serialized on every update causes GC churn.
   - *Action*: Implement partial updates/diffing or separate data channels.
2. **UI Scalability**: `LinearLayout` usage for lists risks OOM.
   - *Action*: Implement a JSON-backed `RecyclerView` adapter.
3. **CSV Parsing**: Naive split(',') in Logic Engine doesn't handle quoted fields.

## Roadmap (Future Features)
- **Advanced CAS**: Integrate `symbolica` for precision math and float error reduction.
- **Search/Filtering**: Extend filtering to other lists.

## Immediate Focus
- Harden input UX: avoid spamming Rust on every character.

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

