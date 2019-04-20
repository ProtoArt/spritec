//! The main spritec executable

// TOOL POLICY:
// - We add tools in order to help *us* improve our code
// - If they are not doing that, we can configure them or even elect to remove them
// - No tool is perfect and we are allowed to disagree with its results
// - If the tool warns about something that isn't actually an issue worth caring about, add it to
//   the list below and explain your change in your PR
// - We don't want to litter our code with #[allow] attributes unnecessarily, so try to either
//   globally disable that aspect of the tool or live with it and do what the tool says
// - If we make a mistake and find that one of these lints shouldn't have been added here, we can
//   always remove it later
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

pub mod color;
pub mod config;
pub mod loaders;
pub mod scale;
pub mod shader;
pub mod tasks;
pub mod model;

mod rayon_polyfill;

use euc::{Pipeline, Target, rasterizer, buffer::Buffer2d};
use vek::{Mat4, Vec3, Vec4, Rgba};

use crate::shader::DiffuseLight;
use crate::model::Model;
use crate::shader::{CelShader, OutlineShader};

pub fn render<T: Target<Item=Rgba<f32>>>(
    color: &mut T,
    depth: &mut Buffer2d<f32>,
    view: Mat4<f32>,
    projection: Mat4<f32>,
    model: &Model,
    outline_thickness: f32,
    outline_color: Rgba<f32>,
) {
    for mesh in &model.meshes {
        // The model matrix
        let model = mesh.transform();
        // Must be multiplied backwards since each point to be multiplied will be on the right
        let mvp = projection * view * model;

        OutlineShader {
            mvp,

            mesh,

            outline_color,
            outline_thickness,
        }.draw::<rasterizer::Triangles<_>, _>(mesh.indices(), color, depth);

        CelShader {
            mvp,
            model_inverse_transpose: model.inverted().transposed(),

            mesh,

            light: DiffuseLight {
                direction: Vec3::from(view * Vec4::up()),
                color: Rgba::white(),
                intensity: 1.0,
            },

            ambient_intensity: 0.5,
        }.draw::<rasterizer::Triangles<_>, _>(mesh.indices(), color, depth);
    }
}
