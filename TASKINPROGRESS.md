# Task In Progress: JWT Decoder

## Status: Implemented
*   **Date:** 2025-12-09
*   **Objective:** Implement a JWT (JSON Web Token) Decoder as part of the "Developer Tools" suite. This allows users to paste a token and inspect its Header and Payload without network calls.
*   **Plan:**
    1.  **State (`rust/src/state.rs`):**
        *   Define `JwtState` struct:
            *   `input_token: String`
            *   `decoded_header: Option<String>`
            *   `decoded_payload: Option<String>`
            *   `error: Option<String>`
    2.  **Implementation (`rust/src/features/jwt.rs`):**
        *   Implement `decode_jwt(token: &str) -> Result<(String, String), String>`:
            *   Split token by `.` (expect 3 parts).
            *   URL-safe Base64 decode parts 0 (header) and 1 (payload).
            *   Parse as JSON and pretty-print using `serde_json`.
        *   Implement `render_jwt_screen(state: &AppState) -> Value`.
            *   UI:
                *   `TextInput` for Token (multiline).
                *   Buttons: "Decode", "Clear", "Paste from Clipboard".
                *   `CodeView` for Header (if present).
                *   `CodeView` for Payload (if present).
    3.  **Integration (`rust/src/router.rs`):**
        *   Register module in `rust/src/features/mod.rs`.
        *   Add `JwtState` to `AppState`.
        *   Add `Action` variants: `JwtScreen`, `JwtDecode`, `JwtClear`, `JwtPaste`.
        *   Handle actions in `handle_command`.
    4.  **Tests:**
        *   **Decode Logic:** Test with valid JWTs (HS256, RS256).
        *   **Error Handling:** Test with invalid Base64, malformed JSON, or non-JWT strings.

## Previous Task: Logical Engine (RDF-like Data Sets)
*   **Status:** Implemented
*   **Date:** 2025-12-09
*   **Summary:** Implemented lightweight RDF triple store and query engine.
