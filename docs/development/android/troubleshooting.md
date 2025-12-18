# Android Troubleshooting Guide

This guide provides solutions to common issues encountered when building and running kistaverk on Android.

## 🐞 Common Build Issues

### Issue: NDK not found

**Error**: `Android NDK not found`

**Symptoms**:
- Build fails with NDK-related errors
- `ANDROID_NDK_HOME` not set

**Solutions**:

1. **Set environment variable**:
   ```bash
   export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/29.0.14206865
   ```

2. **Set in local.properties**:
   ```bash
   echo "ndk.dir=$HOME/Android/Sdk/ndk/29.0.14206865" > app/local.properties
   ```

3. **Install NDK via Android Studio**:
   - Open Android Studio
   - Go to SDK Manager
   - Install NDK (Side by side)

**Verification**:
```bash
ls $ANDROID_NDK_HOME
# Should show NDK directory contents
```

### Issue: Rust target not installed

**Error**: `target not installed: aarch64-linux-android`

**Symptoms**:
- Cargo build fails
- Missing target platform

**Solutions**:

1. **Install target**:
   ```bash
   rustup target add aarch64-linux-android
   rustup target add armv7-linux-androideabi
   rustup target add i686-linux-android
   ```

2. **Verify installation**:
   ```bash
   rustup target list
   ```

**Verification**:
```bash
cargo build --target aarch64-linux-android --release
# Should compile successfully
```

### Issue: Linker errors

**Error**: `linker 'cc' not found`

**Symptoms**:
- Linking fails
- Missing C compiler

**Solutions**:

1. **Install NDK toolchain**:
   ```bash
   $ANDROID_NDK_HOME/build/tools/make_standalone_toolchain.py \
       --arch arm64 --api 26 --install-dir /tmp/android-toolchain
   ```

2. **Set linker path**:
   ```bash
   export PATH=/tmp/android-toolchain/bin:$PATH
   ```

3. **Configure Cargo**:
   ```toml
   # .cargo/config
   [target.aarch64-linux-android]
   linker = "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android26-clang"
   ```

**Verification**:
```bash
which aarch64-linux-android-clang
# Should show path to NDK clang
```

### Issue: AAudio not found at link time

**Error**: `ld.lld: error: unable to find library -laaudio`

**Symptoms**:
- Native build fails during linking
- cargo-ndk uses the default API 21 toolchain

**Solutions**:

1. **Set cargo-ndk platform to API 26**:
   ```bash
   cargo ndk -P 26 -t arm64-v8a -o app/app/src/main/jniLibs build
   ```

2. **Verify Gradle uses platform 26**:
   ```kotlin
   // app/app/build.gradle.kts
   baseArgs.addAll(listOf("ndk", "-P", "26"))
   ```

**Verification**:
```bash
rg -n "cargo ndk -P 26" app/app/build.gradle.kts
```

## 🔧 Runtime Issues

### Issue: App crashes on startup

**Error**: `UnsatisfiedLinkError: dlopen failed`

**Symptoms**:
- App crashes immediately
- Native library loading fails

**Solutions**:

1. **Check library naming**:
   ```bash
   # Ensure library is named correctly
   ls app/src/main/jniLibs/arm64-v8a/
   # Should show libkistaverk_core.so
   ```

2. **Verify ABI compatibility**:
   ```kotlin
   // Check loaded ABI
   val abi = Build.SUPPORTED_ABIS.joinToString(", ")
   Log.d("ABI", "Supported ABIs: $abi")
   ```

3. **Check logcat**:
   ```bash
   adb logcat | grep kistaverk
   ```

**Verification**:
```bash
adb shell pm list packages | grep kistaverk
# Should show package name
```

### Issue: Precision mode crashes

**Error**: `UnsatisfiedLinkError: dlopen failed: cannot locate symbol`

**Symptoms**:
- App works in fast mode
- Crashes when switching to precision mode

**Solutions**:

1. **Verify GMP/MPFR linking**:
   ```rust
   // rust/build.rs
   println!("cargo:rustc-link-lib=static=gmp");
   println!("cargo:rustc-link-lib=static=mpfr");
   println!("cargo:rustc-link-lib=static=mpc");
   ```

2. **Check library presence**:
   ```bash
   ls $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/
   # Should show libgmp.a, libmpfr.a, libmpc.a
   ```

