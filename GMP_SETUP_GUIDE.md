# GMP/MPFR/MPC Setup Guide for Kistaverk

## Overview

This guide explains how to set up the GMP/MPFR/MPC libraries for Kistaverk to ensure that `gmp-mpfr-sys` uses our pre-built libraries instead of attempting to build from source.

## The Problem

The `rug` crate depends on `gmp-mpfr-sys`, which by default attempts to build GMP/MPFR/MPC from source. This causes issues because:

1. **Cross-compilation complexity**: Building GMP from source for Android targets requires proper NDK setup
2. **Build consistency**: We want to use our optimized, pre-built libraries for all platforms
3. **Control**: We need to ensure the exact library versions we've tested with are used

## The Solution

We force `gmp-mpfr-sys` to use our pre-built libraries by:

1. **Environment Variables**: Setting variables that `gmp-mpfr-sys` checks before attempting to build
2. **Cargo Configuration**: Using `.cargo/config.toml` to set these variables for all builds
3. **Pre-built Libraries**: Providing libraries built by our `build_gmp_android.sh` script

## Setup Instructions

### 1. Build the Libraries (if not already built)

Run the build script to create GMP/MPFR/MPC libraries for all target architectures:

```bash
./scripts/build_gmp_android.sh
```

This will create the following structure:

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

### 2. Set Up Environment Variables

Add these environment variables to your shell configuration (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
export GMP_LIB_DIR="$PROJECT_ROOT/rust/libs/android"
export GMP_INCLUDE_DIR="$PROJECT_ROOT/rust/libs/android" 
export GMP_STATIC=1
export GMP_MPFR_SYS_USE_PKG_CONFIG=0
```

Replace `$PROJECT_ROOT` with the actual path to your Kistaverk project.

### 3. Update Cargo Configuration

Ensure your `rust/.cargo/config.toml` includes the GMP environment variables:

```toml
[env]
GMP_LIB_DIR = "${CARGO_MANIFEST_DIR}/libs/android"
GMP_INCLUDE_DIR = "${CARGO_MANIFEST_DIR}/libs/android"
GMP_STATIC = "1"
GMP_MPFR_SYS_USE_PKG_CONFIG = "0"
```

### 4. Build with Precision Feature

Now you can build with the precision feature:

```bash
cd rust
cargo build --features precision
```

For Android targets:

```bash
cargo build --target aarch64-linux-android --features precision
cargo build --target armv7a-linux-androideabi --features precision
# etc.
```

## How It Works

### Environment Variables Explained

- **`GMP_LIB_DIR`**: Tells `gmp-mpfr-sys` where to find the library files
- **`GMP_INCLUDE_DIR`**: Tells `gmp-mpfr-sys` where to find the header files
- **`GMP_STATIC=1`**: Forces static linking of the libraries
- **`GMP_MPFR_SYS_USE_PKG_CONFIG=0`**: Disables pkg-config (not available in Android NDK)

### Build Process Flow

1. Cargo starts the build process
2. `gmp-mpfr-sys` build script runs
3. It checks for the environment variables we set
4. Finding them, it uses our pre-built libraries instead of building from source
5. Our `build.rs` script runs and sets up additional linking flags
6. The final binary is linked with our optimized GMP libraries

## Troubleshooting

### "Library not found" Errors

If you get linker errors about missing GMP libraries:

1. Verify the libraries exist in `rust/libs/android/*/lib/`
2. Check that the environment variables are set correctly
3. Ensure the paths in the variables point to the correct locations

### Build Fails with "gmp.h not found"

This usually means:
- The `GMP_INCLUDE_DIR` variable is not set correctly
- The include files are missing from the library directories
- Run `build_gmp_android.sh` again to regenerate the libraries

### Android NDK Issues

For Android builds, ensure:
- Android NDK is installed and `ANDROID_NDK_HOME` is set
- The NDK version is r21 or newer
- The toolchain files exist in the NDK directory

## Advanced Configuration

### Using Different Library Versions

If you need to use different versions of GMP/MPFR/MPC:

1. Modify `scripts/build_gmp_android.sh` to use your desired versions
2. Update the version variables at the top of the script
3. Rebuild the libraries
4. Clean and rebuild your Rust project

### Custom Build Flags

You can add custom build flags to the GMP libraries by modifying the `configure` commands in `build_gmp_android.sh`. Common options include:

- `--enable-cxx`: Enable C++ support
- `--disable-assembly`: Disable assembly optimizations (for debugging)
- `--with-pic`: Ensure position-independent code

## Platform-Specific Notes

### Linux

- Works out of the box with the provided setup
- Ensure you have standard build tools installed (`make`, `gcc`, etc.)

### macOS (Apple Silicon)

- The same setup works for macOS
- You may need to install additional tools via Homebrew
- Ensure your NDK supports Apple Silicon builds

### Windows (WSL)

- Use Windows Subsystem for Linux (WSL)
- Install Ubuntu or other Linux distribution
- Follow the Linux instructions within WSL
- Access files from Windows file system if needed

## Maintenance

### Updating Libraries

When updating GMP/MPFR/MPC versions:

1. Update version numbers in `build_gmp_android.sh`
2. Run the build script to regenerate libraries
3. Test with your application
4. Commit the updated libraries to the repository

### Cleaning Up

To clean up and start fresh:

```bash
rm -rf rust/libs/android
rm -rf rust/target/debug/build/gmp-mpfr-sys-*
cargo clean
```

## Best Practices

1. **Commit Libraries**: Keep the pre-built libraries in the repository for consistency
2. **Document Versions**: Note which GMP/MPFR/MPC versions are being used
3. **Test Regularly**: Verify that the libraries work with new Rust/crate versions
4. **Automate**: Consider adding library building to your CI/CD pipeline

## License Compliance

Remember that GMP/MPFR/MPC have specific licensing requirements:

- **GMP**: LGPLv3+ or GPLv2+
- **MPFR**: LGPLv3+
- **MPC**: LGPLv3+

Ensure your project complies with these licenses when distributing binaries.

## Summary

This setup ensures that:

✅ All builds (native and cross-compiled) use our pre-built GMP libraries  
✅ No source compilation of GMP/MPFR/MPC occurs during builds  
✅ Consistent library versions across all platforms  
✅ Proper static linking for Android targets  
✅ Easy maintenance and updates

By following this guide, you'll have a robust GMP setup that works reliably across all supported platforms.