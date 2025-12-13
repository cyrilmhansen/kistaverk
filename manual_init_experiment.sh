#!/bin/bash

# Experimental script to try adding DT_INIT manually
# This demonstrates the complexity involved

set -e

echo "=== DT_INIT Manual Addition Experiment ==="

LIB_PATH="./app/app/src/main/jniLibs/arm64-v8a/libkistaverk_core.so"
OUTPUT_PATH="/tmp/libkistaverk_with_manual_init.so"

if [ ! -f "$LIB_PATH" ]; then
    echo "Error: Library not found at $LIB_PATH"
    exit 1
fi

echo "1. Analyzing current library structure..."
readelf -d "$LIB_PATH" | grep -E "(INIT|FINI)" || true

echo "2. Checking for existing init symbols..."
nm -D "$LIB_PATH" | grep -i init || true

echo "3. Attempting to understand ELF structure..."
readelf -S "$LIB_PATH" | head -10

echo "4. Looking at dynamic section details..."
readelf -d "$LIB_PATH"

echo ""
echo "=== Challenges Identified ==="
echo "1. No existing _init symbol to reference"
echo "2. No .init section in the library"
echo "3. Dynamic section has no DT_INIT entry"
echo "4. Adding these manually would require:"
echo "   - Creating new executable code section"
echo "   - Adding symbol table entries"
echo "   - Updating dynamic section"
echo "   - Ensuring proper memory alignment"
echo "   - Handling relocations correctly"

echo ""
echo "=== Why This Is Complex ==="
echo "1. ELF Format Requirements:"
echo "   - Sections must be properly aligned"
echo "   - Symbol tables must reference valid addresses"
echo "   - Dynamic section must have correct entries"

echo "2. Android-Specific Issues:"
echo "   - Android uses custom linker (linker64)"
echo "   - Libraries are built with NDK-specific flags"
echo "   - Different memory layout than standard Linux"

echo "3. Tool Limitations:"
echo "   - objcopy doesn't handle ARM64 Android ELF properly"
echo "   - patchelf doesn't support adding DT_INIT"
echo "   - Manual hex editing is error-prone and unsafe"

echo ""
echo "=== Recommended Approach ==="
echo "Instead of manual manipulation, the proper approach is:"
echo ""
echo "1. Add init function to Rust source (already done):"
echo "   #[no_mangle]"
echo "   pub extern \"C\" fn _init() {}"
echo ""
echo "2. Rebuild with Android NDK:"
echo "   cargo build --release --target aarch64-linux-android"
echo ""
echo "3. Apply UPX compression:"
echo "   upx --best --lzma libkistaverk_core.so"
echo ""
echo "This ensures proper ELF structure and reliable behavior."
