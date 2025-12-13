#!/bin/bash

# Test script to verify the build script is properly set up
# This doesn't actually build anything, just checks the environment

echo "🔍 Testing Android Build Script Setup..."
echo ""

# Check if script exists
if [ ! -f "build_gmp_android.sh" ]; then
    echo "❌ build_gmp_android.sh not found!"
    exit 1
fi

echo "✅ build_gmp_android.sh found"

# Check if script is executable
if [ ! -x "build_gmp_android.sh" ]; then
    echo "❌ build_gmp_android.sh is not executable"
    echo "   Run: chmod +x build_gmp_android.sh"
    exit 1
fi

echo "✅ build_gmp_android.sh is executable"

# Check if README exists
if [ ! -f "ANDROID_BUILD_README.md" ]; then
    echo "❌ ANDROID_BUILD_README.md not found!"
    exit 1
fi

echo "✅ ANDROID_BUILD_README.md found"

# Check Android NDK availability
if [ -z "$ANDROID_NDK_HOME" ] && [ -z "$ANDROID_HOME" ]; then
    echo "⚠️  Android NDK environment variables not set"
    echo "   You'll need to set ANDROID_NDK_HOME or ANDROID_HOME before running the build script"
else
    echo "✅ Android NDK environment variables are set"
fi

# Check basic build tools
MISSING_TOOLS=()
for tool in make curl tar; do
    if ! command -v $tool &> /dev/null; then
        MISSING_TOOLS+=("$tool")
    fi
done

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
    echo "⚠️  Missing build tools: ${MISSING_TOOLS[*]}"
    echo "   These tools are required for building GMP/MPFR/MPC"
else
    echo "✅ All required build tools are available"
fi

echo ""
echo "📋 Build Script Information:"
echo "   - Script: build_gmp_android.sh"
echo "   - Documentation: ANDROID_BUILD_README.md"
echo "   - Output Directory: rust/libs/android/"
echo "   - Target Architectures: aarch64, armv7a, i686, x86_64"
echo ""
echo "🚀 To build the libraries, run:"
echo "   ./build_gmp_android.sh"
echo ""
echo "✅ Setup verification complete!"