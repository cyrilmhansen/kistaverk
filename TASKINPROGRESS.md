# Task In Progress: Symbolic Integration (Math Tool)

## Status: Planning
*   **Date:** 2025-12-08
*   **Objective:** Extend the Symbolic CAS in the Math Tool to support basic indefinite integration (anti-derivatives), as requested in `WORKINPROGRESS.md`.
*   **Plan:**
    1.  **Logic (`features/math_tool.rs`):**
        *   Implement `integrate(expr: &Symbol, var: &str) -> Symbol`.
        *   Support basic rules:
            *   Power rule: `x^n -> x^(n+1)/(n+1)` (handle `n=-1` case `1/x -> ln(x)`).
            *   Linearity: `integrate(a + b) -> integrate(a) + integrate(b)`.
            *   Trig functions: `sin(x) -> -cos(x)`, `cos(x) -> sin(x)`.
            *   Exponential: `e^x -> e^x` (represented as `exp` in parser if needed, or `e^x`).
        *   Update `evaluate_expression` to detect `integ(...)` calls.
    2.  **Parser Update (`features/math_tool.rs`):**
        *   Ensure `tokenize` handles `integ` keyword.
    3.  **Integration (`features/math_tool.rs`):**
        *   Wire `integ(...)` string inputs to the new `integrate` function.
        *   Return symbolic result string similar to `deriv`.
    4.  **Tests:**
        *   Unit tests for basic polynomial and trig integration.

## Previous Task: JSON-Backed RecyclerView Adapter
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Implemented `VirtualList` in JSON protocol and integrated it into the Math Tool history for scalable list rendering.