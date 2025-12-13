# JSON UI DSL Reference

This document defines the JSON widgets the Rust core can emit and the Kotlin renderer must support. All widgets are objects with a `"type"` field. Required props must be present; optional props may be omitted.

## Layout

### Column
- **type**: `"Column"`
- **props**:
  - `children` (array, required): child widgets
  - `padding` (number, optional)
  - `scrollable` (bool, optional)
  - `id` (string, optional)
  - `content_description` (string, optional)

### Section
- **type**: `"Section"`
- **props**:
  - `children` (array, required)
  - `title` (string, optional)
  - `subtitle` (string, optional)
  - `icon` (string, optional)
  - `padding` (number, optional)

### VirtualList
- **type**: `"VirtualList"`
- **props**:
  - `children` (array, required): row widgets
  - `id` (string, optional)
  - `estimated_item_height` (number, optional)

## Inputs

### TextInput
- **type**: `"TextInput"`
- **props**:
  - `bind_key` (string, required)
  - `text` (string, optional)
  - `hint` (string, optional)
  - `action_on_submit` (string, optional)
  - `single_line` (bool, optional)
  - `max_lines` (number, optional)
  - `content_description` (string, optional)
  - `debounce_ms` (number, optional)

### Button
- **type**: `"Button"`
- **props**:
  - `text` (string, required)
  - `action` (string, required)
  - `copy_text` (string, optional)
  - `id` (string, optional)
  - `requires_file_picker` (bool, optional)
  - `payload` (object, optional)
  - `content_description` (string, optional)

## Display

### Text
- **type**: `"Text"`
- **props**:
  - `text` (string, required)
  - `size` (number, optional)
  - `id` (string, optional)
  - `content_description` (string, optional)

### Warning
- **type**: `"Warning"`
- **props**:
  - `text` (string, required)
  - `content_description` (string, optional)

### CodeView
- **type**: `"CodeView"`
- **props**:
  - `text` (string, required)
  - `language` (string, optional)
  - `wrap` (bool, optional)
  - `theme` (string, optional, e.g., `"dark"`/`"light"`)
  - `line_numbers` (bool, optional)
  - `content_description` (string, optional)
  - `id` (string, optional)

## PDF Widgets

### PdfPagePicker
- **type**: `"PdfPagePicker"`
- **props**:
  - `page_count` (number, required)
  - `bind_key` (string, required)
  - `source_uri` (string, required)
  - `selected_pages` (array<number>, optional)
  - `content_description` (string, optional)

### PdfSignPlacement
- **type**: `"PdfSignPlacement"`
- **props**:
  - `source_uri` (string, required)
  - `page_count` (number, required)
  - `selected_page` (number, required)
  - `bind_key_page` (string, required)
  - `bind_key_x_pct` (string, required)
  - `bind_key_y_pct` (string, required)
  - `selected_x_pct` (number, optional)
  - `selected_y_pct` (number, optional)
  - `page_aspect_ratio` (number, optional)
  - `content_description` (string, optional)

### PdfSignPreview
- **type**: `"PdfSignPreview"`
- **props**:
  - `source_uri` (string, required)
  - `page_count` (number, required)
  - `selected_page` (number, required)
  - `bind_key_page` (string, required)
  - `bind_key_x_pct` (string, required)
  - `bind_key_y_pct` (string, required)
  - `selected_x_pct` (number, optional)
  - `selected_y_pct` (number, optional)
  - `page_aspect_ratio` (number, optional)
  - `content_description` (string, optional)

## Example

```json
{
  "type": "Column",
  "padding": 20,
  "children": [
    { "type": "Text", "text": "Pick pages", "size": 16 },
    {
      "type": "PdfPagePicker",
      "bind_key": "pages",
      "page_count": 5,
      "source_uri": "file://doc.pdf"
    }
  ]
}
```
