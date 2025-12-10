# Task In Progress

## Status: Feature Implementation
*   **Date:** 2025-12-11
*   **Objective:** Finalize the Scheduler feature and ensure comprehensive testing.
*   **Current State:** 🔄 Scheduler in progress, previous features validated.

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
*   **Status:** 🚧 IN PROGRESS
*   **Objective:** Allow users to schedule recurring tasks or chains of actions within the application.
*   **Implementation Status:**
    *   ✅ Dependency (`cron` crate)
    *   ✅ State (`SchedulerState` in `state.rs`)
    *   ✅ Core Logic (`features/scheduler.rs`: Task structure, Background runtime)
    *   ✅ UI (`render_scheduler_screen`)
    *   ✅ Integration (`handle_scheduler_action`)
    *   ❌ Unit Tests (Missing `mod tests`)

## Immediate To-Do List
1.  **Add Tests for Scheduler:**
    *   Test `ScheduledTask` struct (serialization/deserialization).
    *   Test Cron parsing and validation.
    *   Test `SchedulerRuntime` (task scheduling, execution triggering).
    *   Test state management (adding/removing tasks).

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
