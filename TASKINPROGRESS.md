# Task In Progress: Input Debouncing

## Status: Implemented
*   **Date:** 2025-12-08
*   **Objective:** Harden the input UX by introducing a `debounce_ms` property to the `TextInput` component.
*   **Plan:**
    1.  **Protocol Update (`ui.rs`):**
        *   Added `debounce_ms: Option<u32>` to the `TextInput` struct and its builder.
    2.  **Implementation (`features/text_viewer.rs`):**
        *   Applied `.debounce_ms(150)` to the Find query input.
    3.  **Implementation (`features/math_tool.rs`):**
        *   Applied `.debounce_ms(150)` to the Math expression input.
    4.  **Tests:**
        *   No new specific unit test for `debounce_ms` serialization was found. This should be added.

## Previous Task: PDF 3x3 Placement Grid (Refinement)
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Refined `PdfSignPlacement` to respect page aspect ratio and confirmed with integration tests.