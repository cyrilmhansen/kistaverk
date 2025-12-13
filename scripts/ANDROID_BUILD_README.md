# Android Build Script for GMP/MPFR/MPC

This directory contains scripts for cross-compiling GMP, MPFR, and MPC libraries for Android targets, which are required for the `rug` crate's arbitrary precision arithmetic.

## Prerequisites

1. **Android NDK**: Must be installed and available in your PATH
   - Set `ANDROID_NDK_HOME` environment variable, or
   - Set `ANDROID_HOME` environment variable pointing to your Android SDK installation

2. **Build Tools**: Standard Unix build tools (make, curl, tar, etc.)

3. **Disk Space**: ~2GB for source code and build artifacts

## Building the Libraries

### Step 1: Run the build script

```bash
./build_gmp_android.sh
```

### Step 2: Verify the output

The script will create the following directory structure:

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

## Build Process Details

The script performs the following steps for each target architecture:

1. **Download Source Code**:
   - GMP (GNU Multiple Precision Arithmetic Library)
   - MPFR (Multiple Precision Floating-Point Reliable Library)
   - MPC (Multiple Precision Complex Library)

2. **Configure and Build**:
   - Configure with Android-specific toolchain settings
   - Build static libraries (.a files) with position-independent code
   - Install to target-specific directories

3. **Clean Up**: Remove temporary build directories

## Target Architectures

The script builds for all major Android ABIs:

- **aarch64-linux-android**: 64-bit ARM (ARMv8-A)
- **armv7a-linux-androideabi**: 32-bit ARM (ARMv7-A)
- **i686-linux-android**: 32-bit x86
- **x86_64-linux-android**: 64-bit x86

## Troubleshooting

### Android NDK Not Found

```
Error: Android NDK not found. Please set ANDROID_NDK_HOME or ANDROID_HOME.
```

**Solution**: Set the environment variables:

```bash
# Option 1: Set ANDROID_NDK_HOME directly
export ANDROID_NDK_HOME=/path/to/your/android-ndk

# Option 2: Set ANDROID_HOME (if NDK is installed under Android SDK)
export ANDROID_HOME=/path/to/your/android-sdk
```

### Build Failures

1. **Check NDK Version**: Ensure you have a recent NDK version (r21+ recommended)

2. **Disk Space**: Ensure you have enough disk space (~2GB)

3. **Network Issues**: The script downloads source archives. If downloads fail, check your internet connection.

4. **Build Tools**: Ensure you have make, curl, tar, and other standard Unix tools installed.

### Missing Libraries

If the build completes but libraries are missing, check:

- The build logs for configuration errors
- That the NDK toolchain is properly installed
- That the API level (21) is supported by your NDK version

## Integration with Rust Build

After successfully building the libraries, you'll need to:

1. Update `rust/build.rs` to link against these libraries (see Task 3.2)
2. Configure your Android app's build.gradle to enable the precision feature (see Task 3.3)

## License Information

The libraries built by this script are subject to their respective licenses:

- **GMP**: LGPLv3+ or GPLv2+
- **MPFR**: LGPLv3+
- **MPC**: LGPLv3+

Please review the license terms in the source packages for compliance requirements.

## Notes

- The script uses API level 21 (Android 5.0+) by default. Adjust if needed.
- Build times vary by hardware but typically take 10-30 minutes per architecture.
- The script can be modified to build for specific architectures only.

## Alternative Approach

If you encounter difficulties with this script, consider:

1. Using pre-built Android libraries from trusted sources
2. Building on a Linux environment (WSL on Windows works well)
3. Adjusting the API level or compiler flags for your specific NDK version