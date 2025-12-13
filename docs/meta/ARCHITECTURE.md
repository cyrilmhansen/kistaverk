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