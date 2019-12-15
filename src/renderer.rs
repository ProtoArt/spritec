mod thread_render_context;
mod geometry_cache;
mod uniform_map;
mod render_mesh;
mod render_material;
mod render_light;

pub use thread_render_context::ThreadRenderContext;
pub use render_mesh::RenderMeshCreationError;
pub use geometry_cache::{ModelRef, RenderLoaderError};

use vek::{Rgba, Mat4, Vec3, Vec4};
use glium::{Frame, Surface};

use thread_render_context::Shaders;
use geometry_cache::GeometryCache;
use uniform_map::UniformMap;
use render_mesh::RenderMesh;
use render_light::RenderDirectionalLight;

/// A renderer that allows you to draw models
pub struct Renderer<'a> {
    shaders: &'a Shaders,
    geometry: &'a GeometryCache,
    target: Frame,
}

impl<'a> Renderer<'a> {
    pub fn clear(&mut self, background: Rgba<f32>) {
        self.target.clear_color_and_depth(background.into_tuple(), 1.0);
    }

    pub fn render(
        &mut self,
        model: ModelRef,
        view: Mat4<f32>,
        projection: Mat4<f32>,
        outline_thickness: f32,
        outline_color: Rgba<f32>,
    ) -> Result<(), glium::DrawError> {
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                // Not enabling backface culling for now because we do not know if the meshes are
                // closed or not. See the last part of the tutorial below:
                // https://github.com/glium/glium/blob/125be3580ccfb4e3924005aa5b092069c050a922/book/tuto-11-backface-culling.md#backface-culling-in-glium
                ..Default::default()
            },
            ..Default::default()
        };

        // Once we have support for lights, light info will come from elsewhere
        let light = RenderDirectionalLight {
            direction: Vec3::from(view * Vec4::up()),
            color: Rgba::white(),
            intensity: 1.0,
        };
        let ambient_intensity = 0.5;

        let mut cel_uniforms = UniformMap::default();
        cel_uniforms.insert("ambient_intensity", ambient_intensity);
        cel_uniforms.insert_nested("light", light.to_uniforms());

        let mut outline_uniforms = UniformMap::default();
        outline_uniforms.insert("outline_thickness", outline_thickness);
        outline_uniforms.insert("outline_color", outline_color.into_array());

        let model = self.geometry.model(model);
        for mesh in &model.meshes {
            let RenderMesh {indices, positions, normals, material, model_transform} = mesh;
            let model_view = view * (*model_transform);
            let mvp = projection * model_view;
            let model_view_inverse_transpose = model_view.inverted().transposed();

            cel_uniforms.insert("mvp", mvp.into_col_arrays());
            cel_uniforms.insert("model_view_inverse_transpose", model_view_inverse_transpose.into_col_arrays());
            cel_uniforms.insert_nested("material", material.to_uniforms());

            self.target.draw((positions, normals), indices, &self.shaders.cel, &cel_uniforms, &params)?;

            outline_uniforms.insert("mvp", mvp.into_col_arrays());

            self.target.draw((positions, normals), indices, &self.shaders.outline, &outline_uniforms, &params)?;
        }

        Ok(())
    }
}
