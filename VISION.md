Here is the completely refactored `ARCHITECTURE.md`. It now serves as the comprehensive technical reference, absorbing the structural details that were previously in the Vision document.

***

# ARCHITECTURE.md

## System Overview
The project follows a **Local Backend-Driven UI** architecture.
Unlike traditional Android apps where logic and UI are tightly coupled in Java/Kotlin, MicroTools acts as a local "Client-Server" system within a single process:
*   **The Server (Core):** Written in **Rust**. It holds the state, business logic, and dictates *what* to display.
*   **The Client (Renderer):** Written in **Kotlin**. It is a "dumb" terminal that renders native Android views based on instructions from the Core.

---

## 1. The Core: "The Rust Fortress"
The heart of the application is a compiled native library (`libmicrotools.so`). It is the **Single Source of Truth**.

### Responsibilities
1.  **State Management:** It holds the current state of the application (e.g., "Screen: HashCalculator", "File: /sdcard/doc.pdf", "Status: Processing").
2.  **UI Generation:** It generates the UI structure dynamically. Instead of hardcoded XML layouts, Rust returns a JSON description of the screen based on the current state.
3.  **Heavy Lifting:** It utilizes the Rust Crates ecosystem for cryptography, PDF manipulation, and image processing.
4.  **Security:** It ensures memory safety and handles sensitive data (pointers, buffers) without exposing them to the Java layer.

### Optional Scripting Layer
*   *Future Expansion:* The Rust core is designed to host an embedded interpreter (e.g., Lua via `mlua`) to allow user-defined scripts or dynamic extensions without recompiling the binary.

---

## 2. The Protocol: JSON DSL
Communication between Core and Renderer is strictly defined by a JSON Domain Specific Language (DSL).

### Screen Definition Structure
When the Renderer asks for a screen, Rust returns:
```json
{
  "screen_id": "sha256_tool",
  "title": "SHA-256 Calculator",
  "layout": {
    "type": "Column",
    "children": [
      { 
        "type": "FilePicker", 
        "bind_key": "input_path", 
        "action": "select_file" 
      },
      { 
        "type": "Button", 
        "text": "Calculate", 
        "action": "compute_hash",
        "visible_if": "input_path != null" 
      }
    ]
  }
}
```

### Key Concepts
*   **Components:** Visual primitives (`Row`, `Column`, `Text`, `Button`, `Input`).
*   **Bindings:** Data links. If `bind_key` is "result_hash", the view automatically updates when that key changes in the state.
*   **Actions:** Events sent back to Rust (e.g., `"compute_hash"`).

---

## 3. The Renderer: Kotlin & Android SDK
The Android layer is thin, native, and stateless.

### Responsibilities
1.  **Native Rendering:** Instantiates standard Android SDK Views (`android.widget.Button`, `LinearLayout`). This ensures:
    *   **Accessibility:** Perfect integration with TalkBack and screen readers.
    *   **Ergonomics:** Native scroll physics, text selection, and copy/paste behavior.
    *   **Theme:** Respects the user's system theme (Dark/Light mode).
2.  **Event Dispatching:** Captures user inputs (clicks, text changes) and forwards them immediately to Rust via JNI.

### Why not Pure Rust UI?
While frameworks like `NativeActivity` allow drawing pixels from Rust, they break accessibility and feel "foreign" on Android. Our approach retains the native User Experience while keeping the logic in Rust.

---

## 4. The Bridge: JNI (Java Native Interface)
The interface between Kotlin and Rust is minimalist to reduce complexity.

### The Interface
```rust
// Rust exports strictly one main entry point:
extern "C" fn Java_com_microtools_core_Bridge_dispatch(
    env: JNIEnv, 
    _: JClass, 
    command_json: JString
) -> JString
```

### Performance & Memory Model
*   **Process:** Both Kotlin and Rust run in the same OS Process and share the same RAM.
*   **Context Switching:** Calling Rust is a synchronous function call, not a network request. The overhead is negligible (microseconds) compared to the UI refresh rate (16ms).
*   **Data Flow:**
    1.  **Action:** User clicks -> Kotlin sends `{"action": "click"}` to Rust.
    2.  **Update:** Rust computes -> Rust returns `{"new_screen_state": ...}`.
    3.  **Render:** Kotlin parses JSON -> Updates Views.

---

## 5. Directory Structure
```text
/
├── app/src/main/
│   ├── java/com/microtools/
│   │   ├── ui/          # ViewFactory, Widget implementations
│   │   └── core/        # JNI Bridge Class
│   └── cpp/             # (Empty, we use external Rust build)
├── rust/                # The Cargo Project
│   ├── src/
│   │   ├── lib.rs       # JNI Entry point
│   │   ├── state.rs     # State Machine
│   │   ├── ui_gen.rs    # JSON Builders
│   │   └── modules/     # (crypto, pdf, image)
│   └── Cargo.toml
└── build.gradle.kts     # Configured to build Rust via cargo-ndk
```

---

## 6. Build System
*   **Rust:** Uses `cargo` and `cargo-ndk` to cross-compile `.so` libraries for `arm64-v8a`, `armeabi-v7a`, and `x86_64`.
*   **Android:** Gradle hooks into the process to copy the generated `.so` files into `src/main/jniLibs` before packaging the APK.


## Top 3 Missing Features (Roadmap)

Given the "Swiss Army Knife" philosophy and the current capability set, these features would provide the highest value:

### 1. File Encryption/Decryption (The "Vault")
Why: The app focuses heavily on privacy and integrity (hashing). The missing piece is confidentiality.
Implementation:
Rust: Integrate the age crate (modern, secure file encryption) or aes-gcm.
UI: A screen to "Encrypt File" (accepts file + password/key, outputs .age file) and "Decrypt File".
Benefit: Fits perfectly with the offline, privacy-first mission.

### 2. Batch Processing (Images & PDFs)
Why: Currently, KotlinImageConversion.kt and pdf.rs handle single files. Users typically need to resize multiple photos for email or merge multiple PDFs at once.
Implementation:
Android: Update the ActivityResultContracts to GetMultipleContents.
Rust (KotlinImageState): Change source_path to Vec<String>.
UI: A VirtualList in Rust showing the queue with a "Process All" button.

### 3. Search/Filtering within Tools
Why: The Archive Viewer and Dependency List can become unwieldy with large inputs.
Implementation:
Rust (archive.rs): Add a filter_query field to ArchiveState. Filter entries before sending them to the VirtualList.
UI: Reuse the TextInput with debounce_ms (which was recently implemented) to trigger live filtering updates in the Rust state.
