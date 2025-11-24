# WORK IN PROGRESS

*Use this file to track the active context for AI Agents. Update it at the end of every coding session.*

## 📅 Current Status
**Last Updated:** 2025-11-24
**Phase:** File-hash demo live + Shader demo + Text tools screen + PDF page tools (extract/delete/merge/title) + PDF signature stamping + About screen + Play packaging (arm64-only)


📝 NEXT STEPS: Project Roadmap
1. Core Architecture: From "Toy" to "Engine"
Currently, we have a global integer counter. We need a robust system to manage complex states (file paths, loading states, navigation history).
State Machine: AppState now holds a `Vec<Screen>` stack (Home-root) and typed `Action`/`TextAction` enums; hardware Back triggers a pop in Rust and returns the previous screen JSON.
Next: wire snapshot/restore into CI and add schema validation for renderer JSON; consider moving UI generation to serde-friendly builders for schema validation.
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
- Kotlin now launches the system file picker (detaching FDs), forwards a `bindings` map with UI state, and renders Text/Button/ShaderToy/TextInput/Checkbox/Progress; Columns wrap in ScrollView and Grids auto-pick columns (1/2). Overlay spinner keeps prior screen visible during loading-only calls.
- Rust owns UI and navigation via a `Vec<Screen>` stack with typed `Action`/`TextAction`; hardware Back from Kotlin calls `back` and Rust pops safely (Home is root). Inline Back buttons only appear when depth > 1. `snapshot`/`restore_state` serialize/rehydrate AppState; Kotlin persists the snapshot in the Activity bundle.
- Features: streaming hashes (SHA-256/SHA-1/MD5/MD4/CRC32/BLAKE3), Shader demo, Kotlin image conversion flow with output dir selection, Text Tools (upper/lower/title/wrap/trim/count/Base64/URL/Hex) with copy/share hooks, Progress demo, File info, Text viewer (text/CSV preview with 256KB cap), QR generator (base64 PNG), Color Converter (Hex↔RGB/HSL with swatch and per-format copy buttons), PDF tools (page picker + extract/delete/merge/title, signature stamping via lopdf, PdfRenderer thumbnails, SignaturePad capture), About screen (version, copyright, GPLv3).
- Renderer guardrails: Kotlin validates JSON with a widget whitelist/required children before rendering; malformed payloads fall back to an inline error screen. Accessibility strings flow through `content_description`. Clipboard copy supported via `copy_text` on buttons; clipboard text is injected into bindings when small.
- Build: release shrinks/obfuscates (`minifyEnabled` + `shrinkResources`), ABI splits arm64-only, symbols stripped; Cargo path resolved from env/PATH. Tests: `cargo test` green; `./gradlew test` currently blocked on some hosts by Gradle wrapper permissions.
- Sensor logger: Rust screen now exposes toggles (accel/gyro/mag/pressure/GPS/battery), interval binding, status/error text, and share gating. Kotlin logger consumes bindings, registers only selected sensors at the chosen interval on a background thread, writes CSV with FileProvider sharing, updates Rust status/path during logging, and requests location permission only when GPS is enabled.

## Known Issues / Risks
- Renderer still trusts incoming JSON and can crash on malformed output; Kotlin has a fallback but we lack schema validation and more granular error UI.
- State is ephemeral; no serialization/restoration path; limited unit tests on dispatch/render; Kotlin JSON parsing is only lightly covered (TextInput/Checkbox tests, not yet in CI).
- Cargo build task still compiles only arm64-v8a but stale armeabi-v7a artifacts may exist locally; packaging ignores them. Image conversion flow depends on MediaStore/SAF; on-device permission UX not yet validated.
- Gradle wrapper download may hit filesystem permission errors on some hosts; rerun with writable ~/.gradle or vendored distribution (current run succeeded with permissions).
- Sensor logger needs on-device validation for TalkBack labels, GPS permission flow, and CSV accuracy/throttling; consider debouncing UI refreshes during high-frequency events.

## Next Implementation Step
1. Validate launcher alias/deep link for PDF signature flow (secondary icon) across devices; ensure intent extra `entry=pdf_signature` opens PdfTools and back/reset behaves correctly.

## Near-Term Tasks
- Kotlin side: schema validation already guards widget types/required fields with fallback UI; extend coverage (e.g., per-widget required props) and add a small loading indicator while hashing or converting.
- Move UI generation toward serde structs/builders to align with schema validation and reduce ad-hoc JSON.
- Wire snapshot/restore path into CI and keep native loading mocked in Robolectric to avoid JNI deps.
- Execute Robolectric tests routinely and wire into CI; fix Gradle wrapper permissions or vendor the distribution to keep tests runnable.
- Align cargo build targets to arm64-only to avoid producing unused v7a libs and remove stale v7a .so; regenerate AAB and verify size with `scripts/size_report.sh`.
- Add UI/UX for signature placement preview and validate PDF alias entry path (activity-alias); verify PdfRenderer/FD flows under TalkBack.
- Ensure deps.json generation stays wired (rust/scripts/generate_deps_metadata.sh hooked to Gradle preBuild) and About shows scrollable deps list from assets.
- Sensor logger: on-device QA (TalkBack labels, permission UX), tune GPS sampling interval and throttling, and validate CSV content/share flow.
- Text viewer: on-device check for large files (truncation UX), binary/invalid UTF-8 handling, and TalkBack labels.

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