3. **Rebuild with precision**:
   ```bash
   cargo clean
   cargo build --target aarch64-linux-android --release --features precision
   ```

**Verification**:
```bash
nm -D app/src/main/jniLibs/arm64-v8a/libkistaverk_core.so | grep gmp
# Should show GMP symbols
```

### Issue: Missing C++ Runtime Symbols

**Error**: `UnsatisfiedLinkError: ... cannot locate symbol "__cxa_pure_virtual"`, `__gxx_personality_v0` or `_ZTISt12length_error`

**Symptoms**:
- App crashes on startup or when initializing audio
- Error mentions missing `__cxa_pure_virtual`, `__gxx_personality_v0`, or other C++ symbols

**Cause**:
- Rust dependencies (like `cpal`/`oboe`) require C++ runtime symbols.
- Linking `libc++_static` with aggressive stripping (`--gc-sections`) often removes these symbols if Rust code doesn't explicitly use them.

**Solutions**:
1.  **Use a C++ Shim (Recommended)**:
    Create `rust/src/android_glue.cpp` that uses standard C++ features (like exceptions) to force the linker to retain the runtime.
    ```cpp
    #include <exception>
    extern "C" {
        void __kistaverk_ensure_cpp_support() {
            try { throw std::exception(); } catch (...) {}
        }
    }
    ```
    Then compile it in `rust/build.rs` using the `cc` crate:
    ```rust
    cc::Build::new().cpp(true).file("src/android_glue.cpp").compile("android_glue");
    ```

2.  **Ensure `libc++_static` is linked**:
    In `rust/build.rs`:
    ```rust
    println!("cargo:rustc-link-lib=c++_static");
    ```

**Verification**:
```bash
nm -D app/src/main/jniLibs/arm64-v8a/libkistaverk_core.so | grep __cxa_pure_virtual
# Should show the symbol defined (T)
```

### Issue: JNI errors

**Error**: `JNI ERROR (app bug): local reference table overflow`

**Symptoms**:
- App crashes after prolonged use
- JNI reference leaks

**Solutions**:

1. **Check JNI reference management**:
   ```rust
   // Ensure proper reference management
   let env = jni::JNIEnv::from_raw(env_ptr)?;
   let result = env.new_string(result_str)?;
   // Don't forget to delete local references
   ```

2. **Use global references**:
   ```rust
   // For long-lived objects
   let global_ref = env.new_global_ref(local_ref)?;
   ```

3. **Check for leaks**:
   ```bash
   adb logcat | grep "JNI ERROR"
   ```

**Verification**:
```bash
# Monitor JNI references
adb shell dumpsys meminfo com.example.kistaverk | grep jni
```

## 📦 Deployment Issues

### Issue: Installation fails

**Error**: `INSTALL_FAILED_INSUFFICIENT_STORAGE`

**Symptoms**:
- App fails to install
- Device storage full

**Solutions**:

1. **Free up space**:
   ```bash
   adb shell pm clear com.example.kistaverk
   ```

2. **Use smaller APK**:
   ```bash
   # Build with size optimization
   cargo build --target aarch64-linux-android --release --profile android-release-size
   ```

3. **Use App Bundle**:
   ```bash
   ./gradlew bundleRelease
   ```

**Verification**:
```bash
adb shell df -h
# Check available storage
```

### Issue: App not compatible with device

**Error**: `INSTALL_FAILED_CPU_ABI_INCOMPATIBLE`

**Symptoms**:
- App fails to install
- Wrong architecture

**Solutions**:

1. **Check device ABI**:
   ```bash
   adb shell getprop ro.product.cpu.abi
   ```

2. **Build for correct ABI**:
   ```bash
   # For arm64
   cargo build --target aarch64-linux-android --release
   
   # For armv7
   cargo build --target armv7-linux-androideabi --release
   ```

3. **Build universal APK**:
   ```bash
   # Build all architectures
   cargo build --target aarch64-linux-android --release
   cargo build --target armv7-linux-androideabi --release
   cargo build --target i686-linux-android --release
   cargo build --target x86_64-linux-android --release
   ```

**Verification**:
```bash
# Check APK ABIs
unzip -l app/build/outputs/apk/debug/app-debug.apk | grep lib/
```

## 🔧 Debugging Techniques

### Logcat Debugging

