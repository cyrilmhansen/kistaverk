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

## Phase 3: Cleanup & Validation (TODO)
**Objective:** Remove dead code and ensure long-term maintainability.

1.  **Remove Dead Code:**
    *   Files `rust/src/features/performance_analysis.rs` and `rust/src/features/visualization.rs` appear to be unused stubs. They should be either integrated or deleted to avoid confusion.
    *   The 13 compiler warnings (dead code/unused variables) identified during `cargo check` should be resolved.

2.  **Enhance Plotting Integration:**
    *   Currently, plotting saves to a CSV. It should be integrated with the app's native plotting screen (`Screen::Plotting`) to automatically load and display the generated data.

3.  **Unit Tests:**
    *   Add comprehensive unit tests for nested expressions (e.g., `sin(x*x + 1)`) to ensure the new AST parser/generator covers all edge cases.
