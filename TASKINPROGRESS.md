# Task In Progress: File Inspector

## Feature Description
Enhance the existing "File Info" tool into a robust "File Inspector". In addition to basic size and MIME type, it will provide a hex dump preview of the file header and a UTF-8 validation check to determine if the file is text-safe.

## Plan

### Step 1: Update Core Logic (`rust/src/features/file_info.rs`)
*   **Goal:** Extend file analysis capabilities.
*   **Actions:**
    1.  Update `FileInfoResult` struct to include:
        *   `hex_dump: Option<String>`
        *   `is_utf8: Option<bool>`
    2.  Modify `info_from_reader` (or create `inspect_reader`):
        *   Read the first 512 bytes.
        *   Generate a hex dump string (Offset | Hex | ASCII).
        *   Check if the read buffer is valid UTF-8.
        *   (Optimization) For UTF-8 check, if the file is small (< 64KB), read all and check; if large, checking the head is a good heuristic for "starts with text", or we can scan the whole file efficiently if needed. For now, header check + extension heuristic is a good start for a "preview".

### Step 2: Update UI Rendering (`rust/src/features/file_info.rs`)
*   **Goal:** Display the new detailed info.
*   **Actions:**
    1.  Update `render_file_info_screen`.
    2.  Add a `UiCodeView` or `UiText` section for the `hex_dump`.
    3.  Add a status label for `is_utf8` (e.g., "Content: Text (UTF-8)" vs "Content: Binary/Unknown").
    4.  Organize layout: File Path -> Meta (Size/MIME/Type) -> Preview (Hex).

### Step 3: Integration (`rust/src/lib.rs`)
*   **Goal:** Expose the new functionality.
*   **Actions:**
    1.  Rename "File info" to "File Inspector" in `feature_catalog`.
    2.  Ensure `handle_command` calls the updated logic for `Action::FileInfo`.

### Step 4: Testing
*   **Actions:**
    1.  Unit test the hex dump formatter.
    2.  Unit test the UTF-8 check with valid text, invalid text, and binary data.
    3.  Manual test with various file types (images, text scripts, binaries).

---

## Completed Tasks
*   **Dithering Tools**: Done.
*   **Multi-hash view**: Done.
*   **Refactoring lib.rs**: Done.
*   **PDF Thumbnail Grid**: Rust side done (Kotlin pending).
*   **Pixel Art Mode**: Implemented (Rust Done, Kotlin Pending).
*   **Regex Tester**: Implemented (Rust Done).
*   **UUID/Random**: Implemented (Rust Done).