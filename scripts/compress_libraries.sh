#!/bin/bash

# Script to compress native libraries using UPX
# This should be run after the libraries are built

set -e

echo "Starting library compression..."

# Check if UPX is available
if ! command -v upx &> /dev/null; then
    echo "UPX not found. Please install UPX first."
    echo "You can download it from: https://upx.github.io/"
    exit 1
fi

# Find all .so files in the jniLibs directory
LIB_DIR="./app/app/src/main/jniLibs"

if [ ! -d "$LIB_DIR" ]; then
    echo "jniLibs directory not found: $LIB_DIR"
    exit 1
fi

# Try to compress each library
for lib in $(find "$LIB_DIR" -name "*.so"); do
    echo "Processing $lib..."
    
    # Try to compress with UPX
    if upx --best "$lib"; then
        # Get original and compressed sizes
        original_size=$(stat -c%s "$lib")
        compressed_size=$(stat -c%s "$lib")
        
        # Calculate compression ratio
        if [ $original_size -gt 0 ]; then
            ratio=$(echo "scale=2; $compressed_size * 100 / $original_size" | bc)
            echo "Compressed $lib from $original_size bytes to $compressed_size bytes ($ratio%)"
        else
            echo "Compressed $lib"
        fi
    else
        echo "Failed to compress $lib - it may not be compatible with UPX"
        echo "This is normal for libraries that don't have a DT_INIT section"
    fi
done

echo "Library compression complete."
