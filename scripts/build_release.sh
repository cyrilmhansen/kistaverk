#!/bin/bash

# Master Build Script for Kistaverk (Android)
# Automates GMP/MPFR/MPC compilation and Gradle build in one command.

set -e

# --- Configuration ---
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GMP_LIBS_DIR="$PROJECT_ROOT/rust/libs/android"
# Check just one architecture to verify presence (assuming all built together)
CHECK_FILE="$GMP_LIBS_DIR/aarch64-linux-android/lib/libgmp.a"

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

cd "$PROJECT_ROOT"
"$PROJECT_ROOT/app/gradlew" "app:$TASK"

echo ""
echo "🎉 Build Complete!"
if [ "$TASK" == "assembleDebug" ]; then
    echo "apk: app/app/build/outputs/apk/debug/"
elif [ "$TASK" == "assembleRelease" ]; then
    echo "apk: app/app/build/outputs/apk/release/"
fi
