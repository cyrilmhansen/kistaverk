# ARM64 Optimization Guide for Kistaverk

This document explains the ARM64 optimization framework and how to use the optional build targets.

## Overview

Kistaverk now supports **multi-level ARM64 optimizations** through Cargo's target configuration system. This allows building optimized binaries for different generations of ARM64 devices while maintaining backward compatibility.

## ARM64 Instruction Set Versions

### ARMv8.0-A (Baseline)
- **Compatibility**: All ARM64 devices
- **Features**: NEON SIMD, FP-ARMv8
- **Use Case**: Maximum compatibility for entry-level devices
- **Build Target**: `aarch64-unknown-linux-gnu.armv8-0`

### ARMv8.1-A (Mid-range)
- **Compatibility**: Cortex-A72, Kryo, and similar
- **Features**: +CRC, +LSE (Large System Extensions)
- **Use Case**: Common mid-range devices (2016-2018 flagship phones)
- **Build Target**: `aarch64-unknown-linux-gnu.armv8-1`

### ARMv8.2-A (High-end)
- **Compatibility**: Cortex-A75, A76
- **Features**: +RDM, +FP16 (half-precision floating-point)
- **Use Case**: High-end devices (2018-2020 flagship phones)
- **Build Target**: `aarch64-unknown-linux-gnu.armv8-2`

### ARMv8.4-A (Premium)
- **Compatibility**: Cortex-A76, A77, A78
- **Features**: +DOTPROD, +FLAGM
- **Use Case**: Premium devices (2020-2022 flagship phones)
- **Build Target**: `aarch64-unknown-linux-gnu.armv8-4`

### ARMv8.5-A (Flagship)
- **Compatibility**: Cortex-X1, X2
- **Features**: +SSBS, +SB (speculation barriers)
- **Use Case**: Latest flagship devices (2022+ high-end phones)
- **Build Target**: `aarch64-unknown-linux-gnu.armv8-5`

## Build Commands

### Default Build (Native CPU Detection)
```bash
# Uses CPU auto-detection for best performance on the build machine
cargo build --target aarch64-unknown-linux-gnu
```

### Specific Instruction Set Version
```bash
# Build for ARMv8.2-A (high-end devices)
cargo build --target aarch64-unknown-linux-gnu.armv8-2

# Build for ARMv8.0-A (baseline compatibility)
cargo build --target aarch64-unknown-linux-gnu.armv8-0
```

### Platform-Specific Builds
```bash
# Android (Cortex-A72 optimized)
cargo build --target aarch64-linux-android

# Apple Silicon (iOS/macOS with native detection)
cargo build --target aarch64-apple-darwin
```

## Optimization Features by Target

| Feature | ARMv8.0 | ARMv8.1 | ARMv8.2 | ARMv8.4 | ARMv8.5 |
|---------|---------|---------|---------|---------|---------|
| NEON SIMD | ✅ | ✅ | ✅ | ✅ | ✅ |
| FP-ARMv8 | ✅ | ✅ | ✅ | ✅ | ✅ |
| CRC | ❌ | ✅ | ✅ | ✅ | ✅ |
| LSE | ❌ | ✅ | ✅ | ✅ | ✅ |
| RDM | ❌ | ❌ | ✅ | ✅ | ✅ |
| FP16 | ❌ | ❌ | ✅ | ✅ | ✅ |
| DOTPROD | ❌ | ❌ | ❌ | ✅ | ✅ |
| FLAGM | ❌ | ❌ | ❌ | ✅ | ✅ |
| SSBS | ❌ | ❌ | ❌ | ❌ | ✅ |
| SB | ❌ | ❌ | ❌ | ❌ | ✅ |

## Performance Considerations

### Trade-offs
- **Higher instruction set versions** provide better performance but reduce compatibility
- **Baseline (ARMv8.0)** ensures maximum compatibility but may not use all CPU features
- **Native detection** (default) provides the best balance for most use cases

### Recommendations

1. **App Stores**: Use baseline (ARMv8.0) for maximum compatibility
2. **Direct Distribution**: Consider multiple APKs/IPAs for different device tiers
3. **Development**: Use native detection for best performance on development machines
4. **CI/CD**: Build multiple versions for comprehensive device coverage

