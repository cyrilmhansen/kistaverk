# Task In Progress: UUID & Random String Generator

## Feature Description
Implement a tool for generating UUIDs (version 4) and random strings with configurable length and character sets. This provides a quick, offline way for developers and users to generate unique identifiers and passwords.

## Plan

### Step 1: Update Rust State (`rust/src/state.rs`)
*   **Goal:** Manage state for generated values and configuration.
*   **Actions:**
    1.  Add `Screen::UuidGenerator` to `Screen` enum.
    2.  Define `StringCharset` enum (Alphanumeric, Numeric, Alpha, Hex).
    3.  Create `UuidGeneratorState` struct:
        *   `last_uuid: Option<String>`
        *   `last_string: Option<String>`
        *   `string_length: u32` (default 16)
        *   `string_charset: StringCharset`
    4.  Add `uuid_generator: UuidGeneratorState` to `AppState`.
    5.  Initialize in `AppState::new()` and `reset_runtime()`.

### Step 2: Dependency Check (`rust/Cargo.toml`)
*   **Goal:** Ensure `uuid` and `rand` crates are available.
*   **Actions:**
    1.  Check `Cargo.toml` for `uuid` (with `v4`, `fast-rng` features) and `rand`.
    2.  Add them if missing.

### Step 3: Implement Core Logic (`rust/src/features/uuid_gen.rs`)
*   **Goal:** Implement generation logic.
*   **Actions:**
    1.  Create `rust/src/features/uuid_gen.rs`.
    2.  Implement `generate_uuid() -> String` using `uuid::Uuid::new_v4()`.
    3.  Implement `generate_string(len: usize, charset: StringCharset) -> String` using `rand`.
    4.  Implement `handle_uuid_action` to process generation requests and update state.

### Step 4: Implement UI Rendering (`rust/src/features/uuid_gen.rs`)
*   **Goal:** Create the user interface.
*   **Actions:**
    1.  Implement `render_uuid_screen(state: &AppState) -> Value`.
    2.  **UUID Section:**
        *   "Generate UUID v4" button.
        *   Result text + Copy button.
    3.  **Random String Section:**
        *   Text input for length (`bind_key: "uuid_str_len"`).
        *   Buttons/Checkbox to select charset.
        *   "Generate String" button.
        *   Result text + Copy button.

### Step 5: Integrate into JNI Dispatch (`rust/src/lib.rs`)
*   **Goal:** Connect actions.
*   **Actions:**
    1.  Add actions:
        *   `UuidScreen`
        *   `UuidGenerate`
        *   `RandomStringGenerate`
    2.  Update `parse_action` and `handle_command`.
        *   `RandomStringGenerate` should read bindings for length and charset preferences.
    3.  Update `render_ui`.
    4.  Add to `feature_catalog`.

### Step 6: Testing
*   **Actions:**
    1.  Unit test UUID generation (valid format).
    2.  Unit test string generation (correct length and charset).
    3.  Manual test on device.

---

## Completed Tasks
*   **Dithering Tools**: Done.
*   **Multi-hash view**: Done.
*   **Refactoring lib.rs**: Done.
*   **PDF Thumbnail Grid**: Rust side done (Kotlin pending).
*   **Pixel Art Mode**: Plan reviewed, pending implementation.
*   **Regex Tester**: Plan reviewed, pending implementation.
