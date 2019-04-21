use std::f32::consts::PI;

use spritec::model::Model;
use euc::{Target, buffer::Buffer2d};
use vek::{Rgba, Mat4};

use crate::image_buffer::ImageBuffer;

/// The full state of the renderer. Stored in web assembly memory and outlives the function it is
/// created in.
#[derive(Debug, Clone)]
pub struct Renderer {
    model: Model,
    image_data: ImageBuffer,
    view_x_rotation: f32,
    view_y_rotation: f32,
    background: Rgba<f32>,
    border_thickness: f32,
    border_color: Rgba<f32>,
}

impl Renderer {
    pub fn new(model: Model, width: usize, height: usize, scale: usize) -> Self {
        Self {
            model,
            view_x_rotation: PI/8.0,
            view_y_rotation: 0.0,
            image_data: ImageBuffer::new(width, height, scale),
            background: Rgba {r: 0.0, g: 0.0, b: 0.0, a: 0.0},
            border_thickness: 0.15,
            border_color: Rgba::black(),
        }
    }

    /// Loads the Renderer from the given pointer. Leaks the value so it is not dropped.
    ///
    /// This is useful for quickly getting some information from a Renderer when you don't expect
    /// that its pointer will become invalidated due to a memory allocation.
    pub unsafe fn from_raw_leak(r_ptr: *mut Renderer) -> &'static mut Self {
        Box::leak(Box::from_raw(r_ptr))
    }

    /// Return a read-only reference to the inner image buffer
    pub fn image_data(&self) -> &ImageBuffer {
        &self.image_data
    }

    /// Render the image, modifying the image buffer in-place
    pub fn render(&mut self) {
        // The transformation that represents the position and orientation of the camera
        //
        // World coordinates -> Camera coordinates
        let view = Mat4::rotation_x(self.view_x_rotation) * Mat4::rotation_y(self.view_y_rotation);
        // The perspective/orthographic/etc. projection of the camera
        //
        // Camera coordinates -> Homogenous coordinates
        let projection = Mat4::perspective_rh_no(0.8*PI, 1.0, 0.01, 100.0)
            * Mat4::<f32>::scaling_3d(0.6);

        let mut depth = Buffer2d::new(self.image_data.size(), 1.0);
        self.image_data.clear(self.background);
        spritec::render(&mut self.image_data, &mut depth, view, projection, &self.model,
            self.border_thickness, self.border_color);
    }
}
