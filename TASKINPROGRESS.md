# Task In Progress: ZIP Creation

## Feature Description
Implement the ability to create `.zip` archives. Users should be able to select a file or a directory (folder) and compress it into a standard ZIP file.

## Plan

### Step 1: Update Core Logic (`rust/src/features/archive.rs`)
*   **Goal:** Add compression logic.
*   **Actions:**
    1.  Implement `create_archive(source_path: &Path, output_path: &Path) -> Result<(), String>`.
    2.  Implement a recursive directory walker (using `std::fs`) to handle folder compression.
    3.  Use `zip::ZipWriter` with `zip::write::FileOptions` (Deflate compression).
    4.  Ensure relative paths in the ZIP are correct (stripping the absolute prefix of the source).

### Step 2: Integration (`rust/src/lib.rs`)
*   **Goal:** Expose the feature.
*   **Actions:**
    1.  Add `Action::ArchiveCompress { path, fd, ... }`.
    2.  Add "Compress to ZIP" entry to the `feature_catalog` (Category: Files).
    3.  In `handle_command`, for `ArchiveCompress`:
        *   Determine output filename (`<source>.zip`).
        *   Show `Screen::Loading`.
        *   Call `create_archive`.
        *   On success, open the *newly created archive* in the Archive Viewer (reusing `Action::ArchiveOpen`).

### Step 3: Testing
*   **Actions:**
    1.  Unit test: Create a ZIP from a temporary directory structure and verify its contents (using the existing `ZipArchive` reader logic).
    2.  Unit test: Create a ZIP from a single file.

---

## Completed Tasks
*   **PDF Page Reordering**: Done.
*   **ZIP Extraction**: Done.
*   **File Inspector**: Done.
*   **Dithering Tools**: Done.
*   **Multi-hash view**: Done.
*   **Refactoring lib.rs**: Done.
*   **PDF Thumbnail Grid**: Rust side done (Kotlin pending).
*   **Pixel Art Mode**: Implemented (Rust Done, Kotlin Pending).
*   **Regex Tester**: Implemented (Rust Done).
*   **UUID/Random**: Implemented (Rust Done).