//! In order to work with the memory in the WASM module, we expose allocation and deallocation
//! methods to JavaScript that can manipulate internal memory.

use std::mem;

/// For reading an exact sized buffer from a WASM memory pointer and taking ownership of the
/// memory. It is assumed that the length and capacity are the same.
pub unsafe fn ptr_to_vec(ptr: *mut u8, len: usize) -> Vec<u8> {
    Vec::from_raw_parts(ptr, len, len)
}

/// Allocates the given `size` of memory using a Vec.
/// It is undefined behaviour to call Vec::from_raw_parts on a pointer that wasn't obtained by
/// calling `into_raw`. Hence why we need both `alloc` and `alloc_str`.
#[no_mangle]
extern fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    // Forget the pointer so that Rust doesn't free the memory we want to give JavaScript
    // access to. (Leaking memory is **safe**, so unsafe { ... } is not necessary.)
    mem::forget(buf);
    ptr
}

/// Frees the given pointer's memory by taking ownership of it
#[no_mangle]
unsafe extern fn dealloc(ptr: *mut u8, cap: usize) {
    // Rust will drop this vector and free the memory
    let _buf = Vec::from_raw_parts(ptr, 0, cap);
}
