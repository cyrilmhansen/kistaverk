fn main() {
    // Set environment variable for Symbolica hobbyist license compliance
    // Note: Symbolica is not currently used due to compilation complexity in mobile environment
    // println!("cargo:rustc-env=SYMBOLICA_SINGLE_CORE=1");

    // Ensure DT_INIT points to our _init shim for UPX (Android targets only)
    if std::env::var("CARGO_CFG_TARGET_OS").map(|v| v == "android").unwrap_or(false) {
        println!("cargo:rustc-link-arg=-Wl,-init=_init");
        // Link the static C++ runtime (libc++_static.a) to satisfy dependencies (e.g., GMP/MIR)
        // that require C++ symbols like __cxa_pure_virtual.
        println!("cargo:rustc-link-lib=c++_static");
    }

    // Re-run if env vars change
    println!("cargo:rerun-if-env-changed=GMP_LIB_DIR");
    println!("cargo:rerun-if-env-changed=GMP_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=GMP_STATIC");
    println!("cargo:rerun-if-env-changed=MPFR_LIB_DIR");
    println!("cargo:rerun-if-env-changed=MPFR_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=MPFR_STATIC");
    println!("cargo:rerun-if-env-changed=MPC_LIB_DIR");
    println!("cargo:rerun-if-env-changed=MPC_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=MPC_STATIC");
}
