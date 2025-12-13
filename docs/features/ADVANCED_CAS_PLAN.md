# Advanced CAS Implementation Plan

This document outlines a phased approach for integrating arbitrary-precision arithmetic into the existing Computer Algebra System (CAS) in `kistaverk`, primarily focusing on the `math_tool` feature. The plan is broken down into smaller, independent, actionable tasks, prioritizing safety and reversibility.

## Phase 1: Foundation & Abstraction (Safe & Reversible)
*These tasks prepare the codebase without introducing heavy external dependencies yet. They are foundational for any future numeric library integration.*

### Task 1.1: Define Numeric Abstraction Layer (`src/features/cas_types.rs`)
*   **Goal:** Create a flexible `Number` enum (or trait) that currently wraps `f64` but is designed for future extension to support arbitrary-precision types.
*   **Action:**
    1.  Create the file `src/features/cas_types.rs`.
    2.  Define an `enum Number { Fast(f64) }`.
    3.  Implement essential arithmetic traits (`Add`, `Sub`, `Mul`, `Div`) for `Number`.
    4.  Implement comparison traits (`PartialEq`, `PartialOrd`).
    5.  Add a constructor (e.g., `from_f64`) to convert `f64` into `Number::Fast`.
*   **Verification:** Write unit tests in `src/features/cas_types.rs` to ensure `Number` instances behave correctly and as expected when performing arithmetic and comparisons, mirroring `f64` behavior.

### Task 1.2: Refactor Tokenizer to Defer Parsing
*   **Goal:** Modify the tokenizer to postpone the conversion of numeric strings to concrete numeric types until the desired precision is known. This allows for dynamic selection of precision at evaluation time.
*   **Action:**
    1.  In `src/features/math_tool.rs`, change the `Token::Number(f64)` variant to `Token::NumberStr(String)`.
    2.  Adjust the `tokenize` function to store the raw string representation of numbers in `Token::NumberStr`.
    3.  Update the logic in the parser/shunting-yard algorithm to handle `Token::NumberStr`. The actual conversion to the `Number` type (from Task 1.1) should now happen later, closer to evaluation.
*   **Verification:** Ensure all existing unit tests for `math_tool.rs` pass after this change. This confirms that the tokenizer still correctly identifies numbers and the parser can process the new `Token::NumberStr` variant without functional regressions.

### Task 1.3: Refactor AST (`Symbol`) and Evaluation to use Abstraction
*   **Goal:** Decouple the Abstract Syntax Tree (AST) and the evaluation logic from the concrete `f64` type, making them compatible with the new `Number` abstraction.
*   **Action:**
    1.  In `src/features/math_tool.rs`, replace all occurrences of `f64` within the `Symbol` enum variants (e.g., `Symbol::Number(f64)`) with the `Number` type (from Task 1.1).
    2.  Modify functions such as `evaluate_expression`, `eval_rpn`, `differentiate`, and `integrate` to operate on the `Number` type instead of `f64`. This will involve updating function signatures and internal arithmetic operations.
*   **Verification:** Run all existing unit tests in `src/features/math_tool.rs`. Use `cargo check` extensively to identify any remaining `f64` usages that need conversion.

## Phase 2: Arbitrary Precision Logic (Linux/Desktop First)
*These tasks introduce the arbitrary-precision library (`rug`) but initially keep it behind a feature flag to isolate its impact, especially on Android builds. Development should ideally occur on a Linux environment.*

### Task 2.1: Add `rug` Dependency with Feature Flag
*   **Goal:** Incorporate `rug` and its dependencies into the project's build system in a way that allows it to be optionally enabled.
*   **Action:**
    1.  In `rust/Cargo.toml`, add `rug = { version = "...", optional = true }` to the `[dependencies]` section.
    2.  Add a `[features]` section if it doesn't exist, and define a new feature: `precision = ["rug"]`.
*   **Verification:** `cargo check --features precision` should complete without errors on a Linux development machine.

### Task 2.2: Implement Precision Variant within `Number`
*   **Goal:** Extend the `Number` enum (from Task 1.1) to natively support `rug::Float` when the `precision` feature is enabled.
*   **Action:**
    1.  In `src/features/cas_types.rs`, add a `Precise(rug::Float)` variant to the `Number` enum, guarding it with `#[cfg(feature = "precision")]`.
    2.  Implement arithmetic and comparison traits for `Number::Precise`, ensuring they delegate to `rug::Float`'s implementations.
    3.  Add conversion methods to `Number` to convert from `rug::Float` and to convert between `Fast` and `Precise` variants (e.g., `to_f64`, `to_rug_float`).
