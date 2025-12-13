#!/bin/bash

# GMP/MPFR/MPC Library Setup for Kistaverk
# This script sets up the environment to use our pre-built math libraries
# instead of letting gmp-mpfr-sys build from source

set -e

echo "🚀 GMP/MPFR/MPC Library Setup for Kistaverk"
echo "============================================"
echo ""

# Function to check if libraries exist
check_libraries() {
    local lib_dir="$1"
    local arch_name="$2"
    
    echo "🔍 Checking $arch_name libraries in: $lib_dir"
    
    local required_files=(
        "lib/libgmp.a"
        "lib/libmpfr.a" 
        "lib/libmpc.a"
        "include/gmp.h"
        "include/mpfr.h"
        "include/mpc.h"
    )
    
    local all_found=true
    
    for file in "${required_files[@]}"; do
        if [ -f "$lib_dir/$file" ]; then
            echo "  ✅ $file"
        else
            echo "  ❌ $file (missing)"
            all_found=false
        fi
    done
    
    if [ "$all_found" = true ]; then
        echo "✅ All $arch_name libraries are present"
        return 0
    else
        echo "❌ Some $arch_name libraries are missing"
        return 1
    fi
}

# Function to build libraries if missing
build_libraries() {
    echo "🛠️  Building GMP/MPFR/MPC libraries..."
    echo ""
    
    if [ ! -f "scripts/build_gmp_android.sh" ]; then
        echo "❌ build_gmp_android.sh script not found!"
        return 1
    fi
    
    echo "🔨 Running build script..."
    ./scripts/build_gmp_android.sh
    
    if [ $? -eq 0 ]; then
        echo "✅ Library build completed successfully"
        return 0
    else
        echo "❌ Library build failed"
        return 1
    fi
}

# Function to set up environment variables
setup_environment() {
    echo "🌐 Setting up environment variables..."
    echo ""
    
    # Determine the appropriate shell configuration file
    local shell_config=""
    if [ -f "$HOME/.bashrc" ]; then
        shell_config="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
        shell_config="$HOME/.bash_profile"
    elif [ -f "$HOME/.zshrc" ]; then
        shell_config="$HOME/.zshrc"
    elif [ -f "$HOME/.profile" ]; then
        shell_config="$HOME/.profile"
    fi
    
    # Check if variables are already set
    local variables_set=false
    
    if [ -n "$shell_config" ]; then
        echo "📝 Checking shell configuration: $shell_config"
        
        # Check if GMP variables are already configured
        if grep -q "GMP_LIB_DIR" "$shell_config" && \
           grep -q "GMP_INCLUDE_DIR" "$shell_config" && \
           grep -q "GMP_STATIC" "$shell_config"; then
            echo "✅ GMP environment variables are already configured"
            variables_set=true
        fi
    fi
    
    if [ "$variables_set" = false ]; then
        echo "📝 Adding GMP environment variables..."
        
        # Add to shell config file
        if [ -n "$shell_config" ]; then
            echo "" >> "$shell_config"
            echo "# GMP/MPFR/MPC Configuration for Kistaverk" >> "$shell_config"
            echo "# Forces gmp-mpfr-sys to use our pre-built libraries instead of building from source" >> "$shell_config"
            echo "export GMP_LIB_DIR="$PWD/rust/libs/android"" >> "$shell_config"
            echo "export GMP_INCLUDE_DIR="$PWD/rust/libs/android"" >> "$shell_config"
            echo "export GMP_STATIC=1" >> "$shell_config"
            echo "export GMP_MPFR_SYS_USE_PKG_CONFIG=0" >> "$shell_config"
            echo "" >> "$shell_config"
            echo "✅ Environment variables added to $shell_config"
        else
            echo "⚠️  Could not determine shell configuration file."
            echo "    Please manually add these variables to your shell config:"
            echo "    export GMP_LIB_DIR="$PWD/rust/libs/android""
            echo "    export GMP_INCLUDE_DIR="$PWD/rust/libs/android""
            echo "    export GMP_STATIC=1"
            echo "    export GMP_MPFR_SYS_USE_PKG_CONFIG=0"
        fi
    fi
    
    # Set variables for current session
    export GMP_LIB_DIR="$PWD/rust/libs/android"
    export GMP_INCLUDE_DIR="$PWD/rust/libs/android"
    export GMP_STATIC=1
    export GMP_MPFR_SYS_USE_PKG_CONFIG=0
    
    echo ""
    echo "🔄 Environment variables set for current session:"
    echo "    GMP_LIB_DIR=$GMP_LIB_DIR"
    echo "    GMP_INCLUDE_DIR=$GMP_INCLUDE_DIR"
    echo "    GMP_STATIC=$GMP_STATIC"
    echo "    GMP_MPFR_SYS_USE_PKG_CONFIG=$GMP_MPFR_SYS_USE_PKG_CONFIG"
    echo ""
    
    echo "💡 To apply the changes permanently, run:"
    if [ -n "$shell_config" ]; then
        echo "    source $shell_config"
    else
        echo "    source your shell configuration file"
    fi
    echo ""
}

