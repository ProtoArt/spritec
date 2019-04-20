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
#![deny(unused_must_use)] // Ignoring Result is the source of many common bugs

use std::mem;
use std::ffi::{CStr, CString};
use std::fmt::Debug;
use std::os::raw::{c_char, c_void};
use std::f32::consts::PI;

use euc::{Target, buffer::Buffer2d};
use vek::{Rgba, Mat4};

/// A shared context to store in the web assembly memory that outlives the function it is
/// created in.
#[derive(Debug, Clone)]
struct Context {
    image_data: ImageBuffer,
}

impl Context {
    pub fn new() -> Self {
        Self {
            image_data: ImageBuffer::new(64, 64, 12),
        }
    }

    /// Loads the Context from the given pointer. Leaks the value so it is not dropped.
    pub unsafe fn from_raw_leak(ctx_ptr: *mut Context) -> &'static mut Self {
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

impl Target for ImageBuffer {
    type Item = Rgba<f32>;

    #[inline(always)]
    fn size(&self) -> [usize; 2] {
        [self.width, self.height]
    }

    #[inline(always)]
    unsafe fn set(&mut self, [x, y]: [usize; 2], item: Self::Item) {
        let scale = self.scale;
        for i in 0..scale {
            let col = x * scale + i;
            for j in 0..scale {
                let row = y * scale + j;
                let index = row * RGBA_COMPONENTS * self.width * scale + col * RGBA_COMPONENTS;
                *self.data.get_unchecked_mut(index + 0) = (255.0 * item.r) as u8;
                *self.data.get_unchecked_mut(index + 1) = (255.0 * item.g) as u8;
                *self.data.get_unchecked_mut(index + 2) = (255.0 * item.b) as u8;
                *self.data.get_unchecked_mut(index + 3) = (255.0 * item.a) as u8;
            }
        }
    }

    #[inline(always)]
    unsafe fn get(&self, [x, y]: [usize; 2]) -> Self::Item {
        let scale = self.scale;
        let index = y * scale * RGBA_COMPONENTS * self.width + x * scale * RGBA_COMPONENTS;
        Rgba {
            r: *self.data.get_unchecked(index + 0) as f32 / 255.0,
            g: *self.data.get_unchecked(index + 1) as f32 / 255.0,
            b: *self.data.get_unchecked(index + 2) as f32 / 255.0,
            a: *self.data.get_unchecked(index + 3) as f32 / 255.0,
        }
    }

    fn clear(&mut self, fill: Self::Item) {
        for chunk in self.data.chunks_exact_mut(RGBA_COMPONENTS) {
            chunk[0] = (255.0 * fill.r) as u8;
            chunk[1] = (255.0 * fill.g) as u8;
            chunk[2] = (255.0 * fill.b) as u8;
            chunk[3] = (255.0 * fill.a) as u8;
        }
    }
}

// Functions preovided by JavaScript, to be called by the WebAssembly generated from Rust
extern "C" {
    fn console_log(message: *const c_char);
}

/// Prints a value out to the JavaScript console (for debugging purposes)
#[cfg(debug_assertions)] // Keep this to debug builds
pub fn debug<T: Debug>(value: T) {
    let raw_str = CString::new(format!("{:?}", value)).unwrap().into_raw();
    unsafe { console_log(raw_str) };
}

// In order to work with the memory in WASM, we expose allocation and deallocation methods
#[no_mangle]
extern "C" fn alloc(size: usize) -> *const c_void {
    let buf = Vec::with_capacity(size);
    let ptr = buf.as_ptr();
    // Forget the pointer so that Rust doesn't free the memory we want to give JavaScript
    // access to. (Leaking memory is **safe**, so unsafe { ... } is not necessary.)
    mem::forget(buf);
    ptr as *const c_void
}

#[no_mangle]
unsafe extern "C" fn dealloc(ptr: *mut c_void, cap: usize) {
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
    let _ = Box::from_raw(ctx_ptr);
}

/// Returns a pointer to the image data stored in the context (compatible with Uint8ClampedArray)
#[no_mangle]
unsafe extern fn context_image_data(ctx_ptr: *mut Context) -> *const u8 {
    let ctx = Context::from_raw_leak(ctx_ptr);
    ctx.image_data.as_ptr()
}

/// Performs a render and returns a new pointer to the context in case the previous one has been
/// invalidated by any allocations. If the pointer has been changed, all pointers to data within
/// the context are also invalidated.
#[no_mangle]
unsafe extern fn context_render(ctx_ptr: *mut Context, rotation: f32) -> *const Context {
    let mut ctx = Box::from_raw(ctx_ptr);

    let mut obj_data: &[u8] = include_bytes!("../../samples/bigboi/obj/bigboi_rigged_000001.obj");
    let model = spritec::loaders::obj::from_reader(&mut obj_data, |_| {
        let mut mtl_data: &[u8] = include_bytes!("../../samples/bigboi/obj/bigboi_rigged_000001.mtl");
        tobj::load_mtl_buf(&mut mtl_data)
    })
        .expect("could not read model");

    let mut depth = Buffer2d::new(ctx.image_data.size(), 1.0);
    // The transformation that represents the position and orientation of the camera
    //
    // World coordinates -> Camera coordinates
    let view = Mat4::rotation_x(PI/8.0) * Mat4::rotation_y(rotation*PI/2.0);
    // The perspective/orthographic/etc. projection of the camera
    //
    // Camera coordinates -> Homogenous coordinates
    let projection = Mat4::perspective_rh_no(0.8*PI, 1.0, 0.01, 100.0)
        * Mat4::<f32>::scaling_3d(0.6);
    let background = Rgba {r: 0.62, g: 0.62, b: 0.62, a: 1.0};
    ctx.image_data.clear(background);
    spritec::render(&mut ctx.image_data, &mut depth, view, projection, &model, 0.15, Rgba::black());

    Box::into_raw(ctx)
}
