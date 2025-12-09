# Task In Progress: Standardize "Save As" Flow (GZIP)

## Status: Implemented
*   **Date:** 2025-12-09
*   **Objective:** Implement a "Save As" flow for the GZIP Compression tool (and standardize the mechanism) to allow users to save processed files to a location of their choice. Currently, results are saved to an internal directory without a convenient way to export them.
*   **Plan:**
    1.  **Kotlin (`MainActivity.kt`):**
        *   **Fix `cacheLastResult`**: Update the parsing logic for "Result saved to:" to use `guessMimeFromPath(path)` instead of hardcoding "application/pdf".
        *   **Update `guessMimeFromPath`**: Add support for `.gz` ("application/gzip") and potentially other common types to ensure correct MIME handling.
        *   **Handle Action**: Add a handler for `gzip_save_as` in the `UiRenderer` callback. It should call `launchSaveAs(lastFileOutputPath, lastFileOutputMime ?: "application/gzip")`.
    2.  **Rust (`router.rs`):**
        *   Update `WorkerJob::Compression` output to prefix the result path with "Result saved to: ". This triggers the path capture in Kotlin.
    3.  **Rust (`features/compression.rs`):**
        *   Update `render_compression_screen` to conditionally display a "Save as…" button (action: `gzip_save_as`) when `state.compression_status` contains "Result saved to:".
    4.  **Tests:**
        *   **Rust UI Test**: Add a test in `features/compression.rs` to verify that the `gzip_save_as` button is rendered when the status message indicates a success.
        *   **MIME Resolution**: Verify `guessMimeFromPath` correctly identifies `.gz` files.

## Previous Task: Input Debouncing
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Implemented `debounce_ms` for `TextInput` and applied it to Math Tool and Text Viewer.
*   **Note:** Needs a unit test for serialization.
