# Task In Progress: PDF 3x3 Placement Grid

## Status: Planning
*   **Date:** 2025-12-08
*   **Objective:** Implement a visual 3x3 grid of buttons for quick PDF signature placement (Top-Left, Top-Center, etc.), as requested in `WORKINPROGRESS.md` ("PDF UX: 3x3 placement grid").
*   **Plan:**
    1.  **Refine UI (`features/pdf.rs`):**
        *   The current `render_pdf_screen` already contains a basic implementation of a 3x3 grid.
        *   I will review it to ensure it uses the `Grid` component effectively and that the actions (`pdf_sign_grid`) are correctly mapped.
        *   I will verify that the layout looks like a grid (using `Grid` type with `columns: 3`).
    2.  **Action Handling (`router.rs`):**
        *   The `Action::PdfSignGrid` variant exists.
        *   I will ensure the handler correctly updates `signature_x_pct`, `signature_y_pct`, and `signature_target_page`.
    3.  **Feedback Loop (`features/pdf.rs`):**
        *   Ensure that selecting a grid position updates the `PdfSignPlacement` and `PdfSignPreview` components to reflect the new coordinates immediately.
    4.  **Tests:**
        *   Add a test case in `router.rs` or `pdf.rs` to verify that `pdf_sign_grid` action updates the state coordinates correctly (e.g., Top-Left -> 0.1, 0.1).

## Previous Task: Symbolic Differentiation (CAS)
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Added symbolic differentiation support to the Math Tool.