# Task In Progress: Symbolic Differentiation (CAS)

## Status: Implemented
*   **Date:** 2025-12-08
*   **Objective:** Extend the Math Tool to support symbolic differentiation (Computer Algebra System).
*   **Plan:**
    1.  **Data Structures (`features/math_tool.rs`):**
        *   Added `Symbol` enum for AST.
    2.  **Parsing Logic (`features/math_tool.rs`):**
        *   Implemented `tokenize` (updated for variables), `shunting_yard`, and `rpn_to_symbol` to build AST.
    3.  **Differentiation Logic (`features/math_tool.rs`):**
        *   Implemented `differentiate` with Power, Chain, Product, Quotient rules, and standard function derivatives.
    4.  **Simplification Logic (`features/math_tool.rs`):**
        *   Implemented `simplify` to fold constants and remove identity ops (x*1, x+0, etc.).
    5.  **Formatting (`features/math_tool.rs`):**
        *   Implemented `render_symbol` for string output.
    6.  **Integration (`features/math_tool.rs`):**
        *   Updated `evaluate_expression` to intercept `deriv(...)` calls.
    7.  **Tests:**
        *   Added unit tests for symbolic differentiation rules (polynomial, trig, chain rule) and simplification.

## Previous Task: Refine PDF Placement Overlay
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Updated `PdfState` to store page aspect ratio and pass it to the UI for accurate placement markers. `load_pdf_metadata` now returns aspect ratio.
