# Kistaverk Vision & Roadmap

## 🌟 The Core Philosophy: "The Offline Digital Swiss Army Knife"

Kistaverk aims to be the definitive utility for privacy-conscious power users on Android.
Unlike the thousands of "Unit Converter" or "PDF Merger" apps laden with ads, cloud dependencies, and tracking, Kistaverk is:

1.  **Strictly Local:** Internet permission is explicitly denied in the manifest.
2.  **High Performance:** Heavy logic (crypto, image processing) runs in compiled Rust.
3.  **Cross-Platform Core:** The business logic is decoupled from Android, enabling future ports (iOS, Desktop, CLI).

---

## ✅ Completed Features

### Core Infrastructure
*   **Local Backend-Driven UI:** Rust state machine drives the UI via JSON.
*   **Worker Thread:** Offloads heavy I/O and computation.
*   **Settings Persistence:** JSON-based preset system.

### Security & Integrity
*   **The Vault:** File encryption/decryption using `age`.
*   **Hash Tools:** MD5, SHA-1, SHA-256, BLAKE3, CRC32.
*   **Hash Verification:** Clipboard integration for instant checks.

### File Manipulation
*   **PDF Tools:** Merge, Split, Reorder, Sign, Delete Pages.
*   **Archive Tools:** ZIP creation, extraction, and inspection.
*   **Compression:** GZIP support.
*   **Batch Processing:** Bulk image conversion/resizing and PDF merging.

### Utilities
*   **Logical Engine:** Offline RDF-like triple store and query engine.
*   **Search/Filtering:** Filterable lists in Archive Viewer and Text Viewer.
*   **QR Tools:** Generation, Scanning (Rust-driven), and "Air Gap" File Transfer protocol.
*   **Sensor Logger:** CSV export of device sensors.
*   **Math Tool:** Symbolic differentiation and expression evaluation.

### Developer Tools
*   **Hex/Binary Editor:** View and patch files at the byte level.
*   **JWT Decoder:** Inspect JSON Web Tokens without network calls.

### Data Analysis (The "Lab")
*   **Plotting:** Generate Line, Scatter, and Histogram charts from CSV files using `poloto`.

---

## 🚀 Upcoming Features (Roadmap)

### 1. Advanced Data Analysis (The "Lab")
*   **CSV/JSON SQL Engine:** Treat local files as tables and run SQL queries (via `polars` or `sqlite`).

### 2. Networking Diagnostics (The "Probe")
*   While the *core* app has no internet, an optional "Probe" build flavor could enable:
    *   **Local Network Scan:** Discover devices (ARP/mDNS).
    *   **Port Scanner:** Check open ports on localhost or LAN.
    *   **Ping/Traceroute:** Diagnostic tools.
*   *Note: This conflicts with the strict "No Internet" philosophy, so it might remain a separate experimental branch or plugin.*

### 3. Automation
*   **Cron/Task Scheduler:** Allow users to chain tools (e.g., "Every day at 9 AM, take sensor log, compress to ZIP, encrypt").

---

## 🧠 Architectural Goals

1.  **Embedded Scripting:** Integrate a Lua or Rhai interpreter to allow users to write custom automation scripts that glue existing tools together.
2.  **Plugin System:** Allow loading external `.so` modules (signed) to extend functionality without recompiling the main app.
3.  **Desktop UI:** Wrap the Rust core in a lightweight GUI (e.g., Iced or Dioxus) for Linux/Windows/macOS parity.