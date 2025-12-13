# Task In Progress

## Status: Feature Implementation
*   **Date:** 2025-12-11
*   **Objective:** Finalize the Scheduler feature and ensure comprehensive testing.
*   **Current State:** ✅ Scheduler logic and tests landed; previous features validated.

## Completed Features
1.  **CSV/JSON SQL Engine** (`features/sql_engine.rs`)
    *   ✅ Core Logic & SQLite Integration
    *   ✅ UI Integration
    *   ✅ Unit Tests (Implemented)
2.  **Embedded Scripting (Rhai)** (`features/scripting.rs`)
    *   ✅ Core Logic & Rhai Integration
    *   ✅ UI Integration
    *   ✅ Unit Tests (Implemented)
3.  **Dependency List with Search** (`features/dependencies.rs`)
    *   ✅ Rust-side rendering
    *   ✅ Filtering logic
    *   ✅ Unit Tests (Implemented)
4.  **Preset Filtering** (`features/presets.rs`)
    *   ✅ Filter UI & Logic
    *   ✅ Unit Tests (Implemented)

## Feature In Progress: Cron/Task Scheduler
*   **Status:** ✅ IMPLEMENTED
*   **Objective:** Allow users to schedule recurring tasks or chains of actions within the application.
*   **Implementation Status:**
    *   ✅ Dependency (`cron` crate)
    *   ✅ State (`SchedulerState` in `state.rs`)
    *   ✅ Core Logic (`features/scheduler.rs`: Task structure, Background runtime)
    *   ✅ UI (`render_scheduler_screen`)
    *   ✅ Integration (`handle_scheduler_action`)
    *   ✅ Unit Tests (`mod tests` for serialization, cron parsing, runtime events)

## Feature Completed: Regex Tester Enhancements
*   **Status:** ✅ IMPLEMENTED
*   **Objective:** Transform the Regex Tester from a single-match validator into a multi-match extraction tool with quality-of-life improvements.
*   **Implementation Status:**
    *   ✅ State (`RegexTesterState` in `state.rs`: Added `global_mode`, `common_patterns`, and enhanced `RegexMatchResult`)
    *   ✅ Core Logic (`features/regex_tester.rs`: Global search mode, common patterns, multiple match results)
    *   ✅ UI (Global mode toggle, common pattern buttons, multiple match display)
    *   ✅ Integration (`handle_regex_action` with pattern initialization and global mode handling)
    *   ✅ Unit Tests (Updated tests for global mode and multiple matches)

## Immediate To-Do List
1.  Monitor scheduler stability and expand coverage if new actions are added.
2.  Test the new Regex Tester features in the Android app to ensure proper UI rendering.

## Planned Features

### Feature 1: Unit Converter
*   **Status:** 📅 PLANNED
*   **Date:** 2025-12-12
*   **Objective:** Implement a dedicated tool for converting common units (Length, Mass, Temperature, Digital Storage).
*   **Rationale:** A core utility expected in a "Swiss Army Knife" app.
*   **Implementation Plan:**
    1.  **Create Module:** `features/unit_converter.rs`.
    2.  **Define State:** `UnitConverterState` in `state.rs` (category, from_unit, to_unit, input_value, output_value).
    3.  **Implement Logic:** Conversion factors for Metric/Imperial/Digital units.
    4.  **Implement UI:** Dropdowns for category and units, numeric input.
    5.  **Add Tests:** Verify conversions (e.g., Meters to Feet, Celsius to Fahrenheit).

### Feature 2: Advanced CAS (Numerica Integration for Arbitrary Precision)
*   **Status:** ⚠️ ON HOLD (NEON Optimizations Added)
*   **Objective:** Upgrade the Math Tool to use `numerica` for arbitrary precision arithmetic, exact error tracking, and robust symbolic math.
*   **Implementation Status:**
    *   ✅ **Dependency:** Added `numerica` to `Cargo.toml` with minimal features (no Python integration).
    *   ❌ **Integration:** Numerica integration proved complex due to:
        - Significant API differences requiring extensive refactoring
        - Type system complexity with arbitrary precision types
        - Integration challenges with existing symbolic math system
    *   ✅ **NEON Optimizations:** Added comprehensive ARM64 optimization framework:
        - **Multi-Level Targets**: Created optional build targets for ARMv8.0 through ARMv8.5
        - **ARMv8.0 Baseline**: Compatible with all ARM64 devices (neon + fp-armv8)
        - **ARMv8.1 (Mid-range)**: Cortex-A72 class devices (adds CRC, LSE)
        - **ARMv8.2 (High-end)**: Cortex-A75/A76 class (adds RDM, FP16)
        - **ARMv8.4 (Premium)**: Cortex-A76/A77/A78 class (adds dotprod, flagm)
        - **ARMv8.5 (Flagship)**: Cortex-X1/X2 class (adds SSBS, SB)
        - **Platform-Specific Defaults**: Optimized configurations for Android, iOS, and Linux
        - **Native Detection**: Default targets use CPU auto-detection for best performance
        - **Build Flexibility**: Optional targets allow manual selection of instruction set versions
        - **Wide Crate Integration**: Full utilization of wide's SIMD capabilities across all targets
    *   📅 **Alternative Approach:** Maintain current implementation with enhanced error tracking:
        - Keep existing `f64` arithmetic with improved precision handling
        - Add epsilon-based error accumulation tracking
        - Implement result validation and precision warnings
        - Maintain all existing functionality and tests
    *   ✅ **Documentation:** Created comprehensive ARM64 optimization guide:
        - Detailed build instructions for all instruction set versions
        - Performance considerations and trade-offs analysis
        - Integration guides for Android, iOS, and CI/CD systems
        - Future enhancement roadmap for dynamic dispatch and PGO

### Completed Features (Recent)
*   **Regex Tester Enhancements:** (Merged 2025-12-12) Added Global Search and Common Patterns.
*   **Extend Symbolic Integration:** (Merged 2025-12-12) Added `exp`, `tan`, `atan` support.


## Roadmap
*   **Optimization:** Review memory usage of in-memory SQLite and Rhai engine.
*   **Integration Tests:** Verify router handling for new actions.
