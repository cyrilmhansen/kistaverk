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
