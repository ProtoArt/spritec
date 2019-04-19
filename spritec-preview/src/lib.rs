//! The spritec-preview WASM library

// See TOOL POLICY in src/lib.rs
#![deny(clippy::all)] // Deny clippy warnings when running clippy (used for CI)
#![allow(
    clippy::identity_op,
    clippy::let_and_return,
    clippy::cast_lossless,
    clippy::redundant_closure,
    clippy::len_without_is_empty,
    clippy::large_enum_variant,
)]
#![deny(bare_trait_objects)] // Prefer Box<dyn Trait> over Box<Trait>

/// Any initialization code that needs to happen exactly once
pub fn initialize() {
}

extern {
    fn alert(s: &str);
}

#[no_mangle]
pub unsafe extern fn add(x: i32, y: i32) -> i32 {
    x + y
}

#[no_mangle]
pub unsafe extern fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}
