mod thread_render_context;
mod shader_material;
mod shader_geometry;
mod joint_matrices_texture;
mod render_node;
mod rendered_image;
mod job;
mod light;
mod camera;

mod layout;
mod shader;
mod imageops;

pub use thread_render_context::*;
pub use shader_material::*;
pub use shader_geometry::*;
pub use joint_matrices_texture::*;
pub use render_node::*;
pub use rendered_image::*;
pub use job::*;
pub use light::*;
pub use camera::*;

use std::sync::Arc;

use glium::{Surface, framebuffer::SimpleFrameBuffer, texture::{Texture2d, DepthTexture2d}};

use crate::math::{Rgba, Rgb, Mat4};

use shader::cel::CelUniforms;
use shader::outline::OutlineUniforms;

/// A renderer that allows you to draw models
pub struct Renderer<'a> {
    // Kept here to allow us to lazily upload geometry to the GPU even while rendering
    display: &'a Display,
    shaders: &'a Shaders,
    target: SimpleFrameBuffer<'a>,
}

impl<'a> Renderer<'a> {
    /// Returns the display being drawn on by this renderer
    pub fn display(&self) -> &Display {
        &self.display
    }

    /// Clears the screen and resets the depth buffer
    pub fn clear(&mut self, background: Rgba) {
        self.target.clear_color_and_depth(background.into_tuple(), 1.0);
    }

    /// Draw the given model with the given parameters
    pub fn render(
        &mut self,
        geometry: &ShaderGeometry,
        lights: &[Arc<Light>],
        ambient_light: Rgb,
        view: Mat4,
        projection: Mat4,
        outline: &Outline,
        color_texture: &Texture2d,
        depth_texture: &DepthTexture2d,
    ) -> Result<(), glium::DrawError> {
        let cel_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            // Not enabling backface culling for now because we do not know if the meshes are
            // closed or not. See the last part of the tutorial below:
            // https://github.com/glium/glium/blob/125be3580ccfb4e3924005aa5b092069c050a922/book/tuto-11-backface-culling.md#backface-culling-in-glium
            ..Default::default()
        };
        let outline_params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            // Enabling backface culling, but flipping the test so that *only* the back faces will
            // be rendered. Without this, the slightly larger outline mesh would always render over
            // the regular cel shaded mesh.
            // backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        let ShaderGeometry {
            indices,
            positions,
            normals,
            tex_coords,
            joint_influences,
            joint_weights,
            joint_matrices,
            material,
            model_transform,
        } = geometry;

        let model_transform = *model_transform;
        let mvp = projection * view * model_transform;
        let model_inverse_transpose = model_transform.inverted().transposed();

        let cel_uniforms = shader::cel::Cel::from(CelUniforms {
            mvp,
            model_transform,
            model_inverse_transpose,
            joint_matrices,
            lights,
            ambient_light,
            material,
        });

        self.target.draw((positions, normals, tex_coords, joint_influences, joint_weights), indices,
            &self.shaders.cel, &cel_uniforms, &cel_params)?;

         let outline_uniforms = glium::uniform! {
             tex: color_texture.sampled().minify_filter(glium::uniforms::MinifySamplerFilter::Nearest),
             depth_tex:depth_texture.sampled().minify_filter(glium::uniforms::MinifySamplerFilter::Nearest),
             // TODO: hard code for now, only works for bigboi
             near_plane: 0.1 as f32,
             far_plane: 1000.0 as f32,
         };

        // Screen Triangle
        // No need to use vertices, just pass in a null layout and specifying the number of vertices
        // it represents. It's a cool graphics trick that saves performance
        let screen_triangle_indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        // self.target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0),1.0);
        self.target.draw(
            glium::vertex::EmptyVertexAttributes { len: 3 },
            &screen_triangle_indices,
            &self.shaders.outline,
            &outline_uniforms,
            &Default::default())?;

        Ok(())
    }
}
