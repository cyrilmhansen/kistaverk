# WORK IN PROGRESS

*Use this file to track the active context for AI Agents. Update it at the end of every coding session.*

## 📅 Current Status
**Last Updated:** 2025-11-24
**Phase:** File-hash demo live (picker → Rust hashes SHA256/SHA1/MD5/MD4) + Shader demo + Play packaging (arm64-only)


📝 NEXT STEPS: Project Roadmap
1. Core Architecture: From "Toy" to "Engine"
Currently, we have a global integer counter. We need a robust system to manage complex states (file paths, loading states, navigation history).
State Machine: Replace the AtomicI32 with a proper AppState struct in Rust (enum Screen { Home, Hash, Pdf }).
Navigation Router: Implement a navigation stack in Rust.
Goal: Handle "Back" button hardware events (Android sends "Back" -> Rust pops state -> Rust returns previous screen JSON).
Action Dispatcher: Create a typed enum Action in Rust (instead of raw string matching) to handle events cleanly (Action::NavigateTo(ToolId), Action::SelectFile(Path)).
2. The UI Engine: Expanding the Vocabulary
The Kotlin UiRenderer needs to support more than just Text and Buttons to be useful.
Inputs & Forms: Add TextField (with on_change events sent to Rust) and Checkbox.
File Pickers: This is critical.
Challenge: Rust cannot open files directly on modern Android (Scoped Storage).
Solution: The JSON requests a FilePicker. Kotlin opens the system picker, gets a File Descriptor (FD), and passes the FD (int) to Rust.
Dynamic Lists: Implement ListView or GridView for the main menu and PDF page thumbnails.
Accessibility (A11y):
Update the JSON schema to include content_description.
Kotlin Renderer must map this field to view.contentDescription for TalkBack support.
3. Internationalization (I18n)
Since the UI is defined in Rust, the text should be handled there to maintain portability (e.g., for a future Desktop version).
Strategy: Do not use Android's strings.xml.
Implementation: Use a Rust crate like fluent or gettext embedded in the core.
Rust detects the locale (passed via init from Kotlin).
Rust selects the correct string dictionary.
Rust generates JSON with already translated text (e.g., "text": "Ouvrir" instead of "text": "Open").
4. Modularization & Dependency Management
To keep the "All-in-One" promise without creating a 100MB binary, we must manage Rust dependencies smartly.
Cargo Workspace: Split the Rust code into logical crates inside the folder:
core (UI logic, State).
modules/crypto (SHA, MD5).
modules/pdf (LoPDF or MuPDF bindings).
modules/image (Image crate).
Feature Flags: Use Cargo.toml features.
If a dependency (e.g., an image decoder) conflicts or is too heavy, we can disable it via flags.
Note: Rust handles version conflicts well (it statically links both versions if necessary), but we want to avoid this for size. We will audit the tree with cargo tree.
5. Feature Implementation Order
We will build the tools one by one to validate the engine components.
Main Menu (The Hub):
UI: Grid of Cards (Icon + Title).
Logic: Navigation routing.
Tool A: Hash Calculator (The MVP):
Tech: File Input -> Streaming Read -> SHA256.
Validation: Proves we can read large files from Android storage in Rust.
Tool B: Image Converter:
Tech: Bitmap decoding/encoding.
Validation: Proves we can handle heavy CPU tasks without freezing the UI (needs background threading in Rust).
Tool C: PDF Manipulator:
Tech: Complex binary parsing.


## Snapshot
- Kotlin now launches the system file picker, copies the chosen URI to cache, and sends the path to Rust via JSON (escaped with `JSONObject`). Renderer buttons support `requires_file_picker`; the sample screen shows “Select file and hash” and renders the resulting SHA-256 or an error message.
- Rust computes streaming hashes (SHA-256/SHA-1/MD5/MD4), stores `last_hash`/`last_hash_algo`/`last_error`, and returns updated UI JSON. Counter stub remains but is unused. Added Shader demo screen emitting a GLSL fragment for a simple cosine color wave; shader screen can load a fragment from file.
- Home menu now comes from a feature dictionary grouped by categories (Hashes, Graphics); hash buttons request file picker, shader opens demo.
- Build: release now shrinks/obfuscates (`minifyEnabled` + `shrinkResources`), enables ABI splits for Play, strips Rust symbols with size-focused Rust profile, excludes unused META-INF resources, disables BuildConfig, and targets arm64-v8a only (APK ~0.5 MB). Cargo.lock still not refreshed after adding `sha2`; cargo path remains hardcoded. Gradle density splits removed (AAB handles density).

## Known Issues / Risks
- JNI dispatch can unwind or abort: `STATE.lock().unwrap()` plus no `catch_unwind` means any panic poisons the mutex and may crash the VM (rust/src/lib.rs).
- Renderer still trusts incoming JSON and will crash on malformed output; no panic guard in JNI to backstop it.
- State is ephemeral; no serialization/restoration path; no tests around dispatch or renderer parsing; Cargo.lock not updated for `sha2`. Cargo build task still compiles armeabi-v7a even though APK is arm64-only; stale armeabi-v7a .so remains on disk (not packaged).

## Next Implementation Step
1. Harden JNI dispatch: wrap `Java_aeska_kistaverk_MainActivity_dispatch` in `catch_unwind`, replace `unwrap` with error propagation, and return a minimal error-screen JSON instead of letting panics cross the boundary.

## Near-Term Tasks
- Introduce typed `Command`/`Action` + richer `Screen` enum in Rust (home + placeholder hash screen), and generate UI via serde builders instead of manual json! literals.
- Kotlin side: guard `render` with try/catch and show a fallback error view instead of crashing; add a small loading indicator while hashing.
- Make cargo path portable in `app/app/build.gradle.kts` (use `commandLine("cargo", ...)` or look up from PATH); refresh Cargo.lock after adding `sha2`.
- Align cargo build targets to arm64-only to avoid producing unused v7a libs and remove stale v7a .so; refresh Cargo.lock after adding `sha2`; regenerate AAB and verify size with `scripts/size_report.sh`.

## MVP / Easy Wins
- Add a text input widget to the renderer with a simple binding map so Rust can ask for “Enter hash to compare” and receive it on submit.
- Add lightweight unit tests for Rust dispatch/state transitions and Kotlin renderer JSON parsing to lock in behavior as widgets expand.
