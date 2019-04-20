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

/// A shared context to store in the web assembly memory that outlives the function it is
/// created in.
#[derive(Debug, Clone)]
struct Context {
    image_data: ImageBuffer,
}

impl Context {
    pub fn new() -> Self {
        Self {
            image_data: ImageBuffer::new(64, 64, 1),
        }
    }

    /// Loads the Context from the given pointer. Leaks the value so it is not dropped.
    pub unsafe fn from_raw_leak(ctx_ptr: *mut Context) -> &'static Self {
        Box::leak(Box::from_raw(ctx_ptr))
    }
}

/// The number of components in an RGBA value (always 4)
const RGBA_COMPONENTS: usize = 4;

/// An image data buffer (compatible with JavaScript's ImageData)
#[derive(Debug, Clone)]
struct ImageBuffer {
    data: Vec<u8>,
    width: usize,
    height: usize,
    scale: usize,
}

impl ImageBuffer {
    pub fn new(width: usize, height: usize, scale: usize) -> Self {
        Self {
            data: vec![0; RGBA_COMPONENTS * width * scale * height * scale],
            width,
            height,
            scale,
        }
    }

    /// Returns a pointer to the data buffer compatible with JavaScript's Uint8ClampedArray
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
}

/// Create a new context.
///
/// IMPORTANT NOTE: If the allocator re-allocates the web assembly memory, this pointer will
/// become *INVALIDATED*.
#[no_mangle]
extern fn context_new() -> *mut Context {
    let ctx = Box::new(Context::new());
    // Ownership of this value is given to the caller. It is *not* freed at the end of this function.
    Box::into_raw(ctx)
}

/// Delete a previously created context
#[no_mangle]
unsafe extern fn context_delete(ctx_ptr: *mut Context) {
    // Loads the value and then immediately drops it
    Box::from_raw(ctx_ptr);
}

/// Returns a pointer to the image data stored in the context (compatible with Uint8ClampedArray)
#[no_mangle]
unsafe extern fn context_image_data(ctx_ptr: *mut Context) -> *const u8 {
    let ctx = Context::from_raw_leak(ctx_ptr);
    ctx.image_data.as_ptr()
}
