# Remediation Plan: Function Analysis & MIR Features

## Phase 1: Core Logic Repair (COMPLETED)
**Objective:** Replace hardcoded "fake" logic with functional recursive algorithms.

*   [x] **Audit `c_based_ad.rs`:** Identified hardcoded string matching.
*   [x] **Implement AST Parser:** Added `ExpressionAST` and a recursive descent parser (migrated/adapted from `automatic_differentiation.rs` logic).
*   [x] **Implement Recursive Code Gen:** Rewrote `generate_computation_graph` to traverse the AST and emit valid C code for dual numbers.
*   [x] **Restore Missing Function:** Restored `evaluate_derivative` which was accidentally removed during refactoring.

## Phase 2: UI Implementation (COMPLETED)
**Objective:** Make the "Function Analysis" screen actually functional.

*   [x] **Performance Analysis:** Implemented `handle_function_analysis_action` -> `function_analysis_performance` to run a 50-iteration benchmark.
*   [x] **Stability Test:** Implemented `function_analysis_stability` to check for numerical instability at edge case inputs.
*   [x] **Plotting:** Implemented `function_analysis_plot` to generate CSV data for the derivative.

## Phase 3: Cleanup & Validation (COMPLETED)
**Objective:** Remove dead code and ensure long-term maintainability.

1.  **Remove Dead Code:**
    *   [x] Files `rust/src/features/performance_analysis.rs` and `rust/src/features/visualization.rs` deleted.
    *   [x] Resolved compiler warnings in `mir_math.rs` and `c_based_ad.rs`.

2.  **Enhance Plotting Integration:**
    *   [x] Integrated `function_analysis_plot` with `Screen::Plotting`. It now automatically loads the generated CSV and displays the graph.

3.  **Unit Tests:**
    *   [x] Added `test_complex_nested_expression` to verify `sin(x*x + 1)`.
    *   [x] Verified all tests pass with `cargo test`.