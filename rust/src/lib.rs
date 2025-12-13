mod features;
mod i18n;
mod router;
mod state;
mod ui;

pub use i18n::*;
pub use router::*;

#[macro_use]
extern crate rust_i18n;

i18n!("locales");

// Dummy init function to satisfy UPX compression requirements
#[cfg(target_os = "android")]
#[export_name = "_init"]
pub extern "C" fn init_for_upx() {}

// Keep _init referenced so the linker emits init hooks even with LTO/optimizations
#[cfg(target_os = "android")]
#[used]
#[link_section = ".init_array"]
static INIT_ARRAY: [extern "C" fn(); 1] = [init_for_upx];
