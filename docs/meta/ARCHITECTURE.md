# Kistaverk Architecture

## 🏛 System Overview

Kistaverk follows a **Unidirectional Data Flow** architecture, heavily inspired by The Elm Architecture (TEA) or Redux, but adapted for a cross-language context (Kotlin ↔ Rust).

### The Core Loop

1.  **State (Rust):** The single source of truth. The application state lives entirely inside the Rust `Core`.
2.  **View (Rust -> JSON):** The `view()` function transforms the State into a JSON Virtual DOM (The "UI Protocol").
3.  **Render (Kotlin):** The Android layer observes the JSON, diffs it (naively or strictly), and updates the native Android Views (`RecyclerView`, `TextView`, etc.).
4.  **Message (Kotlin -> Rust):** User interactions (clicks, text input) are sent as string-based "Commands" (e.g., `["text_update", "input_1", "Hello"]`) back to Rust.
5.  **Update (Rust):** The `update(msg)` function mutates the State based on the message.

---

## 🧱 The Stack

### 1. The Native UI (Android/Kotlin)
*   **Role:** Dumb renderer. It knows *how* to draw a Button, but not *what* the button does.
*   **Key Component:** `UIRenderer` (The "Browser"). It parses the custom JSON layout protocol and builds Android Views.
*   **Dependency:** `WebView` is used *only* for specific rich-text content (PrismJS code highlighting) or special visualizations. The rest is native Widgets.

### 2. The Bridge (JNI)
*   **Role:** Message passing interface.
*   **Serialization:** JSON is used for the UI description. Primitive types are used for commands.
*   **Concurrency:** Rust manages its own Worker Thread for heavy tasks. The UI thread in Kotlin is never blocked.

### 3. The Core (Rust)
*   **Role:** Business logic, State management, Cryptography, File I/O.
*   **Crate:** `kistaverk_core` (`cdylib`).
*   **Architecture:**
    *   `State`: A giant enum or struct holding the data for the active screen.
    *   `Features`: Modules (e.g., `features/vault.rs`) implementing specific tools.
    *   **Notable Dependencies:**
        *   `symbolica`: Used for CAS and high-precision math. *Note: Restricted to single-core execution for non-commercial use (Hobbyist License). MIT-licensed modules used where applicable.*
        *   `wide`: **ARM64 SIMD Optimization Framework** - Provides automatic NEON utilization for numerical computations. Always available on AArch64 devices.
          - **Purpose**: Accelerate math-intensive operations (matrix math, vector operations, cryptography)
          - **Performance**: 2-4x speedup for vectorizable operations
          - **Usage**: Automatically utilized through Rust's SIMD intrinsics
          - **Targets**: Multiple ARM64 instruction set versions (ARMv8.0-ARMv8.5) for device-specific optimization
          - **Documentation**: See `rust/ARM64_OPTIMIZATIONS.md` for build instructions

### 4. Testing Strategy
* **Host unit tests:** `cargo test` for fast, platform-agnostic logic coverage.
* **Android instrumented tests:** Espresso/UI flows that load the UPX-packed `libkistaverk_core.so` on device/emulator to validate JNI load/init hooks and critical screens. Add these to ensure 16 KB alignment and UPX packaging work end-to-end on real Android runtimes.

---

## 🛡️ Error Handling & Stability

To ensure a compact binary size and deterministic behavior, the Rust core is compiled with `panic = "abort"`.

### The "No-Panic" Policy
Since panics result in an immediate application crash (SIGABRT) without stack unwinding:
1.  **Expected Errors:** MUST be handled using `Result<T, E>`.
2.  **Runtime Panics:** (e.g., `unwrap()`, `expect()`, array indexing) MUST be avoided on dynamic data.
3.  **Boundary Protection:** The FFI layer should ideally catch any residual errors, though `abort` makes `catch_unwind` impossible. The strategy is **prevention**.

### Safe Pattern Example
**❌ BAD (Crashes the App):**
```rust
// If parsing fails, the entire Android app dies.
let value = inputs.pop().unwrap(); 
let num: i32 = value.parse().unwrap();
```

**✅ GOOD (Returns Error to UI):**
```rust
// Propagates error state, UI shows a snackbar or error text.
let value = inputs.pop().ok_or("Stack underflow")?;
let num: i32 = value.parse().map_err(|_| "Invalid number")?;
```

---

## 🌍 Internationalization & RTL Support

### Current Implementation
The system supports **Left-to-Right (LTR) languages** through the `rust-i18n` crate with locale files for English (`en`) and Icelandic (`is`). French (`fr`) support is in development.

### RTL Support Architecture

**Feasibility:** ✅ **High** - Android's native components support RTL out of the box since API 17+. The main integration work is in the JSON UI protocol.

**Required Protocol Extensions:**
```json
{
  "type": "Text",
  "text": "مرحبا",  // Arabic text
  "align": "start",  // Direction-aware (vs "left")
  "rtl_aware": true,  // Opt-in RTL handling
  "text_direction": "auto"  // auto | ltr | rtl
}
```

**Implementation Components:**

1. **JSON Protocol Extensions** (~1-2 days)
   - Add `text_direction` field to all text widgets
   - Support `start`/`end` instead of `left`/`right` for alignment
   - Add RTL awareness flags to containers

2. **Rust UI Renderer Updates** (~2-3 days)
   - Pass direction information to Android views
   - Handle bidirectional text mixing (LTR within RTL)
   - Mirror icon positions and visual indicators

3. **Android Configuration** (~1 hour)
   ```xml
   <!-- AndroidManifest.xml -->
   <application android:supportsRtl="true">
   ```

4. **Locale File Support** (Ongoing)
   - Add `ar.yml` (Arabic) locale file
   - Extend `normalize_locale()` to handle RTL languages
   - Test bidirectional text rendering

### Current Limitations
- ❌ No RTL language files yet (Arabic, Hebrew, Persian)
- ❌ JSON protocol doesn't have direction awareness
- ❌ No bidirectional text algorithm integration
- ❌ Layout mirroring not implemented

### Future Roadmap
1. **Phase 1 (Current):** Complete French LTR implementation
2. **Phase 2 (Post-French):** Add RTL protocol extensions
3. **Phase 3 (Demand-driven):** Add Arabic locale and test RTL rendering
4. **Phase 4 (Optimization):** Implement layout mirroring for complex screens

**Estimated Total Cost:** ~1 week development + testing
**Potential Market Expansion:** ~10-15% additional user base
**Technical Risk:** Low (Android handles most complexity)

## 📦 Data Protocol (DSL)

See `DSL.md` for the complete widget specification.

**Core Widgets:**
*   `Column`, `Row`, `Grid`: Layout containers.
*   `Text`, `Button`, `TextInput`: Basic interaction.
*   `Card`, `Section`: Grouping.

**Smart Widgets:**
*   `CodeView`: Syntax highlighting.
*   `PdfPagePicker`: Custom view for selecting pages.
*   `SignaturePad`: Capture vectors/bitmap.
