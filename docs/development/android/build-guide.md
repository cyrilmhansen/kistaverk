# Android Build Guide

This guide provides step-by-step instructions for setting up the Android development environment and building kistaverk for Android.

## 🛠️ Prerequisites

### System Requirements

- **Operating System**: Linux (Ubuntu 20.04+ recommended) or macOS
- **RAM**: 8GB minimum, 16GB recommended
- **Disk Space**: 20GB minimum, 50GB recommended
- **CPU**: x86_64 with virtualization support

### Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| Java JDK | 11+ | Android development |
| Android Studio | Latest | Android IDE |
| Android NDK | 29+ | Native development |
| Rust | 1.70+ | Rust toolchain |
| Cargo | Latest | Rust package manager |
| Gradle | 8.0+ | Build system |
| CMake | 3.22+ | Build system |

## 🚀 Setup Instructions

### 1. Install Java JDK

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install openjdk-11-jdk

# macOS (using Homebrew)
brew install openjdk@11
```

Verify installation:
```bash
java -version
javac -version
```

### 2. Install Android Studio

Download from: https://developer.android.com/studio

Install Android SDK components:
- Android SDK Platform
- Android SDK Build-Tools
- Android SDK Command-line Tools
- Android Emulator (optional)

### 3. Install Android NDK

```bash
# Through Android Studio SDK Manager
# Or manually download from: https://developer.android.com/ndk/downloads

# Set environment variables
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/29.0.14206865
export ANDROID_HOME=$HOME/Android/Sdk
```

### 4. Install Rust Toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android
```

### 5. Install Additional Tools

```bash
# Ubuntu/Debian
sudo apt install cmake ninja-build pkg-config libssl-dev

# macOS
brew install cmake ninja pkg-config openssl
```

## 📁 Project Structure

```
kistaverk/
├── app/                    # Android application
│   ├── build.gradle.kts     # Android build configuration
│   └── src/                 # Android source code
├── rust/                   # Rust core library
│   ├── Cargo.toml           # Rust dependencies
│   ├── build.rs             # Rust build script
│   └── src/                 # Rust source code
└── scripts/                 # Build and setup scripts
```

## 🔧 Build Configuration

### Android Gradle Configuration

```kotlin
// app/build.gradle.kts
android {
    compileSdk = 36
    
    defaultConfig {
        minSdk = 26
        targetSdk = 36
    }
    
    buildTypes {
        release {
            isMinifyEnabled = false
            isShrinkResources = false
            ndk {
                debugSymbolLevel = "SYMBOL_TABLE"
            }
        }
        debug {
            ndk {
                debugSymbolLevel = "FULL"
            }
        }
    }
    
}
```

### Rust Configuration

```toml
# rust/Cargo.toml
[lib]
name = "kistaverk_core"
crate-type = ["cdylib"]

[target.'cfg(target_os = "android")'.dependencies]
android_log-sys = "0.3"
```
### Audio Backend

The synthesizer uses Android's AAudio backend (API 26+).
- **Requirement**: Set `minSdk = 26` and ensure cargo-ndk targets platform 26.
- **Configuration**: handled in `app/app/build.gradle.kts` by passing `cargo ndk -P 26`.

## 🏗️ Building the Project

### Build Steps

```bash
# 1. Navigate to project directory
cd kistaverk

# 2. Build Rust library for Android
cd rust
cargo ndk -t arm64-v8a -P 26 -o ../app/app/src/main/jniLibs build --release

# 3. Build Android application
cd ../app
./gradlew assembleDebug

# 4. Install on device/emulator
./gradlew installDebug
```

### Build Variants

| Variant | Command | Purpose |
|---------|---------|---------|
| Debug | `assembleDebug` | Development with debugging |
| Release | `assembleRelease` | Production build |
| Benchmark | `assembleBenchmark` | Performance testing |

### Multi-architecture Builds

```bash
# Build for all Android architectures
cargo ndk -t arm64-v8a -P 26 -o ../app/app/src/main/jniLibs build --release
cargo ndk -t armeabi-v7a -P 26 -o ../app/app/src/main/jniLibs build --release
cargo build --target i686-linux-android --release
cargo build --target x86_64-linux-android --release
```

## 🐞 Common Build Issues

### Issue: NDK not found

**Error**: `Android NDK not found`

**Solution**:
```bash
export ANDROID_NDK_HOME=/path/to/ndk
# Or set in local.properties
echo "ndk.dir=/path/to/ndk" > app/local.properties
```

### Issue: Rust target not installed

**Error**: `target not installed: aarch64-linux-android`

**Solution**:
```bash
rustup target add aarch64-linux-android
```

