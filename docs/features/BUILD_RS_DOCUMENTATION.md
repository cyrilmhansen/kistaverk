# build.rs Android Integration Documentation

This document explains the Android linking integration in `rust/build.rs` for the precision feature.

## Overview

The `build.rs` file has been enhanced to automatically detect Android targets and configure proper linking when the `precision` feature is enabled.

## Key Features

### 1. Android Detection

```rust
#[cfg(all(target_os = "android", feature = "precision"))]
{
    // Android-specific linking logic
}
```

The build script automatically detects when:
- Target OS is Android (`target_os = "android"`)
- Precision feature is enabled (`feature = "precision"`)

### 2. Architecture Mapping

The script maps Rust target architectures to Android library directories:

| Rust Architecture | Android ABI | Library Path |
|------------------|------------|--------------|
| `aarch64` | aarch64-linux-android | `rust/libs/android/aarch64-linux-android/lib` |
| `arm` | armv7a-linux-androideabi | `rust/libs/android/armv7a-linux-androideabi/lib` |
| `x86` | i686-linux-android | `rust/libs/android/i686-linux-android/lib` |
| `x86_64` | x86_64-linux-android | `rust/libs/android/x86_64-linux-android/lib` |

### 3. Library Linking

When both conditions are met, the build script:

1. **Sets link search path**: `cargo:rustc-link-search=native={lib_path}`
2. **Links static libraries**:
   - `cargo:rustc-link-lib=static=gmp`
   - `cargo:rustc-link-lib=static=mpfr`
   - `cargo:rustc-link-lib=static=mpc`
3. **Links system libraries**:
   - `cargo:rustc-link-lib=c` (libc)
   - `cargo:rustc-link-lib=m` (libm for math functions)
4. **Provides feedback**: `cargo:warning=Android precision feature enabled`

### 4. Error Handling

If the required libraries are not found:

```rust
if !std::path::Path::new(lib_path).exists() {
    eprintln!("Error: Android precision libraries not found in: {}", lib_path);
    eprintln!("Please run scripts/build_gmp_android.sh to build the required libraries");
    panic!("Android precision libraries missing - see error above");
}
```

### 5. Non-Android Platforms

For non-Android platforms with precision enabled:

```rust
#[cfg(all(not(target_os = "android"), feature = "precision"))]
{
    println!("cargo:warning=Precision feature enabled for non-Android platform");
}
```

This provides a helpful warning but doesn't require special linking since `rug` handles its own dependencies on non-Android platforms.

### 6. Precision Disabled

When precision feature is disabled:

```rust
#[cfg(not(feature = "precision"))]
{
    println!("cargo:warning=Building without precision feature (using f64)");
}
```

## Build Scenarios

### Scenario 1: Android with Precision

```bash
# Build for Android with precision feature
cargo build --target aarch64-linux-android --features precision
```

**Expected Output**:
- ✅ Links against GMP/MPFR/MPC from `rust/libs/android/aarch64-linux-android/lib/`
- ✅ Provides warning about Android precision feature
- ✅ Successful build if libraries are present

### Scenario 2: Android without Precision

```bash
# Build for Android without precision feature
cargo build --target aarch64-linux-android
```

**Expected Output**:
- ✅ Uses standard f64 arithmetic (no special linking)
- ✅ Provides warning about building without precision
- ✅ Successful build

### Scenario 3: Non-Android with Precision

```bash
# Build for Linux/macOS/Windows with precision
cargo build --features precision
```

**Expected Output**:
- ✅ Uses `rug` crate directly (no special linking needed)
- ✅ Provides warning about precision on non-Android platform
- ✅ Successful build

### Scenario 4: Non-Android without Precision

```bash
# Standard build
cargo build
```

**Expected Output**:
- ✅ Uses standard f64 arithmetic
- ✅ Provides warning about building without precision
- ✅ Successful build

## Prerequisites

### For Android Builds

