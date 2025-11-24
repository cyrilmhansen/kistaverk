# WORK IN PROGRESS

*Use this file to track the active context for AI Agents. Update it at the end of every coding session.*

## 📅 Current Status
**Last Updated:** 2025-11-26
**Phase:** File-hash demo live (picker → Rust hashes SHA256/SHA1/MD5/MD4/CRC32/BLAKE3) + Shader demo + Text tools screen (wrap/trim/count + checkbox) + Play packaging (arm64-only)


📝 NEXT STEPS: Project Roadmap
1. Core Architecture: From "Toy" to "Engine"
Currently, we have a global integer counter. We need a robust system to manage complex states (file paths, loading states, navigation history).
State Machine: Replace the AtomicI32 with a proper AppState struct in Rust (enum Screen { Home, Hash, Pdf }).
Navigation Router: Implement a navigation stack in Rust.
Goal: Handle "Back" button hardware events (Android sends "Back" -> Rust pops state -> Rust returns previous screen JSON).
Action Dispatcher: Create a typed enum Action in Rust (instead of raw string matching) to handle events cleanly (Action::NavigateTo(ToolId), Action::SelectFile(Path)).
2. The UI Engine: Expanding the Vocabulary
The Kotlin UiRenderer now supports TextInput, Checkbox, Progress with bindings and propagates `content_description` for TalkBack. Grids render the menu with an auto column heuristic (1 column on narrow screens, 2 otherwise, or explicit override). MainActivity can overlay a spinner on the current screen for loading-only calls.
Renderer guardrails: unknown/malformed widget types render inline error text instead of crashing; missing children show a warning row.
File Pickers: This is critical.
Challenge: Rust cannot open files directly on modern Android (Scoped Storage).
Solution: The JSON requests a FilePicker. Kotlin opens the system picker, gets a File Descriptor (FD), and passes the FD (int) to Rust.
Dynamic Lists: Implement ListView or GridView for the main menu and PDF page thumbnails.
Accessibility (A11y):
Continue passing `content_description` everywhere and add accessibility labels to new widgets; validate with TalkBack.
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
- Kotlin now launches the system file picker (detaching FDs), forwards a `bindings` map with UI state, and renders TextInput alongside Text/Button/ShaderToy. Buttons support `requires_file_picker`; Columns are wrapped in ScrollView. Content descriptions flow to Text/Button/Column/ShaderToy/TextInput.
- Rust computes streaming hashes (SHA-256/SHA-1/MD5/MD4/CRC32/BLAKE3), stores `last_hash`/`last_hash_algo`/`last_error`, and returns updated UI JSON. Catch_unwind guards JNI panics, poisoned locks recover. Added Shader demo, Kotlin image conversion screen, and a Rust-driven Text Tools screen (upper/lower/title/wrap/trim/word & char counts) with inline result block.
- Home menu is generated from a feature catalog grouped by category (Hashes, Graphics, Media, Text). Text tools are reachable from the menu without pickers.
- Build: release shrinks/obfuscates (`minifyEnabled` + `shrinkResources`), enables ABI splits for Play, strips Rust symbols with size-focused profile, excludes unused META-INF resources, disables BuildConfig, targets arm64-v8a only. Cargo task resolves `cargo` from PATH/`CARGO`. Gradle density splits removed (AAB handles density).
## Snapshot
- Kotlin now launches the system file picker (detaching FDs), forwards a `bindings` map with UI state, and renders TextInput, Checkbox, Progress, Text/Button/ShaderToy. Buttons support `requires_file_picker`; Columns are wrapped in ScrollView. Grids auto-pick columns (1 on narrow, 2 otherwise unless overridden). Content descriptions flow to Text/Button/Column/ShaderToy/TextInput/Checkbox/Progress. MainActivity can show a translucent overlay spinner while keeping the last screen visible.
- Renderer guardrails added: unknown widgets and missing children produce inline error rows rather than crashing.
- Rust computes streaming hashes (SHA-256/SHA-1/MD5/MD4/CRC32/BLAKE3), stores `last_hash`/`last_hash_algo`/`last_error`, and returns updated UI JSON. Catch_unwind guards JNI panics, poisoned locks recover. Added Shader demo, Kotlin image conversion screen, a Rust-driven Text Tools screen (upper/lower/title/wrap/trim/word & char counts + aggressive trim checkbox + Base64/URL encode/decode) with inline result block, and a Progress demo feature.
- Home menu is generated from a feature catalog grouped by category (Hashes, Graphics, Media, Text, Experiments). Text tools and Progress demo are reachable without pickers.
- Build: release shrinks/obfuscates (`minifyEnabled` + `shrinkResources`), enables ABI splits for Play, strips Rust symbols with size-focused profile, excludes unused META-INF resources, disables BuildConfig, targets arm64-v8a only. Cargo task resolves `cargo` from PATH/`CARGO`. Gradle density splits removed (AAB handles density).
- Tests: `cargo test` passes; `./gradlew test` passes with Robolectric TextInput/Checkbox/Progress binding tests.

