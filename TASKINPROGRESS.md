# Task In Progress

## Feature 1: Embedded Scripting (Rhai)
*   **Status:** 📅 PLANNED
*   **Date:** 2025-12-10
*   **Objective:** Integrate a lightweight scripting engine to allow users to write and execute custom automation scripts.
*   **Rationale:** Aligns with the "Automation" and "Embedded Scripting" goals in `VISION.md`. Empower power users to chain tools and perform custom logic.
*   **Implementation Plan:**
    1.  **Dependency:** Add `rhai` to `rust/Cargo.toml`.
    2.  **State:** Create `ScriptingState` in `state.rs` to hold script input, output log, and execution status.
    3.  **Core Logic:** Create `features/scripting.rs`.
        *   Initialize `rhai::Engine`.
        *   Register custom functions (e.g., `print`, `math`, maybe access to other Kistaverk tools if feasible).
        *   Implement `run_script` function to evaluate user input.
    4.  **UI:**
        *   Add a "Scripting Lab" screen.
        *   `TextInput` (multiline) for code editing.
        *   `Text` view for capturing stdout/result.
        *   `Button` to "Run".
    5.  **Integration:** Register in `router.rs`.

## Feature 2: Dependency List with Search
*   **Status:** 📅 PLANNED
*   **Date:** 2025-12-10
*   **Objective:** Implement a searchable, Rust-rendered list of open-source dependencies.
*   **Rationale:** Addresses "Search/Filtering: Extend filtering to other lists" in `WORKINPROGRESS.md`. Improves the "About" screen experience and demonstrates handling large static datasets in Rust.
*   **Implementation Plan:**
    1.  **Data Source:** Use `include_str!` to compile `app/app/src/main/assets/deps.json` directly into the Rust binary.
    2.  **Data Structure:** Define `Dependency` struct (name, version, url, license).
    3.  **State:** Create `DependencyState` in `state.rs` with `search_query` and a filtered list cache.
    4.  **Core Logic:** Create `features/dependencies.rs`.
        *   Parse JSON on startup (or lazy load).
        *   Implement filtering logic based on `search_query`.
    5.  **UI:**
        *   Implement `render_dependency_screen`.
        *   Use `VirtualList` for performance (the list can be long).
        *   Add `TextInput` for searching.
    6.  **Integration:** Add route in `router.rs` and link from the System Info or Settings screen.

## Previous Tasks
*   **CSV/JSON SQL Engine:** ✅ COMPLETED (2025-12-10)