1. **Android NDK**: Must be installed and configured
2. **Prebuilt Libraries**: Run `scripts/build_gmp_android.sh` first
3. **Library Structure**: Ensure `rust/libs/android/{arch}/lib/` contains the required `.a` files

### Build Script Requirements

The build script expects the following directory structure:

```
rust/libs/android/
├── aarch64-linux-android/
│   └── lib/
│       ├── libgmp.a
│       ├── libmpc.a
│       └── libmpfr.a
├── armv7a-linux-androideabi/
│   └── lib/
│       ├── libgmp.a
│       ├── libmpc.a
│       └── libmpfr.a
├── i686-linux-android/
│   └── lib/
│       ├── libgmp.a
│       ├── libmpc.a
│       └── libmpfr.a
└── x86_64-linux-android/
    └── lib/
        ├── libgmp.a
        ├── libmpc.a
        └── libmpfr.a
```

## Troubleshooting

### "Android precision libraries not found"

**Cause**: The build script cannot find the prebuilt libraries.

**Solution**:
1. Run `scripts/build_gmp_android.sh` to build the libraries
2. Ensure `ANDROID_NDK_HOME` is set correctly
3. Verify the libraries are in the expected locations

### "Unknown Android architecture"

**Cause**: You're targeting an unsupported Android architecture.

**Solution**: Use one of the supported architectures: `aarch64`, `arm`, `x86`, or `x86_64`

### Linking Errors

**Cause**: Missing dependencies or incorrect library paths.

**Solution**:
1. Verify the libraries exist in the expected locations
2. Check that the libraries are built for the correct target architecture
3. Ensure the NDK toolchain is properly configured

## Testing

### Verify Build Configuration

```bash
./scripts/test_android_linking.sh
```

This script checks that:
- `build.rs` exists and is properly configured
- Android-specific code is present
- Library linking code is correct
- Architecture mapping is implemented
- Error handling is in place

### Test Different Build Scenarios

```bash
# Test 1: Standard build (no precision)
cargo build

# Test 2: Precision on non-Android
cargo build --features precision

# Test 3: Android without precision (requires NDK)
cargo build --target aarch64-linux-android

# Test 4: Android with precision (requires NDK and libraries)
cargo build --target aarch64-linux-android --features precision
```

## Integration with Android App

### Gradle Configuration

To use this in an Android app, you'll need to configure `build.gradle.kts`:

```kotlin
android {
    // ... other configuration ...
    
    externalNativeBuild {
        cmake {
            arguments += listOf("-DCARGO_FEATURES=precision")
        }
    }
}
```

### Expected Behavior

- **Without precision**: App uses standard f64 arithmetic (smaller APK)
- **With precision**: App uses arbitrary precision arithmetic (larger APK, more functionality)

## Performance Considerations

### APK Size
- **Without precision**: Standard size (uses f64)
- **With precision**: Significantly larger (~5-10MB increase from GMP/MPFR/MPC libraries)

### Runtime Performance
- **f64**: Fast, hardware-accelerated
- **Arbitrary precision**: Slower, software-based, but more accurate

### Build Time
- **Without precision**: Fast build times
- **With precision**: Longer build times due to native library linking

## Future Enhancements

### Dynamic Feature Modules
Consider using Android's dynamic feature modules to:
- Reduce initial APK size
- Download precision libraries only when needed
- Provide better user experience

### Selective Architecture Support
Modify the build script to:
- Support only specific architectures
- Reduce build time and complexity
- Target only the architectures your app supports

### Improved Error Messages
Enhance error handling to provide:
- More specific guidance for common issues
- Links to documentation
- Suggestions for troubleshooting

## Conclusion

The `build.rs` integration provides a robust foundation for Android precision support. It automatically handles the complex linking requirements while providing clear feedback and error messages. The implementation is ready for integration with Android apps and supports all major Android architectures.

**Next Steps**:
- Task 3.3: Configure Android app's build.gradle to enable precision feature
- Test on actual Android devices
- Optimize APK size and build times
- Document the full Android integration process