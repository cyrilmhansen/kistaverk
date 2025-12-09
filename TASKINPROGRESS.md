# Task In Progress: Search/Filtering within Tools

## Status: Implemented
*   **Date:** 2025-12-09
*   **Objective:** Implement search/filtering capabilities in the Archive Viewer to help users find specific files within large archives.
*   **Plan:**
    1.  **State (`rust/src/features/archive.rs`):**
        *   Update `ArchiveState` struct to include `filter_query: Option<String>`.
        *   Initialize `filter_query` to `None` in `ArchiveState::new`.
    2.  **Implementation (`rust/src/features/archive.rs`):**
        *   Update `render_archive_screen` to include a `TextInput` for `archive_filter`.
        *   Configure the `TextInput` with `debounce_ms` (e.g., 200ms) to trigger updates without explicit submission.
        *   Filter the displayed `entries` based on the `filter_query` before rendering the list.
        *   Ensure the "Extract" buttons use the correct index from the original (unfiltered) list or map indices correctly. *Correction:* Since `extract_entry` takes an index, we need to be careful. It might be safer to pass the entry name or ensure the UI index maps back to the `ZipArchive` index. However, `ZipArchive` works by index. A simple way is to store the original index in `ArchiveEntry`.
    3.  **Refactoring (`rust/src/features/archive.rs`):**
        *   Update `ArchiveEntry` struct to include `original_index: usize`.
        *   Update `read_archive_entries` to populate `original_index`.
        *   Update `extract_entry` calls in `render_archive_screen` to use `entry.original_index`.
    4.  **Integration (`rust/src/router.rs`):**
        *   Handle the binding update for `archive_filter` implicitly via state update (or explicit action if needed, but `TextInput` binding should suffice if we refresh the screen).
        *   Actually, `TextInput` updates `bindings` map in Kotlin, and we need an action to sync it to Rust state. We can use a generic "refresh" or specific "archive_filter_update" action triggered by `debounce_ms`.
        *   Let's add `ArchiveFilter { query: String }` to `Action` and `WorkerJob` is not needed if filtering happens in `render`. Wait, `render` is pure function of `state`. We need to update `state.archive.filter_query`.
        *   So, add `Action::ArchiveFilter` which updates state and re-renders.
    5.  **Tests:**
        *   **Filtering Logic:** Create a test with a mock `ArchiveState` containing multiple entries, apply a filter, and verify the rendered list contains only matches.
        *   **Index Integrity:** Verify that extracting a filtered item uses the correct original index.

## Previous Task: Batch Processing (Images & PDFs)
*   **Status:** Implemented
*   **Date:** 2025-12-09
*   **Summary:** Implemented batch processing for images and PDFs. Added multi-file picking, queues in state, and batch operations in Rust core.