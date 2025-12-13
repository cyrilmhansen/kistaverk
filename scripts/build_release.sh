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

# --- Step 2: Run Gradle Build ---
echo ""
echo "🚀 Starting Android Gradle Build..."

# Determine task (default to assembleDebug)
TASK="${1:-assembleDebug}"

cd "$PROJECT_ROOT/app"
./gradlew -PuseUpx="$USE_UPX" "app:$TASK"

echo ""
echo "🎉 Build Complete!"
if [ "$TASK" == "assembleDebug" ]; then
    echo "apk: app/app/build/outputs/apk/debug/"
elif [ "$TASK" == "assembleRelease" ]; then
    echo "apk: app/app/build/outputs/apk/release/"
fi
