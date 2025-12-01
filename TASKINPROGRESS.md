# Task In Progress: GZIP Compression

## Feature Description
Implement GZIP compression and decompression for single files. This allows users to compress a file into a `.gz` archive and restore it.

## Plan

### Step 1: Dependencies (`rust/Cargo.toml`)
*   **Goal:** Enable GZIP support.
*   **Actions:**
    1.  Add `flate2 = "1.0"` to dependencies.

### Step 2: Core Logic (`rust/src/features/compression.rs`)
*   **Goal:** Implement compression logic.
*   **Actions:**
    1.  Create `compression.rs`.
    2.  Implement `gzip_compress(source: &Path) -> Result<PathBuf, String>`.
        *   Use `flate2::write::GzEncoder`.
        *   Target: `source.gz` (in output dir).
    3.  Implement `gzip_decompress(source: &Path) -> Result<PathBuf, String>`.
        *   Use `flate2::read::GzDecoder`.
        *   Target: Strip `.gz` extension or append `_decoded`.

### Step 3: Integration (`rust/src/lib.rs` & `rust/src/features/mod.rs`)
*   **Goal:** Expose actions.
*   **Actions:**
    1.  Register module in `mod.rs`.
    2.  Add `Action::GzipCompress` and `Action::GzipDecompress` in `lib.rs`.
    3.  Add menu entries to "Files" or "Utilities" category.

### Step 4: Testing
*   **Actions:**
    1.  Unit test: Roundtrip compression/decompression of a sample text file.

---

## Completed Tasks
*   **ZIP Creation**: Done.
*   **PDF Reordering**: Done.
*   **ZIP Extraction**: Done.
*   **File Inspector**: Done.
*   **Dithering Tools**: Done.
*   **Multi-hash view**: Done.
*   **Refactoring lib.rs**: Done.
*   **PDF Thumbnail Grid**: Rust side done (Kotlin pending).
*   **Pixel Art Mode**: Implemented (Rust Done, Kotlin Pending).
*   **Regex Tester**: Implemented (Rust Done).
*   **UUID/Random**: Implemented (Rust Done).
