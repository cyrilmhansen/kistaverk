mod features;
mod router;
mod state;
mod ui;

pub use router::*;

// Dummy init function to satisfy UPX compression requirements
#[export_name = "_init"]
pub extern "C" fn init_for_upx() {}

// Keep _init referenced so the linker emits init hooks even with LTO/optimizations
#[used]
#[link_section = ".init_array"]
static INIT_ARRAY: [extern "C" fn(); 1] = [init_for_upx];