Voici une synthèse structurée et priorisée des nouvelles fonctionnalités identifiées lors de nos échanges, intégrée à la vision globale du projet Kistaverk.
Cette liste est filtrée par notre philosophie : Rust Core (Logique) + Kotlin Native (UI/System) + Zero-Bloat (< 5MB).
🟢 Priorité 1 : Le "Cœur" (MVP Consolidation)
Ces fonctionnalités complètent la base actuelle pour rendre l'outil indispensable au quotidien.
Gestion PDF "Chirurgicale"
Besoin : Extraire/Supprimer des pages, Fusionner.
Tech (Kotlin) : Utiliser l'API native android.graphics.pdf.PdfRenderer (dispo depuis Android 5.0) pour générer les aperçus (Bitmap) des pages afin que l'utilisateur puisse les sélectionner dans l'UI. Pas de WebView (trop lourd/imprévisible).
Tech (Rust) : Utiliser la crate lopdf pour manipuler la structure du fichier PDF et sauvegarder le résultat sans perte de qualité.
Gestionnaire d'Archives (Beyond Zip)
Besoin : Ouvrir des tar.gz, tar.bz2, xz sur Android.
Tech (Rust) : Les crates tar, flate2, bzip2, xz2 sont performantes et sûres.
UI : Une vue arborescente simple (Tree View) générée via le JSON.
Dithering (Tramage) Rétro
Besoin : Esthétique "Pixel Art", préparation d'images pour écrans e-ink ou imprimantes thermiques.
Tech (Rust) : Implémentation des algorithmes Floyd-Steinberg, Atkinson et Ordered Dithering. C'est du calcul pur sur les pixels, parfait pour Rust.
🟡 Priorité 2 : La "Geek Suite" (Developer Tools)
Outils spécialisés pour les développeurs, accessibles via le mode standard ou Geek.
Solveur d'Équations (Style "Mercury/Eureka")
Besoin : Résoudre des systèmes d'équations (
2
x
+
y
=
10
2x+y=10
) ou trouver des racines (
x
2
−
4
=
0
x 
2
 −4=0
).
Tech (Rust) : Plutôt que d'intégrer un énorme moteur C++ comme Giac/Xcas (trop gros), nous implémenterons un solveur numérique itératif (Méthode de Newton-Raphson) ou un petit moteur symbolique en pur Rust.
Input : Un champ texte multi-lignes où l'utilisateur tape ses équations comme dans le manuel Mercury fourni.
Convertisseur Vectoriel (SVG <-> VectorDrawable)
Besoin : Les devs Android détestent convertir manuellement des SVG en XML VectorDrawable.
Tech (Rust) : Parsing XML du SVG et réécriture en XML Android. Utilité immédiate pour le dev mobile.
Simplificateur Logique (Tableau de Karnaugh)
Besoin : Simplifier des expressions booléennes ((A AND B) OR (A AND NOT B) -> A).
Tech (Rust) : Algorithme de Quine-McCluskey.
UI : Une grille interactive représentant le tableau de Karnaugh où l'utilisateur toggle les 0 et 1.
Calculatrice RPN & Convertisseur de Base
Validé précédemment : Pile infinie, conversion Hex/Bin/Dec en temps réel.
🔴 Priorité 3 : Le "Labo" (Mode Expert / Easter Eggs)
Fonctionnalités cachées derrière les "7 taps", expérimentales ou très avancées.
Kista-Forth (Langage de Script)
Validé précédemment : Interpréteur Forth complet pour scripter les fonctions internes de l'app (hash, convert, math).
Automata Lab (Wolfram NKS)
Validé précédemment : Génération d'automates cellulaires 1D (Rule 30, 110) avec rendu Bitmap.
Transfert de Données Haute Densité (Color QR / JAB Code)
Besoin : Transférer un fichier (ex: une clé GPG, un petit fichier de conf) d'un écran à un autre sans réseau (AirGap).
Tech (Rust) : Encodage binaire vers une matrice de couleurs (Cyan, Magenta, Jaune, Noir).
Tech (Kotlin) : Affichage plein écran haute luminosité. Note : La lecture (scan caméra) est complexe à faire en "Zero-Bloat", on se limitera peut-être à la génération (émetteur) dans un premier temps.
Capteurs & Hardware (Système Android)
Besoin : Debugger le matériel.
Tech (Kotlin) : Utiliser SensorManager pour lire Magnétomètre, Gyroscope, Pression, et BatteryManager.
Tech (Rust) : Recevoir les données brutes, appliquer des filtres (Kalman ?) ou des stats, et renvoyer le JSON pour afficher des graphiques en temps réel.
❌ Idées écartées (Pour l'instant)
WebView pour le PDF : Trop lourd, trop variable selon les versions d'Android, risque de failles de sécurité. On préfère PdfRenderer (natif).
Intégration Giac/Xcas complète : Trop lourd (plusieurs Mo). On préfère un solveur Rust léger "fait maison" inspiré de Mercury.
🗺️ Synthèse de la Roadmap Technique
Architecture : Finaliser la machine à état Rust (Stack de navigation).
UI Engine : Ajouter les widgets manquants (TreeView pour archives, Canvas/Bitmap pour Dithering/Automates).
Implémentation P1 : PDF + Archives + Dithering.
Implémentation P2 : Solveur + Logic tools.
Implémentation P3 : Le "Mode Geek" (Forth + Automates).
