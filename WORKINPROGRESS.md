# WORK IN PROGRESS

Keep this file short and actionable. Update it at the end of each session.

## Status (2025-11-28)
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

## Immediate Focus
- Harden input UX: avoid spamming Rust on every character; consider focusing updates on submit/blur unless diffing is smarter. Verify keyboard doesn’t dismiss when background actions refresh UI.
- On-device QA for text viewer (large logs), TalkBack labels, theme/line-number toggles.
- Ensure Back buttons stay wired for all nested flows (QR, archive, text tools, sensors, color, Kotlin image, PDF sign placement); add guardrail for unsafe back pops.
- Robolectric coverage: add tests for `CodeView`/Prism payloads, PdfSignPlacement tap mapping, and Back button presence in QR screen.
- PDF UX: refine placement overlay (pinch-zoom? page thumbnails carousel) and consider auto-open viewer after save.
- Rust core: mitigate long-held STATE mutex (queue/channel, parking_lot + timeout) and migrate FDs to OwnedFd to avoid leaks.
- Text viewer: on-device QA for chunked loads/large logs; verify windowed paging + find UX stay stable; tune window size if needed.
- DSL grouping: add renderer tests for Section/Card and apply to more screens as we refactor layouts.
  - Robolectric coverage added for Section/Card validation and rendering.
 - Compass/Barometer/Magnetometer: consider smoothing/filtering and exposing calibration/error hints in UI; current sync is throttled to reduce redraw pulses.

## Near-Term
- Schema hardening: move UI generation fully to typed builders; expand renderer validation coverage.
- Snapshot/restore: wire into CI; keep native load mocked in tests.
- Packaging: verify arm64-only output, clean stale ABIs, track size (`scripts/size_report.sh`).
- Sensor logger: on-device QA for permissions, GPS intervals, CSV accuracy, TalkBack; Foreground Service with status indicator now in place.
- Text viewer roadmap: WebView search bar via `findAllAsync` instead of Rust re-render; polish chunked pagination UX (progress, keyboard focus).
- PDF UX: add 3x3 placement grid and thumbnail overlay preview for signature positioning.
- Outputs: offer ACTION_CREATE_DOCUMENT “Save As” for PDF/image results; keep power-user directory control.
- Image tools: add resize/quality controls (scale %, WebP/JPEG quality) to fit VISION’s mail-use case.
- DSL polish: keep Section/Card grouping widget readable; keep emoji iconography.

## Notes
- Prism assets are MIT; license stored in `assets/prism/PRISM_LICENSE.txt`.
- Keep APK slim (<5 MB): avoid adding heavy WebView deps or extra language packs; keep Prism component set minimal.***
