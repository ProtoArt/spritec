mod cel;
mod light;

use std::path::Path;
use euc::{Pipeline, rasterizer, buffer::Buffer2d, Target};
use minifb::{self, Key, KeyRepeat};
use tobj;
use vek::{Mat4, Vec3, Rgba};

use crate::cel::CelShader;
use crate::light::DiffuseLight;

fn scale_buffer<T: Clone + Copy>(target: &mut Buffer2d<T>, source: &Buffer2d<T>) {
    let target_size = target.size();
    let source_size = source.size();
    let scale_x = target_size[0] / source_size[0];
    let scale_y = target_size[1] / source_size[1];

    // Check for truncating division
    assert_eq!(source_size[0] * scale_x, target_size[0]);
    assert_eq!(source_size[1] * scale_y, target_size[1]);

    // Blit the pixels with no anti-aliasing
    for i in 0..source_size[0] {
        for j in 0..source_size[1] {
            // Unsafe because we are guaranteeing that these indexes are not out of bounds
            let color = unsafe { *source.get([i, j]) };

            // Copy the color to every pixel in the scaled box
            for sx in 0..scale_x {
                for sy in 0..scale_y {
                    // Unsafe because we are guaranteeing that these indexes are not out of bounds
                    unsafe {
                        target.set([i * scale_x + sx, j * scale_y + sy], color);
                    }
                }
            }
        }
    }
}

fn main() {
    let width = 32;
    let height = 32;
    let scale = 32;

    let mut color = Buffer2d::new([width, height], 0);
    let mut depth = Buffer2d::new([width, height], 1.0);
    // Scaled screen buffer
    let mut screen = Buffer2d::new([width * scale, height * scale], 0);

    let mut win = minifb::Window::new(
        "Test Project",
        width * scale,
        height * scale,
        minifb::WindowOptions::default()
    ).unwrap();

    let obj = tobj::load_obj(&Path::new("samples/bigboi.obj")).unwrap();
    let indices = &obj.0[0].mesh.indices;
    let positions = obj.0[0].mesh.positions.chunks(3).map(|sl| Vec3::from_slice(sl)).collect::<Vec<_>>();
    let normals = obj.0[0].mesh.normals.chunks(3).map(|sl| Vec3::from_slice(sl)).collect::<Vec<_>>();

    for i in 0.. {
        // The transformation that represents the center of the model, all points in the model are
        // relative to this
        // Also known as the "world" transformation
        //
        // Model coordinates -> World coordinates
        let model = Mat4::rotation_x((i as f32 * 0.0004).sin() * 8.0)
            * Mat4::rotation_y((i as f32 * 0.0008).cos() * 4.0)
            * Mat4::rotation_z((i as f32 * 0.0016).sin() * 2.0);
        // The transformation that represents the position and orientation of the camera
        //
        // World coordinates -> Camera coordinates
        let view = Mat4::identity();
        // The perspective/orthographic/etc. projection of the camera
        //
        // Camera coordinates -> Homogenous coordinates
        let projection = Mat4::perspective_rh_no(2.3, 1.00, 0.01, 100.0)
            * Mat4::<f32>::scaling_3d(0.50);;

        // Must be multiplied backwards since each point to be multiplied will be on the right
        let mvp = projection * view * model;

        color.clear(0);
        depth.clear(1.0);

        CelShader {
            mvp,
            model_inverse_transpose: model.inverted().transposed(),

            positions: &positions,
            normals: &normals,

            light: DiffuseLight {
                direction: Vec3 {x: 1.0, y: 0.0, z: 0.0},
                color: Rgba {r: 1.0, g: 1.0, b: 1.0, a: 1.0},
                intensity: 1.0,
            },

            outline_color: Rgba {r: 0.0, g: 0.0, b: 0.0, a: 1.0},
            outline_thickness: 0.03,
        }.draw::<rasterizer::Triangles<_>, _>(indices, &mut color, &mut depth);

        scale_buffer(&mut screen, &color);

        if win.is_open() && !win.is_key_pressed(Key::Escape, KeyRepeat::No) {
            win.update_with_buffer(screen.as_ref()).unwrap();
        } else {
            break;
        }
    }
}