*   **Verification:** Write specific unit tests for `cas_types.rs` that are enabled by `#[cfg(feature = "precision")]`. These tests should verify `rug::Float` functionality through the `Number` enum. Run `cargo test --features precision`.

### Task 2.3: Integrate Precision into Evaluation Context & Logic
*   **Goal:** Allow the `math_tool` to dynamically switch between `f64` and `rug::Float` based on a user-defined precision setting.
*   **Action:**
    1.  Add a `precision_bits: u32` field to `MathToolState` in `src/state.rs`. This value will control the precision of `rug::Float` operations (e.g., `0` for `f64`, `64` for standard `rug::Float` precision, `128` for higher precision).
    2.  Modify the `evaluate_expression` function in `src/features/math_tool.rs` to accept a `precision: u32` parameter.
    3.  Inside `evaluate_expression` and `eval_rpn`, implement logic to:
        *   If `precision_bits` is 0 (or a designated value), use `Number::Fast(f64)`.
        *   If `precision_bits` > 0 and `precision` feature is enabled, parse numeric strings into `rug::Float` and perform all calculations using `Number::Precise(rug::Float)`.
*   **Verification:** Add unit tests to `math_tool.rs` that test evaluation with different precision settings. This requires `#[cfg(feature = "precision")]` for tests involving `rug::Float`.

## Phase 3: Android Integration (High Difficulty & Risk)
*These tasks address the challenging cross-compilation requirements for `rug` and its C dependencies on Android. This phase should only begin after Phase 2 is stable.*

### Task 3.1: Prototype Android Build Script for GMP/MPFR
*   **Goal:** Develop a standalone script capable of cross-compiling GMP, MPFR, and MPC (the C libraries underlying `rug`) for Android targets.
*   **Action:**
    1.  Create a new shell script `scripts/build_gmp_android.sh`.
    2.  This script should:
        *   Download the source code for GMP, MPFR, and MPC.
        *   Configure these libraries using the Android NDK toolchain for target architectures (e.g., `aarch64-linux-android`, `armv7-linux-androideabi`).
        *   Build the libraries as static archives (`.a` files).
    3.  Store the compiled static libraries in a designated location (e.g., `rust/libs/android/<abi>/`).
*   **Verification:** Manually run the script and confirm that static library files (`.a`) are successfully generated for at least one Android ABI (e.g., `aarch64-linux-android`).

### Task 3.2: Integrate with Rust `build.rs` for Android Linking
*   **Goal:** Instruct Cargo's build system to link against the cross-compiled C libraries when building for Android with the `precision` feature enabled.
*   **Action:**
    1.  Modify `rust/build.rs`.
    2.  Add logic to detect the Android target (`cfg(target_os = "android")`) and check if the `precision` feature is enabled.
    3.  If both conditions are met, use `println!("cargo:rustc-link-search=native=/path/to/prebuilt/libs");` and `println!("cargo:rustc-link-lib=static=gmp");` (and for MPFR, MPC) to point Cargo to the precompiled libraries from Task 3.1.
*   **Verification:** Attempt to `cargo build --target aarch64-linux-android --features precision` (or similar) on a development machine with the Android NDK configured. Look for successful compilation without linking errors related to GMP/MPFR.

### Task 3.3: Enable Precision Feature in Android App Build
*   **Goal:** Ensure the Android application (Gradle) invokes the Rust build with the `precision` feature, enabling arbitrary-precision math in the final APK.
*   **Action:**
    1.  Examine `app/build.gradle.kts` (or equivalent build configuration for the Android project).
    2.  Identify where the Rust library is compiled and add the `--features precision` flag to the Cargo build command when appropriate (e.g., for release builds or specific build variants).
*   **Verification:** Build the Android application (`./gradlew assembleRelease`). Verify that the resulting `.apk` includes the increased size expected from `rug` and its dependencies. Test the application on an Android device to confirm the arbitrary-precision functionality.

## Recommendation
It is highly recommended to complete **Phase 1** entirely before proceeding. This will create a robust and flexible foundation for any future numeric backend.
Phase 2 should then be tackled on a non-Android platform (e.g., Linux desktop) to minimize initial build complexity.
Phase 3 (Android Integration) carries significant risk and complexity and may require specialized Android NDK/build system expertise. Consideration should be given to alternative pure-Rust arbitrary precision crates if the build complexity becomes unmanageable.
