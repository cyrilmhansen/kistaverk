# Kistaverk

**The Offline Digital Swiss Army Knife.**

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Platform](https://img.shields.io/badge/platform-Android-green)
![Stack](https://img.shields.io/badge/stack-Rust%20%2B%20Kotlin-orange)

**Kistaverk** (Icelandic for *"Chest of works"*) is a privacy-first, lightweight Android toolbox for power users. It provides essential file and data utilities—hashing, PDF manipulation, image conversion, sensor logging—without the bloat, ads, or cloud dependency of typical "free" apps.

**Target Size:** < 5 MB.  
**Philosophy:** 100% Local. No Internet Permission.

---

## 🛠 Features

Everything runs locally on your device using a high-performance Rust core.

### 🔐 File Security & Integrity
- **Hash Calculator:** Compute MD5, SHA-1, SHA-256, SHA-512, BLAKE3, and CRC32.
- **Integrity Check:** Paste a hash from your clipboard to verify a file's integrity instantly.

### 📄 PDF Tools
- **PDF Splitter:** Extract specific pages to a new file.
- **PDF Merge:** Concatenate multiple PDF files.
- **PDF Signing:** Sign documents with a visual overlay for signature placement.
- **Page Removal:** Delete unwanted pages.

### 🖼 Image & Media
- **Converter:** Convert images between JPEG, PNG, and WebP.
- **Compression:** Resize and compress images for email/messaging (bypass attachment limits).
- **QR Code Generator:** Create QR codes for text or URLs.

### 📊 System & Sensors
- **Sensor Logger:** Log accelerometer, gyroscope, magnetometer, GPS, and barometer data to CSV files.
- **System Info:** View basic device details (model, architecture).

### 📝 Text & Dev Tools
- **Text Viewer:** Syntax highlighting for JSON, Markdown, Rust, TOML, etc. (via PrismJS). Supports large logs via windowed loading.
- **Regex Tester:** Test regular expressions against text input.
- **Archive Viewer:** Peek inside ZIP files without extracting them.

---

## 🏗 Architecture

Kistaverk uses a unique **Local Backend-Driven UI** architecture to maximize performance and maintainability while keeping the APK size tiny.

- **Core (Rust):** The "Brain". Handles all business logic, state management, and IO. It dictates the UI by sending a JSON DSL to the Android layer.
- **Renderer (Kotlin):** The "Dumb Terminal". A thin layer that renders standard Android Views (LinearLayout, Button, TextView) based on the JSON instructions from Rust.

This approach allows:
1.  **Cross-platform logic:** 95% of the code is in Rust.
2.  **Native Performance:** No webviews (except for syntax highlighting), no heavy UI frameworks like Flutter/React Native. Just raw native Android views.
3.  **Security:** Complex parsing (PDF, Zip) happens in memory-safe Rust.

See [ARCHITECTURE.md](ARCHITECTURE.md) for a deep dive.

---

## 🚀 Building from Source

### Prerequisites
- **Rust:** Stable toolchain (`rustup install stable`).
- **Android SDK:** API 36 (Command-line tools + NDK).
- **cargo-ndk:** `cargo install cargo-ndk`.
- **JDK:** Version 17 or higher.

### Build Steps

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/aeska/kistaverk.git
    cd kistaverk
    ```

2.  **Build the APK:**
    ```bash
    # This will compile the Rust core and the Android app
    cd app
    ./gradlew :app:assembleRelease
    ```

    *Note: The build is currently configured for `arm64-v8a` only to optimize iteration speed and size.*

3.  **Install:**
    ```bash
    adb install app/build/outputs/apk/release/app-release.apk
    ```

---

## 📂 Project Structure

```text
/
├── app/                   # Android Application Project
│   ├── app/src/main/java  # Kotlin UI Renderer & JNI Bridge
│   └── build.gradle.kts   # Gradle build with Cargo hooks
├── rust/                  # Rust Core Logic
│   ├── src/               # Source code (State, Features, UI DSL)
│   └── Cargo.toml         # Rust dependencies
├── scripts/               # Utility scripts (size reports, metadata)
└── ...                    # Documentation
```

---

## 🤝 Contributing

Contributions are welcome! Whether it's a new Rust feature or a UI renderer improvement.
Please read [CONTRIBUTING.md](CONTRIBUTING.md) and [VISION.md](VISION.md) before starting.

## 📄 License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)** - see the [LICENSE](LICENSE) file for details.