### Issue: Linker errors

**Error**: `linker 'cc' not found`

**Solution**:
```bash
# Install NDK toolchain
$ANDROID_NDK_HOME/build/tools/make_standalone_toolchain.py \
    --arch arm64 --api 24 --install-dir /tmp/android-toolchain
```

### Issue: Missing C++ libraries

**Error**: `c++_shared not found`

**Solution**:
```bash
# Install C++ support in Android Studio SDK Manager
# Or manually:
$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager "ndk;25.2.9519653"
```

## 🔧 Advanced Build Configuration

### Custom Build Scripts

```bash
# scripts/build_android.sh
#!/bin/bash
set -e

export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/25.2.9519653
export ANDROID_HOME=$HOME/Android/Sdk

# Build Rust library
cd rust
cargo build --target aarch64-linux-android --release

# Build Android app
cd ../app
./gradlew assembleRelease

echo "Build completed successfully!"
```

### Build Optimization

```toml
# rust/Cargo.toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = "fat"          # Link-time optimization
codegen-units = 1    # Better optimization
panic = "abort"      # Smaller binaries
strip = "symbols"    # Strip symbols
```

## 📦 Deployment

### Device Deployment

```bash
# Install on connected device
./gradlew installDebug

# Run on specific device
./gradlew installDebug -Pandroid.serial=device_id
```

### Emulator Deployment

```bash
# Create AVD (Android Virtual Device)
$ANDROID_HOME/cmdline-tools/latest/bin/avdmanager \
    create avd -n test_device -k "system-images;android-34;google_apis;arm64-v8a"

# Start emulator
$ANDROID_HOME/emulator/emulator -avd test_device

# Install app
./gradlew installDebug
```

## 🧪 Testing

### Unit Tests

```bash
# Run Rust tests
cd rust
cargo test

# Run Android tests
cd app
./gradlew test
```

### Integration Tests

```bash
# Run Android instrumented tests
./gradlew connectedAndroidTest
```

### Performance Testing

```bash
# Run benchmark tests
./gradlew assembleBenchmark
./gradlew runBenchmark
```

## 🔄 Continuous Integration

### CI Configuration Example

```yaml
# .github/workflows/android.yml
name: Android CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-22.04
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up JDK
      uses: actions/setup-java@v3
      with:
        java-version: '11'
        distribution: 'temurin'
    
    - name: Set up Android SDK
      uses: android-actions/setup-android@v2
    
    - name: Set up Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: aarch64-linux-android
        override: true
    
    - name: Build Rust library
      run: cd rust && cargo build --target aarch64-linux-android --release
    
    - name: Build Android app
      run: cd app && ./gradlew assembleDebug
    
    - name: Run tests
      run: cd app && ./gradlew test
```

## 📈 Performance Optimization

### Build Size Optimization

```bash
# Analyze APK size
./gradlew :app:bundleRelease
./gradlew :app:analyzeApk

# Use UPX compression (experimental)
upx --best app/build/outputs/apk/release/app-release.apk
```

### Execution Performance

```toml
# rust/Cargo.toml
[profile.android-release]
inherits = "release"
opt-level = 3          # Maximum optimization
lto = "thin"          # Thin LTO for faster builds
debug = 0              # No debug info
strip = "symbols"     # Strip symbols
```

## 🛡️ Security Considerations

### Code Signing

```bash
# Create keystore
keytool -genkey -v -keystore release.keystore \
    -alias kistaverk -keyalg RSA -keysize 2048 -validity 10000

# Configure signing in build.gradle.kts
android {
    signingConfigs {
        release {
            storeFile = file("release.keystore")
            storePassword = System.getenv("KEYSTORE_PASSWORD")
            keyAlias = "kistaverk"
            keyPassword = System.getenv("KEY_PASSWORD")
        }
    }
    
    buildTypes {
        release {
            signingConfig = signingConfigs.release
        }
    }
}
```

### ProGuard Configuration

```pro
# app/proguard-rules.pro
# Basic ProGuard rules for Rust JNI
-keep class * { native <methods>; }
-keepclassmembers class * { native <methods>; }
-keepclassmembers class * { public private *; }

# Keep Rust symbols for debugging
-keep class com.example.kistaverk.** { *; }
```

## 📚 Related Documents

- **[Android Precision Setup](precision-setup.md)** - Precision math configuration
- **[Android Troubleshooting](troubleshooting.md)** - Common issues and solutions
- **[System Architecture](../../architecture/overview.md)** - Overall system architecture
- **[MIR JIT Integration](../../architecture/mir-integration.md)** - MIR integration details

**Last updated:** 2025-12-14