## Implementation Details

### NEON SIMD
- **Always Available**: All ARMv8-A devices support NEON
- **Wide Crate**: Automatically utilizes NEON through Rust's SIMD intrinsics
- **Performance**: 2-4x speedup for vectorizable operations

### Crypto Instructions
- **Hardware Acceleration**: SHA-1, SHA-2, AES acceleration
- **Security**: Speculation barrier instructions for Spectre mitigation

### Floating-Point
- **FP16**: Half-precision floating-point for ML and graphics
- **RDM**: Rounding double multiply for financial calculations

## Integration with Build Systems

### Gradle (Android)
```gradle
android {
    externalNativeBuild {
        cmake {
            arguments "-DCMAKE_SYSTEM_NAME=Android",
                    "-DCMAKE_SYSTEM_VERSION=21",
                    "-DCMAKE_ANDROID_ARCH_ABI=arm64-v8a",
                    "-DCMAKE_ANDROID_NDK=$ANDROID_NDK"
        }
    }
    
    // For multiple ABIs
    splits {
        abi {
            enable true
            reset()
            include 'arm64-v8a', 'armeabi-v7a', 'x86', 'x86_64'
            universalApk false
        }
    }
}
```

### Xcode (iOS)
```bash
# Build for iOS with native optimizations
cargo build --target aarch64-apple-ios

# Create universal binary
lipo -create \
    target/aarch64-apple-ios/debug/libkistaverk_core.a \
    target/x86_64-apple-ios/debug/libkistaverk_core.a \
    -output libkistaverk_core.a
```

### CI/CD Pipeline Example
```yaml
# GitHub Actions example
jobs:
  build:
    strategy:
      matrix:
        target: [
          'aarch64-unknown-linux-gnu.armv8-0',  # Baseline
          'aarch64-unknown-linux-gnu.armv8-2',  # High-end
          'aarch64-unknown-linux-gnu'           # Native (for dev)
        ]
    
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
      - run: cargo build --target ${{ matrix.target }} --release
      - run: strip target/${{ matrix.target }}/release/libkistaverk_core.so
      - uses: actions/upload-artifact@v3
        with:
          name: kistaverk-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/libkistaverk_core.so
```

## Testing Optimizations

### Verify NEON Usage
```bash
# Check for NEON instructions in the binary
aarch64-linux-gnu-objdump -d libkistaverk_core.so | grep -i neon
```

### Benchmark Different Targets
```bash
# Build and compare performance
hyperfine \
    'cargo build --target aarch64-unknown-linux-gnu.armv8-0 --release' \
    'cargo build --target aarch64-unknown-linux-gnu.armv8-2 --release' \
    'cargo build --target aarch64-unknown-linux-gnu --release'
```

## Future Enhancements

### Dynamic Dispatch
Consider runtime CPU feature detection for single binary that adapts to the device:
```rust
#[cfg(target_arch = "aarch64")]
fn detect_cpu_features() -> CpuFeatures {
    // Use CPUID or similar to detect available features
    // Return appropriate feature set
}
```

### Profile-Guided Optimization
```bash
# Collect profiling data
cargo build --release
RUSTFLAGS="-C profile-generate" cargo run --release

# Apply PGO
RUSTFLAGS="-C profile-use" cargo build --release
```

## References

- [ARM Architecture Reference](https://developer.arm.com/architectures)
- [Rust SIMD Guide](https://doc.rust-lang.org/stable/core/arch/)
- [Wide Crate Documentation](https://docs.rs/wide)
- [Cargo Target Configuration](https://doc.rust-lang.org/cargo/reference/config.html)

## Support Matrix

| Device Class | Recommended Target | Notes |
|--------------|-------------------|-------|
| Entry-level phones | armv8-0 | Maximum compatibility |
| Mid-range phones | armv8-1 | Cortex-A72 class |
| High-end phones | armv8-2 | Cortex-A75/A76 class |
| Premium phones | armv8-4 | Cortex-A76/A77/A78 class |
| Flagship phones | armv8-5 | Cortex-X1/X2 class |
| Development machines | native | Best performance on build machine |