# Function to create Cargo configuration
create_cargo_config() {
    echo "📦 Creating Cargo configuration..."
    echo ""
    
    local cargo_config="rust/.cargo/config.toml"
    local backup_config="rust/.cargo/config.toml.backup"
    
    # Backup existing config if it exists
    if [ -f "$cargo_config" ]; then
        echo "📝 Backing up existing Cargo config to: $backup_config"
        cp "$cargo_config" "$backup_config"
    fi
    
    # Create new config with GMP settings
    cat > "$cargo_config" << 'EOF'
# Cargo configuration for Kistaverk with GMP/MPFR/MPC support

# ============================================================================
# GMP/MPFR/MPC Configuration
# ============================================================================
# Force all builds to use our pre-built GMP libraries instead of building from source

# Environment variables to prevent gmp-mpfr-sys from building GMP from source
[env]
GMP_LIB_DIR = "${CARGO_MANIFEST_DIR}/libs/android"
GMP_INCLUDE_DIR = "${CARGO_MANIFEST_DIR}/libs/android"
GMP_STATIC = "1"
GMP_MPFR_SYS_USE_PKG_CONFIG = "0"

# ============================================================================
# Platform-Specific Optimizations
# ============================================================================

# Enable optimizations for all profiles
[profile.dev]
opt-level = 1  # Basic optimizations for development

[profile.release]
opt-level = "z"  # Optimize for size
lto = "fat"     # Link-time optimization
codegen-units = 1  # Better optimization
panic = "abort"  # Reduce binary size
strip = "symbols"  # Strip debug symbols

# ============================================================================
# ARM64 Instruction Set Version Targets
# ============================================================================
# These provide optional build targets for different generations of ARM64 devices
# while maintaining backward compatibility.

# Baseline ARMv8.0-A (Compatible with all ARM64 devices)
[target.aarch64-unknown-linux-gnu.armv8-0]
inherits = "aarch64-unknown-linux-gnu"
rustflags = [
    "-C", "target-cpu=generic",        # Baseline ARMv8.0-A
    "-C", "target-feature=+neon",      # NEON is always available
    "-C", "target-feature=+fp-armv8",  # ARMv8 floating-point
    "-C", "link-arg=-march=armv8-a",
]

# ARMv8.1-A (Cortex-A72, Kryo, etc.) - Common in mid-range devices
[target.aarch64-unknown-linux-gnu.armv8-1]
inherits = "aarch64-unknown-linux-gnu"
rustflags = [
    "-C", "target-cpu=cortex-a72",     # ARMv8.1-A
    "-C", "target-feature=+neon",      # NEON SIMD
    "-C", "target-feature=+fp-armv8",  # ARMv8 FP
    "-C", "target-feature=+crc",       # Hardware CRC
    "-C", "target-feature=+lse",       # Large System Extensions
    "-C", "link-arg=-march=armv8.1-a",
]

# ARMv8.2-A (Cortex-A75, A76) - High-end devices
[target.aarch64-unknown-linux-gnu.armv8-2]
inherits = "aarch64-unknown-linux-gnu"
rustflags = [
    "-C", "target-cpu=cortex-a75",     # ARMv8.2-A
    "-C", "target-feature=+neon",      # NEON SIMD
    "-C", "target-feature=+fp-armv8",  # ARMv8 FP
    "-C", "target-feature=+crc",       # Hardware CRC
    "-C", "target-feature=+lse",       # Large System Extensions
    "-C", "target-feature=+rdm",       # Round Double Multiply
    "-C", "target-feature=+fp16",      # Half-precision floating-point
    "-C", "link-arg=-march=armv8.2-a",
]

# ARMv8.4-A (Cortex-A76, A77, A78) - Premium devices
[target.aarch64-unknown-linux-gnu.armv8-4]
inherits = "aarch64-unknown-linux-gnu"
rustflags = [
    "-C", "target-cpu=cortex-a76",     # ARMv8.4-A
    "-C", "target-feature=+neon",      # NEON SIMD
    "-C", "target-feature=+fp-armv8",  # ARMv8 FP
    "-C", "target-feature=+crc",       # Hardware CRC
    "-C", "target-feature=+lse",       # Large System Extensions
    "-C", "target-feature=+rdm",       # Round Double Multiply
    "-C", "target-feature=+fp16",      # Half-precision FP
    "-C", "target-feature=+dotprod",   # Dot Product instructions
    "-C", "target-feature=+flagm",     # Flag manipulation
    "-C", "link-arg=-march=armv8.4-a",
]

# ARMv8.5-A (Cortex-X1, X2) - Flagship devices
[target.aarch64-unknown-linux-gnu.armv8-5]
inherits = "aarch64-unknown-linux-gnu"
rustflags = [
    "-C", "target-cpu=cortex-x1",      # ARMv8.5-A
    "-C", "target-feature=+neon",      # NEON SIMD
    "-C", "target-feature=+fp-armv8",  # ARMv8 FP
    "-C", "target-feature=+crc",       # Hardware CRC
    "-C", "target-feature=+lse",       # Large System Extensions
    "-C", "target-feature=+rdm",       # Round Double Multiply
    "-C", "target-feature=+fp16",      # Half-precision FP
    "-C", "target-feature=+dotprod",   # Dot Product
    "-C", "target-feature=+flagm",     # Flag manipulation
    "-C", "target-feature=+ssbs",      # Speculative Store Bypass Safe
    "-C", "target-feature=+sb",        # Speculation Barrier
    "-C", "link-arg=-march=armv8.5-a",
]

# ============================================================================
# Primary Target Configurations (Used by default)
# ============================================================================

# Default Linux target - uses native CPU detection
[target.aarch64-unknown-linux-gnu]
rustflags = [
    "-C", "target-cpu=native",           # Auto-detect best CPU features
    "-C", "target-feature=+neon",        # NEON SIMD (always available)
    "-C", "target-feature=+fp-armv8",    # ARMv8 FP
    "-C", "target-feature=+crc",        # Hardware CRC
    "-C", "target-feature=+crypto",     # Crypto instructions
    "-C", "target-feature=+lse",        # Large System Extensions
    "-C", "link-arg=-march=armv8-a+neon+crypto",
]

# Apple Silicon (iOS/macOS) - uses native detection
[target.aarch64-apple-darwin]
rustflags = [
    "-C", "target-cpu=native",           # Auto-detect Apple Silicon features
    "-C", "target-feature=+neon",        # NEON SIMD (always available)
    "-C", "target-feature=+fp-armv8",    # ARMv8 FP
    "-C", "target-feature=+crypto",     # Crypto instructions
]

# Android - targeted at Cortex-A72 (common in high-end devices)
[target.aarch64-linux-android]
rustflags = [
    "-C", "target-cpu=cortex-a72",       # ARMv8.1-A (common high-end Android)
    "-C", "target-feature=+neon",        # NEON SIMD (always available)
    "-C", "target-feature=+fp-armv8",    # ARMv8 FP
    "-C", "target-feature=+crypto",     # Crypto instructions
    "-C", "link-arg=-march=armv8.1-a+neon+crypto",
]

# ============================================================================
# Android GMP/MPFR/MPC Configuration
# ============================================================================
# Force Android builds to use our pre-built GMP libraries

[target.aarch64-linux-android]
rustflags = [
    "-C", "target-cpu=cortex-a72",
    "-C", "target-feature=+neon",
    "-C", "target-feature=+fp-armv8",
    "-C", "target-feature=+crypto",
    "-C", "link-arg=-march=armv8.1-a+neon+crypto",
    "-L", "${CARGO_MANIFEST_DIR}/libs/android/aarch64-linux-android/lib",
    "-l", "static=gmp",
    "-l", "static=mpfr",
    "-l", "static=mpc"
]

[target.armv7a-linux-androideabi]
rustflags = [
    "-L", "${CARGO_MANIFEST_DIR}/libs/android/armv7a-linux-androideabi/lib",
    "-l", "static=gmp",
    "-l", "static=mpfr",
    "-l", "static=mpc"
]

[target.i686-linux-android]
rustflags = [
    "-L", "${CARGO_MANIFEST_DIR}/libs/android/i686-linux-android/lib",
    "-l", "static=gmp",
    "-l", "static=mpfr",
    "-l", "static=mpc"
]

[target.x86_64-linux-android]
rustflags = [
    "-L", "${CARGO_MANIFEST_DIR}/libs/android/x86_64-linux-android/lib",
    "-l", "static=gmp",
    "-l", "static=mpfr",
    "-l", "static=mpc"
]
EOF
    
    echo "✅ Cargo configuration created with GMP support"
    echo ""
}

