use std::ffi::CString;
use std::os::raw::c_char;

/// For reading a NULL-terminated string from a WASM memory pointer and taking ownership of the
/// memory.
pub unsafe fn ptr_to_str(ptr: *mut c_char) -> String {
    CString::from_raw(ptr).into_string().expect("Unable to convert character ptr to valid UTF-8")
}

/// Specifically for allocating NULL-terminated strings without knowing their length in advance.
/// It is undefined behaviour to call CString::from_raw on a pointer that wasn't obtained by
/// calling `into_raw`. Hence why we need both `alloc` and `alloc_str`.
///
/// The `size` argument must *not* include the NULL-terminator at the end. The allocated memory
/// will already be NULL-terminated. The actual size of the allocation from this function is
/// thus `size + 1`.
///
/// No guarantees are made about the contents of the memory allocated. Do not read from it without
/// initializing it first.
///
/// See `CString::into_raw` for more information:
/// https://doc.rust-lang.org/std/ffi/struct.CString.html#method.into_raw
#[no_mangle]
unsafe extern fn alloc_str(size: usize) -> *mut c_char {
    CString::from_vec_unchecked(vec![0; size]).into_raw()
}

/// Specifically for deallocating NULL-terminated strings without knowing their length in advance
///
/// See `CString::from_raw` for more information:
/// https://doc.rust-lang.org/std/ffi/struct.CString.html#method.from_raw
#[no_mangle]
pub unsafe extern fn dealloc_str(ptr: *mut c_char) {
    let _ = CString::from_raw(ptr);
}
