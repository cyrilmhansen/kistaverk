# TECHNICAL ARCHITECTURE

## System Overview
The application follows a **Rendering Engine (Kotlin) <-> JNI Bridge <-> Core Logic (Rust)** model.

### 1. The UI Layer (Kotlin): "The Renderer"
There are no classic XML layout files for individual screens.
- **Single Activity:** `MainActivity` handles the global lifecycle and delegates rendering/navigation to JSON coming from Rust.
- **UiRenderer:** Accepts JSON from Rust and dynamically instantiates native Android `View` objects (LinearLayout, TextView, Button, optional ScrollView wrapper for Columns/Grids via a `scrollable` flag, plus a ShaderToy view). Validation enforces required fields (Button text + action/copy_text, TextInput/Checkbox bind_key, PdfPagePicker data, etc.) and falls back to an inline error screen on schema violations.
- **Feature modules:** Feature-specific Kotlin code lives under `app/src/main/java/aeska/kistaverk/features` (e.g., Kotlin-side image conversion) so platform logic stays isolated from the activity.
- **Async boundary:** Blocking work runs on `Dispatchers.IO`; UI updates happen on the main thread after Rust returns JSON (or after Kotlin completes a local pipeline and reports the outcome back to Rust).
- **Media saves:** Image conversions default to MediaStore into `Pictures/kistaverk` (gallery-visible). A user-chosen SAF directory overrides this, with Rust still rendering the result screen.
- **Render fallback:** `MainActivity` catches `UiRenderer` errors and shows a minimal error screen (with a Back action), preventing renderer crashes from killing the app.
- **Inputs & bindings:** `UiRenderer` renders `TextInput` (EditText) widgets and forwards a `bindings` map with user-entered strings on every action; buttons still carry `requires_file_picker` hints for picker routing.
- **Checkboxes:** `UiRenderer` now supports `Checkbox` nodes with `bind_key`/`checked`/`content_description`, updating bindings on toggle and optionally firing an action (e.g., to refresh Rust UI).
- **Loading/Progress:** `UiRenderer` renders a `Progress` widget (indeterminate spinner + optional label) for loading states; used by Rust “Computing…” screens.
- **Overlay spinner:** MainActivity wraps content in a frame with a translucent overlay spinner for loading-only calls (hashes, progress demo) so the prior screen stays visible.
- **Grid layout:** The home menu renders categories as 2-column grids (auto-falls back to 1 column on narrow screens unless `columns` is set explicitly) to reduce scroll.
- **PDF widgets:** `PdfPagePicker` renders page thumbnails via `PdfRenderer` with multi-select bindings (now in a horizontal carousel strip); `SignaturePad` captures a drawn signature to base64 PNG and reports bitmap dimensions/DPI so Rust can scale and position accurately in PDF points and prefill width/height based on the image aspect ratio.
- **PDF metadata:** UI wiring allows updating the PDF Title (Info dictionary) via Rust lopdf, alongside page ops and signature stamping.
- **Hardware Back:** `OnBackPressedDispatcher` sends a `back` action into Rust, which pops the navigation stack and returns the previous screen JSON; inline Back buttons are only shown when stack depth > 1.
- **State persistence:** `MainActivity` saves a Rust snapshot during `onSaveInstanceState` and restores it on recreate. Robolectric tests cover the snapshot/restore path with JNI loading stubbed to avoid native deps.
- **Schema guardrails:** Renderer validates incoming JSON (widget whitelist + required children for Column/Grid + required fields for Button/TextInput/Checkbox/ImageBase64/ColorSwatch/PdfPagePicker) before rendering and falls back to an inline error screen if malformed. Robolectric tests cover malformed PdfPagePicker payloads.
- **Clipboard & copy actions:** Buttons may specify `copy_text` to push values to clipboard client-side; Color converter buttons copy Hex/RGB/HSL independently. MainActivity injects small clipboard text into bindings and reuses clipboard Hex for “Put Hex in input”.
- **Accessibility:** JSON `content_description` is applied on Text, Button, Column, ShaderToy, TextInput, Checkbox, Grid, and Progress to cover TalkBack without XML layouts.
- **Tests:** Robolectric tests exercise `UiRenderer` TextInput/Checkbox/Progress parsing and binding delivery to actions to catch JSON/render regressions early.
- **Renderer guardrails:** Unknown or malformed widgets render error TextViews instead of crashing; missing children in columns/grids show a clear error row.
- **Text tools:** Rust-side text utilities include casing, wrap/trim, counts, Base64, URL, and Hex encode/decode; implemented with std code paths (no extra deps) and rendered via JSON buttons. Result actions can be copied/shared via Android clipboard/share sheet.
- **Launcher alias:** A secondary `activity-alias` exposes a “Sign PDF” icon (distinct badge/icon) pointing to `MainActivity` with an `entry=pdf_signature` extra to land on the PDF tools/signature flow while keeping a single APK.
- **About screen:** Rust renders an About view (version, copyright, GPLv3) reachable from the home menu; navigation/back handled by the Rust screen stack.
- **Dependency listing:** A build-time script (`rust/scripts/generate_deps_metadata.sh`) writes `deps.json` from `cargo metadata` into Android assets; `DepsList` widget renders it in a scrollable list inside About (now emitted via a typed DSL helper; tested with a stubbed asset to ensure rendering works when the JSON is present). Archive viewer lists ZIP contents with entry caps and truncated/empty states.
- **Sensor logger:** Rust screen exposes sensor toggles (accel/gyro/mag/pressure/GPS/battery), interval binding, status/error text, and share gating. Kotlin registers only selected sensors at the requested interval on a background thread, writes CSV to Documents when possible (fallback app-private), throttles status UI updates to keep the screen responsive, and shares via FileProvider. GPS paths request location permission only when needed. A volatile `isLogging` flag guards late callbacks so writers are not touched after stop.
- **Text viewer:** Rust drives a text/CSV preview screen (256 KB cap) with a file picker or direct intent-view from Files. Kotlin filters picker MIME types to text/CSV (and handles ACTION_VIEW intents), opens a detached FD, and relays path + fd to Rust for safe UTF-8/lossey decoding with inline error reporting. Rendering now uses a lightweight WebView that loads bundled Prism (MIT) assets from `assets/prism/` for syntax highlighting (JSON/Markdown/TOML/YAML/Rust), keeping everything offline and size-capped while improving readability (monospace, optional wrap, line numbers, theme toggle, internal scroll instead of nested ScrollView). Zip archive entries that look like text become clickable and open directly in the viewer.
- **Back buttons:** Feature screens include inline Back buttons when depth > 1 (including QR/text tools/archive viewer/sensor logger/color converter/Kotlin image flows) to mirror hardware Back and keep navigation consistent.

