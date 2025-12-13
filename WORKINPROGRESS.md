# WORK IN PROGRESS

Keep this file short and actionable. Update it at the end of each session.

## Status (2025-12-13)
- **Advanced CAS**: Completed Phase 3 (Android Integration). Implemented build scripts, `build.rs` logic, and Gradle tasks for precision builds.
- **Math Tool**: Added cumulative floating-point error display.
- **Test Coverage**: Added cumulative FP error tests (currently orphaned).
- **ARM64 Optimizations**: Implemented NEON SIMD support and build targets.
- **Documentation**: Updated `TASKINPROGRESS.md` with AI Agent Protocol.

## Status (2025-12-12)
- **Regex Tester Enhancements**: Added Global Search and Common Patterns (`features/regex_tester.rs`).
- **Unit Converter**: Implemented core logic, UI, state, and unit tests (`features/unit_converter.rs`).
- **Symbolic Integration**: Merged extensions for `exp`, `tan`, `atan` in `math_tool.rs`.
- **Cron/Task Scheduler**: Implemented core logic, UI, state, and unit tests (`features/scheduler.rs`).
- **Embedded Scripting (Rhai)**: Implemented "Scripting Lab" (`features/scripting.rs`).
- **SQL Query Lab**: Implemented `features/sql_engine.rs`).
- **Dependency List**: Implemented searching and filtering (`features/dependencies.rs`).

## Status (2025-12-11)
- **Preset Filtering**: Added real-time filtering to the Preset Manager.

## Technical Debt & Issues (High Priority)
1. **Test Integration**: `rust/src/features/math_tool_test.rs` is not part of the module tree and is not executing.
2. **JSON Overhead**: Full UI tree serialized on every update causes GC churn.
   - *Action*: Implement partial updates/diffing or separate data channels.
3. **UI Scalability**: `LinearLayout` usage for lists risks OOM.
   - *Action*: Implement a JSON-backed `RecyclerView` adapter.
4. **CSV Parsing**: Naive split(',') in Logic Engine doesn't handle quoted fields.

## Roadmap (Future Features)
- **Search/Filtering**: Extend filtering to other lists.

## Immediate Focus
- Integrate `math_tool_test.rs` into the build or move tests to `math_tool.rs`.
- Verify Android build with precision enabled.

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
