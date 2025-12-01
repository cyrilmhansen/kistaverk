# Task In Progress: System Panels

## Feature Description
Implement a "System Panels" screen that aggregates device information. This will serve as a dashboard for:
*   **Storage**: Usage stats (Total/Free) for internal and external storage.
*   **Network**: Active interface status (Wi-Fi/Mobile), IP address (local), and basic connectivity check.
*   **Battery**: Level, status (charging/discharging), and health.
*   **App Inspector**: Basic OS info (Android version, SDK, Device model).

*Note: Since the core logic is in Rust, we will need to define the data structures and UI in Rust, but rely heavily on the Kotlin side to feed this data via `bindings` or specific actions, as standard Rust crates for Android system info are limited without JNI calls to Android APIs.*

## Plan

### Step 1: Core Logic (`rust/src/features/system_info.rs`)
*   **Goal:** Define data structures and state.
*   **Actions:**
    1.  Create `system_info.rs`.
    2.  Define structs for `StorageInfo`, `NetworkInfo`, `BatteryInfo`, `DeviceInfo`.
    3.  Update `AppState` to hold `SystemInfoState`.

### Step 2: UI Rendering (`rust/src/features/system_info.rs`)
*   **Goal:** Display the dashboard.
*   **Actions:**
    1.  Implement `render_system_info_screen`.
    2.  Use `UiCard` and `UiSection` to group info.
    3.  Add a "Refresh" button to trigger a data fetch.

### Step 3: Integration (`rust/src/lib.rs`)
*   **Goal:** Wire up actions.
*   **Actions:**
    1.  Add `Action::SystemInfoScreen`.
    2.  Add `Action::SystemInfoUpdate { bindings }`. This is crucial: The Kotlin layer will likely need to be triggered to "gather info" and then call back into Rust with the data in the `bindings` map.
    *   *Architecture Decision*: To keep Rust as the source of truth, `Action::SystemInfoScreen` will push the screen *and* return a side-effect/command telling Kotlin "Go fetch system info".
    *   Alternatively, the "Refresh" button sends an action to Rust, which returns a JSON payload that the Kotlin renderer interprets as "Run `getSystemInfo()` and send result back to Rust via `Action::SystemInfoUpdate`".

### Step 4: Testing
*   **Actions:**
    1.  Unit test the parsing of binding data into structs.
    2.  Verify the UI layout generation.

---

## Completed Tasks
*   **GZIP Compression**: Done.
*   **ZIP Creation**: Done.
*   **PDF Reordering**: Done.
*   **ZIP Extraction**: Done.
*   **File Inspector**: Done.
*   **Dithering Tools**: Done.
*   **Multi-hash view**: Done.
*   **Refactoring lib.rs**: Done.
*   **PDF Thumbnail Grid**: Rust side done (Kotlin pending).
*   **Pixel Art Mode**: Implemented (Rust Done, Kotlin Pending).
*   **Regex Tester**: Implemented (Rust Done).
*   **UUID/Random**: Implemented (Rust Done).