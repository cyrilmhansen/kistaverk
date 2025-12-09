# Task In Progress: Logical Engine (RDF-like Data Sets)

## Status: Implemented
*   **Date:** 2025-12-09
*   **Objective:** Implement a lightweight RDF/logic module to enable structured data inspection, simple queries, and set operations offline.
*   **Plan:**
    1.  **Dependencies (`rust/Cargo.toml`):**
        *   No new heavy dependencies needed; can implement a simple triple store in pure Rust or use a lightweight crate if available (e.g., `oxigraph` might be too heavy, so a custom `Vec<Triple>` implementation is preferred for "lightweight").
    2.  **State (`rust/src/state.rs`):**
        *   Define `LogicState` struct:
            *   `triples: Vec<(String, String, String)>` (Subject, Predicate, Object)
            *   `query: Option<String>`
            *   `results: Vec<(String, String, String)>`
            *   `import_error: Option<String>`
    3.  **Implementation (`rust/src/features/logic.rs`):**
        *   Implement `TripleStore` struct with methods:
            *   `add(subject, predicate, object)`
            *   `query(subject_pattern, predicate_pattern, object_pattern)` -> returns matches.
            *   `import_csv(content: &str)` -> parses CSV into triples.
        *   Implement `render_logic_screen(state: &AppState) -> Value`.
            *   UI:
                *   Input for adding triples manually (S, P, O).
                *   Button to "Import CSV" (Subject, Predicate, Object).
                *   Query inputs (S, P, O) with wildcards (empty or "*").
                *   Results list (`VirtualList`).
    4.  **Integration (`rust/src/router.rs`):**
        *   Register module in `rust/src/features/mod.rs`.
        *   Add `LogicState` to `AppState`.
        *   Add `Action` variants: `LogicScreen`, `LogicAddTriple`, `LogicImportCsv`, `LogicQuery`.
        *   Handle actions in `handle_command`.
    5.  **Kotlin (`MainActivity.kt`):**
        *   Ensure file picker handles CSV for import.
    6.  **Tests:**
        *   **Store Logic:** Test adding and querying triples.
        *   **Import:** Test parsing simple CSV data.
        *   **Query:** Test wildcard matching.

## Previous Task: Search/Filtering within Tools
*   **Status:** Implemented
*   **Date:** 2025-12-09
*   **Summary:** Implemented search/filtering in Archive Viewer. Added `filter_query` to state, updated renderer to filter entries, and ensured original indices are preserved for extraction actions.