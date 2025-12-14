#![cfg(test)]

use mir_sys::*;
use std::ffi::{CStr, CString};
use std::ptr;
use libc::{self, c_int, c_void};

struct StringReader {
    data: Vec<u8>,
    cursor: usize,
}

unsafe extern "C" fn getc_func(data: *mut c_void) -> c_int {
    let reader = &mut *(data as *mut StringReader);
    if reader.cursor < reader.data.len() {
        let byte = reader.data[reader.cursor];
        reader.cursor += 1;
        byte as c_int
    } else {
        -1 // EOF
    }
}

#[test]
fn test_mir_load_from_string_and_exec() {
    unsafe {
        #[cfg(unix)]
        let mut code_alloc = mir_sys::code_alloc::unix_mmap();
        #[cfg(unix)]
        let ctx = _MIR_init(ptr::null_mut(), &mut code_alloc);
        #[cfg(not(unix))]
        let ctx = _MIR_init(ptr::null_mut(), ptr::null_mut());

        MIR_gen_init(ctx);
        MIR_gen_set_optimize_level(ctx, 2);

        let mir_source = CString::new(r#"
m_calc:   module
          export add_nums
add_nums: func i64, i64:a, i64:b
          local i64:r
          add r, a, b
          ret r
          endfunc
          endmodule
"#).unwrap();

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

#[test]
fn test_c2mir_compile_sieve() {
    unsafe {
        #[cfg(unix)]
        let mut code_alloc = mir_sys::code_alloc::unix_mmap();
        #[cfg(unix)]
        let ctx = _MIR_init(ptr::null_mut(), &mut code_alloc);
        #[cfg(not(unix))]
        let ctx = _MIR_init(ptr::null_mut(), ptr::null_mut());

        MIR_gen_init(ctx);
        MIR_gen_set_optimize_level(ctx, 1);

        c2mir_init(ctx);

        MIR_load_external(
            ctx,
            CString::new("printf").unwrap().as_ptr(),
            libc::printf as *mut c_void,
        );
        MIR_load_external(
            ctx,
            CString::new("abort").unwrap().as_ptr(),
            libc::abort as *mut c_void,
        );

        let c_source = r##"#
void printf (const char *fmt, ...);
void abort (void);
#if defined(_WIN32) || !defined(SIEVE_BENCH)
#define SieveSize 8190
#define Expected 1027
#else
#define SieveSize 819000
#define Expected 65333
#endif
#define N_ITER 1000
int sieve (int n) {
  long i, k, count, iter, prime;
  char flags[SieveSize];

  for (iter = 0; iter < n; iter++) {
    count = 0;
    for (i = 0; i < SieveSize; i++) flags[i] = 1;
    for (i = 2; i < SieveSize; i++)
      if (flags[i]) {
        prime = i + 1;
        for (k = i + prime; k < SieveSize; k += prime) flags[k] = 0;
        count++;
      }
  }
  return count;
}
int main (void) {
  int n = sieve (N_ITER);
  printf ("%d iterations of sieve for %d: result = %d\n", N_ITER, SieveSize, n);
  if (n != Expected) abort ();
  return 0;
}
"##;
        let mut reader = StringReader {
            data: c_source.bytes().collect(),
            cursor: 0,
        };

        let mut options: c2mir_options = std::mem::zeroed();
        
        let result = c2mir_compile(
            ctx,
            &mut options,
            Some(getc_func),
            &mut reader as *mut _ as *mut c_void,
            b"sieve.c\0".as_ptr() as *const _,
            ptr::null_mut(),
        );

        assert_eq!(result, 1, "Compilation of sieve.c failed");

        let module_list = MIR_get_module_list(ctx);
        let module = (*module_list).tail;
        assert!(!module.is_null());

        MIR_load_module(ctx, module);
        MIR_link(ctx, Some(MIR_set_gen_interface), None);

        let target_func_name = CString::new("main").unwrap();
        let mut func_item = (*module).items.head;
        let mut found_func = ptr::null_mut();

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
        assert!(!found_func.is_null(), "Function 'main' not found");

        let fun_ptr = MIR_gen(ctx, found_func);
        assert!(!fun_ptr.is_null());

        let rust_func: extern "C" fn() -> c_int = std::mem::transmute(fun_ptr);
        
        println!("Running sieve...");
        let result_code = rust_func();
        println!("Sieve returned: {}", result_code);
        
        assert_eq!(result_code, 0);

        c2mir_finish(ctx);
        MIR_gen_finish(ctx);
        MIR_finish(ctx);
    }
}
