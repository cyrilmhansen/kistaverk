#![cfg(test)]

use mir_sys::*;
use std::ffi::{CStr, CString};
use std::ptr;

#[cfg(unix)]
mod allocator {
    use super::*;
    use libc::{
        c_int, c_void, mmap, mprotect, munmap, size_t, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE,
        PROT_EXEC, PROT_READ, PROT_WRITE,
    };
    use std::ptr;

    unsafe extern "C" fn test_mem_map(len: size_t, _user_data: *mut c_void) -> *mut c_void {
        let ptr = mmap(
            ptr::null_mut(),
            len,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        );
        if ptr == MAP_FAILED { ptr::null_mut() } else { ptr }
    }

    unsafe extern "C" fn test_mem_unmap(ptr: *mut c_void, len: size_t, _user_data: *mut c_void) -> c_int {
        munmap(ptr, len)
    }

    unsafe extern "C" fn test_mem_protect(
        ptr: *mut c_void,
        len: size_t,
        prot: MIR_mem_protect_t,
        _user_data: *mut c_void,
    ) -> c_int {
        let native_prot = if prot == MIR_mem_protect_PROT_WRITE_EXEC {
            PROT_READ | PROT_WRITE
        } else if prot == MIR_mem_protect_PROT_READ_EXEC {
            PROT_READ | PROT_EXEC
        } else {
            return -1;
        };
        if mprotect(ptr, len, native_prot) != 0 { -1 } else { 0 }
    }

    pub(super) fn get_test_allocator() -> MIR_code_alloc {
        MIR_code_alloc {
            mem_map: Some(test_mem_map),
            mem_unmap: Some(test_mem_unmap),
            mem_protect: Some(test_mem_protect),
            user_data: ptr::null_mut(),
        }
    }
}

#[test]
fn test_mir_load_from_string_and_exec() {
    unsafe {
        #[cfg(unix)]
        let mut code_alloc = allocator::get_test_allocator();
        #[cfg(unix)]
        let ctx = _MIR_init(ptr::null_mut(), &mut code_alloc);
        #[cfg(not(unix))]
        let ctx = _MIR_init(ptr::null_mut(), ptr::null_mut());

        MIR_gen_init(ctx);
        MIR_gen_set_optimize_level(ctx, 0);

        let mir_source = CString::new(
            r#"
m_calc:   module
          export add_nums
add_nums: func i64, i64:a, i64:b
          local i64:r
          add r, a, b
          ret r
          endfunc
          endmodule
"#,
        )
        .unwrap();

        MIR_scan_string(ctx, mir_source.as_ptr());

        let module_list_ptr = MIR_get_module_list(ctx);
        let module = (*module_list_ptr).tail;
        assert!(!module.is_null(), "Failed to parse module");

        MIR_load_module(ctx, module);

        let target_func_name = CString::new("add_nums").unwrap();
        let mut func_item = (*module).items.head;
        let mut found_func: MIR_item_t = ptr::null_mut();

        while !func_item.is_null() {
            if (*func_item).item_type == MIR_item_type_t_MIR_func_item {
                let name_ptr = MIR_item_name(ctx, func_item);
                let name = CStr::from_ptr(name_ptr);
                if name == target_func_name.as_c_str() {
                    found_func = func_item;
                    break;
                }
            }
            func_item = (*func_item).item_link.next;
        }

        assert!(!found_func.is_null(), "Function 'add_nums' not found in module");

        MIR_link(ctx, Some(MIR_set_gen_interface), None);

        let fun_ptr = MIR_gen(ctx, found_func);
        assert!(!fun_ptr.is_null());

        let rust_func: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(fun_ptr);
        let result = rust_func(100, 50);

        println!("MIR String execution result: {}", result);
        assert_eq!(result, 150);

        MIR_gen_finish(ctx);
        MIR_finish(ctx);
    }
}