```bash
# View all logs
adb logcat

# Filter for kistaverk
adb logcat | grep kistaverk

# Save to file
adb logcat -d > kistaverk.log

# Clear logs
adb logcat -c
```

### Rust Debugging

```rust
// Add logging to Rust code
#[cfg(target_os = "android")]
fn logcat(msg: &str) {
    unsafe {
        let tag = b"kistaverk-rust\0";
        let c_msg = CString::new(msg).unwrap();
        android_log_sys::__android_log_print(
            android_log_sys::LogPriority::INFO as _,
            tag.as_ptr() as *const _,
            b"%s\0".as_ptr() as *const _,
            c_msg.as_ptr(),
        );
    }
}

// Usage
logcat("Starting calculation...");
```

### Android Studio Debugging

1. **Attach debugger**:
   - Run app in debug mode
   - Attach Android Studio debugger
   - Set breakpoints in Kotlin code

2. **Native debugging**:
   - Use LLDB for Rust debugging
   - Set breakpoints in Rust code
   - Inspect native variables

### Performance Profiling

```bash
# CPU profiling
adb shell dumpsys cpuinfo | grep kistaverk

# Memory profiling
adb shell dumpsys meminfo com.example.kistaverk

# GPU profiling
adb shell dumpsys gfxinfo com.example.kistaverk
```

## 📊 Performance Issues

### Issue: Slow startup

**Error**: App takes too long to start

**Symptoms**:
- Long launch time
- ANR (Application Not Responding)

**Solutions**:

1. **Optimize Rust initialization**:
   ```rust
   // Lazy initialization
   static MIR_GLOBAL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
   ```

2. **Reduce JNI calls**:
   ```rust
   // Batch operations
   fn batch_operations(operations: Vec<Operation>) -> Vec<Result>
   ```

3. **Use background threads**:
   ```kotlin
   // Move initialization to background
   viewModelScope.launch(Dispatchers.IO) {
       initializeRustCore()
   }
   ```

**Verification**:
```bash
# Measure startup time
adb shell am start -S -W com.example.kistaverk
```

### Issue: High memory usage

**Error**: App uses too much memory

**Symptoms**:
- App crashes with OOM
- High memory usage in task manager

**Solutions**:

1. **Reduce precision cache**:
   ```rust
   // Limit cache size
   struct PrecisionCache {
       cache: LruCache<String, Number>,
       max_size: usize,
   }
   ```

2. **Free unused resources**:
   ```rust
   // Clean up after operations
   impl Drop for MirScriptingState {
       fn drop(&mut self) {
           // Free MIR context
       }
   }
   ```

3. **Use smaller precision**:
   ```rust
   // Use appropriate precision
   if error < 1e-10 {
       state.precision_bits = 64; // Reduce precision
   }
   ```

**Verification**:
```bash
# Monitor memory usage
adb shell dumpsys meminfo com.example.kistaverk
```

### Issue: High CPU usage

**Error**: App uses too much CPU

**Symptoms**:
- Battery drain
- Device overheating
- Slow UI response

**Solutions**:

1. **Optimize MIR execution**:
   ```rust
   // Cache compiled functions
   struct JITCache {
       cache: HashMap<String, *mut c_void>,
   }
   ```

2. **Throttle calculations**:
   ```kotlin
   // Limit calculation frequency
   fun calculateWithThrottle(expr: String) {
       if (lastCalculationTime + 100 > System.currentTimeMillis()) {
           return
       }
       lastCalculationTime = System.currentTimeMillis()
       // Perform calculation
   }
   ```

3. **Use background threads**:
   ```kotlin
   // Offload to background
   withContext(Dispatchers.Default) {
       performHeavyCalculation()
   }
   ```

**Verification**:
```bash
# Monitor CPU usage
adb shell top -n 1 | grep kistaverk
```

## 🔄 Update and Migration Issues

### Issue: Gradle sync fails

**Error**: `Gradle sync failed`

**Symptoms**:
- Android Studio shows sync errors
- Build.gradle issues

**Solutions**:

1. **Clean and resync**:
   ```bash
   ./gradlew clean
   rm -rf .gradle/
   ```

2. **Update Gradle**:
   ```bash
   # Update Gradle wrapper
   ./gradlew wrapper --gradle-version 8.0
   ```

3. **Check dependencies**:
   ```bash
   # Update dependencies
   ./gradlew dependencies
   ```

