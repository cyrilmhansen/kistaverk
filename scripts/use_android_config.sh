#!/bin/bash

# Script to set up Android build configuration
# This creates a symlink to use the Android-specific Cargo config

echo "Setting up Android build configuration..."

# Create symlink to Android config
if [ -f "rust/.cargo/config.toml" ]; then
    mv rust/.cargo/config.toml rust/.cargo/config.toml.original
fi

ln -sf rust/.cargo/config-android.toml rust/.cargo/config.toml

echo "✅ Android configuration activated"
echo "You can now build for Android targets:"
echo "  cargo build --target aarch64-linux-android --features precision"
echo "  cargo build --target armv7a-linux-androideabi --features precision"
echo "  etc."

echo "To restore the original configuration, run:"
echo "  ./scripts/restore_original_config.sh"