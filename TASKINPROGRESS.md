# Task In Progress: Dithering Tools

## Feature Description
Implement a "Dithering Tool" that allows users to apply various dithering algorithms (e.g., Floyd-Steinberg, Bayer) and retro color palettes (e.g., Monochrome, CGA, Game Boy) to images. The tool will process images locally in Rust and allow users to save or share the result.

## Plan

### Step 1: Update Rust State Management (`rust/src/state.rs`)
*   **Goal:** support the dithering screen state.
*   **Actions:**
    1.  Add `Screen::Dithering` to the `Screen` enum.
    2.  Define `DitheringMode` enum (FloydSteinberg, Bayer4x4, Bayer8x8).
    3.  Define `DitheringPalette` enum (Monochrome, Cga, GameBoy).
    4.  Add `dithering_source_path`, `dithering_result_path`, `dithering_mode`, `dithering_palette`, and `dithering_error` fields to `AppState`.
    5.  Initialize these fields in `AppState::new()` and `AppState::reset_runtime()`.

### Step 2: Implement Core Dithering Logic (`rust/src/features/dithering.rs`)
*   **Goal:** Implement the image processing algorithms.
*   **Actions:**
    1.  Create `rust/src/features/dithering.rs`.
    2.  Implement palette quantization logic (finding nearest color).
    3.  Implement dithering algorithms:
        *   **Floyd-Steinberg**: Error diffusion.
        *   **Ordered**: Using Bayer matrices.
    4.  Implement `process_dithering(path: &str, mode: DitheringMode, palette: DitheringPalette) -> Result<String, String>`:
        *   Load image using `image` crate.
        *   Apply selected algorithm and palette.
        *   Save result to a temporary file (e.g., in cache dir).
        *   Return the path to the saved file.

### Step 3: Implement UI Rendering (`rust/src/features/dithering.rs`)
*   **Goal:** Create the UI for the tool.
*   **Actions:**
    1.  Implement `render_dithering_screen(state: &AppState) -> Value`.
    2.  UI Components:
        *   Title "Retro Dithering".
        *   "Pick Image" button (`requires_file_picker: true`).
        *   Selection controls for Mode (Algorithm) and Palette.
        *   "Apply" button to trigger processing (if an image is selected).
        *   Display of the processed image result (path/status).
        *   "Save/Share" button for the result.

### Step 4: Integrate into JNI Dispatch (`rust/src/lib.rs`)
*   **Goal:** Connect UI actions to logic.
*   **Actions:**
    1.  Add new `Action` variants: `DitheringScreen`, `DitheringPickImage`, `DitheringSetMode`, `DitheringSetPalette`, `DitheringApply`.
    2.  Update `parse_action` to map JSON strings to these variants.
    3.  Update `handle_command`:
        *   `DitheringScreen`: Reset state, push screen.
        *   `DitheringPickImage`: Update source path.
        *   `DitheringSetMode/Palette`: Update state options.
        *   `DitheringApply`: Call `process_dithering` (potentially with `loading_only` pattern).
    4.  Update `render_ui` to match `Screen::Dithering`.
    5.  Add the feature to `feature_catalog`.

### Step 5: Dependency Check (`rust/Cargo.toml`) - **Completed**
*   **Goal:** Ensure image processing libraries are available.
*   **Actions:**
    1.  Verified `image` crate is present (v0.24) with `png`, `jpeg`, `webp` features enabled.

### Step 6: Testing & Validation
*   **Goal:** Verify functionality.
*   **Actions:**
    1.  Write unit tests for dithering logic in `dithering.rs`.
        *   **Test:** Verify palette quantization (e.g., closest color to pure white is white).
        *   **Test:** Verify output dimensions match input dimensions.
        *   **Test:** Check behavior with 0-size or invalid images (should return error, not panic).
    2.  Manual testing on device:
        *   Load image -> Select algorithm/palette -> Apply.
        *   **Edge Case:** Test with a very large image (e.g., camera photo) to ensure no OOM or excessive lag (consider resizing if > 2048px).
        *   **Edge Case:** Test with non-image file (should show error).
        *   Verify visual output matches expectations (retro look).
        *   Verify saving/sharing works.
