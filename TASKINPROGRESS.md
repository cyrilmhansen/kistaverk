# Task In Progress: Offload Blocking File I/O to Worker Thread Pool

## Status: Planning
*   **Date:** 2025-12-08
*   **Objective:** Eliminate UI freezes and ANRs caused by synchronous file I/O operations on the main JNI thread by moving them to the asynchronous worker thread pool.
*   **Plan:**
    1.  **Define Worker Jobs (`router.rs`):**
        *   Extend `WorkerJob` enum with:
            *   `PdfLoad { fd: i32, uri: Option<String> }`
            *   `PdfTitle { fd: i32, uri: Option<String>, title: String }`
            *   `PdfSign { ... }` (capture all sign args)
            *   `FileInfo { fd: Option<i32>, path: Option<String> }`
            *   `TextLoad { fd: Option<i32>, path: Option<String> }`
    2.  **Define Worker Results (`router.rs`):**
        *   Extend `WorkerResult` enum with corresponding results:
            *   `PdfLoaded { ... }`
            *   `PdfSaved { out_path: String }`
            *   `FileInfoReady { json: String }`
            *   `TextLoaded { content: String, ... }`
    3.  **Implement Execution Logic (`router.rs`):**
        *   In `run_worker_job`, move the body of `handle_pdf_select`, `handle_file_info`, etc., into the worker thread.
        *   Ensure file descriptors are correctly owned/closed by the worker.
    4.  **Update Action Handlers (`router.rs`, `features/*.rs`):**
        *   Refactor `Action::PdfSelect`, `Action::FileInfo`, etc., to:
            *   Enqueue the job.
            *   Set `state.loading_message`.
            *   Switch to `Screen::Loading`.
    5.  **Handle Results (`router.rs`):**
        *   In `apply_worker_results`, handle the new result variants to update state and switch screens (e.g., back to `PdfTools` or `FileInfo`).
    6.  **Verification:**
        *   Use `TEST_FORCE_ASYNC_WORKER` to verify the loading state appears and then resolves.

## Previous Task: WebView Text Search
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Implemented search bar in Text Viewer. Exposed `find_query` in JSON payload for Kotlin integration.
