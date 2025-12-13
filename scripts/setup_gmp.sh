#!/bin/bash

# Simple GMP Setup Script for Kistaverk
# Sets up environment variables for using pre-built GMP libraries

echo "🚀 Setting up GMP environment for Kistaverk..."
echo ""

# Get the project root directory
PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

echo "📁 Project root: $PROJECT_ROOT"
echo ""

# Set environment variables
export GMP_LIB_DIR="$PROJECT_ROOT/rust/libs/android"
export GMP_INCLUDE_DIR="$PROJECT_ROOT/rust/libs/android"
export GMP_STATIC=1
export GMP_MPFR_SYS_USE_PKG_CONFIG=0

echo "✅ Environment variables set:"
echo "    GMP_LIB_DIR=$GMP_LIB_DIR"
echo "    GMP_INCLUDE_DIR=$GMP_INCLUDE_DIR"
echo "    GMP_STATIC=$GMP_STATIC"
echo "    GMP_MPFR_SYS_USE_PKG_CONFIG=$GMP_MPFR_SYS_USE_PKG_CONFIG"
echo ""

# Check if libraries exist
echo "🔍 Checking GMP libraries..."

if [ -d "$GMP_LIB_DIR" ]; then
    echo "✅ GMP library directory found"
    
    # Count libraries
    lib_count=$(find "$GMP_LIB_DIR" -name "*.a" 2>/dev/null | wc -l)
    if [ "$lib_count" -gt 0 ]; then
        echo "✅ Found $lib_count library files"
    else
        echo "⚠️  No library files found in $GMP_LIB_DIR"
        echo "    Run ./scripts/build_gmp_android.sh to build libraries"
    fi
else
    echo "❌ GMP library directory not found: $GMP_LIB_DIR"
    echo "    Run ./scripts/build_gmp_android.sh to create it"
fi

echo ""
echo "🎯 You can now build with precision support:"
echo "    cd rust"
echo "    cargo build --features precision"
echo ""
echo "📚 For Android builds:"
echo "    cargo build --target aarch64-linux-android --features precision"
echo "    cargo build --target armv7a-linux-androideabi --features precision"
echo "    etc."
echo ""
echo "💡 To make these settings permanent, add the export lines above"
echo "    to your shell configuration file (~/.bashrc, ~/.zshrc, etc.)"