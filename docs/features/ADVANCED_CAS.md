# Advanced CAS Implementation Report

**Date:** 2025-12-13
**Status:** Investigation Phase
**Objective:** Determine refactoring steps to upgrade `math_tool` from `f64` to arbitrary precision using `rug` or `numerica`.

## 1. Current Implementation Analysis

The current CAS (Computer Algebra System) in `src/features/math_tool.rs` is tightly coupled to the `f64` primitive type.

*   **Tokenization:** The `tokenize` function immediately parses numeric strings into `f64`.
    ```rust
    // Current
    Token::Number(n) // n is f64
    ```
*   **AST (Symbol):** The `Symbol` enum uses `f64` for constants and coefficients.
    ```rust
    // Current
    Symbol::Number(f64)
    ```
*   **Evaluation:** The `evaluate_expression` and `eval_rpn` functions return `Result<f64, String>` and perform standard floating-point arithmetic.
*   **Storage:** `MathHistoryEntry` stores results as `String`, which is good (decoupled), but the internal processing is all `f64`.

## 2. Candidate Libraries Analysis

### 2.1. `rug` (Recommended by Roadmap)
*   **Description:** A safe Rust wrapper around GMP (integers), MPFR (floats), and MPC (complex).
*   **Pros:**
    *   Industry-standard performance and correctness.
    *   Arbitrary precision with controllable rounding.
    *   Extensive mathematical functions (trig, exp, log, gamma, etc.).
*   **Cons:**
    *   **Heavy C Dependencies:** Requires linking against GMP, MPFR, and MPC.
    *   **Android Build Complexity:** Extremely high. Requires cross-compiling these C libraries for `aarch64-linux-android`, `armv7-linux-androideabi`, etc., and configuring `build.rs` to find them.
    *   **License:** GMP is LGPL.

### 2.2. `numerica`
*   **Description:** A Rust math library for exact and floating-point computations.
*   **Dependency:** Depends on `rug` for arbitrary precision.
*   **Verdict:** Inherits all the build complexity of `rug`. If `rug` is used, `numerica` might offer higher-level abstractions (matrices, roots), but for a basic CAS, `rug` alone might be sufficient or `symbolica` (which also uses `rug`) might be the target.

### 2.3. Pure Rust Alternatives (For Reference)
*   `dashu` or `num-bigfloat`: Pure Rust implementations.
*   **Pros:** Trivial to build on Android (just `cargo build`).
*   **Cons:** Slower than GMP; fewer advanced mathematical functions implemented.

## 3. Refactoring Roadmap

To support `rug` (or any arbitrary precision type), the codebase requires significant refactoring to decouple logic from `f64`.

### Phase 1: Type Definition & Abstraction
Create a wrapper type that can hold either `f64` (for fast mode) or `rug::Float` (for precision mode).

```rust
// src/features/cas_types.rs
#[cfg(feature = "precision")]
use rug::Float;

#[derive(Clone, Debug)]
pub enum Number {
    Fast(f64),
    #[cfg(feature = "precision")]
    Precise(Float),
}

// Implement Add, Sub, Mul, Div for Number
```

### Phase 2: Tokenizer Update
Modify `Token` to store the raw string or the abstract `Number` type, postponing parsing until the precision mode is known.

```rust
// src/features/math_tool.rs
enum Token {
    NumberStr(String), // Parse later based on desired precision
    // ...
}
```

### Phase 3: AST & Evaluation Refactoring
Update `Symbol` and `eval_rpn` to use the `Number` type.

1.  **Replace `f64`:** Search and replace `f64` with `Number` (or a generic `T: Numeric`).
2.  **Context Passing:** Evaluation functions need to know the desired precision (e.g., 50 bits vs 1000 bits).
    ```rust
    pub fn evaluate(expr: &str, precision: u32) -> Result<Number, String> { ... }
    ```

### Phase 4: Android Build Configuration (The Hard Part)
If `rug` is selected, the `kistaverk` build system must be updated.

1.  **Precompiled Libs:** Create a script `scripts/build_gmp_android.sh` to download and cross-compile GMP/MPFR for Android targets.
2.  **Build Script:** Update `rust/build.rs` to link against these static libraries when targeting Android.
3.  **CI:** Ensure CI runners have the NDK and tools to build these deps.

## 4. Feasibility & Risk
*   **Code Refactoring:** Medium difficulty. Mostly mechanical type substitution and fixing compilation errors.
*   **Build System:** **High difficulty.** Getting GMP/MPFR to link correctly inside an Android Gradle/Cargo build is error-prone.
*   **Size:** `rug` + deps will add significant size (several MBs) to the `.so` / APK.

## 5. Recommendation
If strictly arbitrary precision floats are needed and pure Rust crates (`dashu`, `rust_decimal`) are insufficient:
1.  **Prototype on Linux first:** Implement the refactoring behind a feature flag `feature = "precision"`.
2.  **Use `gmp-mpfr-sys` crate:** It attempts to build GMP automatically, but often fails on Android cross-compilation.
3.  **Consider `symbolica`:** If the goal is symbolic math (simplification, derivatives), use `symbolica` (which depends on `rug` usually, but check if it has a pure-rust feature).

**Immediate Action:**
Start by defining the `Number` abstraction to prepare the codebase, regardless of the backend chosen.
