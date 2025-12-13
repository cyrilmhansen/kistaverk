#!/bin/bash

# Master Build Script for Kistaverk (Android)
# Automates GMP/MPFR/MPC compilation and Gradle build in one command.

set -e

# --- Configuration ---
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GMP_LIBS_DIR="$PROJECT_ROOT/rust/libs/android"
# Check just one architecture to verify presence (assuming all built together)
CHECK_FILE="$GMP_LIBS_DIR/aarch64-linux-android/lib/libgmp.a"
# Enable UPX compression by default (set USE_UPX=false to skip)
USE_UPX="${USE_UPX:-true}"
# Allow ABI selection (arm64, armv7, both)
ABI_PROP="${abi:-}"
# Precision feature (default: false)
ENABLE_PRECISION="${enablePrecision:-false}"

# --- Setup Environment from local.properties ---
LOCAL_PROPS="$PROJECT_ROOT/app/local.properties"
if [ -f "$LOCAL_PROPS" ]; then
    echo "📄 Reading configuration from $LOCAL_PROPS"
    
    # Extract sdk.dir
    SDK_DIR=$(grep "^sdk.dir" "$LOCAL_PROPS" | cut -d'=' -f2)
    if [ -n "$SDK_DIR" ]; then
        export ANDROID_HOME="$SDK_DIR"
        export ANDROID_SDK_ROOT="$SDK_DIR"
        echo "   ANDROID_HOME set to $ANDROID_HOME"
    fi
    
    # Extract ndk.dir
    NDK_DIR=$(grep "^ndk.dir" "$LOCAL_PROPS" | cut -d'=' -f2)
    if [ -n "$NDK_DIR" ]; then
        export ANDROID_NDK_HOME="$NDK_DIR"
        echo "   ANDROID_NDK_HOME set to $ANDROID_NDK_HOME"
    fi
fi

# Fallback: Try to find NDK in standard locations if not set
if [ -z "$ANDROID_NDK_HOME" ] && [ -n "$ANDROID_HOME" ]; then
    if [ -d "$ANDROID_HOME/ndk" ]; then
        # Use the latest NDK version found
        LATEST_NDK=$(ls -1 "$ANDROID_HOME/ndk" | sort -V | tail -n1)
        if [ -n "$LATEST_NDK" ]; then
            export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/$LATEST_NDK"
            echo "   Found NDK in standard location: $ANDROID_NDK_HOME"
        fi
    fi
fi

# --- Step 1: Check/Build Native Libraries ---
echo "🔍 Checking for pre-built math libraries..."
echo "   Config: ABI=${ABI_PROP:-arm64} USE_UPX=${USE_UPX} enablePrecision=${ENABLE_PRECISION} SKIP_TESTS=${SKIP_TESTS:-false}"

if [ -f "$CHECK_FILE" ]; then
    echo "✅ Pre-built libraries found at:"
    echo "   $CHECK_FILE"
    echo "   Skipping native library compilation."
else
    echo "⚠️  Libraries not found. Starting compilation..."
    echo "   This may take a few minutes (GMP, MPFR, MPC)."
    
    # Run the build script
    "$PROJECT_ROOT/scripts/build_gmp_android.sh"
    
    # Verify again
    if [ ! -f "$CHECK_FILE" ]; then
        echo "❌ Error: Library build failed. $CHECK_FILE not created."
        exit 1
    fi
fi

# --- Step 1b: Run Rust tests (unless skipped) ---
if [ "${SKIP_TESTS:-false}" != "true" ]; then
    echo ""
    echo "🧪 Running Rust tests..."
    pushd "$PROJECT_ROOT/rust" >/dev/null
    cargo test --locked
    popd >/dev/null

    # --- Step 1c: Run Kotlin/JVM unit tests ---
    echo ""
    echo "🧪 Running Kotlin unit tests..."
    pushd "$PROJECT_ROOT/app" >/dev/null
    ./gradlew test
    popd >/dev/null
else
    echo "⏭️  SKIP_TESTS=true, skipping Rust and Kotlin tests."
fi

# --- Step 2: Run Gradle Build ---
echo ""
echo "🚀 Starting Android Gradle Build..."

# Determine task (default to assembleDebug)
TASK="${1:-assembleDebug}"

cd "$PROJECT_ROOT/app"
if [ -n "$ABI_PROP" ]; then
    ./gradlew -PuseUpx="$USE_UPX" -Pabi="$ABI_PROP" -PenablePrecision="$ENABLE_PRECISION" "app:$TASK"
else
    ./gradlew -PuseUpx="$USE_UPX" -PenablePrecision="$ENABLE_PRECISION" "app:$TASK"
fi

echo ""
echo "🎉 Build Complete!"
if [ "$TASK" == "assembleDebug" ]; then
    echo "apk: app/app/build/outputs/apk/debug/"
elif [ "$TASK" == "assembleRelease" ]; then
    echo "apk: app/app/build/outputs/apk/release/"
fi
