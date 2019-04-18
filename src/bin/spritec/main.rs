//! The spritec command line interface

mod args;

use std::f32::consts::PI;

use structopt::StructOpt;
use euc::{buffer::Buffer2d, Target};
use image::ImageBuffer;
use vek::{Mat4, Rgba};
use spritec::{
    render,
    loaders,
    geometry::Mesh,
    color::vek_rgba_to_image_rgba,
};

use crate::args::AppArgs;

fn main() {
    let args = AppArgs::from_args();

    let (image_width, image_height) = args.size.into();

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

    save_poses(image_width, image_height, model, view, projection, &frames);
    save_spritesheet(image_width, image_height, model, view, projection, &frames);
}

fn save_poses(
    image_width: usize,
    image_height: usize,
    model: Mat4<f32>,
    view: Mat4<f32>,
    projection: Mat4<f32>,
    frames: &[Vec<Mesh>],
) {
    let background = Rgba {r: 0.0, g: 0.0, b: 0.0, a: 0.0};
    let mut color = Buffer2d::new([image_width, image_height], background);
    let mut depth = Buffer2d::new([image_width, image_height], 1.0);

    for (i, frame) in frames.iter().enumerate() {
        render(&mut color, &mut depth, model, view, projection, frame, 0.15);

        let mut img = ImageBuffer::new(image_width as u32, image_height as u32);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            // Unsafe because we are guaranteeing that these indexes are not out of bounds
            let rgba = unsafe { *color.get([x as usize, y as usize]) };
            *pixel = vek_rgba_to_image_rgba(rgba);
        }
        img.save(&format!("pose{:06}.png", i + 1)).expect("unable to write image");

        color.clear(background);
        depth.clear(1.0);
    }
}

fn save_spritesheet(
    image_width: usize,
    image_height: usize,
    model: Mat4<f32>,
    view: Mat4<f32>,
    projection: Mat4<f32>,
    frames: &[Vec<Mesh>],
) {
    let rows = 2;
    let columns = 4;
    assert!(frames.len() <= rows * columns, "not enough room on spritesheet for all sprites");

    let mut img = ImageBuffer::new((image_width * columns) as u32, (image_height * rows) as u32);

    let background = Rgba {r: 0.0, g: 0.0, b: 0.0, a: 0.0};
    let mut color = Buffer2d::new([image_width, image_height], background);
    let mut depth = Buffer2d::new([image_width, image_height], 1.0);

    for (i, frame) in frames.iter().enumerate() {
        render(&mut color, &mut depth, model, view, projection, frame, 0.15);

        let column = i % columns;
        let row = i / columns;

        for x in 0..image_width {
            for y in 0..image_height {
                // Unsafe because we are guaranteeing that these indexes are not out of bounds
                let rgba = unsafe { *color.get([x, y]) };

                let pixel = img.get_pixel_mut(
                    (column * image_width + x) as u32,
                    (row * image_height + y) as u32,
                );

                *pixel = vek_rgba_to_image_rgba(rgba);
            }
        }

        color.clear(background);
        depth.clear(1.0);
    }

    img.save("spritesheet.png").expect("unable to write image");
}
