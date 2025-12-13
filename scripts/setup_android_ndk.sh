#!/bin/bash

# Android NDK Setup Helper for Kistaverk
# This script helps configure the Android NDK environment for building Android targets

set -e  # Exit on error

echo "🚀 Android NDK Setup Helper for Kistaverk"
echo "========================================"
echo ""

# Function to detect NDK installations
 detect_ndk_installations() {
    echo "🔍 Searching for Android NDK installations..."
    echo ""
    
    found_ndks=()
    ndk_paths=()
    
    # Common NDK installation locations
    local search_paths=(
        "$ANDROID_NDK_HOME"
        "$ANDROID_HOME/ndk/*"
        "$HOME/Android/Sdk/ndk/*"
        "$HOME/android-ndk-*"
        "$HOME/android-ndk"
        "/usr/local/android-ndk-*"
        "/usr/local/android-ndk"
        "/opt/android-ndk-*"
        "/opt/android-ndk"
        "/usr/lib/android-ndk-*"
        "/usr/lib/android-ndk"
    )
    
    # Check if ANDROID_NDK_HOME is already set
    if [ -n "$ANDROID_NDK_HOME" ] && [ -d "$ANDROID_NDK_HOME" ]; then
        echo "✅ Found NDK via ANDROID_NDK_HOME: $ANDROID_NDK_HOME"
        found_ndks+=("$ANDROID_NDK_HOME")
        ndk_paths+=("$ANDROID_NDK_HOME")
    fi
    
    # Check if ANDROID_HOME is set and look for NDK under it
    if [ -n "$ANDROID_HOME" ] && [ -d "$ANDROID_HOME" ]; then
        echo "📂 Checking ANDROID_HOME for NDK installations: $ANDROID_HOME"
        if [ -d "$ANDROID_HOME/ndk" ]; then
            # Look for NDK versions under ANDROID_HOME/ndk
            for ndk_dir in "$ANDROID_HOME"/ndk/*; do
                if [ -d "$ndk_dir" ]; then
                    echo "✅ Found NDK via ANDROID_HOME: $ndk_dir"
                    found_ndks+=("$ndk_dir")
                    ndk_paths+=("$ndk_dir")
                fi
            done
        fi
    fi
    
    # Search common installation paths
    for path in "${search_paths[@]}"; do
        # Skip empty paths
        if [ -z "$path" ]; then
            continue
        fi
        
        # Expand wildcards
        if [[ "$path" == *"*"* ]]; then
            for expanded_path in $path; do
                if [ -d "$expanded_path" ]; then
                    # Check if it looks like an NDK directory
                    if [ -f "$expanded_path/ndk-build" ] || [ -f "$expanded_path/source.properties" ]; then
                        echo "✅ Found NDK: $expanded_path"
                        found_ndks+=("$expanded_path")
                        ndk_paths+=("$expanded_path")
                    fi
                fi
            done
        else
            if [ -d "$path" ]; then
                # Check if it looks like an NDK directory
                if [ -f "$path/ndk-build" ] || [ -f "$path/source.properties" ]; then
                    echo "✅ Found NDK: $path"
                    found_ndks+=("$path")
                    ndk_paths+=("$path")
                fi
            fi
        fi
    done
    
    # Also check common download locations
    echo "🔍 Checking common download locations..."
    if [ -d "$HOME/Downloads" ]; then
        find "$HOME/Downloads" -maxdepth 2 -name "android-ndk-*" -type d 2>/dev/null | while read ndk_dir; do
            if [ -f "$ndk_dir/ndk-build" ] || [ -f "$ndk_dir/source.properties" ]; then
                echo "✅ Found NDK in Downloads: $ndk_dir"
                found_ndks+=("$ndk_dir")
                ndk_paths+=("$ndk_dir")
            fi
        done
    fi
    
    if [ ${#found_ndks[@]} -eq 0 ]; then
        echo "❌ No Android NDK installations found."
        echo ""
        return 1
    else
        echo ""
        echo "📋 Found ${#found_ndks[@]} NDK installation(s):"
        for i in "${!found_ndks[@]}"; do
            echo "  $((i+1)). ${found_ndks[$i]}"
        done
        echo ""
        return 0
    fi
}

# Function to check NDK version
check_ndk_version() {
    local ndk_path="$1"
    
    if [ -f "$ndk_path/source.properties" ]; then
        local version=$(grep "Pkg.Revision" "$ndk_path/source.properties" | cut -d'=' -f2 | tr -d ' ')
        echo "📊 NDK Version: $version"
        
        # Check if version is recent enough (r21+ recommended)
        local major_version=$(echo "$version" | cut -d'.' -f1)
        if [ "$major_version" -ge 21 ]; then
            echo "✅ NDK version is suitable for building \(r21+\)"
            return 0
        else
            echo "⚠️  NDK version might be too old. r21+ is recommended."
            return 1
        fi
    else
        echo "⚠️  Could not determine NDK version"
        return 1
    fi
}

# Function to check required NDK components
check_ndk_components() {
    local ndk_path="$1"
    
    echo "🔧 Checking NDK components..."
    
    local required_tools=(
        "ndk-build"
        "prebuilt/$(uname -m | sed 's/x86_64/x86_64/;s/arm64/aarch64/')-linux-android/bin/clang"
        "prebuilt/$(uname -m | sed 's/x86_64/x86_64/;s/arm64/aarch64/')-linux-android/bin/clang++"
        "prebuilt/$(uname -m | sed 's/x86_64/x86_64/;s/arm64/aarch64/')-linux-android/bin/llvm-ar"
        "prebuilt/$(uname -m | sed 's/x86_64/x86_64/;s/arm64/aarch64/')-linux-android/bin/llvm-ranlib"
    )
    
    local all_found=true
    
    for tool in "${required_tools[@]}"; do
        if [ -f "$ndk_path/$tool" ] || [ -f "$ndk_path/$tool" ]; then
            echo "  ✅ $tool"
        else
            echo "  ❌ $tool \(missing\)"
            all_found=false
        fi
    done
    
    if [ "$all_found" = true ]; then
        echo "✅ All required NDK components found"
        return 0
    else
        echo "⚠️  Some NDK components are missing"
        return 1
    fi
}

# Function to set up environment variables
setup_environment() {
    local ndk_path="$1"
    
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
    
    if [ -n "$shell_config" ]; then
        echo "📝 Adding NDK environment variables to: $shell_config"
        
        # Check if variables are already set
        if grep -q "ANDROID_NDK_HOME" "$shell_config"; then
            echo "⚠️  ANDROID_NDK_HOME is already configured in $shell_config"
            echo "    You may want to update it manually."
        else
            echo "" >> "$shell_config"
            echo "# Android NDK Configuration for Kistaverk" >> "$shell_config"
            echo "export ANDROID_NDK_HOME=\"$ndk_path\"" >> "$shell_config"
            echo "export PATH=\"\$ANDROID_NDK_HOME:\$PATH\"" >> "$shell_config"
            echo "" >> "$shell_config"
            echo "✅ Environment variables added to $shell_config"
        fi
        
        echo ""
        echo "💡 To apply the changes, run:"
        echo "    source $shell_config"
        echo ""
        
        # Also provide immediate export for current session
        export ANDROID_NDK_HOME="$ndk_path"
        export PATH="$ndk_path:$PATH"
        
        echo "🔄 Environment variables set for current session:"
        echo "    ANDROID_NDK_HOME=$ANDROID_NDK_HOME"
        echo ""
    else
        echo "⚠️  Could not determine shell configuration file."
        echo "    Please manually add these variables to your shell config:"
        echo "    export ANDROID_NDK_HOME="$ndk_path""
        echo "    export PATH="$ndk_path:\$PATH""
        echo ""
        
        # Set for current session
        export ANDROID_NDK_HOME="$ndk_path"
        export PATH="$ndk_path:$PATH"
    fi
}

# Function to provide NDK download instructions
download_instructions() {
    echo "📥 Android NDK Download Instructions"
    echo "==================================="
    echo ""
    
    echo "You can download the Android NDK from these sources:"
    echo ""
    
    echo "1. Official Android NDK Download \(Recommended\):"
    echo "   🌐 https://developer.android.com/ndk/downloads"
    echo ""
    
    echo "2. Via Android Studio:"
    echo "   - Install Android Studio"
    echo "   - Go to Tools > SDK Manager"
    echo "   - Select 'SDK Tools' tab"
    echo "   - Check 'NDK \(Side by side\)' and install"
    echo ""
    
    echo "3. Command line download \(Linux/macOS\):"
    echo "   wget https://dl.google.com/android/repository/android-ndk-r26b-linux.zip"
    echo "   unzip android-ndk-r26b-linux.zip"
    echo "   mv android-ndk-r26b ~/android-ndk"
    echo ""
    
    echo "Recommended NDK Versions:"
    echo "  - r26b \(latest stable\)"
    echo "  - r25c"
    echo "  - r21+ \(minimum required\)"
    echo ""
    
    echo "After downloading, run this setup script again."
}

# Function to test NDK setup
test_ndk_setup() {
    echo "🧪 Testing NDK Setup..."
    echo ""
    
    if [ -z "$ANDROID_NDK_HOME" ]; then
        echo "❌ ANDROID_NDK_HOME is not set"
        return 1
    fi
    
    if [ ! -d "$ANDROID_NDK_HOME" ]; then
        echo "❌ ANDROID_NDK_HOME directory does not exist: $ANDROID_NDK_HOME"
        return 1
    fi
    
    # Test basic NDK functionality
    if [ ! -f "$ANDROID_NDK_HOME/ndk-build" ]; then
        echo "❌ ndk-build not found in NDK directory"
        return 1
    fi
    
    # Test toolchain
    local toolchain_path="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$(uname -m | sed 's/x86_64/x86_64/;s/arm64/aarch64/')-linux-android/bin"
    
    if [ ! -f "$toolchain_path/clang" ]; then
        echo "❌ clang compiler not found in toolchain"
        return 1
    fi
    
    echo "✅ NDK setup appears to be working correctly"
    echo ""
    
    # Test with a simple compilation
    echo "🔨 Testing basic compilation..."
    local test_file=$(mktemp --suffix=.c)
    cat > "$test_file" << 'EOF'
#include <stdio.h>
int main() {
    printf("NDK test successful!\n");
    return 0;
}
EOF
    
    if "$toolchain_path/clang" --target=aarch64-linux-android -o /tmp/ndk_test "$test_file" 2>/dev/null; then
        echo "✅ Basic compilation test passed"
        rm -f /tmp/ndk_test "$test_file"
        return 0
    else
        echo "⚠️  Basic compilation test failed \(this might be expected\)"
        rm -f /tmp/ndk_test "$test_file"
        return 1
    fi
}

# Main script execution
main() {
    # Check if we're running on a supported platform
    local platform=$(uname -s)
    local arch=$(uname -m)
    
    echo "📋 System Information:"
    echo "  Platform: $platform"
    echo "  Architecture: $arch"
    echo ""
    
    if [ "$platform" != "Linux" ] && [ "$platform" != "Darwin" ]; then
        echo "⚠️  This script is designed for Linux and macOS."
        echo "    Windows users should use WSL or install Android Studio."
        echo ""
    fi
    
    # Detect existing NDK installations
    if detect_ndk_installations; then
        echo "🎯 NDK Configuration Options"
        echo "============================"
        echo ""
        
        # If only one NDK found, use it automatically
        if [ ${#found_ndks[@]} -eq 1 ]; then
            local selected_ndk="${found_ndks[0]}"
            echo "🔧 Using the only NDK found: $selected_ndk"
            echo ""
            
            check_ndk_version "$selected_ndk"
            echo ""
            check_ndk_components "$selected_ndk"
            echo ""
            
            read -p "Do you want to set up this NDK for Kistaverk? [Y/n] " -n 1 -r
            echo ""
            if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
                setup_environment "$selected_ndk"
                echo ""
                test_ndk_setup
            else
                echo "🎭 NDK setup cancelled."
            fi
        else
            # Multiple NDKs found, let user choose
            echo "🎯 Multiple NDK installations found. Please select one:"
            echo ""
            
            for i in "${!found_ndks[@]}"; do
                local ndk_path="${found_ndks[$i]}"
                echo "  $((i+1)). $ndk_path"
                
                # Try to get version
                if [ -f "$ndk_path/source.properties" ]; then
                    local version=$(grep "Pkg.Revision" "$ndk_path/source.properties" | cut -d'=' -f2 | tr -d ' ')
                    echo "     Version: $version"
                fi
                echo ""
            done
            
            read -p "Enter your choice \(1-${#found_ndks[@]}\): " choice
            echo ""
            
            if [[ "$choice" =~ ^[0-9]+$ ]] && [ "$choice" -ge 1 ] && [ "$choice" -le ${#found_ndks[@]} ]; then
                local selected_index=$((choice-1))
                local selected_ndk="${found_ndks[$selected_index]}"
                
                echo "🔧 Selected NDK: $selected_ndk"
                echo ""
                
                check_ndk_version "$selected_ndk"
                echo ""
                check_ndk_components "$selected_ndk"
                echo ""
                
                read -p "Do you want to set up this NDK for Kistaverk? [Y/n] " -n 1 -r
                echo ""
                if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
                    setup_environment "$selected_ndk"
                    echo ""
                    test_ndk_setup
                else
                    echo "🎭 NDK setup cancelled."
                fi
            else
                echo "❌ Invalid choice."
            fi
        fi
    else
        # No NDK found
        echo "📥 No Android NDK found on your system."
        echo ""
        
        read -p "Do you want to see download instructions? [Y/n] " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
            download_instructions
        fi
    fi
    
    echo ""
    echo "🎯 Next Steps for Android Development"
    echo "===================================="
    echo ""
    
    if [ -n "$ANDROID_NDK_HOME" ] && [ -d "$ANDROID_NDK_HOME" ]; then
        echo "✅ NDK is configured: $ANDROID_NDK_HOME"
        echo ""
        echo "To build for Android:"
        echo "  1. Activate Android configuration:"
        echo "     ./scripts/use_android_config.sh"
        echo ""
        echo "  2. Build for your target architecture:"
        echo "     cd rust"
        echo "     cargo build --target aarch64-linux-android --features precision"
        echo ""
        echo "  3. When done, restore normal configuration:"
        echo "     ./scripts/restore_original_config.sh"
        echo ""
    else
        echo "⚠️  NDK is not configured yet."
        echo ""
        echo "After installing the NDK:"
        echo "  1. Run this setup script again"
        echo "  2. Follow the prompts to configure your NDK"
        echo "  3. Proceed with Android builds as shown above"
        echo ""
    fi
    
    echo "📚 Additional Resources:"
    echo "  - Android NDK Documentation: https://developer.android.com/ndk/guides"
    echo "  - Kistaverk Android Guide: scripts/ANDROID_BUILD_README.md"
    echo "  - Troubleshooting: ANDROID_GMP_SOLUTION.md"
    echo ""
    
    echo "🎉 Setup complete!"
}

# Run the main function
main