**Verification**:
```bash
# Sync successfully
./gradlew assembleDebug
```

### Issue: Rust version mismatch

**Error**: `rustc version mismatch`

**Symptoms**:
- Cargo build fails
- Version compatibility issues

**Solutions**:

1. **Update Rust toolchain**:
   ```bash
   rustup update
   ```

2. **Specify Rust version**:
   ```bash
   rustup install 1.70.0
   rustup default 1.70.0
   ```

3. **Update rust-toolchain file**:
   ```toml
   # rust-toolchain
   [toolchain]
   channel = "1.70.0"
   components = ["rustfmt", "clippy"]
   targets = ["aarch64-linux-android"]
   ```

**Verification**:
```bash
rustc --version
# Should show correct version
```

## 🛡️ Security Issues

### Issue: JNI security vulnerabilities

**Error**: `SecurityException: JNI access violation`

**Symptoms**:
- Security-related crashes
- JNI access denied

**Solutions**:

1. **Use proper JNI security**:
   ```rust
   // Validate all JNI inputs
   fn validate_jni_input(env: &JNIEnv, input: JString) -> Result<String, String> {
       let input_str = env.get_string(input)?;
       if input_str.contains("\0") {
           return Err("Null bytes not allowed".to_string());
       }
       Ok(input_str.into())
   }
   ```

2. **Sandbox JNI operations**:
   ```rust
   // Limit JNI operations
   struct JNISandbox {
       max_string_length: usize,
       allowed_classes: HashSet<String>,
   }
   ```

3. **Use secure JNI practices**:
   ```rust
   // Always check for exceptions
   if let Err(e) = env.exception_check() {
       env.exception_describe();
       env.exception_clear();
       return Err("JNI exception occurred".to_string());
   }
   ```

**Verification**:
```bash
# Check for security issues
adb logcat | grep "SecurityException"
```

### Issue: Native library vulnerabilities

**Error**: `Native code security violation`

**Symptoms**:
- Security warnings
- Native code access issues

**Solutions**:

1. **Use memory-safe Rust**:
   ```rust
   // Rust is memory-safe by default
   // No manual memory management needed
   ```

2. **Validate all inputs**:
   ```rust
   // Validate MIR source code
   fn validate_mir_source(source: &str) -> Result<(), String> {
       if source.contains("\0") {
           return Err("Null bytes not allowed in MIR source".to_string());
       }
       if source.len() > 100000 {
           return Err("MIR source too large".to_string());
       }
       Ok(())
   }
   ```

3. **Limit resource usage**:
   ```rust
   // Set execution limits
   struct ExecutionLimits {
       max_time: Duration,
       max_memory: usize,
       max_instructions: u64,
   }
   ```

**Verification**:
```bash
# Check for memory issues
adb shell dumpsys meminfo com.example.kistaverk | grep native
```

## 🚀 Advanced Troubleshooting

### Custom Build Scripts

```bash
# scripts/debug_build.sh
#!/bin/bash
set -e

export RUST_BACKTRACE=1
export RUST_LOG=debug

export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/25.2.9519653
export ANDROID_HOME=$HOME/Android/Sdk

# Build with debug symbols
cd rust
cargo build --target aarch64-linux-android --debug

# Install debug app
cd ../app
./gradlew installDebug

# Start logcat
adb logcat -c
adb logcat | grep kistaverk
```

### Remote Debugging

```bash
# Connect to device
adb tcpip 5555
adb connect device_ip:5555

# Port forwarding
adb forward tcp:5005 tcp:5005

# Remote debugging
lldb -p attach --name com.example.kistaverk
```

### Performance Analysis

```bash
# CPU profiling
adb shell perfetto --txt -c - -o /data/misc/perfetto-traces/trace
adb pull /data/misc/perfetto-traces/trace

# Memory analysis
adb shell am dumpheap com.example.kistaverk /data/local/tmp/heap.hprof
adb pull /data/local/tmp/heap.hprof
```

## 📚 Related Documents

- **[Android Build Guide](build-guide.md)** - Basic Android build setup
- **[Android Precision Setup](precision-setup.md)** - Precision math configuration
- **[System Architecture](../../architecture/overview.md)** - Overall system architecture
- **[MIR JIT Integration](../../architecture/mir-integration.md)** - MIR integration details

**Last updated:** 2025-12-14
