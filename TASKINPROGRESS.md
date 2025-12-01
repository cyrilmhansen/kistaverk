# Task In Progress: PDF Thumbnail Grid & Preview

## Feature Description
Implement a PDF viewer that displays a grid of page thumbnails and allows users to view individual pages in full screen. This leverages the Android `PdfRenderer` API on the Kotlin side, with navigation and state managed by the Rust core.

## Plan

### Step 1: Update Rust State (`rust/src/state.rs`)
*   **Goal:** Track preview state.
*   **Actions:**
    1.  Add `Screen::PdfPreview` to `Screen` enum.
    2.  Add `preview_page: Option<u32>` to `PdfState` struct.
    3.  Initialize/reset this field in `PdfState::new()` and `PdfState::reset()`.

### Step 2: Implement Rust UI Generation (`rust/src/features/pdf.rs`)
*   **Goal:** Define the JSON UI for the grid and single-page view.
*   **Actions:**
    1.  Implement `render_pdf_preview_screen`.
    2.  **Grid View:** If `preview_page` is `None`, render a `PdfPreviewGrid` widget (custom type) containing the `source_uri` and `page_count`.
    3.  **Page View:** If `preview_page` is `Some(n)`, render a `PdfSinglePage` widget with navigation buttons ("Prev", "Next", "Grid").

### Step 3: Update JNI Dispatch (`rust/src/lib.rs`)
*   **Goal:** Handle navigation actions.
*   **Actions:**
    1.  Add actions: `PdfPreviewScreen`, `PdfPageOpen { page: u32 }`, `PdfPageClose`.
    2.  Update `parse_action` and `handle_command`.
    3.  Add "PDF Viewer" to `feature_catalog`.

### Step 4: Implement Kotlin Renderer Widgets (`app/.../UiRenderer.kt`)
*   **Goal:** Render the actual PDF bitmaps.
*   **Actions:**
    1.  **`PdfPreviewGrid`**: A grid layout that uses `PdfRenderer` to generate thumbnails for all pages. Each thumbnail is a clickable button dispatching `pdf_page_open`.
    2.  **`PdfSinglePage`**: A view rendering a single high-res page from `PdfRenderer`. Supports zooming (basic) or just fits screen.

### Step 5: Testing
*   **Actions:**
    1.  Verify grid loads for small and large PDFs.
    2.  Verify navigation between grid and single page.
    3.  Check memory usage (PdfRenderer bitmaps can be large; ensure they are recycled or cache is managed).

---

## Completed Tasks
*   **Dithering Tools**: Implemented core logic, UI, and integration.
*   **Multi-hash view**: Implemented and refactored.
*   **Refactoring lib.rs**: Completed.