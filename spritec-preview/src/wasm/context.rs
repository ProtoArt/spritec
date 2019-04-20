use std::f32::consts::PI;

use euc::{Target, buffer::Buffer2d};
use vek::{Rgba, Mat4};

use crate::context::Context;

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
unsafe extern fn image_data(ctx_ptr: *mut Context) -> *const u8 {
    let ctx = Context::from_raw_leak(ctx_ptr);
    ctx.image_data().as_ptr()
}

/// Performs a render and returns a new pointer to the context in case the previous one has been
/// invalidated by any allocations. If the pointer has been changed, all pointers to data within
/// the context are also invalidated.
#[no_mangle]
unsafe extern fn context_render(ctx_ptr: *mut Context, rotation: f32) -> *const Context {
    let mut ctx = Box::from_raw(ctx_ptr);

    let mut obj_data: &[u8] = include_bytes!("../../../samples/bigboi/obj/bigboi_rigged_000001.obj");
    let model = spritec::loaders::obj::from_reader(&mut obj_data, |_| {
        let mut mtl_data: &[u8] = include_bytes!("../../../samples/bigboi/obj/bigboi_rigged_000001.mtl");
        tobj::load_mtl_buf(&mut mtl_data)
    })
        .expect("could not read model");

    let mut depth = Buffer2d::new(ctx.image_data().size(), 1.0);
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

    let image_data = ctx.image_data_mut();
    image_data.clear(background);
    spritec::render(image_data, &mut depth, view, projection, &model, 0.15, Rgba::black());

    Box::into_raw(ctx)
}
