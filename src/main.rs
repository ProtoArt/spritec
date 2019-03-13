mod cel;
mod light;
mod outline;
mod color;
mod scale;
mod geometry;
mod material;

use std::f32::consts::PI;
use std::time::Duration;
use std::path::Path;
use std::thread;
use std::rc::Rc;

use euc::{Pipeline, rasterizer, buffer::Buffer2d, Target};
use minifb::{self, Key, KeyRepeat};
use tobj;
use vek::{Mat4, Vec3, Vec4, Rgba};
use image::ImageBuffer;

use crate::cel::CelShader;
use crate::outline::OutlineShader;
use crate::light::DiffuseLight;
use crate::scale::scale_buffer;
use crate::geometry::Mesh;
use crate::material::Material;
use crate::color::{rgba_to_bgra_u32, bgra_u32_to_rgba, vek_rgba_to_image_rgba};

fn main() {
    let image_width = 64;
    let image_height = 64;

    let load_frame = |filename: &str| {
        let (meshes, materials) = tobj::load_obj(&Path::new(filename)).unwrap();
        let materials: Vec<_> = materials.into_iter().map(|mat| Rc::new(Material::from(mat))).collect();
        let meshes: Vec<_> = meshes.into_iter().map(|model| Mesh::new(model.mesh, &materials)).collect();
        meshes
    };

    let frames: Vec<_> = (1..=8).map(|i| {
        load_frame(&format!("samples/bigboi_rigged_{:06}.obj", i))
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
    preview_window(image_width, image_height, model, view, projection, &frames);
}

fn save_poses(
    image_width: usize,
    image_height: usize,
    model: Mat4<f32>,
    view: Mat4<f32>,
    projection: Mat4<f32>,
    frames: &[Vec<Mesh>],
) {
    let mut color = Buffer2d::new([image_width, image_height], 0);
    let mut depth = Buffer2d::new([image_width, image_height], 1.0);

    for (i, frame) in frames.into_iter().enumerate() {
        render(&mut color, &mut depth, model, view, projection, frame);

        let mut img = ImageBuffer::new(image_width as u32, image_height as u32);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            // Unsafe because we are guaranteeing that these indexes are not out of bounds
            let color = unsafe { color.get([x as usize, y as usize]) };
            let rgba = bgra_u32_to_rgba(*color);
            *pixel = vek_rgba_to_image_rgba(rgba);
        }
        img.save(&format!("pose{:06}.png", i + 1)).expect("unable to write image");

        color.clear(0);
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

    let mut color = Buffer2d::new([image_width, image_height], 0);
    let mut depth = Buffer2d::new([image_width, image_height], 1.0);

    for (i, frame) in frames.into_iter().enumerate() {
        render(&mut color, &mut depth, model, view, projection, frame);

        let column = i % columns;
        let row = i / columns;

        for x in 0..image_width {
            for y in 0..image_height {
                // Unsafe because we are guaranteeing that these indexes are not out of bounds
                let color = unsafe { color.get([x, y]) };
                let rgba = bgra_u32_to_rgba(*color);

                let pixel = img.get_pixel_mut(
                    (column * image_width + x) as u32,
                    (row * image_height + y) as u32,
                );

                *pixel = vek_rgba_to_image_rgba(rgba);
            }
        }

        color.clear(0);
        depth.clear(1.0);
    }

    img.save("spritesheet.png").expect("unable to write image");
}

fn preview_window(
    image_width: usize,
    image_height: usize,
    model: Mat4<f32>,
    view: Mat4<f32>,
    projection: Mat4<f32>,
    frames: &[Vec<Mesh>],
) {
    let scale = 16;
    let background = rgba_to_bgra_u32(Rgba {r: 0.62, g: 0.62, b: 0.62, a: 1.0});

    let mut color = Buffer2d::new([image_width, image_height], background);
    let mut depth = Buffer2d::new([image_width, image_height], 1.0);

    // Scaled screen buffer
    let mut screen = Buffer2d::new([image_width * scale, image_height * scale], 0);

    let mut win = minifb::Window::new(
        "Test Project",
        image_width * scale,
        image_height * scale,
        minifb::WindowOptions::default()
    ).unwrap();

    // Keep the program from ending
    let mut i = 0;
    while win.is_open() && !win.is_key_pressed(Key::Escape, KeyRepeat::No) {
        color.clear(background);
        depth.clear(1.0);

        let meshes = &frames[i];
        render(&mut color, &mut depth, model, view, projection, meshes);

        scale_buffer(&mut screen, &color);

        win.update_with_buffer(screen.as_ref()).unwrap();

        // No need to use 100% CPU for no reason
        thread::sleep(Duration::from_millis(1000 / 10));

        i = (i + 1) % frames.len();
    }
}

fn render(
    color: &mut Buffer2d<u32>,
    depth: &mut Buffer2d<f32>,
    model: Mat4<f32>,
    view: Mat4<f32>,
    projection: Mat4<f32>,
    meshes: &[Mesh],
) {
    // Must be multiplied backwards since each point to be multiplied will be on the right
    let mvp = projection * view * model;

    for mesh in meshes {
        OutlineShader {
            mvp,

            mesh,

            outline_color: Rgba::black(),
            outline_thickness: 0.15,
        }.draw::<rasterizer::Triangles<_>, _>(mesh.indices(), color, depth);

        CelShader {
            mvp,
            model_inverse_transpose: model.inverted().transposed(),

            mesh,

            light: DiffuseLight {
                direction: Vec3::from(view * Vec4::forward_lh()),
                color: Rgba::white(),
                intensity: 1.0,
            },

            ambient_intensity: 0.5,
        }.draw::<rasterizer::Triangles<_>, _>(mesh.indices(), color, depth);
    }
}
