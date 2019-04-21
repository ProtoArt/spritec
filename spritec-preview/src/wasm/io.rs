use std::path::Path;
use std::os::raw::c_char;
use std::ffi::CString;

use super::alloc::ptr_to_vec;

// Functions provided by JavaScript, to be called by the WebAssembly generated from Rust
extern {
    /// Reads a file with the given path and stores the length in the given length pointer (big-endian).
    /// Returns a pointer to the location in WASM memory where the file contents were stored.
    fn read_file(path: *const c_char, len_ptr: *mut usize) -> *mut u8;
}

/// Reads an entire file into memory
pub fn read_file_buf(path: impl AsRef<Path>) -> Vec<u8> {
    let path_str = path.as_ref().to_str().expect("Unable to convert path to string");
    // Ownership of this string will be passed to JavaScript. JavaScript will be responsible for
    // freeing the memory.
    let path_ptr = CString::new(path_str).expect("CString::new failed").into_raw();
    let mut data_len = 0usize;

    let data;
    unsafe {
        let data_ptr = read_file(path_ptr, &mut data_len as *mut usize);
        let data_len = usize::from_be(data_len);
        data = ptr_to_vec(data_ptr, data_len);
    }

    data
}
