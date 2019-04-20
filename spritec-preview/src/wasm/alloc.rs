//! In order to work with the memory in the WASM module, we expose allocation and deallocation
//! methods to JavaScript that can manipulate internal memory.

use std::mem;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

#[no_mangle]
extern fn alloc(size: usize) -> *const c_void {
    let buf = Vec::with_capacity(size);
    let ptr = buf.as_ptr();
    // Forget the pointer so that Rust doesn't free the memory we want to give JavaScript
    // access to. (Leaking memory is **safe**, so unsafe { ... } is not necessary.)
    mem::forget(buf);
    ptr as *const c_void
}

#[no_mangle]
unsafe extern fn dealloc(ptr: *mut c_void, cap: usize) {
    // Rust will drop this vector and free the memory
    let _buf = Vec::from_raw_parts(ptr, 0, cap);
}

/// Specifically for deallocating NULL-terminated strings without knowing their length in advance
///
/// See `CString::into_raw` for more information:
/// https://doc.rust-lang.org/std/ffi/struct.CString.html#method.into_raw
#[no_mangle]
unsafe extern fn dealloc_str(ptr: *mut c_char) {
    let _ = CString::from_raw(ptr);
}