### 2. The DSL Protocol (JSON)
Each screen is described by a JSON string generated by the Rust Core.
Typical structure:
```json
{
  "type": "Column",
  "children": [
    { "type": "Text", "bind_text": "status_msg" },
    { "type": "Button", "action": "do_compute" }
  ]
}
```

With inputs/bindings:
```json
{
  "type": "Column",
  "children": [
    { "type": "TextInput", "bind_key": "text_input", "text": "hello", "hint": "Type text" },
    { "type": "Button", "action": "text_tools_upper" }
  ]
}
```
Actions from Kotlin now include the `bindings` map (key/value strings) so Rust can consume user input without parsing Android widgets.

## 3. The Core Logic: "The Rust Fortress"
We utilize **Rust** as the native layer. It serves two purposes:
1.  **Safety & Performance:** It handles heavy lifting (PDF parsing, Crypto) with memory safety guarantees.
2.  **Orchestration:** It owns navigation, state, and screen rendering even when a feature's heavy lifting runs on Kotlin (e.g., image conversion that reuses Android codecs).

### Layered Structure
1.  **JNI Interface (Rust):**
    *   Exposes a single entry point `extern "C" fn dispatch(...)` to Kotlin.
    *   Deserializes the JSON command.
2.  **Service Layer (Rust Modules):**
    *   Uses crates like `sha2` (Crypto) and future small modules as needed.
    *   Feature states and renderers live in their own modules (e.g., `features::hashes`, `features::kotlin_image`).
