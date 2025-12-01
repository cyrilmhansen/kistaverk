# Task In Progress: Regex Tester

## Feature Description
Implement a "Regex Tester" tool that allows users to input a regular expression pattern and a sample text string. The tool will compile the regex, test it against the sample, and display whether it matches, along with any captured groups or syntax errors.

## Plan

### Step 1: Update Rust State (`rust/src/state.rs`)
*   **Goal:** Manage state for the regex tester.
*   **Actions:**
    1.  Add `Screen::RegexTester` to the `Screen` enum.
    2.  Create `RegexMatchResult` struct to hold match details (full match, groups).
    3.  Create `RegexTesterState` struct with fields:
        *   `pattern: String`
        *   `sample_text: String`
        *   `match_result: Option<RegexMatchResult>`
        *   `error: Option<String>`
    4.  Add `regex_tester: RegexTesterState` to `AppState`.
    5.  Initialize in `AppState::new()` and `reset_runtime()`.

### Step 2: Implement Core Logic (`rust/src/features/regex_tester.rs`)
*   **Goal:** Implement regex compilation and matching.
*   **Actions:**
    1.  Create `rust/src/features/regex_tester.rs`.
    2.  **Dependency Check:** `regex` crate (v1) is present with `std` and `unicode-perl` features.
    3.  Implement `test_regex(pattern: &str, text: &str) -> Result<RegexMatchResult, String>`:
        *   Compile with `regex::Regex::new`.
        *   On error, return the error string.
        *   On success, use `captures` to find the first match and extract groups.
    4.  Implement `handle_regex_action` to update state from bindings and run the test.

### Step 3: Implement UI Rendering (`rust/src/features/regex_tester.rs`)
*   **Goal:** Create the user interface.
*   **Actions:**
    1.  Implement `render_regex_tester_screen(state: &AppState) -> Value`.
    2.  UI Elements:
        *   Header "Regex Tester".
        *   Input field for Pattern (`regex_pattern`).
        *   Input field for Sample Text (`regex_sample`, potentially multi-line).
        *   "Test" button (trigger `regex_test` action).
        *   **Results Area:**
            *   Syntax Error display (if any).
            *   Match Status (Match/No Match).
            *   List of captured groups.

### Step 4: Integrate into JNI Dispatch (`rust/src/lib.rs`)
*   **Goal:** Connect actions.
*   **Actions:**
    1.  Add actions:
        *   `RegexTesterScreen`
        *   `RegexTest`
    2.  Update `parse_action` to map `regex_tester_screen` and `regex_test`.
    3.  Update `handle_command` to call `handle_regex_action`.
    4.  Update `render_ui` map.
    5.  Add to `feature_catalog`.

### Step 5: Testing
*   **Actions:**
    1.  Unit test `test_regex` with:
        *   **Valid patterns:** Simple literals, character classes, quantifiers.
        *   **Invalid patterns:** Unclosed groups, invalid escapes (verify error messages).
        *   **Capture groups:** Named and unnamed groups.
        *   **No match:** Pattern valid but text doesn't match.
        *   **Complex/Edge cases:** Empty pattern, empty text, unicode characters.
        *   *Note:* Rust's `regex` crate guarantees linear time execution, so catastrophic backtracking is less of a concern than with PCRE, but valid inputs that might be slow should be considered (though specific performance tests might be out of scope for a simple tool).
    2.  Manual test on device to verify UI flow and input handling.

---

## Completed Tasks
*   **Dithering Tools**: Done.
*   **Multi-hash view**: Done.
*   **Refactoring lib.rs**: Done.
*   **PDF Thumbnail Grid**: Rust side done (Kotlin pending).
*   **Pixel Art Mode**: Implemented (assumed per workflow).