# Android Precision Feature Guide

This guide explains how to enable and use the precision feature in the Android app, which provides arbitrary precision arithmetic using the `rug` crate.

## Overview

The precision feature enables arbitrary precision arithmetic in the Math Tool, allowing for exact calculations that go beyond the limitations of standard floating-point arithmetic.

## Build Configuration

### Option 1: Using Gradle Tasks (Recommended)

```bash
# Build WITH precision feature (arbitrary precision)
cd app
./gradlew buildWithPrecision

# Build WITHOUT precision feature (standard f64 - default)
cd app
./gradlew buildWithoutPrecision
```

### Option 2: Using Command Line Property

```bash
# Build WITH precision feature
cd app
./gradlew assembleDebug -PenablePrecision=true

# Build WITHOUT precision feature (default)
cd app
./gradlew assembleDebug -PenablePrecision=false
```

### Option 3: Direct Cargo Build

```bash
# Build WITH precision feature
cd app
./gradlew cargoBuild -PenablePrecision=true

# Build WITHOUT precision feature
cd app
./gradlew cargoBuild -PenablePrecision=false
```

## Build Scenarios

### 1. Standard Build (Default)

```bash
./gradlew assembleDebug
```

**Characteristics**:
- ✅ Fastest build time
- ✅ Smallest APK size
- ✅ Uses standard f64 arithmetic
- ✅ No external dependencies needed

### 2. Precision Build

```bash
./gradlew buildWithPrecision
# or
./gradlew assembleDebug -PenablePrecision=true
```

**Characteristics**:
- ⚠️ Longer build time
- ⚠️ Larger APK size (~5-10MB increase)
- ✅ Arbitrary precision arithmetic
- ❌ Requires prebuilt Android libraries

## Prerequisites

### For Precision Builds

1. **Android NDK**: Must be installed and configured
2. **Prebuilt Libraries**: Run the build script first:
   ```bash
   cd scripts
   ./build_gmp_android.sh
   ```
3. **Library Location**: Ensure libraries are in `rust/libs/android/{arch}/lib/`

### Expected Library Structure

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
└── ... (other architectures)
```

## Performance Comparison

### APK Size

| Configuration | APK Size Impact | Use Case |
|--------------|----------------|----------|
| Without Precision | Baseline size | General use, most users |
| With Precision | +5-10MB | Advanced math, scientific calculations |

### Build Time

| Configuration | Build Time | Notes |
|--------------|------------|-------|
| Without Precision | Fast (~1-2 min) | Standard f64 arithmetic |
| With Precision | Slower (~3-5 min) | Native library linking |

### Runtime Performance

| Configuration | Speed | Accuracy | Use Case |
|--------------|-------|----------|----------|
| f64 | ⚡ Fastest | Limited (~15 decimal digits) | General calculations |
| Arbitrary Precision | 🐢 Slower | Unlimited (user-defined) | Scientific, financial |

## When to Use Precision

### Use Precision Feature When:

- ✅ You need exact decimal calculations
- ✅ Working with very large or very small numbers
- ✅ Financial calculations requiring exact results
- ✅ Scientific computing with high precision needs
- ✅ Cryptographic applications

### Use Standard f64 When:

- ✅ General purpose calculations
- ✅ Performance is critical
- ✅ APK size must be minimized
- ✅ Most real-world applications
- ✅ User interface calculations

## Implementation Details

### Gradle Integration

The build system automatically:

1. **Detects precision flag**: Checks `-PenablePrecision` property
2. **Configures cargo**: Adds `--features precision` when enabled
3. **Provides feedback**: Clear build messages indicate configuration
4. **Handles errors**: Graceful error messages for missing libraries

### Build Output

**Without Precision**:
```
📊 Building without precision feature (standard f64)
```

**With Precision**:
```
🔧 Precision feature enabled for Rust build
   This will enable arbitrary precision arithmetic using GMP/MPFR/MPC
   Note: Requires prebuilt Android libraries from scripts/build_gmp_android.sh
```

## Testing

### Verify Precision Feature

```bash
# Check if precision is enabled in the build
./gradlew cargoBuild
```

### Test Math Operations

The precision feature affects the Math Tool:

```kotlin
// Standard precision (f64)
val result1 = evaluateExpression("1/3 + 1/3 + 1/3", 0)  // ~1.0 (with floating-point error)

// Arbitrary precision
val result2 = evaluateExpression("1/3 + 1/3 + 1/3", 100) // Exactly 1.0
```

## Troubleshooting

### "Android precision libraries not found"

**Solution**:
1. Run the build script: `cd scripts && ./build_gmp_android.sh`
2. Ensure `ANDROID_NDK_HOME` is set
3. Verify libraries are in `rust/libs/android/{arch}/lib/`

### Build fails with linking errors

**Solution**:
1. Check NDK installation and version
2. Verify library paths in `build.rs`
3. Ensure correct target architecture

### APK size too large

**Solution**:
1. Consider using dynamic feature modules
2. Only enable precision for specific build variants
3. Use standard f64 for most users

## Best Practices

### 1. Default to Standard Precision

Most users don't need arbitrary precision. Use it only for specific build variants:

```gradle
// build.gradle.kts
android {
    buildTypes {
        release {
            // Standard precision for production (smaller APK)
            isMinifyEnabled = true
        }
        
        debug {
            // Optional: Enable precision for debugging
            // Requires manual configuration
        }
    }
}
```

### 2. Document Precision Requirements

If you distribute apps with precision enabled:
- Document the increased APK size
- Explain the performance implications
- Provide guidance on when to use precision

### 3. Test on Multiple Architectures

Ensure precision works on all target architectures:
- `arm64-v8a` (most common)
- `armeabi-v7a` (legacy devices)
- `x86` and `x86_64` (emulators, some devices)

### 4. Consider Dynamic Delivery

For large apps, consider:
- Dynamic feature modules for precision
- Download precision libraries on demand
- Reduce initial APK size

## Future Enhancements

### 1. Automatic Library Download

Future versions could:
- Download prebuilt libraries automatically
- Reduce manual setup requirements
- Improve developer experience

### 2. Selective Architecture Support

Optimize by:
- Supporting only common architectures
- Reducing build complexity
- Targeting specific device markets

### 3. Precision Feature Toggle

Allow users to:
- Enable/disable precision at runtime
- Choose between speed and accuracy
- Optimize for their use case

## Conclusion

The precision feature provides powerful arbitrary precision arithmetic for Android apps. It's designed to be:

- **Easy to enable**: Simple Gradle property
- **Transparent**: Automatic detection and configuration
- **Flexible**: Works with all build variants
- **Well-documented**: Clear guidance for developers

**Recommendation**: Start with standard precision (f64) for most users, and enable arbitrary precision only when specifically needed for advanced mathematical operations.

## Reference

- **Build Script**: `scripts/build_gmp_android.sh`
- **Rust Integration**: `rust/build.rs`
- **Gradle Configuration**: `app/app/build.gradle.kts`
- **Documentation**: `scripts/BUILD_RS_DOCUMENTATION.md`