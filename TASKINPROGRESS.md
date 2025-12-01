# Task In Progress: ZIP Extraction

## Feature Description
Enhance the "Archive Viewer" (currently read-only) to support extracting files. Users should be able to extract all files or selected individual files to a target directory.

## Plan

### Step 1: Update Core Logic (`rust/src/features/archive.rs`)
*   **Goal:** Add extraction logic.
*   **Actions:**
    1.  Implement `extract_all(archive_path: &str, output_dir: &str) -> Result<String, String>`.
        *   Iterate over all entries.
        *   Sanitize paths (zip slip protection).
        *   Write files to `output_dir`.
    2.  Implement `extract_entry(archive_path: &str, entry_index: usize, output_dir: &str) -> Result<String, String>`.
        *   Extract a single file.

### Step 2: Update UI Rendering (`rust/src/features/archive.rs`)
*   **Goal:** Add "Extract" controls.
*   **Actions:**
    1.  Add a global "Extract All" button to the header.
    2.  Add "Extract" button next to each file entry (or a context menu equivalent if space permits, for now just a button or rely on "Extract All" for MVP).
    *   *Decision*: For mobile UI, an "Extract All" button is high value. For individual files, maybe "Open" (view text) vs "Extract" (save) is the distinction.
    *   Let's add an "Extract" button to each row.

### Step 3: Integration (`rust/src/lib.rs`)
*   **Goal:** Wire up actions.
*   **Actions:**
    1.  Add `Action::ArchiveExtractAll { output_dir: Option<String> }`.
    2.  Add `Action::ArchiveExtractEntry { index: usize, output_dir: Option<String> }`.
    3.  Handle "Extract" actions by resolving an output directory (using `storage::output_dir_for` or similar) and calling the logic.

### Step 4: Testing
*   **Actions:**
    1.  Unit test the extraction logic (using a temp dir).
    2.  Test zip slip vulnerability prevention (path traversal).

---

## Completed Tasks
*   **File Inspector**: Done.
*   **Dithering Tools**: Done.
*   **Multi-hash view**: Done.
*   **Refactoring lib.rs**: Done.
*   **PDF Thumbnail Grid**: Rust side done (Kotlin pending).
*   **Pixel Art Mode**: Implemented (Rust Done, Kotlin Pending).
*   **Regex Tester**: Implemented (Rust Done).
*   **UUID/Random**: Implemented (Rust Done).