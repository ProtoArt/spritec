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

#[derive(Debug)]
struct DiffuseLight {
    /// The **normalized** direction of the diffuse light being cast on the model
    direction: Vec3<f32>,
    /// The color of the diffuse light
    color: Rgba<f32>,
    /// The intensity of the diffuse light
    intensity: f32,
}

// Initial version of the toon shader is loosely based on this article:
// http://rbwhitaker.wikidot.com/toon-shader
//
// Global assumptions:
// * Color values (red, green, blue, alpha) are all between 0.0 and 1.0
// * Direction vectors are normalized
#[derive(Debug)]
struct ToonShader<'a> {
    // TRANSFORMATIONS

    /// The model-view-projection matrix
    mvp: Mat4<f32>,
    /// The transpose of the inverse of the world transformation, used for transforming the
    /// vertex's normal
    model_inverse_transpose: Mat4<f32>,

    // INPUT TO THE SHADER

    /// The position of each vertex of the model, relative to the model's center
    positions: &'a [Vec3<f32>],
    /// The normal of each vertex of the model
    normals: &'a [Vec3<f32>],

    // DIFFUSE LIGHT PROPERTIES

    light: DiffuseLight,

    // TOON SHADER PROPERTIES

    /// The color for drawing the outline
    outline_color: Rgba<f32>,
    /// The thickness of the outlines. This may need to change, depending on the scale of the
    /// objects you are drawing.
    outline_thickness: f32,

    // TEXTURE PROPERTIES
    //TODO
}

impl<'a> Pipeline for ToonShader<'a> {
    type Vertex = u32; // Vertex index
    type VsOut = Vec3<f32>; // Normal
    type Pixel = u32; // BGRA

    /// The vertex shader that does cel shading.
    ///
    /// It really only does the basic transformation of the vertex location, and normal, and copies
    /// the texture coordinate over.
    #[inline(always)]
    fn vert(&self, v_index: &Self::Vertex) -> ([f32; 3], Self::VsOut) {
        let v_index = *v_index as usize;
        // Find vertex position
        let v_pos = Vec4::from_point(self.positions[v_index]);
        // Calculate vertex position in camera space
        let v_pos_cam = Vec3::from(self.mvp * v_pos).into_array();
        // Find vertex normal
        let v_norm = self.normals[v_index];

        //TODO: Pass along a texture coordinate calculated based on the v_index

        (v_pos_cam, v_norm)
    }

    /// The fragment/pixel shader that does cel shading. Basically, it calculates the color like it
    /// should, and then it discretizes the color into one of four colors.
    #[inline(always)]
    fn frag(&self, norm: &Self::VsOut) -> Self::Pixel {
        // The amount of ambient light to include
        let ambient_intensity = 0.2;

        // Calculate diffuse light amount
        // max() is used to bottom out at zero if the dot product is negative
        let diffuse_intensity = norm.dot(self.light.direction).max(0.0);

        let specular_intensity = self.light.direction
            .reflected(Vec3::from(self.mvp * Vec4::from(*norm)).normalized())
            .dot(-Vec3::unit_z())
            .powf(20.0);

        //TODO: Sample the color from the texture based on the texture coordinate or get it from a
        // material via linear interpolation
        let tex_color = Rgba::new(1.0, 0.7, 0.1, 1.0);

        // Calculate what would normally be the final color, including texturing and diffuse lighting
        let light_intensity = ambient_intensity + diffuse_intensity + specular_intensity;
        let color = tex_color * self.light.intensity;

        // Discretize the intensity, based on a few cutoff points
        let alpha = color.a;
        let mut final_color = match light_intensity {
            intensity if intensity > 0.95 => color,
            intensity if intensity > 0.5 => color * 0.7,
            intensity if intensity > 0.05 => color * 0.35,
            _ => color * 0.1,
        };
        final_color.a = alpha;
        // Clamp the color values between 0.0 and 1.0
        let final_color = final_color.clamped(Rgba::zero(), Rgba::one());

        let bytes = (final_color * 255.0).map(|e| e as u8).into_array();
        (bytes[2] as u32) << 0 |
        (bytes[1] as u32) << 8 |
        (bytes[0] as u32) << 16 |
        (bytes[3] as u32) << 24
    }
}

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

        ToonShader {
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
