# WORK IN PROGRESS

*Use this file to track the active context for AI Agents. Update it at the end of every coding session.*

## 📅 Current Status
**Last Updated:** YYYY-MM-DD
**Phase:** Initialization / PoC

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