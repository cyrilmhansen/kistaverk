# Task In Progress: Refine PDF Placement Overlay

## Status: Planning
*   **Date:** 2025-12-08
*   **Objective:** Improve the UX of the PDF signature placement overlay by ensuring the "tap to place" marker accurately reflects the scaled position on the rendered page thumbnail.
*   **Plan:**
    1.  **Analyze Current State (`features/pdf.rs`):**
        *   The current `PdfSignPlacement` component (in Kotlin, driven by JSON) sends `pdf_signature_x_pct` and `pdf_signature_y_pct`.
        *   The preview thumbnail (`PdfSignPreview`) renders a marker based on these percentages.
    2.  **Refinement Logic (`features/pdf.rs`):**
        *   Update `render_pdf_screen` to ensure the `PdfSignPreview` component receives the aspect ratio of the page if available (from `page_dimensions`).
        *   Currently, `page_dimensions` is internal. We might need to expose it or cache the page aspect ratio in `PdfState`.
    3.  **UI Update (`features/pdf.rs`):**
        *   When rendering `PdfSignPreview`, pass an explicit `aspect_ratio` field if known.
        *   This allows the client (Kotlin) to draw the preview box with the correct aspect ratio, ensuring the normalized coordinates (0.0-1.0) visually map to the correct physical location on the PDF page.
    4.  **Tests:**
        *   Add a test case in `pdf.rs` to verify that `page_dimensions` extracts the correct media box and calculates the aspect ratio.

## Previous Task: Offload Blocking File I/O to Worker Thread Pool
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Moved blocking file I/O operations (PDF load, File Info, Text View) to the asynchronous worker thread pool to prevent UI freezes.