fn main() {
    // Set environment variable for Symbolica hobbyist license compliance
    // Note: Symbolica is not currently used due to compilation complexity in mobile environment
    // println!("cargo:rustc-env=SYMBOLICA_SINGLE_CORE=1");

    // Diagnostic: Print GMP environment variables to verify they are being passed correctly
    println!("cargo:warning=Checking GMP environment variables...");
    if let Ok(val) = std::env::var("GMP_LIB_DIR") {
        println!("cargo:warning=GMP_LIB_DIR is set to: {}", val);
    } else {
        println!("cargo:warning=GMP_LIB_DIR is NOT set.");
    }

    if let Ok(val) = std::env::var("GMP_STATIC") {
        println!("cargo:warning=GMP_STATIC is set to: {}", val);
    }
}