# Function to test the setup
test_setup() {
    echo "🧪 Testing GMP library setup..."
    echo ""
    
    # Test environment variables
    if [ -z "$GMP_LIB_DIR" ] || [ -z "$GMP_INCLUDE_DIR" ]; then
        echo "❌ GMP environment variables not set"
        return 1
    fi
    
    if [ ! -d "$GMP_LIB_DIR" ]; then
        echo "❌ GMP library directory does not exist: $GMP_LIB_DIR"
        return 1
    fi
    
    # Test a simple Rust build with precision feature
    echo "🔨 Testing Rust build with precision feature..."
    
    cd rust
    if cargo build --features precision --quiet 2>&1 | grep -q "error"; then
        echo "❌ Rust build failed"
        cd ..
        return 1
    else
        echo "✅ Rust build with precision feature successful"
        cd ..
        return 0
    fi
}

# Main function
main() {
    echo "📋 This script will set up GMP/MPFR/MPC libraries for Kistaverk"
    echo "    It ensures that gmp-mpfr-sys uses our pre-built libraries"
    echo "    instead of attempting to build from source."
    echo ""
    
    # Check if we're in the correct directory
    if [ ! -f "scripts/build_gmp_android.sh" ]; then
        echo "❌ This script must be run from the project root directory"
        echo "    (where scripts/build_gmp_android.sh is located)"
        return 1
    fi
    
    # Check if libraries already exist
    echo "🔍 Checking for existing GMP libraries..."
    echo ""
    
    local all_libraries_present=true
    
    # Check each architecture
    local architectures=(
        "aarch64-linux-android:AARCH64"
        "armv7a-linux-androideabi:ARMv7"
        "i686-linux-android:x86"
        "x86_64-linux-android:x86_64"
    )
    
    for arch_info in "${architectures[@]}"; do
        local arch=$(echo "$arch_info" | cut -d':' -f1)
        local name=$(echo "$arch_info" | cut -d':' -f2)
        
        if [ -d "rust/libs/android/$arch" ]; then
            check_libraries "rust/libs/android/$arch" "$name"
            if [ $? -ne 0 ]; then
                all_libraries_present=false
            fi
        else
            echo "❌ $name libraries directory not found: rust/libs/android/$arch"
            all_libraries_present=false
        fi
        echo ""
    done
    
    # Build libraries if missing
    if [ "$all_libraries_present" = false ]; then
        echo "🛠️  Some GMP libraries are missing. Would you like to build them?"
        echo "    This requires the Android NDK to be installed."
        echo ""
        
        read -p "Build GMP libraries now? [Y/n] " -n 1 -r
        echo ""
        
        if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
            if ! build_libraries; then
                echo "❌ Library build failed. You can try building manually later."
                echo "    Run: ./scripts/build_gmp_android.sh"
                echo ""
            else
                echo "✅ Libraries built successfully"
                echo ""
            fi
        else
            echo "🎭 Library build skipped. You can build them later by running:"
            echo "    ./scripts/build_gmp_android.sh"
            echo ""
        fi
    fi
    
    # Set up environment variables
    setup_environment
    
    # Create Cargo configuration
    create_cargo_config
    
    # Test the setup
    if test_setup; then
        echo "✅ Setup completed successfully!"
        echo ""
        echo "🎯 You can now build with precision support:"
        echo "    cd rust"
        echo "    cargo build --features precision"
        echo ""
        echo "📚 For Android builds, use:"
        echo "    cargo build --target aarch64-linux-android --features precision"
        echo "    cargo build --target armv7a-linux-androideabi --features precision"
        echo "    etc."
    else
        echo "⚠️  Setup completed with warnings. Check the output above."
        echo "    You may need to install additional dependencies."
    fi
    
    echo ""
    echo "📝 Environment variables have been set for your current session."
    echo "    To make them permanent, add these lines to your shell config:"
    echo "    export GMP_LIB_DIR="$PWD/rust/libs/android""
    echo "    export GMP_INCLUDE_DIR="$PWD/rust/libs/android""
    echo "    export GMP_STATIC=1"
    echo "    export GMP_MPFR_SYS_USE_PKG_CONFIG=0"
    echo ""
    
    echo "🎉 Setup complete!"
}

# Run the main function
main