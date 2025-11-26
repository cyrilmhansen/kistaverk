# WORK IN PROGRESS

Keep this file short and actionable. Update it at the end of each session.

## Status (2025-11-25)
- App state/navigation: Rust owns a `Vec<Screen>` stack; hardware Back pops safely. Inline Back buttons present when depth > 1 (QR, text tools, archive, sensor logger, color, Kotlin image).
- Text viewer: Prism-backed WebView (MIT assets in `assets/prism/`), language guessing, wrap toggle, theme toggle, line numbers. Archive text entries open directly in viewer.
- Tests: `cargo test` green. JNI guarded by `catch_unwind`; renderer validation in Kotlin prevents malformed payload crashes.

## Immediate Focus
- On-device QA for text viewer (large files, binary/UTF-8 errors, TalkBack labels, theme/line-number toggles).
- Ensure Back buttons stay wired for all nested flows (QR, archive, text tools, sensors, color, Kotlin image).
- Robolectric coverage: add tests for `CodeView`/Prism payloads and Back button presence in QR screen.

## Near-Term
- Schema hardening: move UI generation fully to typed builders; expand renderer validation coverage.
- Snapshot/restore: wire into CI; keep native load mocked in tests.
- Packaging: verify arm64-only output, clean stale ABIs, track size (`scripts/size_report.sh`).
- Sensor logger: on-device QA for permissions, GPS intervals, CSV accuracy, TalkBack.

## Notes
- Prism assets are MIT; license stored in `assets/prism/PRISM_LICENSE.txt`.
- Keep APK slim (<5 MB): avoid adding heavy WebView deps or extra language packs; keep Prism component set minimal.***
