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

### Feature 1: Extend Symbolic Integration
*   **Status:** 📅 PLANNED
*   **Date:** 2025-12-11
*   **Objective:** Enhance the existing symbolic integration capabilities in the Math Tool.
*   **Rationale:** Directly addresses the roadmap item "Symbolic Integration: Extend math tool to support basic integration" and builds upon existing functionality in `features/math_tool.rs`.
*   **Implementation Plan:**
    1.  **Analyze current `integrate` function:** Identify patterns of currently `∫unsupported` expressions.
    2.  **Implement new rules:** Add support for more integration techniques (e.g., product rule for integration by parts, basic trigonometric substitutions, or simple rational functions).
    3.  **Add Tests:** Create a small set of unit tests in `math_tool.rs` for each newly implemented integration rule.

## Roadmap
*   **Optimization:** Review memory usage of in-memory SQLite and Rhai engine.
*   **Integration Tests:** Verify router handling for new actions.
