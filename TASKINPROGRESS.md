# Task In Progress: PDF Page Reordering

## Feature Description
Extend the PDF tools to allow reordering pages. Users should be able to specify a new order for the pages (e.g., "3, 1, 2" or "1, 3-5, 2"). This builds upon the existing extract/delete infrastructure.

## Plan

### Step 1: Update Core Logic (`rust/src/features/pdf.rs`)
*   **Goal:** Add `reorder_pages` logic.
*   **Actions:**
    1.  Add `PdfOperation::Reorder` variant.
    2.  Implement `reorder_pages(doc, page_order: &[u32]) -> Result<Document, String>`.
        *   Create a new document structure.
        *   Copy pages from the source document in the specified order.
        *   Handle duplicates (if a user wants to duplicate a page) or strictly reorder (permutation). *Decision: Allow duplication/subsetting as it's more flexible.*
        *   Validation: Ensure all input page numbers exist in the source.

### Step 2: Update UI Rendering (`rust/src/features/pdf.rs`)
*   **Goal:** Add reorder input.
*   **Actions:**
    1.  Add a `TextInput` for "Page Order" (defaulting to "1, 2, 3...").
    2.  Add a "Reorder" button.
    3.  (Ideally) drag-and-drop UI is hard with the current JSON DSL, so a text input with comma-separated values is the pragmatic MVP.

### Step 3: Integration (`rust/src/lib.rs`)
*   **Goal:** Wire up action.
*   **Actions:**
    1.  Add `Action::PdfReorder { fd, uri, order: Vec<u32> }`.
    2.  Parse the comma-separated string from the bindings into a `Vec<u32>`.
    3.  Call `handle_pdf_operation` with `PdfOperation::Reorder`.

### Step 4: Testing
*   **Actions:**
    1.  Unit test the reordering logic (mocking a document structure or using a minimal PDF generator if available, otherwise rely on logic verification).
    2.  Verify behavior with invalid page numbers.

---

## Completed Tasks
*   **ZIP Extraction**: Done.
*   **File Inspector**: Done.
*   **Dithering Tools**: Done.
*   **Multi-hash view**: Done.
*   **Refactoring lib.rs**: Done.
*   **PDF Thumbnail Grid**: Rust side done (Kotlin pending).
*   **Pixel Art Mode**: Implemented (Rust Done, Kotlin Pending).
*   **Regex Tester**: Implemented (Rust Done).
*   **UUID/Random**: Implemented (Rust Done).
