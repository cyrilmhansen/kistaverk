# Task In Progress: WebView Text Search

## Status: Implemented
*   **Date:** 2025-12-08
*   **Objective:** Implement text search functionality within the WebView-based Text Viewer.
*   **Plan:**
    1.  **UI Update (`features/text_viewer.rs`):**
        *   Added a search bar (TextInput + Prev/Next/Clear buttons) to `render_text_viewer_screen`.
        *   Bound controls to `text_view_find_query` and find actions.
    2.  **Logic (`features/text_viewer.rs`):**
        *   Exposed `find_query` in the root JSON payload so the Kotlin renderer can trigger `webView.findAllAsync(query)`.
    3.  **Refined Plan (Protocol Update):**
        *   Confirmed `Action::TextViewerFind` in `router.rs` updates the state correctly.
    4.  **Tests:**
        *   Added a unit test `find_query_is_exposed_in_render` to verify the JSON contract.

## Previous Task: Sensor Data Smoothing
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Implemented Low-Pass Filters for Compass, Barometer, and Magnetometer.