## Known Issues / Risks
- Renderer still trusts incoming JSON and can crash on malformed output; Kotlin has a fallback but we lack schema validation and more granular error UI.
- State is ephemeral; no serialization/restoration path; limited unit tests on dispatch/render; Kotlin JSON parsing is only lightly covered (TextInput/Checkbox tests, not yet in CI).
- Cargo build task still compiles only arm64-v8a but stale armeabi-v7a artifacts may exist locally; packaging ignores them. Image conversion flow depends on MediaStore/SAF; on-device permission UX not yet validated.
- Gradle wrapper download may hit filesystem permission errors on some hosts; rerun with writable ~/.gradle or vendored distribution (current run succeeded with permissions).

## Next Implementation Step
1. Add list/grid rendering for the menu (or feature cards) to avoid long vertical scroll as features grow.

## Near-Term Tasks
- Introduce typed `Command`/`Action` + richer `Screen` enum in Rust (navigation stack) and move UI generation to builders/serde structs.
- Kotlin side: add schema validation/guardrails for renderer JSON; add a small loading indicator while hashing or converting; ensure fallback renders on malformed JSON.
- Execute Robolectric tests routinely and wire into CI; fix Gradle wrapper permissions or vendor the distribution to keep tests runnable.
- Align cargo build targets to arm64-only to avoid producing unused v7a libs and remove stale v7a .so; regenerate AAB and verify size with `scripts/size_report.sh`.

## MVP / Easy Wins
- Add lightweight unit tests for Kotlin renderer JSON parsing beyond TextInput/Checkbox (e.g., unknown type handling).

## Feature Ideas (low dependency)
- QR code generator: Rust `qrcode` crate (tiny), output PNG bytes; Kotlin decodes with `BitmapFactory.decodeByteArray` and renders in `ImageView`.
- Text tools: word/char/line counts via Rust std; Base64/hex/URL encode-decode with `data-encoding` or `percent-encoding` (small); UTF-8/UTF-16 conversions via std.
- Compression: Zip list/extract/create with `zip` crate (deflate/store only). Tar/gzip via `tar` + `flate2` if needed; avoid 7z/rar.
- Image conversion: prefer Kotlin `BitmapFactory`/`Bitmap.compress` for PNG/JPEG/WebP to avoid new Rust crates; Rust std has no codecs.
- Hash/Checksum expansion: add CRC32/CRC64 (`crc32fast`/`crc`) or BLAKE3 (`blake3` crate, small/fast).
- File info: MIME sniff via `infer` (light), hex viewer via std IO, file metadata via std.
- Sensor logging: Kotlin `SensorManager` on a background thread, log to CSV in app storage; no new deps.
- Color/encoding utilities: color conversions (hex/RGB/HSL) via std math; password/random bytes with `rand` minimal features.


Phase 1: Stability & Foundation (Immediate)
Harden JNI Boundary:
Refactor lib.rs to wrap logic in std::panic::catch_unwind.
Create a standardized Error JSON response structure.
Zero-Copy File Access:
Update Kotlin MainActivity to open a ParcelFileDescriptor instead of copying streams.
Update Rust Command struct to accept an optional fd: i32 field.
Refactor features/hashes.rs to read from the raw FD.
UI Scrolling:
Update UiRenderer.kt: Wrap the returned View in a ScrollView if the root type is a Column.
Phase 2: Core Components (Next Session)
State Machine Evolution:
Replace String based routing with a Vec<Screen> stack in Rust to support "Back" functionality.
Implement the restore_state logic.
UI Widget Expansion:
Implement TextInput (for text hashing/comparison).
Implement ProgressBar (for long operations).
Thread Management:
Currently, handle_hash_action blocks the thread calling dispatch. Even if Kotlin calls it on IO, it prevents sending other events (like "Cancel").
Goal: Move hash computation to a Rust background thread (using std::thread or rayon), and have the main Rust state return a "Computing..." screen immediately. Poll for results or use a callback.
Phase 3: Packaging
Cleanup Gradle:
Translate comments.
Remove the armeabi-v7a build argument from build.gradle.kts if the target is strictly arm64-v8a to speed up compilation.
