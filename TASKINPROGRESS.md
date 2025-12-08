# Task In Progress: Input Debouncing

## Status: Planning
*   **Date:** 2025-12-08
*   **Objective:** Harden the input UX by introducing a `debounce_ms` property to the `TextInput` component. This instructs the renderer to delay sending state updates to Rust, reducing the frequency of JNI calls during rapid typing.
*   **Plan:**
    1.  **Protocol Update (`ui.rs`):**
        *   Add `debounce_ms: Option<u64>` to the `TextInput` struct.
        *   Add a builder method `debounce_ms(ms: u64)`.
    2.  **Implementation (`features/text_viewer.rs`):**
        *   Apply `.debounce_ms(300)` to the Find query input.
    3.  **Implementation (`features/math_tool.rs`):**
        *   Apply `.debounce_ms(300)` to the Math expression input.
    4.  **Tests:**
        *   Add a unit test in `ui.rs` (or `mod.rs`) to verify that `debounce_ms` is correctly serialized into the JSON payload.

## Previous Task: PDF 3x3 Placement Grid (Refinement)
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Refined `PdfSignPlacement` to respect page aspect ratio and confirmed with integration tests.