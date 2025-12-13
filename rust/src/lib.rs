mod features;
mod router;
mod state;
mod ui;

pub use router::*;

// Dummy init function to satisfy UPX compression requirements
#[no_mangle]
pub extern "C" fn _init() {
    // This function is required by UPX for compression
    // It doesn't need to do anything
}
