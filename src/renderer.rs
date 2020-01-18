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

use glium::{Surface, framebuffer::SimpleFrameBuffer};

use crate::math::{Rgba, Rgb, Mat4, Radians};
use crate::scene::LightType;

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
        view: Mat4,
        projection: Mat4,
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
        let lights = &[
            Light {
                data: LightType::Spot {
                    color: Rgb::green(),
                    intensity: 1.0,
                    range: None,
                    inner_cone_angle: Radians::from_degrees(3.0),
                    outer_cone_angle: Radians::from_degrees(6.0),
                },
                world_transform: Mat4::model_look_at_rh((-5.0, 20.0, 10.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0)),
            },

            Light {
               data: LightType::Point {
                   color: Rgb::white(),
                   intensity: 0.5,
                   range: None,
               },
               world_transform: Mat4::translation_3d((10.0, 5.0, 20.0)),
            },
        ];
        let ambient_light = Rgb::white() * 0.3;

        let ShaderGeometry {indices, positions, normals, material, model_transform} = geometry;
        let model_transform = *model_transform;
        let mvp = projection * view * model_transform;
        let model_inverse_transpose = model_transform.inverted().transposed();

        let cel_uniforms = shader::cel::Cel::from(CelUniforms {
            mvp,
            model_transform,
            model_inverse_transpose,
            lights,
            ambient_light,
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
