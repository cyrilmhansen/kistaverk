mod features;
mod i18n;
mod router;
mod state;
mod ui;

pub use i18n::*;
pub use router::*;

#[cfg(test)]
mod mir_tests;

use jni::JNIEnv;
use jni::objects::{JObject, JString};

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

fn mir_version_string() -> String {
    // bindgen converts `#define MIR_API_VERSION 0.2` into a `const f64`.
    format!("{}", mir_sys::MIR_API_VERSION)
}

// Android/NDK: MIR uses `__builtin___clear_cache`, which may lower to a call to `__clear_cache`.
// Some link setups leave this symbol unresolved at runtime, causing `dlopen` to fail.
// Provide an implementation for arm64-v8a so the JIT can flush the instruction cache.
#[cfg(all(target_os = "android", target_arch = "aarch64"))]
#[no_mangle]
pub unsafe extern "C" fn __clear_cache(begin: *mut core::ffi::c_void, end: *mut core::ffi::c_void) {
    let begin = begin as *mut u8;
    let end = end as *mut u8;
    if begin.is_null() || end.is_null() || begin >= end {
        return;
    }

    // AArch64 cache maintenance for self-modifying/JIT code:
    // - Clean D-cache to PoU for modified range (`dc cvau`)
    // - DSB to ensure completion
    // - Invalidate I-cache to PoU for range (`ic ivau`)
    // - DSB + ISB to ensure visibility to instruction fetch
    //
    // Cache line size is typically 64 bytes on Android arm64; using 64 is a pragmatic default.
    const LINE: usize = 64;
    let start = (begin as usize) & !(LINE - 1);
    let end = end as usize;

    let mut ptr = start;
    while ptr < end {
        core::arch::asm!("dc cvau, {}", in(reg) ptr, options(nostack));
        ptr += LINE;
    }
    core::arch::asm!("dsb ish", options(nostack));

    ptr = start;
    while ptr < end {
        core::arch::asm!("ic ivau, {}", in(reg) ptr, options(nostack));
        ptr += LINE;
    }
    core::arch::asm!("dsb ish", "isb", options(nostack));
}

#[no_mangle]
pub extern "system" fn Java_aeska_kistaverk_MainActivity_mirVersion<'local>(
    env: JNIEnv<'local>,
    _this: JObject<'local>,
) -> JString<'local> {
    env.new_string(mir_version_string())
        .expect("Couldn't create java string")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mir_version_string_matches_mir_sys_constant() {
        println!("MIR version: {}", mir_version_string());
        assert_eq!(mir_version_string(), format!("{}", mir_sys::MIR_API_VERSION));
        assert!(!mir_version_string().is_empty());
    }
}
