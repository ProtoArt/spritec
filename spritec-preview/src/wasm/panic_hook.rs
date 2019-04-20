//! Hooks into Rust's panic system so we get nicer error messages when a panic occurs.
//!
//! Based on: https://github.com/rustwasm/console_error_panic_hook

use std::panic;

use log::error;

/// A panic hook for use with
/// [`std::panic::set_hook`](https://doc.rust-lang.org/nightly/std/panic/fn.set_hook.html)
/// that logs panics into
/// [`console.error`](https://developer.mozilla.org/en-US/docs/Web/API/Console/error).
///
/// On non-wasm targets, prints the panic to `stderr`.
pub fn hook(info: &panic::PanicInfo) {
    error!("{}", info);
}

/// Set the `console.error` panic hook the first time this is called. Subsequent
/// invocations do nothing.
#[inline]
pub fn set_once() {
    use std::sync::{ONCE_INIT, Once};
    static SET_HOOK: Once = ONCE_INIT;
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(hook));
    });
}
