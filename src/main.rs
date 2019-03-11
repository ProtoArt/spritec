mod cel;
mod light;
mod outline;
mod scale;
mod geometry;
mod material;

use std::f32::consts::PI;
use std::path::Path;
use std::rc::Rc;

use euc::{Pipeline, rasterizer, buffer::Buffer2d, Target};
use minifb::{self, Key, KeyRepeat};
use tobj;
use vek::{Mat4, Vec3, Rgba};

use crate::cel::CelShader;
use crate::outline::OutlineShader;
use crate::light::DiffuseLight;
use crate::scale::scale_buffer;
use crate::geometry::Mesh;
use crate::material::Material;

/// Converts an Rgba color to a bgra u32 suitable for use in minifb
#[inline(always)]
fn rgba_to_bgra_u32(Rgba {r, g, b, a}: Rgba<f32>) -> u32 {
    // Truncating conversion to u8 from f32 in range 0.0 to 1.0
    let to_u8 = |x| (x * 255.0) as u8;

    (to_u8(b) as u32) << 0 |
    (to_u8(g) as u32) << 8 |
    (to_u8(r) as u32) << 16 |
    (to_u8(a) as u32) << 24
}

fn main() {
    let width = 32;
    let height = 32;
    let scale = 32;
    let background = rgba_to_bgra_u32(Rgba {r: 0.62, g: 0.62, b: 0.62, a: 1.0});

    let mut color = Buffer2d::new([width, height], background);
    let mut depth = Buffer2d::new([width, height], 1.0);
    // Scaled screen buffer
    let mut screen = Buffer2d::new([width * scale, height * scale], 0);

    let mut win = minifb::Window::new(
        "Test Project",
        width * scale,
        height * scale,
        minifb::WindowOptions::default()
    ).unwrap();

    let (meshes, materials) = tobj::load_obj(&Path::new("samples/bigboi.obj")).unwrap();
    let materials: Vec<_> = materials.into_iter().map(|mat| Rc::new(Material::from(mat))).collect();
    let meshes: Vec<_> = meshes.into_iter().map(|model| Mesh::new(model.mesh, &materials)).collect();

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
        let projection = Mat4::perspective_rh_no(0.3*PI, (width as f32)/(height as f32), 0.01, 100.0)
            * Mat4::<f32>::scaling_3d(0.50);

        // Must be multiplied backwards since each point to be multiplied will be on the right
        let mvp = projection * view * model;

        color.clear(background);
        depth.clear(1.0);

        for mesh in &meshes {
            OutlineShader {
                mvp,

                mesh,

                outline_color: Rgba::black(),
                outline_thickness: 0.15,
            }.draw::<rasterizer::Triangles<_>, _>(mesh.indices(), &mut color, &mut depth);

            CelShader {
                mvp,
                model_inverse_transpose: model.inverted().transposed(),

                mesh,

                light: DiffuseLight {
                    direction: Vec3 {x: 1.0, y: 0.0, z: 0.0},
                    color: Rgba::white(),
                    intensity: 1.0,
                },

                ambient_intensity: 0.5,
            }.draw::<rasterizer::Triangles<_>, _>(mesh.indices(), &mut color, &mut depth);
        }

        scale_buffer(&mut screen, &color);

        if win.is_open() && !win.is_key_pressed(Key::Escape, KeyRepeat::No) {
            win.update_with_buffer(screen.as_ref()).unwrap();
        } else {
            break;
        }
    }
}
