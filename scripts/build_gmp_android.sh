#!/bin/bash

# Android GMP/MPFR/MPC Build Script
# This script cross-compiles GMP, MPFR, and MPC for Android targets
# Required: Android NDK installed and in PATH

set -e  # Exit on error

# Configuration
if [ -n "$ANDROID_NDK_HOME" ]; then
    NDK_PATH="$ANDROID_NDK_HOME"
elif [ -n "$ANDROID_HOME" ] && [ -d "$ANDROID_HOME/ndk" ]; then
    # Find latest NDK
    LATEST_NDK=$(ls -1 "$ANDROID_HOME/ndk" | sort -V | tail -n1)
    if [ -n "$LATEST_NDK" ]; then
        NDK_PATH="$ANDROID_HOME/ndk/$LATEST_NDK"
    fi
fi

if [ -z "$NDK_PATH" ] || [ ! -d "$NDK_PATH" ]; then
    echo "Error: Android NDK not found."
    echo "Please set ANDROID_NDK_HOME environment variable,"
    echo "or ensure ANDROID_HOME is set and contains an 'ndk' directory."
    exit 1
fi

# Target architectures and toolchain prefixes
TARGETS=(
    "aarch64-linux-android"
    "armv7a-linux-androideabi"
    "i686-linux-android"
    "x86_64-linux-android"
)

# API level (adjust as needed)
API_LEVEL=21

# Source versions
GMP_VERSION="6.3.0"
MPFR_VERSION="4.2.1"
MPC_VERSION="1.3.1"

# Output directory
OUTPUT_DIR="rust/libs/android"
mkdir -p "$OUTPUT_DIR"

echo "Building GMP, MPFR, and MPC for Android..."
echo "NDK Path: $NDK_PATH"
echo "Output Directory: $OUTPUT_DIR"
echo "API Level: $API_LEVEL"
echo ""

# Build for each target architecture
for TARGET in "${TARGETS[@]}"; do
    echo "=== Building for $TARGET ==="
    
    # Determine toolchain prefix
    case "$TARGET" in
        aarch64-linux-android)
            TOOLCHAIN="aarch64-linux-android"
            HOST="aarch64-linux-android"
            ;;
        armv7a-linux-androideabi)
            TOOLCHAIN="arm-linux-androideabi"
            HOST="arm-linux-androideabi"
            ;;
        i686-linux-android)
            TOOLCHAIN="i686-linux-android"
            HOST="i686-linux-android"
            ;;
        x86_64-linux-android)
            TOOLCHAIN="x86_64-linux-android"
            HOST="x86_64-linux-android"
            ;;
        *)
            echo "Unknown target: $TARGET"
            continue
            ;;
    esac
    
    # Create build directory
    BUILD_DIR="build_$TARGET"
    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"
    
    # Set up environment
    export PATH="$NDK_PATH/toolchains/llvm/prebuilt/$(uname -m | sed 's/x86_64/x86_64/;s/arm64/aarch64/')-linux-android/bin:$PATH"
    export CC="${TOOLCHAIN}-clang"
    export CXX="${TOOLCHAIN}-clang++"
    export AR="llvm-ar"
    export RANLIB="llvm-ranlib"
    export STRIP="llvm-strip"
    export CFLAGS="--target=$TARGET --sysroot=$NDK_PATH/toolchains/llvm/prebuilt/$(uname -m | sed 's/x86_64/x86_64/;s/arm64/aarch64/')-linux-android/sysroot -fPIC"
    export CXXFLAGS="$CFLAGS"
    export LDFLAGS="--target=$TARGET --sysroot=$NDK_PATH/toolchains/llvm/prebuilt/$(uname -m | sed 's/x86_64/x86_64/;s/arm64/aarch64/')-linux-android/sysroot"
    
    # Build GMP
    echo "Building GMP..."
    if [ ! -f "gmp-${GMP_VERSION}.tar.xz" ]; then
        curl -L -O "https://gmplib.org/download/gmp/gmp-${GMP_VERSION}.tar.xz"
    fi
    tar -xf "gmp-${GMP_VERSION}.tar.xz"
    cd "gmp-${GMP_VERSION}"
    ./configure \
        --host=$HOST \
        --prefix="$OUTPUT_DIR/$TARGET" \
        --enable-static \
        --disable-shared \
        --with-pic \
        ABI=$API_LEVEL
    make -j$(nproc)
    make install
    cd ..
    
    # Build MPFR (depends on GMP)
    echo "Building MPFR..."
    if [ ! -f "mpfr-${MPFR_VERSION}.tar.xz" ]; then
        curl -L -O "https://www.mpfr.org/mpfr-${MPFR_VERSION}/mpfr-${MPFR_VERSION}.tar.xz"
    fi
    tar -xf "mpfr-${MPFR_VERSION}.tar.xz"
    cd "mpfr-${MPFR_VERSION}"
    ./configure \
        --host=$HOST \
        --prefix="$OUTPUT_DIR/$TARGET" \
        --enable-static \
        --disable-shared \
        --with-pic \
        --with-gmp="$OUTPUT_DIR/$TARGET" \
        ABI=$API_LEVEL
    make -j$(nproc)
    make install
    cd ..
    
    # Build MPC (depends on GMP and MPFR)
    echo "Building MPC..."
    if [ ! -f "mpc-${MPC_VERSION}.tar.gz" ]; then
        curl -L -O "https://www.multiprecision.org/mpc/download/mpc-${MPC_VERSION}.tar.gz"
    fi
    tar -xf "mpc-${MPC_VERSION}.tar.gz"
    cd "mpc-${MPC_VERSION}"
    ./configure \
        --host=$HOST \
        --prefix="$OUTPUT_DIR/$TARGET" \
        --enable-static \
        --disable-shared \
        --with-pic \
        --with-gmp="$OUTPUT_DIR/$TARGET" \
        --with-mpfr="$OUTPUT_DIR/$TARGET" \
        ABI=$API_LEVEL
    make -j$(nproc)
    make install
    cd ..
    
    # Clean up
    cd ..
    rm -rf "$BUILD_DIR"
    
    echo "✅ Successfully built libraries for $TARGET"
    echo "   Libraries available in: $OUTPUT_DIR/$TARGET/lib/"
    echo ""
done

echo "🎉 All Android libraries built successfully!"
echo ""
echo "Summary:"
for TARGET in "${TARGETS[@]}"; do
    if [ -d "$OUTPUT_DIR/$TARGET/lib" ]; then
        echo "  ✓ $TARGET: $(ls $OUTPUT_DIR/$TARGET/lib/*.a 2>/dev/null | wc -l) static libraries"
    else
        echo "  ✗ $TARGET: Build failed"
    fi
done