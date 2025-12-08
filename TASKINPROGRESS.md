# Task In Progress: Sensor Data Smoothing

## Status: Implemented
*   **Date:** 2025-12-08
*   **Objective:** Implement signal smoothing (Low-Pass Filter) for Compass, Barometer, and Magnetometer to reduce UI jitter.
*   **Plan:**
    1.  **New Module (`features/sensor_utils.rs`):**
        *   Implemented `LowPassFilter` logic (scalar & angular).
    2.  **State Update (`state.rs`):**
        *   Added filter state fields (`compass_filter_angle`, etc.) to `AppState`.
    3.  **Integration (`router.rs`):**
        *   Updated `CompassSet`, `BarometerSet`, `MagnetometerSet` to use smoothing.
    4.  **Tests:**
        *   Added unit tests in `sensor_utils.rs`.

## Previous Task: Math Expression Evaluator
*   **Status:** Implemented
*   **Date:** 2025-12-08
*   **Summary:** Implemented a math expression parser and evaluator with history support. Verified via unit tests.
