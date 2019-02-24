use std::path::Path;
use euc::{
    Pipeline,
    rasterizer,
    buffer::Buffer2d,
    Target,
};
use minifb::{self, Key, KeyRepeat};
use tobj;
use vek::*;

struct Teapot<'a> {
    mvp: Mat4<f32>,
    positions: &'a [Vec3<f32>],
    normals: &'a [Vec3<f32>],
    light_dir: Vec3<f32>,
}

impl<'a> Pipeline for Teapot<'a> {
    type Vertex = u32; // Vertex index
    type VsOut = Vec3<f32>; // Normal
    type Pixel = u32; // BGRA

    #[inline(always)]
    fn vert(&self, v_index: &Self::Vertex) -> ([f32; 3], Self::VsOut) {
        let v_index = *v_index as usize;
        // Find vertex position
        let v_pos = self.positions[v_index] + Vec3::new(0.0, -0.5, 0.0); // Offset to center the teapot
        (
            // Calculate vertex position in camera space
            Vec3::from(self.mvp * Vec4::from_point(v_pos)).into_array(),
            // Find vertex normal
            self.normals[v_index],
        )
    }

    #[inline(always)]
    fn frag(&self, norm: &Self::VsOut) -> Self::Pixel {
        let ambient = 0.2;
        let diffuse = norm.dot(self.light_dir).max(0.0) * 0.5;
        let specular = self.light_dir.reflected(Vec3::from(self.mvp * Vec4::from(*norm)).normalized()).dot(-Vec3::unit_z()).powf(20.0);

        let light = ambient + diffuse + specular;
        let color = (Rgba::new(1.0, 0.7, 0.1, 1.0) * light).clamped(Rgba::zero(), Rgba::one());

        let bytes = (color * 255.0).map(|e| e as u8).into_array();
        (bytes[2] as u32) << 0 |
        (bytes[1] as u32) << 8 |
        (bytes[0] as u32) << 16 |
        (bytes[3] as u32) << 24
    }
}

const W: usize = 800;
const H: usize = 600;

fn main() {
    let mut color = Buffer2d::new([W, H], 0);
    let mut depth = Buffer2d::new([W, H], 1.0);

    let mut win = minifb::Window::new("Teapot", W, H, minifb::WindowOptions::default()).unwrap();

    let obj = tobj::load_obj(&Path::new("samples/bigboi.obj")).unwrap();
    let indices = &obj.0[0].mesh.indices;
    let positions = obj.0[0].mesh.positions.chunks(3).map(|sl| Vec3::from_slice(sl)).collect::<Vec<_>>();
    let normals = obj.0[0].mesh.normals.chunks(3).map(|sl| Vec3::from_slice(sl)).collect::<Vec<_>>();

    for i in 0.. {
        let mvp =
            Mat4::perspective_rh_no(1.3, 1.35, 0.01, 100.0) *
            Mat4::<f32>::scaling_3d(0.2) *
            Mat4::rotation_x((i as f32 * 0.002).sin() * 8.0) *
            Mat4::rotation_y((i as f32 * 0.004).cos() * 4.0);
            Mat4::rotation_z((i as f32 * 0.008).sin() * 2.0);

        color.clear(0);
        depth.clear(1.0);

        Teapot {
            mvp,
            positions: &positions,
            normals: &normals,
            light_dir: Vec3::new(1.0, 1.0, 1.0).normalized(),
        }
            .draw::<rasterizer::Triangles<_>, _>(
                indices,
                &mut color,
                &mut depth,
            );

            win.update_with_buffer(color.as_ref()).unwrap();
        if win.is_open() && !win.is_key_pressed(Key::Escape, KeyRepeat::No) {
        } else {
            break;
        }
    }
}
