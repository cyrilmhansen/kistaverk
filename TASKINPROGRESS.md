# Task In Progress: JSON-Backed RecyclerView Adapter

## Status: Planning
*   **Date:** 2025-12-08
*   **Objective:** Address the "UI Scalability" technical debt by implementing a virtualized list component (`VirtualList`) in the JSON protocol. This allows the Android `RecyclerView` to render large datasets without OOM errors, replacing the current `LinearLayout` approach for history/logs.
*   **Plan:**
    1.  **Protocol Update (`ui.rs`):**
        *   Define a new component type `VirtualList` in the JSON schema.
        *   It should accept a list of items (data) and a template (how to render each item).
    2.  **State Management (`state.rs`):**
        *   No global state changes needed immediately, but features like `SensorLogger` (logs) or `MathTool` (history) will be consumers.
    3.  **Implementation (`ui.rs`):**
        *   Implement `VirtualList` struct and `Serialize` impl.
    4.  **Integration (Feature: Math Tool):**
        *   Update `render_math_tool_screen` to use `VirtualList` for the history instead of a manual `Column` loop.
    5.  **Tests:**
        *   Verify that `VirtualList` serializes correctly to the expected JSON structure.

## Previous Task: PDF 3x3 Placement Grid
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Added a 3x3 grid for quick signature placement in the PDF tool.
