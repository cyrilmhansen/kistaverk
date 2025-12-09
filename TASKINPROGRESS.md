# Task In Progress: Batch Processing (Images & PDFs)

## Status: Implemented
*   **Date:** 2025-12-09
*   **Objective:** Enable batch processing for images (resizing/conversion) and PDFs (merging) to improve user efficiency. Currently, tools only handle single files.
*   **Plan:**
    1.  **UI Protocol (`rust/src/ui.rs`):**
        *   Update `Button` struct: Add `allow_multiple_files: Option<bool>`.
    2.  **Android Renderer (`MainActivity.kt`):**
        *   Update `pickFileLauncher`: Switch to `ActivityResultContracts.OpenMultipleDocuments()` when `allow_multiple_files` is true.
        *   Handle list of URIs in `handlePickerResult`.
        *   Update `dispatch` to send `path_list` (JSON array of strings) in extras when multiple files are picked.
    3.  **Rust State (`rust/src/state.rs`):**
        *   Update `KotlinImageState` to include `batch_queue: Vec<String>`.
        *   Update `PdfState` to include `merge_queue: Vec<String>`.
    4.  **Rust Logic:**
        *   **Images (`rust/src/features/kotlin_image.rs`):**
            *   Update `render_kotlin_image_screen`: Show a `VirtualList` of selected files if queue is not empty.
            *   Add "Process Batch" button.
            *   Implement batch processing loop in `handle_kotlin_image_result` or via a new worker job `BatchImageProcess`.
        *   **PDFs (`rust/src/features/pdf.rs`):**
            *   Update merge flow to accept a list of files instead of just primary/secondary.
            *   Update `PdfOperation::Merge` to take `Vec<i32>` (fds) or `Vec<String>` (uris).
    5.  **Integration (`rust/src/router.rs`):**
        *   Handle `path_list` in `Command` struct.
        *   Route batch selections to appropriate state fields.

## Previous Task: File Encryption/Decryption (The "Vault")
*   **Status:** Implemented
*   **Date:** 2025-12-09
*   **Summary:** Implemented secure file encryption/decryption using the `age` crate. Added `VaultState`, `WorkerJob::Vault`, and updated UI/Android layers to support password masking and "Save As".