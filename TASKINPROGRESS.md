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

### Completed Features (Recent)
*   **Regex Tester Enhancements:** (Merged 2025-12-12) Added Global Search and Common Patterns.
*   **Extend Symbolic Integration:** (Merged 2025-12-12) Added `exp`, `tan`, `atan` support.


## Roadmap
*   **Optimization:** Review memory usage of in-memory SQLite and Rhai engine.
*   **Integration Tests:** Verify router handling for new actions.