3.  **Result:**
    *   Serializes output structs to JSON strings passed back to Android.

### Why this stack?
*   **Dependency Management:** `Cargo` handles cross-compilation for Android much better than CMake.
*   **Security:** Rust prevents memory corruption. Lua (if used) provides a sandboxed environment for logic execution.

## 4. The Core Logic (Rust)

Instead of C/C++, we choose **Rust** for the low-level core.
*   **Why?** Rust provides memory safety guarantees at compile-time without the runtime overhead of a Garbage Collector. It prevents entire classes of security vulnerabilities (buffer overflows) common in file manipulation tools.
*   **Toolchain:** Standard Android NDK with Rust support (Cargo-NDK).
*   **Structure:**
    *   The core is a Rust library compiled as a JNI `.so`.
    *   It handles navigation, state, and pure-Rust features (streaming hashes) and also orchestrates screens whose heavy lifting happens in Kotlin (image conversion).
    *   Navigation uses a `Vec<Screen>` stack managed in Rust; typed `Action`/`TextAction` enums replace stringly dispatch, and back pops safely without underflow.
    *   Snapshot/restore: Rust serializes `AppState` to JSON on `snapshot`, and `restore_state` rehydrates with navigation guarded; Kotlin persists this in the Activity bundle.
    *   Typed UI builders: Core screens (home, loading, shader, file info) now emit JSON via serde-backed structs to reduce malformed output risk and align with renderer validation.
    *   Panic Strategy: The Rust core catches panics (`std::panic::catch_unwind`), recovers from poisoned state locks, and returns error JSON to the UI instead of crashing.
    *   PDF pipeline: lopdf-powered extract/delete/merge and signature stamping work off detached FDs; outputs save to temp paths and flow back into AppState/UI.
    *   Android build uses Gradle to call Cargo; cargo path is resolved from `CARGO` env or PATH (not hardcoded), keeping NDK/strip settings intact and building arm64-v8a only.
    *   Example: the Text Tools screen is rendered fully from Rust (TextInput + grouped action buttons for uppercase/lower/title/wrap/trim/count), and Kotlin simply renders native views and relays bindings.

### Why not a Scripting Language (Lua/JS)?
To maintain the "Micro-tool" philosophy (< 5MB APK), we avoid embedding interpreters.
*   Scripting engines add runtime weight.
*   Rust offers high-level abstractions (like Python) but compiles down to efficient machine code.


### 5. UI Philosophy: "Backend-Driven UI"
We treat the Local Rust Core as a "Server" and the Android View as a "Client".
*   **Screen Construction:** Android never hardcodes screens. It requests the screen definition from Rust (e.g., `cmd: "get_home_screen"`).
*   **State updates:** When an action occurs, Rust returns the *entire new state description* of the UI (Virtual DOM style). Kotlin-owned pipelines (like image conversion) run locally but immediately report results back through `dispatch` so Rust still drives the screen and navigation, including MediaStore/SAF target info.
*   **Benefits:** Complex logic (e.g., "Show this specific error if SHA-256 fails but MD5 succeeds") resides in Rust. Kotlin focuses on platform integrations, Android-provided codecs, and storage APIs (MediaStore/SAF).


-----

