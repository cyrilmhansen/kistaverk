# Kistaverk

**A tiny offline Swiss-knife for your files.**

Kistaverk is a lightweight, privacy-friendly Android toolbox for everyday “power user” tasks:
file hashes, PDF surgery, image conversion, and other small utilities – all in a single,
minimal app that runs entirely on your device.

No accounts, no cloud, no tracking. Just tools.

---

## Goals

- **All-in-one, but small**  
  One APK, multiple utilities (hashing, PDF, images, text tools…), with a strong size budget
  (target: < 5 MB).

- **Local-only & auditable**  
  No unsolicited network access. Logic implemented in Rust, open source, reviewable.

- **Long-term maintainable**  
  Clear separation between a thin Android UI layer and a Rust “core” with a simple JSON DSL.

---

## Features (planned)

Early scope (subject to change):

- **File integrity**
  - Compute and verify hashes (SHA-256, SHA-512, etc.).
  - Compare with a reference hash (clipboard / pasted / scanned via QR).

- **PDF utilities**
  - Extract a subset of pages into a new PDF.
  - Concatenate multiple PDFs (simple merge).
  - Remove selected pages.

- **Image utilities**
  - Resize images for mail / messaging size limits (e.g. “< 200 kB”).
  - Format conversion (JPEG / PNG / WebP, where supported).
  - Basic quality / size trade-off presets.

- **Sharing & export**
  - Copy results to clipboard.
  - Share via Android’s share sheet.
  - Optional QR code display for hashes or short text.

- **Sensor logger**
  - Configurable sampling interval and sensor selection (accelerometer, gyroscope, magnetometer, barometer, GPS, battery stats).
  - Logs to CSV in app-private storage with FileProvider sharing; GPS requires location permission.

---

## Architecture (high-level)

Kistaverk uses a **local backend-driven UI**:

- **Core (Rust)**
  - Holds all application state.
  - Implements business logic (crypto, PDF, image processing).
  - Exposes a single `dispatch(json)` entry point via JNI.
  - Returns full screen descriptions as JSON (a small UI DSL).

- **UI (Android / Kotlin)**
  - Very thin “renderer” on top of the standard Android SDK.
  - Parses JSON from Rust and instantiates native Views (`LinearLayout`, `TextView`, `Button`, etc.).
  - Forwards user actions back to Rust as JSON commands.

This keeps the logic safe and explicit in Rust, while preserving native Android ergonomics
and accessibility.

For more details, see:

- `VISION.md` – project philosophy and goals.  
- `ARCHITECTURE.md` – technical architecture and module layout.  
- `METHODOLOGY.md` – rules for AI-assisted and human development.  

---

## Project layout

Planned structure:

```text
/
├── app/                   # Android frontend (Kotlin)
│   └── src/main/java/com/kistaverk/toolnest/
│       ├── ui/            # JSON -> View renderer
│       └── core/          # JNI bridge and app shell
├── rust/                  # Rust core (Cargo project)
│   ├── src/
│   │   ├── lib.rs         # JNI entry + dispatch
│   │   ├── state.rs       # application state machine
│   │   ├── ui_gen.rs      # JSON DSL builders
│   │   └── modules/       # crypto, pdf, image, etc.
│   └── Cargo.toml
├── VISION.md
├── ARCHITECTURE.md
├── METHODOLOGY.md
├── CONTRIBUTING.md
└── WORKINPROGRESS.md

---

## Build & Packaging

- Release builds are shrunk/obfuscated (R8 + resource shrinking) with stripped Rust libs and size-focused Rust profile flags.  
- ABI outputs: arm64-v8a only (APK/AAB). Per-ABI splits are enabled; no density splits (Play handles density).  
- Build release APKs/AAB: from `app/`, run `./gradlew clean :app:assembleRelease` or `./gradlew :app:bundleRelease`.  
- Size audit: run `./scripts/size_report.sh <apk-or-aab>` to break down dex/lib/res and per-ABI .so sizes.
