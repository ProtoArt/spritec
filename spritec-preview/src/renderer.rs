use std::fmt::Write;
use std::f32::consts::PI;

use spritec::model::Model;
use euc::{Target, buffer::Buffer2d};
use vek::{Rgba, Mat4};
use strum::{IntoEnumIterator, EnumCount};
use strum_macros::{EnumIter, EnumDiscriminants, AsRefStr, IntoStaticStr, EnumCount};

use crate::image_buffer::ImageBuffer;

/// Represents all valid configuration options for the renderer.
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(name(ConfigOpts))]
#[strum_discriminants(derive(EnumIter, AsRefStr, IntoStaticStr, EnumCount))]
pub enum ConfigureRenderer {
    Width(usize),
    Height(usize),
    Scale(usize),
    ViewXRotation(f32),
    ViewYRotation(f32),
    Background(Rgba<f32>),
    BorderThickness(f32),
    BorderColor(Rgba<f32>),
}

impl ConfigureRenderer {
    /// Returns a JSON string that maps the option name to its serialized form as a number
    pub fn options() -> String {
        // Note that write!() can never panic for strings unless we get OOM
        let mut json = String::from("{");

        let count = ConfigOpts::count();
        for (i, opt) in ConfigOpts::iter().enumerate() {
            if i < count-1 {
                write!(json, r#""{}": {},"#, opt.as_ref(), opt as u8).unwrap();
            } else {
                write!(json, r#""{}": {}"#, opt.as_ref(), opt as u8).unwrap();
            }
        }

        json += "}";
        json
    }

    /// Return the configuration corresponding to the given option
    pub fn from_usize(opt: ConfigOpts, arg: usize) -> Self {
        use ConfigureRenderer::*;
        match opt {
            ConfigOpts::Width => Width(arg),
            ConfigOpts::Height => Height(arg),
            ConfigOpts::Scale => Scale(arg),

            ConfigOpts::ViewXRotation |
            ConfigOpts::ViewYRotation |
            ConfigOpts::BorderThickness |
            ConfigOpts::Background |
            ConfigOpts::BorderColor => unreachable!(),
        }
    }

    /// Return the configuration corresponding to the given option
    pub fn from_f32(opt: ConfigOpts, arg: f32) -> Self {
        use ConfigureRenderer::*;
        match opt {
            ConfigOpts::ViewXRotation => ViewXRotation(arg),
            ConfigOpts::ViewYRotation => ViewYRotation(arg),
            ConfigOpts::BorderThickness => BorderThickness(arg),

            ConfigOpts::Width |
            ConfigOpts::Height |
            ConfigOpts::Scale |
            ConfigOpts::Background |
            ConfigOpts::BorderColor => unreachable!(),
        }
    }

    /// Return the configuration corresponding to the given option
    pub fn from_rgba(opt: ConfigOpts, arg: Rgba<f32>) -> Self {
        use ConfigureRenderer::*;
        match opt {
            ConfigOpts::Background => Background(arg),
            ConfigOpts::BorderColor => BorderColor(arg),

            ConfigOpts::Width |
            ConfigOpts::Height |
            ConfigOpts::Scale |
            ConfigOpts::ViewXRotation |
            ConfigOpts::ViewYRotation |
            ConfigOpts::BorderThickness => unreachable!(),
        }
    }
}

impl From<u8> for ConfigOpts {
    fn from(value: u8) -> Self {
        ConfigOpts::iter().nth(value as usize).expect("Value for ConfigOpts out of range")
    }
}

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

    /// Set one of the configurable options for this renderer
    pub fn config(&mut self, opt: ConfigureRenderer) {
        use ConfigureRenderer::*;
        match opt {
            Width(arg) => self.image_data.set_width(arg),
            Height(arg) => self.image_data.set_height(arg),
            Scale(arg) => self.image_data.set_scale(arg),
            ViewXRotation(arg) => self.view_x_rotation = arg,
            ViewYRotation(arg) => self.view_y_rotation = arg,
            Background(arg) => self.background = arg,
            BorderThickness(arg) => self.border_thickness = arg,
            BorderColor(arg) => self.border_color = arg,
        }
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
