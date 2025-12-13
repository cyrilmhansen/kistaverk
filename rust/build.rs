fn main() {
    // Set environment variable for Symbolica hobbyist license compliance
    // Note: Symbolica is not currently used due to compilation complexity in mobile environment
    // println!("cargo:rustc-env=SYMBOLICA_SINGLE_CORE=1");

    // Android precision feature integration
    #[cfg(all(target_os = "android", feature = "precision"))]
    {
        use std::env;
        
        // Determine the target architecture for Android
        let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".to_string());
        
        // Map Rust target architectures to our Android library directories
        let lib_path = match target_arch.as_str() {
            "aarch64" => "rust/libs/android/aarch64-linux-android/lib",
            "arm" => "rust/libs/android/armv7a-linux-androideabi/lib",
            "x86" => "rust/libs/android/i686-linux-android/lib",
            "x86_64" => "rust/libs/android/x86_64-linux-android/lib",
            _ => {
                eprintln!("Warning: Unknown Android architecture: {}", target_arch);
                eprintln!("Supported architectures: aarch64, arm, x86, x86_64");
                return;
            }
        };
        
        // Check if the library directory exists
        if std::path::Path::new(lib_path).exists() {
            println!("cargo:rustc-link-search=native={}", lib_path);
            println!("cargo:rustc-link-lib=static=gmp");
            println!("cargo:rustc-link-lib=static=mpfr");
            println!("cargo:rustc-link-lib=static=mpc");
            
            // Also link against Android's libc and libm for math functions
            println!("cargo:rustc-link-lib=c");
            println!("cargo:rustc-link-lib=m");
            
            println!("cargo:warning=Android precision feature enabled - linking against GMP/MPFR/MPC");
        } else {
            eprintln!("Error: Android precision libraries not found in: {}", lib_path);
            eprintln!("Please run scripts/build_gmp_android.sh to build the required libraries");
            panic!("Android precision libraries missing - see error above");
        }
    }
    
    // Non-Android platforms with precision feature just need to ensure rug is available
    #[cfg(all(not(target_os = "android"), feature = "precision"))]
    {
        println!("cargo:warning=Precision feature enabled for non-Android platform");
    }
    
    // When precision feature is disabled, no special linking is needed
    #[cfg(not(feature = "precision"))]
    {
        println!("cargo:warning=Building without precision feature (using f64)");
    }
}