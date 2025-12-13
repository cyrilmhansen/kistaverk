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
else
    # Try to read local.properties
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
    LOCAL_PROPS="$PROJECT_ROOT/app/local.properties"
    
    if [ -f "$LOCAL_PROPS" ]; then
        NDK_DIR=$(grep "^ndk.dir" "$LOCAL_PROPS" | cut -d'=' -f2)
        if [ -n "$NDK_DIR" ]; then
            NDK_PATH="$NDK_DIR"
        else
            SDK_DIR=$(grep "^sdk.dir" "$LOCAL_PROPS" | cut -d'=' -f2)
            if [ -n "$SDK_DIR" ] && [ -d "$SDK_DIR/ndk" ]; then
                LATEST_NDK=$(ls -1 "$SDK_DIR/ndk" | sort -V | tail -n1)
                if [ -n "$LATEST_NDK" ]; then
                    NDK_PATH="$SDK_DIR/ndk/$LATEST_NDK"
                fi
            fi
        fi
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

# Download URLs (can be overridden by environment variables)
GMP_URL="${GMP_URL:-https://gmplib.org/download/gmp/gmp-${GMP_VERSION}.tar.xz}"
MPFR_URL="${MPFR_URL:-https://www.mpfr.org/mpfr-${MPFR_VERSION}/mpfr-${MPFR_VERSION}.tar.xz}"
MPC_URL="${MPC_URL:-https://ftp.gnu.org/gnu/mpc/mpc-${MPC_VERSION}.tar.gz}"

# Output directory
mkdir -p "rust/libs/android"
OUTPUT_DIR="$(cd "rust/libs/android" && pwd)"

echo "Building GMP, MPFR, and MPC for Android..."
echo "NDK Path: $NDK_PATH"
echo "Output Directory: $OUTPUT_DIR"
echo "API Level: $API_LEVEL"
echo ""

# Build for each target architecture
for TARGET in "${TARGETS[@]}"; do
    echo "=== Building for $TARGET ==="
    
    # Determine toolchain prefix and GMP ABI
    case "$TARGET" in
        aarch64-linux-android)
            TOOLCHAIN="aarch64-linux-android"
            COMPILER_PREFIX="aarch64-linux-android${API_LEVEL}"
            HOST="aarch64-linux-android"
            GMP_ABI="64"
            ;;
        armv7a-linux-androideabi)
            TOOLCHAIN="arm-linux-androideabi"
            COMPILER_PREFIX="armv7a-linux-androideabi${API_LEVEL}"
            HOST="arm-linux-androideabi"
            GMP_ABI="32"
            ;;
        i686-linux-android)
            TOOLCHAIN="i686-linux-android"
            COMPILER_PREFIX="i686-linux-android${API_LEVEL}"
            HOST="i686-linux-android"
            GMP_ABI="32"
            ;;
        x86_64-linux-android)
            TOOLCHAIN="x86_64-linux-android"
            COMPILER_PREFIX="x86_64-linux-android${API_LEVEL}"
            HOST="x86_64-linux-android"
            GMP_ABI="64"
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
    
    # Determine host tag for NDK toolchain
    HOST_TAG="linux-x86_64"
    if [ "$(uname -s)" == "Darwin" ]; then
        HOST_TAG="darwin-x86_64"
    fi

    # Set up environment
    export PATH="$NDK_PATH/toolchains/llvm/prebuilt/$HOST_TAG/bin:$PATH"
    export CC="${COMPILER_PREFIX}-clang"
    export CXX="${COMPILER_PREFIX}-clang++"
    export AR="llvm-ar"
    export RANLIB="llvm-ranlib"
    export STRIP="llvm-strip"
    export CFLAGS="-fPIC"
    export CXXFLAGS="$CFLAGS"
    export LDFLAGS=""
    
    # Build GMP
    echo "Building GMP..."
    if [ ! -f "gmp-${GMP_VERSION}.tar.xz" ]; then
        curl -f -L -O "$GMP_URL"
    fi
    tar -xf "gmp-${GMP_VERSION}.tar.xz"
    cd "gmp-${GMP_VERSION}"
    ./configure \
        --host=$HOST \
        --prefix="$OUTPUT_DIR/$TARGET" \
        --enable-static \
        --disable-shared \
        --with-pic \
        ABI=$GMP_ABI
    make -j$(nproc)
    make install
    cd ..
    
    # Build MPFR (depends on GMP)
    echo "Building MPFR..."
    if [ ! -f "mpfr-${MPFR_VERSION}.tar.xz" ]; then
        curl -f -L -O "$MPFR_URL"
    fi
    tar -xf "mpfr-${MPFR_VERSION}.tar.xz"
    cd "mpfr-${MPFR_VERSION}"
    ./configure \
        --host=$HOST \
        --prefix="$OUTPUT_DIR/$TARGET" \
        --enable-static \
        --disable-shared \
        --with-pic \
        --with-gmp="$OUTPUT_DIR/$TARGET"
    make -j$(nproc)
    make install
    cd ..
    
    # Build MPC (depends on GMP and MPFR)
    echo "Building MPC..."
    if [ ! -f "mpc-${MPC_VERSION}.tar.gz" ]; then
        curl -f -L -O "$MPC_URL"
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
        --with-mpfr="$OUTPUT_DIR/$TARGET"
    make -j$(nproc)
    make install
    
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