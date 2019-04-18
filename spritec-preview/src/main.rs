use std::f32::consts::PI;
use std::time::Duration;
use std::thread;

use vek::{Mat4, Vec2, Rgba};
use euc::{buffer::Buffer2d, Target};
use minifb::{Window, WindowOptions, Key, KeyRepeat};
use spritec::{
    render,
    loaders,
    geometry::Mesh,
    color::rgba_to_bgra_u32,
    scale::scale_buffer_map,
};

fn main() {
    let image_width = 64;
    let image_height = 64;

    let frames: Vec<_> = (1..=8).map(|i| {
        loaders::obj::load_file(&format!("samples/bigboi/obj/bigboi_rigged_{:06}.obj", i))
    }).collect();

    // The transformation that represents the center of the model, all points in the model are
    // relative to this
    // Also known as the "world" transformation
    //
    // Model coordinates -> World coordinates
    let model = Mat4::identity();
    // The transformation that represents the position and orientation of the camera
    //
    // World coordinates -> Camera coordinates
    let view = Mat4::rotation_x(PI/8.0) * Mat4::rotation_y(0.0*PI/2.0);
    // The perspective/orthographic/etc. projection of the camera
    //
    // Camera coordinates -> Homogenous coordinates
    let projection = Mat4::perspective_rh_no(0.8*PI, (image_width as f32)/(image_height as f32), 0.01, 100.0)
        * Mat4::<f32>::scaling_3d(0.6);

    let scale = 16;
    let background = Rgba {r: 0.62, g: 0.62, b: 0.62, a: 1.0};

    let mut color = Buffer2d::new([image_width, image_height], background);
    let mut depth = Buffer2d::new([image_width, image_height], 1.0);

    // Scaled screen buffer
    let (screen_width, screen_height) = (image_width * scale, image_height * scale);
    let mut screen = Buffer2d::new([screen_width, screen_height], 0);

    let (axis_width, axis_height) = (128, 128);
    let axis = if screen_width > axis_width && screen_height > axis_height {
        let mut axis_color = Buffer2d::new([axis_width, axis_height], background);
        render_axis(&mut axis_color, view);
        Some(axis_color)
    } else { None };

    let mut win = Window::new(
        "Test Project",
        image_width * scale,
        image_height * scale,
        WindowOptions::default()
    ).unwrap();

    // Keep the program from ending
    let mut i = 0;
    while win.is_open() && !win.is_key_pressed(Key::Escape, KeyRepeat::No) {
        color.clear(background);
        depth.clear(1.0);

        let meshes = &frames[i];
        render(&mut color, &mut depth, model, view, projection, meshes, 0.15);

        scale_buffer_map(&mut screen, &color, |color| rgba_to_bgra_u32(color));

        if let Some(axis) = &axis {
            // Unsafe because we are guaranteeing that the provided offset is not out of bounds
            unsafe { copy(&mut screen, &axis, (0, screen_height - axis_height)); }
        }

        win.update_with_buffer(screen.as_ref()).unwrap();

        // No need to use 100% CPU for no reason
        thread::sleep(Duration::from_millis(1000 / 10));

        i = (i + 1) % frames.len();
    }
}

/// Copy the entire source buffer into the given target buffer starting at the given offset
///
/// Unsafe because no bounds checking is performed.
unsafe fn copy(target: &mut Buffer2d<u32>, source: &Buffer2d<Rgba<f32>>, (x, y): (usize, usize)) {
    let Vec2 {x: source_width, y: source_height} = Vec2::from(source.size());

    for i in 0..source_width {
        for j in 0..source_height {
            let value = rgba_to_bgra_u32(*source.get([i, j]));
            target.set([x + i, y + j], value);
        }
    }
}

/// Renders a set of axis that match the orientation of the given view matrix
fn render_axis(
    axis_color: &mut Buffer2d<Rgba<f32>>,
    view: Mat4<f32>,
) {
    // Only want to load this once
    thread_local! {
        static AXIS_MESHES: Vec<Mesh> = loaders::obj::load_file("samples/axis/axis.obj");
    }

    let axis_size = axis_color.size();
    let projection = Mat4::perspective_rh_no(0.35*PI, (axis_size[0] as f32)/(axis_size[1] as f32), 0.01, 100.0)
        * Mat4::<f32>::scaling_3d(0.5);

    let mut depth = Buffer2d::new(axis_color.size(), 1.0);
    AXIS_MESHES.with(|meshes| {
        render(axis_color, &mut depth, Mat4::identity(), view, projection, meshes, 0.0)
    });
}
