# Task In Progress: Global Mutex Contention Refactoring

## Status: Planning
*   **Date:** 2025-12-06
*   **Objective:** Refactor `lib.rs` to prevent the global `STATE` mutex from blocking JNI calls during long-running operations.
*   **Plan:**
    1.  **Analyze `lib.rs`:** Identify the current usage of `STATE.lock()` and pin-point heavy operations (Hashing, PDF Merge, Zip).
    2.  **Refactor `GlobalState`:**
        *   Introduce an asynchronous command queue (`mpsc` or similar) for heavy tasks.
        *   Ensure the main JNI dispatch function returns immediately with a "Loading" or "Pending" state after enqueueing a job.
    3.  **Worker Thread:**
        *   Update the worker thread to process jobs.
        *   Crucially: Ensure the worker only locks `STATE` *briefly* to write results, not during the computation.
    4.  **UI Feedback:**
        *   Implement a polling or signaling mechanism so the Kotlin layer knows when to re-fetch the state (using the existing `auto_refresh` or similar).
    5.  **Verification:**
        *   Add a test case simulating concurrent JNI calls to verify the main thread is not blocked.

## Last Completed Task: 16KB Page Alignment
*   **Status:** Implemented & Reviewed.
*   **Date:** 2025-12-06
*   **Summary:** Updated `app/app/build.gradle.kts` to enforce 16KB page alignment for native libraries (`-z max-page-size=16384`), satisfying Android 15+ requirements.
