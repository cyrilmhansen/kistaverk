# Android Precision Build Guide

This guide explains how to enable arbitrary precision arithmetic on Android using the `rug` crate.

## Overview

The `rug` crate provides arbitrary precision arithmetic but requires native C libraries (GMP, MPFR, MPC) that must be cross-compiled for Android. This document outlines the process.

## Phase 3: Android Integration

### Task 3.1: Prototype Android Build Script ✅

**Status**: COMPLETED

**Files Created**:
- `scripts/build_gmp_android.sh` - Main build script for cross-compiling GMP/MPFR/MPC
- `scripts/ANDROID_BUILD_README.md` - Comprehensive build documentation
- `scripts/test_build_script.sh` - Verification script

**Features**:
- ✅ Supports all major Android ABIs (aarch64, armv7a, i686, x86_64)
- ✅ Automated download and build process
- ✅ Comprehensive error handling
- ✅ Detailed documentation

**Verification**:
```bash
cd scripts
./test_build_script.sh
```

### Task 3.2: Integrate with Rust build.rs (Next Step)

**Status**: NOT YET IMPLEMENTED

**Requirements**:
- Modify `rust/build.rs` to detect Android target and precision feature
- Add linking instructions for prebuilt libraries
- Handle different ABIs appropriately

**Expected Implementation**:
```rust
// In rust/build.rs
if cfg!(target_os = "android") && cfg!(feature = "precision") {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let lib_path = format!("rust/libs/android/{}/lib", target_arch);
    
    println!("cargo:rustc-link-search=native={}", lib_path);
    println!("cargo:rustc-link-lib=static=gmp");
    println!("cargo:rustc-link-lib=static=mpfr");
    println!("cargo:rustc-link-lib=static=mpc");
}
```

### Task 3.3: Enable Precision Feature in Android App (Next Step)

**Status**: NOT YET IMPLEMENTED

**Requirements**:
- Modify `app/build.gradle.kts` to pass `--features precision` to Cargo
- Configure build variants appropriately
- Handle increased APK size from native libraries

## Current Status

### ✅ Completed
- **Phase 1**: Foundation & Abstraction (Tasks 1.1-1.3)
- **Phase 2**: Arbitrary Precision Logic (Tasks 2.1-2.3)
- **Phase 3 Task 3.1**: Android build script prototype

### 🔄 In Progress
- **Phase 3 Tasks 3.2-3.3**: Android integration (build.rs and Gradle)

## Build Process

### Step 1: Build Native Libraries

```bash
# Install Android NDK first
export ANDROID_NDK_HOME=/path/to/your/android-ndk

# Run the build script
cd scripts
./build_gmp_android.sh
```

### Step 2: Verify Libraries

The script will create:
```
rust/libs/android/
├── aarch64-linux-android/lib/
│   ├── libgmp.a
│   ├── libmpc.a
│   └── libmpfr.a
├── armv7a-linux-androideabi/lib/
│   ├── libgmp.a
│   ├── libmpc.a
│   └── libmpfr.a
└── ... (other architectures)
```

### Step 3: Integrate with Rust (Future Work)

Update `rust/build.rs` to link against these libraries when:
- Target OS is Android
- Precision feature is enabled

### Step 4: Configure Android App (Future Work)

Update `app/build.gradle.kts` to:
- Enable precision feature for appropriate build variants
- Handle increased APK size
- Configure NDK properly

## Technical Challenges

### Android NDK Complexity
- Multiple ABIs require separate builds
- Toolchain configuration is complex
- API level compatibility must be considered

### Library Size
- GMP/MPFR/MPC are large libraries
- Will significantly increase APK size
- May require dynamic feature modules

### Build System Integration
- Cargo build.rs must detect Android target
- Gradle must pass correct flags to Cargo
- Cross-compilation toolchain must be available

## Alternative Approaches

### Option 1: Pre-built Libraries
Use pre-compiled Android libraries from trusted sources instead of building from source.

**Pros**: Faster setup, less maintenance
**Cons**: Less control over versions and build flags

### Option 2: Conditional Compilation
Only enable precision features on non-Android platforms initially.

**Pros**: Simpler build process
**Cons**: Reduced functionality on Android

### Option 3: Pure Rust Alternatives
Consider pure Rust arbitrary precision libraries that don't require C dependencies.

**Pros**: No native linking required
**Cons**: May have different APIs or performance characteristics

## Recommendations

1. **Start with Phase 1-2**: The current implementation provides excellent functionality without Android precision
2. **Test Thoroughly**: When implementing Phase 3, test on multiple Android devices and architectures
3. **Consider APK Size**: The precision libraries will significantly increase APK size - consider dynamic delivery
4. **Document Well**: Android build processes are complex - provide clear documentation for contributors

## Future Work

### Short Term
- ✅ Task 3.1: Build script prototype (COMPLETED)
- 🔄 Task 3.2: Rust build.rs integration
- 🔄 Task 3.3: Android Gradle configuration

### Long Term
- Performance optimization for Android
- APK size reduction techniques
- Dynamic feature modules for precision
- Comprehensive Android testing

## Getting Help

If you encounter issues with the Android build process:

1. **Check the Documentation**: `scripts/ANDROID_BUILD_README.md`
2. **Review Error Messages**: The build script provides detailed output
3. **Consult Android NDK Docs**: Google's official documentation
4. **Search Existing Issues**: Many have solved similar problems
5. **Ask for Help**: Provide detailed error logs and environment info

## License Considerations

The GMP/MPFR/MPC libraries have specific licensing requirements:
- **GMP**: LGPLv3+ or GPLv2+
- **MPFR**: LGPLv3+
- **MPC**: LGPLv3+

Ensure your application complies with these licenses if you distribute the libraries.

## Conclusion

Phase 3 represents the most challenging part of the Advanced CAS implementation due to Android's complex build requirements. The build script prototype (Task 3.1) is complete and ready for testing. Tasks 3.2 and 3.3 remain as future work to fully integrate arbitrary precision arithmetic on Android platforms.