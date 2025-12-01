# Task In Progress: Pixel Art Mode

## Feature Description
Implement a "Pixel Art" tool that processes images to give them a retro, pixelated aesthetic. This is achieved by downscaling the image and then upscaling it back using nearest-neighbor interpolation.

## Plan

### Step 1: Update Rust State (`rust/src/state.rs`)
*   **Goal:** Manage state for the pixel art tool.
*   **Actions:**
    1.  Add `Screen::PixelArt` to the `Screen` enum.
    2.  Create a `PixelArtState` struct with fields:
        *   `source_path: Option<String>`
        *   `result_path: Option<String>`
        *   `scale_factor: u32` (default to 4 or 8)
        *   `error: Option<String>`
    3.  Add `pixel_art: PixelArtState` to `AppState`.
    4.  Initialize in `AppState::new()` and `reset_runtime()`.

### Step 2: Implement Core Logic (`rust/src/features/pixel_art.rs`)
*   **Goal:** Implement the image resizing logic.
*   **Actions:**
    1.  Create `rust/src/features/pixel_art.rs`.
    2.  Implement `process_pixel_art(path: &str, factor: u32) -> Result<String, String>`:
        *   Load image via `image` crate.
        *   Calculate new dimensions (width/factor, height/factor).
        *   `resize` to small dimensions using `FilterType::Nearest`.
        *   `resize` back to original dimensions using `FilterType::Nearest`.
        *   Save to temp file and return path.

### Step 3: Implement UI Rendering (`rust/src/features/pixel_art.rs`)
*   **Goal:** Create the user interface.
*   **Actions:**
    1.  Implement `render_pixel_art_screen(state: &AppState) -> Value`.
    2.  UI Elements:
        *   Header "Pixel Artifier".
        *   "Pick Image" button.
        *   Scale controls (Buttons for 2x, 4x, 8x, 16x).
        *   "Apply" button.
        *   Result preview/path.
        *   "Save" button.

### Step 4: Integrate into JNI Dispatch (`rust/src/lib.rs`)
*   **Goal:** Connect actions.
*   **Actions:**
    1.  Add actions:
        *   `PixelArtScreen`
        *   `PixelArtPick { path: Option<String>, fd: Option<i32> }`
        *   `PixelArtSetScale { scale: u32 }`
        *   `PixelArtApply`
    2.  Update `parse_action` and `handle_command`.
        *   `PixelArtApply` should probably use `loading_only: true`.
    3.  Update `render_ui` map.
    4.  Add to `feature_catalog`.

### Step 5: Testing
*   **Actions:**
    1.  Unit test `process_pixel_art` with various scenarios:
        *   **Standard image:** Verify output dimensions and visual correctness (e.g., specific pixel colors if possible).
        *   **Edge cases for `scale_factor`:** Test with `scale_factor = 1` (no change), and cases where `width / factor` or `height / factor` would result in a zero dimension or very small image.
        *   **Different image types:** Test with PNG, JPEG, WebP, including images with transparency (if supported).
        *   **Error handling:** Provide non-existent paths, corrupted images, or non-image files to ensure graceful error handling.
    2.  Manual test on device:
        *   Verify the full UI flow: picking image, setting scale, applying, previewing, and saving the result.
        *   **Performance:** Test with large images (e.g., high-resolution photos) to identify potential bottlenecks. Consider adding an automatic pre-resize step in Kotlin for images exceeding a certain dimension (e.g., 2048px on any side) to improve performance and avoid OOM errors.
        *   Verify output file size and quality.

---

## Completed Tasks
*   **Dithering Tools**: Done.
*   **Multi-hash view**: Done.
*   **Refactoring lib.rs**: Done.
*   **PDF Thumbnail Grid**: Rust side done (Kotlin pending).
*   **Pixel Art Mode**: Plan reviewed, pending implementation.
