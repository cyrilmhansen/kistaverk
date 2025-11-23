# CONTRIBUTING GUIDE

## 🛠️ Build Environment Setup

### 1. Standard Android Requirements
*   Android Studio (latest stable).
*   Android SDK & NDK (Side-by-side).

### 2. Rust Requirements (The Core)
You don't need to be a Rust expert, but you need the toolchain.
1.  **Install Rust:** `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2.  **Add Android Targets:**
    ```bash
    rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
    ```
3.  **Install Cargo-NDK:** (Helper tool to link correctly)
    ```bash
    cargo install cargo-ndk
    ```

## 🚀 How to add a Feature

### Example: Adding a UUID Generator

1.  **Kotlin UI (JSON):**
    Add a button in the DSL that sends action `generate_uuid`.

2.  **Rust Logic (`/rust/src/lib.rs`):**
    *   Add `uuid = { version = "1.0", features = ["v4"] }` to `Cargo.toml`.
    *   In the dispatch loop:
        ```rust
        "generate_uuid" => {
            let id = uuid::Uuid::new_v4();
            json!({ "result": id.to_string() })
        }
        ```
    *   *Notice:* No Makefiles to edit! Cargo downloads and compiles the UUID library automatically.
    
