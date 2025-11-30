# Task In Progress: Refactoring lib.rs

## Goal
Reduce the size of `rust/src/lib.rs` (currently ~3000 lines) by extracting UI rendering logic into feature modules. This improves build times, code navigability, and separation of concerns.

## Plan

### Step 1: Move Shared UI Helpers - **Completed**
*   **Target:** `rust/src/ui.rs`
*   **Action:** Move `maybe_push_back` and `format_bytes` from `lib.rs` to `ui.rs`. Make them public.

### Step 2: Create `features/misc_screens.rs` - **Completed**
*   **Target:** `rust/src/features/misc_screens.rs`
*   **Action:** Create this module. Move the following from `lib.rs`:
    *   `render_about_screen`
    *   `render_progress_demo_screen`
    *   `render_compass_screen`
    *   `render_barometer_screen`
    *   `render_magnetometer_screen`
    *   `render_shader_screen` and `SAMPLE_SHADER`
    *   `render_loading_screen`
*   **Register:** Add `pub mod misc_screens;` to `rust/src/features/mod.rs`.

### Step 3: Move Feature-Specific Screens - **Completed**
*   **Target:** Existing feature modules.
*   **Action:** Move functions from `lib.rs`:
    *   `render_file_info_screen` -> `rust/src/features/file_info.rs`
    *   `render_hash_verify_screen` -> `rust/src/features/hashes.rs`
    *   `render_text_viewer_screen` -> `rust/src/features/text_viewer.rs`
    *   `render_sensor_logger_screen` -> `rust/src/features/sensor_logger.rs`

### Step 4: Update `lib.rs` - **Completed**
*   **Target:** `rust/src/lib.rs`
*   **Action:**
    *   Remove the moved functions and constants.
    *   Add necessary imports (`use crate::ui::{maybe_push_back, format_bytes};`, `use features::misc_screens::{...};`, etc.).
    *   Update `render_ui` match arms to point to the new locations.

### Step 5: Verify - **Completed**
*   **Action:** Run `cargo check` and `cargo test` to ensure everything still compiles and links correctly.