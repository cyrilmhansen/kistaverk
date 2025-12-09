# Task In Progress: File Encryption/Decryption (The "Vault")

## Status: Implemented
*   **Date:** 2025-12-09
*   **Objective:** Implement secure file encryption and decryption using the `age` crate. This aligns with the app's privacy-first mission ("The Vault").
*   **Plan:**
    1.  **Dependencies (`rust/Cargo.toml`):**
        *   Add `age = "0.10"` (or compatible version).
        *   Add `secrecy` or similar if needed for password handling, or just use `String` for simplicity in this prototype.
    2.  **State (`rust/src/state.rs`):**
        *   Define `VaultState` struct:
            *   `input_path: Option<String>`
            *   `password: String`
            *   `status: Option<String>`
            *   `error: Option<String>`
            *   `is_processing: bool`
    3.  **UI Protocol (`rust/src/ui.rs`):**
        *   Update `TextInput` struct and builder to include `password_mask: Option<bool>`.
        *   This is required to visually mask the password in the Android client.
    4.  **Implementation (`rust/src/features/vault.rs`):**
        *   Implement `encrypt_file(path: &str, password: &str) -> Result<PathBuf, String>`.
            *   Use `age::Encryptor::with_user_passphrase`.
            *   Output: `filename.age`.
        *   Implement `decrypt_file(path: &str, password: &str) -> Result<PathBuf, String>`.
            *   Use `age::Decryptor::new`.
            *   Output: Remove `.age` suffix or append `.dec`.
        *   Implement `render_vault_screen(state: &AppState) -> Value`.
            *   UI Components: FilePicker, PasswordInput (with `password_mask(true)`), Encrypt Button, Decrypt Button.
            *   "Save as…" button (similar to GZIP feature) on success.
    5.  **Integration:**
        *   Register module in `rust/src/features/mod.rs`.
        *   Add `VaultState` to `AppState`.
        *   Add `WorkerJob::Vault { ... }` and `WorkerResult::Vault { ... }` in `rust/src/router.rs`.
        *   Handle actions: `vault_screen`, `vault_pick`, `vault_encrypt`, `vault_decrypt`, `vault_save_as`.
    6.  **Kotlin (`MainActivity.kt`):**
        *   Reuse existing `launchSaveAs` logic for `vault_save_as`.
        *   Ensure `.age` extension is handled in `guessMimeFromPath` (application/age-encryption or application/octet-stream).
    7.  **Tests:**
        *   **Roundtrip Test:** Encrypt a sample file -> Decrypt it -> Assert content matches original.
        *   **Bad Password Test:** Encrypt a file -> Attempt decrypt with wrong password -> Assert error.
        *   **Invalid File Test:** Attempt decrypt on a random file -> Assert error.
        *   **UI Password Mask Test:** Verify `TextInput` serializes `password_mask: true` correctly.
        *   **UI Success State Test:** Verify "Save As" button appears only on success.

## Previous Task: Standardize "Save As" Flow (GZIP)
*   **Status:** Implemented
*   **Date:** 2025-12-09
*   **Summary:** Implemented "Save As" for GZIP tools. Fixed MIME type detection in Kotlin and updated Rust worker output to support path capture.