A. The Async/Threading Model
Since Rust cannot block the main thread, we must define an async boundary.
Recommended Strategy: "Kotlin-Side Async"
Kotlin: Uses Coroutines (Dispatchers.IO) to call the blocking Rust function.
Rust: Remains synchronous (simpler, smaller binary). It just calculates and returns.
UI: While waiting for Rust, Kotlin displays a loading spinner overlay (since the UI JSON hasn't returned yet).


A. State Serialization (The "Don't Lose My Work" Rule)
Android frequently kills background apps to save memory. When the user returns, MainActivity is recreated, but the Rust memory (the static Mutex) is wiped clean.
The Missing Protocol:
Serialize: On onSaveInstanceState (Kotlin), send a get_state_snapshot command to Rust. Rust serializes AppState to a JSON string or Byte Array and returns it. Kotlin saves this in the Bundle.
Restore: On onCreate (Kotlin), check if a Bundle exists. If yes, extract the snapshot and send restore_state(snapshot) to Rust.
Rust Impl: The AppState struct must derive Serialize and Deserialize.
B. The File Descriptor Bridge (Zero-Copy)
To solve the file copying issue mentioned in the feedback:
Mechanism:
Kotlin: Uses ContentResolver.openFileDescriptor(uri, "r").
JNI: Passes the int fd to Rust.
Rust:
code
Rust
use std::os::unix::io::FromRawFd;
use std::fs::File;

// UNSAFE: We must ensure Kotlin keeps the FD open while we read, 
// and that we don't double-close it if not intended.
let file = unsafe { File::from_raw_fd(fd) }; 
// Read file...
// std::mem::forget(file) // logic to prevent Rust from closing the FD if Kotlin owns it

C. Navigation & Feature Ownership
- **Screens:** `AppState` now keeps a stack of `Screen` values (Home, Shader demo, Kotlin image flow, etc.) rather than a single current_screen. Back pops to the prior screen; Home is the root and is never underflown.
- **Feature isolation:** Rust modules own state and rendering. Kotlin feature helpers own platform work (e.g., bitmap codecs) but send outcomes back through `dispatch`, so Rust drives UI and history.
- **Menu/actions:** Buttons carry an `id` and `action`. Actions are parsed into typed enums (e.g., `Action::Hash(HashAlgo)`, `Action::TextTools(TextAction)`) to avoid stringly routing. Some actions open sub-screens (like the Kotlin image screen) and then request the picker; results flow back to Rust to render success/error, display chosen MediaStore/SAF targets, and keep navigation consistent.




This project demonstrates a very strong adherence to the "Zero-Bloat" and "Backend-Driven UI" philosophy. The separation of concerns is clean, and the code is remarkably readable for a JNI project.
However, moving from a "Prototype" to a "Production Engine" requires addressing specific architectural bottlenecks, particularly regarding state management and type safety.
Here is the review of the architecture, bugs, and the i18n strategy.
1. Architecture Review & Improvements
A. State Management: The "God Object" Problem
Current Status: AppState is a massive struct containing fields for every single feature (pdf, image, text_input, sensor_...).
Risk: As you add features (Archives, Math, Logic), this struct will become unmaintainable. Every time you touch state.rs, you risk breaking unrelated features.
Improvement: Implement a Modular State / Feature Trait.
Define a Feature trait in Rust that handles its own state reduction and UI rendering.
AppState should hold a generic Box<dyn Feature> or an enum of active features, plus a shared GlobalState (navigation stack, clipboard).
Benefit: features/pdf.rs would own PdfState completely. lib.rs wouldn't need to know about pdf.source_uri.
B. The JSON "Stringly Typed" Weakness
Current Status: You are constructing JSON using json!({ "type": "Button", ... }) macros.
Risk: It is extremely easy to make a typo in "bind_key" or "action", causing the Kotlin renderer to fail silently or fallback to error views.
Improvement: Use Strongly Typed Structs for the UI DSL in Rust.
You already started this in src/ui.rs (Button, Column), but it's not used everywhere (e.g., render_pdf_screen uses raw json!).
Enforce usage of ui::Button::new(...) everywhere.
Benefit: Compile-time verification of your UI schema. If you change a widget field, the Rust compiler will yell at you.
C. Navigation Data Passing
Current Status: To pass data from the File Picker back to the screen, you rely on global fields like state.pdf.source_uri.
Risk: Race conditions or stale state if the user navigates away and back quickly, or if two features use similar fields.
Improvement: Implement Action payloads.
When Action::PdfSelect is fired, the result should not just update a global variable; it should ideally transition the state machine specific to that screen.
Consider making the Screen enum hold data: Screen::PdfTools(PdfState).
2. Obvious Bugs & Fixes
🐛 Bug 1 : Sensor Logger Race Condition (Threading) — **Fixed**
Location: MainActivity.kt
Resolution: Added `@Volatile isLogging` and early exits in both sensor and GPS listeners so late callbacks cannot write after stop; stop flips the flag before unregistering/closing.
🐛 Bug 2 : PDF Coordinate System Mismatch — **Fixed**
Location: features/pdf.rs & MainActivity.kt (SignaturePad)
Resolution: SignaturePad now sends bitmap dimensions + DPI; Rust derives px→pt, preserves aspect ratio, reads page MediaBox height, and flips Y (`pdf_y = page_height - top_px_in_pt - height`) to align top-left view coordinates with bottom-left PDF coordinates.
🐛 Bug 3 : ScrollView Logic in Renderer — **Fixed**
Location: UiRenderer.kt -> render()
Resolution: Root wrapping is now controlled by a `scrollable` flag (default true); roots can opt out to avoid nested scrolling conflicts with future list/recycler widgets.
Additional guardrails: Buttons must now provide either an `action` or `copy_text`, and renderer validation covers required fields for TextInput/Checkbox/PdfPagePicker/SignaturePad to reduce malformed payloads from Rust.


Internationalization (i18n) Strategy
Since you want the Core to remain the "Single Source of Truth", Rust must handle translations. Do not use Android's strings.xml.
Here is the most extensible way to implement this without bloating the binary:
Step 1: Define Locales in Rust
Create a lightweight translation module. Avoid heavy crates like gettext. Use a compile-time map or rust-i18n.
rust/src/i18n.rs
code
Rust
use std::collections::HashMap;
use std::sync::Mutex;

// Simple Key-Value store. 
// For <5MB, we can embed strings directly or load them from a JSON asset at runtime.
// Embedding is faster and safer.

pub enum Locale {
    En,
    Fr,
    Is, // Icelandic
}

impl Locale {
    pub fn from_str(s: &str) -> Self {
        if s.starts_with("fr") { Locale::Fr }
        else if s.starts_with("is") { Locale::Is }
        else { Locale::En }
    }
}

pub fn t(key: &str, locale: &Locale) -> String {
    match locale {
        Locale::En => match key {
            "app_name" => "Kistaverk",
            "menu_hash" => "Hash Tools",
            _ => key,
        },
        Locale::Fr => match key {
            "app_name" => "Kistaverk",
            "menu_hash" => "Outils de Hachage",
            _ => key,
        },
        _ => key, // Fallback
    }.to_string()
}
Step 2: Inject Locale from Kotlin
In MainActivity.kt, detect the system locale and send it during the init action.
code
Kotlin
// MainActivity.kt
private fun getSystemLocale(): String {
    return resources.configuration.locales[0].language
}

// In onCreate or initial dispatch
val initData = mapOf("locale" to getSystemLocale())
dispatchWithOptionalLoading("init", bindings = initData)
Step 3: Store Locale in AppState
Update AppState to hold the current Locale.
code
Rust
// state.rs
pub struct AppState {
    pub locale: Locale,
    // ... other fields
}
Step 4: Use it in UI Generation
Update your render functions to accept the state (which they already do) and use a helper macro or function.
code
Rust
// features/mod.rs
pub fn render_menu(state: &AppState) -> Value {
    let l = &state.locale;
    let mut children = vec![
        // "Tool menu" becomes dynamic
        serde_json::to_value(UiText::new(&t("menu_title", l)).size(22.0)).unwrap(), 
    ];
    // ...
}
Why this is best for Kistaverk:
Extensible: Adding a language is just adding a match arm or a file in Rust.
Hot-Swap: You can change the language instantly without restarting the Android Activity (just re-render the JSON).
Consistency: The same Rust core can be compiled for iOS or Desktop later, and the translations travel with it.
Use Arrow Up and Arrow Down to select a turn, Enter to jump to it, and Escape to return to the chat.
control for lists).
