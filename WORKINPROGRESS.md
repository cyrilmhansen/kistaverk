# WORK IN PROGRESS

Keep this file short and actionable. Update it at the end of each session.

## Status (2025-11-25)
- App state/navigation: Rust owns a `Vec<Screen>` stack; hardware Back pops safely. Inline Back buttons present when depth > 1 (QR, text tools, archive, sensor logger, color, Kotlin image).
- Renderer now does basic diffing: stable node IDs (explicit `id`, `bind_key`, or action) allow view reuse and keyboard/focus stability instead of full `removeAllViews`.
- Text viewer: Prism-backed WebView (MIT assets bundled into `prism-bundle.min.js`), language guessing, wrap toggle, theme toggle, line numbers. Archive text entries open directly in viewer.
- Tests: `cargo test` green. JNI guarded by `catch_unwind`; renderer validation in Kotlin prevents malformed payload crashes. Rust PDF loader uses `memmap2` to avoid loading whole files into heap.

## Immediate Focus
- Harden input UX: avoid spamming Rust on every character; consider focusing updates on submit/blur unless diffing is smarter. Verify keyboard doesn’t dismiss when background actions refresh UI.
- On-device QA for text viewer (large logs), TalkBack labels, theme/line-number toggles.
- Ensure Back buttons stay wired for all nested flows (QR, archive, text tools, sensors, color, Kotlin image); add guardrail for unsafe back pops.
- Robolectric coverage: add tests for `CodeView`/Prism payloads and Back button presence in QR screen.

## Near-Term
- Schema hardening: move UI generation fully to typed builders; expand renderer validation coverage.
- Snapshot/restore: wire into CI; keep native load mocked in tests.
- Packaging: verify arm64-only output, clean stale ABIs, track size (`scripts/size_report.sh`).
- Sensor logger: on-device QA for permissions, GPS intervals, CSV accuracy, TalkBack; plan Foreground Service with status indicator.
- Text viewer roadmap: pagination (“next chunk” read), WebView search bar via `findAllAsync` instead of Rust re-render.
- PDF UX: add 3x3 placement grid and thumbnail overlay preview for signature positioning.
- Outputs: offer ACTION_CREATE_DOCUMENT “Save As” for PDF/image results; keep power-user directory control.
- Image tools: add resize/quality controls (scale %, WebP/JPEG quality) to fit VISION’s mail-use case.
- DSL polish: add Section/Card grouping widget to improve readability; keep emoji iconography.

## Notes
- Prism assets are MIT; license stored in `assets/prism/PRISM_LICENSE.txt`.
- Keep APK slim (<5 MB): avoid adding heavy WebView deps or extra language packs; keep Prism component set minimal.***
