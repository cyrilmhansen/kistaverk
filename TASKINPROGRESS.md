# Task In Progress: Offload Blocking File I/O to Worker Thread Pool

## Status: Planning
*   **Date:** 2025-12-08
*   **Objective:** Eliminate UI freezes and ANRs caused by synchronous file I/O operations on the main JNI thread by moving them to the asynchronous worker thread pool.
*   **Plan:**
    1.  **Identify Blocking I/O Hotspots:**
        *   Review functions like `handle_pdf_select`, `handle_file_info`, `handle_text_viewer_open`, `handle_pdf_title`, `handle_pdf_sign` for direct file reads/writes.
    2.  **Extend Worker API (`router.rs`):**
        *   Add new `WorkerJob` variants for each identified blocking I/O task (e.g., `WorkerJob::LoadPdf`, `WorkerJob::GetFileInfo`, `WorkerJob::LoadTextFile`, `WorkerJob::SetPdfTitle`, `WorkerJob::SignPdf`).
        *   These jobs will encapsulate all necessary parameters (file descriptors, paths, binding values, etc.).
        *   Add corresponding `WorkerResult` variants to return the operation's outcome (success, error, data).
    3.  **Implement Worker Job Execution (`run_worker_job` in `router.rs`):**
        *   Add `match` arms for the new `WorkerJob` variants.
        *   Execute the actual blocking file I/O operations within these `match` arms.
        *   Return the outcome wrapped in the appropriate `WorkerResult`.
    4.  **Modify UI-Facing Action Handlers (`router.rs` and feature modules):**
        *   For actions that trigger these blocking I/O operations, change them to:
            *   Enqueue the relevant `WorkerJob` using `STATE.worker().enqueue()`.
            *   Immediately set the UI state to `Screen::Loading` and display a loading message.
            *   Return the loading UI.
        *   Remove direct calls to blocking I/O functions from these handlers.
    5.  **Update `apply_worker_results` (`router.rs`):**
        *   Add `match` arms for the new `WorkerResult` variants.
        *   Update the `AppState` with the results received from the worker.
        *   Transition the UI from `Screen::Loading` to the appropriate destination screen (e.g., `Screen::PdfTools`, `Screen::FileInfo`, `Screen::TextViewer`).
    6.  **Tests:**
        *   Add integration tests to verify that these previously blocking operations now correctly enqueue jobs, display loading screens, and update the UI asynchronously when results are ready. This will involve using the `TEST_FORCE_ASYNC_WORKER` and `TEST_WORKER_DELAY_MS` mechanisms.

## Previous Task: Sensor Data Smoothing
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Implemented Low-Pass Filters for Compass, Barometer, and Magnetometer sensor data to reduce UI jitter.