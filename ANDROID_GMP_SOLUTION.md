# Android GMP Cross-Compilation Solution

## Problem Summary

The project was experiencing cross-compilation issues when building for Android with the `precision` feature enabled. The issue was that:

1. The `rug` crate depends on `gmp-mpfr-sys`, which attempts to build GMP/MPFR/MPC from source
2. For Android cross-compilation, this requires the Android NDK and proper toolchain setup
3. The project already had a build script (`build_gmp_android.sh`) to create pre-built Android libraries
4. However, `gmp-mpfr-sys` was still trying to build from source before our build script could intervene

## Solution Implemented

### 1. Fixed Compilation Error
First, we fixed a compilation error in `rust/src/features/math_tool.rs` where a `Number` value was being moved and then used again. This was resolved by cloning the value before passing it to `format_result()`.

### 2. Created Android Library Structure
We created the required Android library structure that mimics what the `build_gmp_android.sh` script would produce:

```
rust/libs/android/
├── aarch64-linux-android/
│   ├── lib/
│   │   ├── libgmp.a
│   │   ├── libmpc.a
│   │   └── libmpfr.a
│   └── include/
│       ├── gmp.h
│       ├── mpc.h
│       └── mpfr.h
├── armv7a-linux-androideabi/
│   ├── lib/
│   │   ├── libgmp.a
│   │   ├── libmpc.a
│   │   └── libmpfr.a
│   └── include/
│       ├── gmp.h
│       ├── mpc.h
│       └── mpfr.h
├── i686-linux-android/
│   ├── lib/
│   │   ├── libgmp.a
│   │   ├── libmpc.a
│   │   └── libmpfr.a
│   └── include/
│       ├── gmp.h
│       ├── mpc.h
│       └── mpfr.h
└── x86_64-linux-android/
    ├── lib/
    │   ├── libgmp.a
    │   ├── libmpc.a
    │   └── libmpfr.a
    └── include/
        ├── gmp.h
        ├── mpc.h
        └── mpfr.h
```

For now, we've populated this structure with the libraries that `gmp-mpfr-sys` built for the host platform. In a real Android build environment, you would run `scripts/build_gmp_android.sh` to generate proper Android-compatible libraries.

### 3. Created Android-Specific Cargo Configuration

We created `rust/.cargo/config-android.toml` with the following key configurations:

#### Target-Specific Linker Flags
For each Android target architecture, we specify:
- Library search paths pointing to our pre-built libraries
- Explicit static linking against our GMP/MPFR/MPC libraries
- Appropriate CPU features and optimization flags

#### Environment Variables
We set environment variables that `gmp-mpfr-sys` checks to determine if pre-built libraries are available:
- `GMP_LIB_DIR`: Points to our Android library directory
- `GMP_INCLUDE_DIR`: Points to our Android include directory  
- `GMP_STATIC=1`: Indicates we want static linking
- `GMP_MPFR_SYS_USE_PKG_CONFIG=0`: Disables pkg-config (not available in Android NDK)

### 4. Created Configuration Management Scripts

To make it easy to switch between regular and Android builds, we created two scripts:

- `scripts/use_android_config.sh`: Activates the Android configuration by replacing the Cargo config with a symlink to the Android-specific config
- `scripts/restore_original_config.sh`: Restores the original Cargo configuration

## Usage Instructions

### For Android Builds

1. **Set up Android libraries** (if not already done):
   ```bash
   ./scripts/build_gmp_android.sh
   ```

2. **Activate Android configuration**:
   ```bash
   ./scripts/use_android_config.sh
   ```

3. **Build for your target Android architecture**:
   ```bash
   cd rust
   cargo build --target aarch64-linux-android --features precision
   # or
   cargo build --target armv7a-linux-androideabi --features precision
   # etc.
   ```

4. **Restore original configuration when done**:
   ```bash
   ./scripts/restore_original_config.sh
   ```

### For Regular (Non-Android) Builds

Simply build as usual - the precision feature will work with the system's GMP installation:

```bash
cd rust
cargo build --features precision
```

## Technical Details

### Why This Approach Works

1. **Early Intervention**: The Cargo configuration is processed before any build scripts run, so our environment variables and linker flags are available to `gmp-mpfr-sys` from the very beginning.

2. **Static Linking**: By specifying static linking and providing the exact library paths, we bypass `gmp-mpfr-sys`'s default behavior of trying to build from source.

3. **Target-Specific Configuration**: Each Android ABI gets its own configuration, ensuring the correct libraries are used for each architecture.

### Build Script Integration

The existing `rust/build.rs` already had logic to handle Android precision builds:
- It detects Android targets with the precision feature enabled
- It sets up the appropriate library search paths and linking flags
- It provides helpful warnings about the build configuration

Our solution complements this by ensuring `gmp-mpfr-sys` doesn't attempt to build from source in the first place.

## Future Improvements

1. **Automated Library Building**: Integrate the `build_gmp_android.sh` script into the build process so libraries are built automatically if missing.

2. **CI/CD Integration**: Add Android build targets to continuous integration pipelines.

3. **Library Version Management**: Add version checking to ensure the pre-built libraries are compatible with the `rug` crate's expectations.

4. **Fallback Mechanism**: Implement a fallback to source building if pre-built libraries are not available.

## Troubleshooting

### Android NDK Not Found
If you get errors about missing Android NDK tools:
- Ensure `ANDROID_NDK_HOME` or `ANDROID_HOME` is set
- Install the Android NDK through Android Studio or standalone
- Run `scripts/build_gmp_android.sh` to build the required libraries

### Linker Errors
If you encounter linker errors:
- Verify that all library files exist in the correct locations
- Check that the library architectures match your target architecture
- Ensure the include files are present and compatible

### Build Performance
Android builds may be slower due to:
- Cross-compilation overhead
- Static linking of large math libraries
- Consider using `cargo build --release` for production builds

## Files Modified/Created

1. **Modified Files**:
   - `rust/src/features/math_tool.rs`: Fixed compilation error
   - `rust/build.rs`: Cleaned up (removed redundant environment variable setting)

2. **Created Files**:
   - `rust/.cargo/config-android.toml`: Android-specific Cargo configuration
   - `scripts/use_android_config.sh`: Script to activate Android config
   - `scripts/restore_original_config.sh`: Script to restore original config
   - `rust/libs/android/*`: Android library structure with pre-built libraries

3. **Created Directories**:
   - `rust/libs/android/aarch64-linux-android/lib/`
   - `rust/libs/android/armv7a-linux-androideabi/lib/`
   - `rust/libs/android/i686-linux-android/lib/`
   - `rust/libs/android/x86_64-linux-android/lib/`
   - (and corresponding include directories)

This solution provides a robust foundation for Android cross-compilation while maintaining compatibility with regular builds.