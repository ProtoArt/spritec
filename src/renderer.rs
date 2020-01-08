mod thread_render_context;
mod shader_geometry;
mod render_node;
mod rendered_image;
mod job;
mod light;
mod camera;

mod layout;
mod shader;
mod imageops;

pub use thread_render_context::*;
pub use shader_geometry::*;
pub use render_node::*;
pub use rendered_image::*;
pub use job::*;
pub use light::*;
pub use camera::*;

use vek::{Rgba, Mat4, Vec3, Vec4};
use glium::{Surface, framebuffer::SimpleFrameBuffer};

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
    pub fn clear(&mut self, background: Rgba<f32>) {
        self.target.clear_color_and_depth(background.into_tuple(), 1.0);
    }

    /// Draw the given model with the given parameters
    pub fn render(
        &mut self,
        geometry: &ShaderGeometry,
        view: Mat4<f32>,
        projection: Mat4<f32>,
        outline: &Outline,
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
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        //TODO: Once we have support for lights, light info will come from elsewhere
        let light = DirectionalLight {
            direction: Vec3::from(view * Vec4::up()),
            color: Rgba::white(),
            intensity: 1.0,
        };
        let ambient_intensity = 0.5;

        let ShaderGeometry {indices, positions, normals, material, model_transform} = geometry;
        let model_view = view * (*model_transform);
        let mvp = projection * model_view;
        let model_view_inverse_transpose = model_view.inverted().transposed();

        let cel_uniforms = shader::cel::Cel::from(CelUniforms {
            mvp,
            model_view_inverse_transpose,
            light: &light,
            ambient_intensity,
            material: &*material,
        });

        self.target.draw((positions, normals), indices, &self.shaders.cel,
            &cel_uniforms, &cel_params)?;

        let outline_uniforms = shader::outline::Outline::from(OutlineUniforms {
            mvp,
            outline_thickness: outline.thickness,
            outline_color: outline.color,
        });

        self.target.draw((positions, normals), indices, &self.shaders.outline,
            &outline_uniforms, &outline_params)?;

        Ok(())
    }
}
