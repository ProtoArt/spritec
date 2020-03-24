// Quote from: https://www.khronos.org/opengl/wiki/OpenGL_Context
//
// ---
// An OpenGL context represents many things. A context stores all of the state associated with this
// instance of OpenGL. It represents the (potentially visible) default framebuffer that rendering
// commands will draw to when not drawing to a framebuffer object. Think of a context as an object
// that holds all of OpenGL; when a context is destroyed, OpenGL is destroyed.
//
// Contexts are localized within a particular process of execution (an application, more or less)
// on an operating system. A process can create multiple OpenGL contexts. Each context can
// represent a separate viewable surface, like a window in an application.
//
// Each context has its own set of OpenGL Objects, which are independent of those from other
// contexts. A context's objects can be shared with other contexts. Most OpenGL objects are
// sharable, including Sync Objects and GLSL Objects. Container Objects are not sharable, nor are
// Query Objects.
//
// Any object sharing must be made explicitly, either as the context is created or before a newly
// created context creates any objects. However, contexts do not have to share objects; they can
// remain completely separate from one another.
//
// In order for any OpenGL commands to work, a context must be current; all OpenGL commands affect
// the state of whichever context is current. The current context is a thread-local variable, so a
// single process can have several threads, each of which has its own current context. However, a
// single context cannot be current in multiple threads at the same time.
// ---
//
// This is why we try to keep a single context per thread. That context is made current and we
// leave it that way.

use std::num::NonZeroU32;

use glium::{
    Program,
    framebuffer::SimpleFrameBuffer,
    texture::{
        RawImage2d,
        Texture2d,
        UncompressedFloatFormat,
        MipmapsOption,
        DepthTexture2d,
        DepthFormat,
    },
};
use glium::glutin::{
    ContextBuilder,
    dpi::PhysicalSize,
    event_loop::EventLoop,
};
use image::{RgbaImage, imageops::flip_vertical_in_place};
use thiserror::Error;

use crate::query3d::{QueryBackend, QueryError};

use super::{
    Renderer,
    RenderedImage,
    Size,
    FileQuery,
    Camera,
    layout::{LayoutNode, LayoutError},
    imageops::{scale_to_fit, copy},
};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum ContextCreationError {
    CreationError(#[from] glium::glutin::CreationError),
    DisplayCreationError(#[from] glium::backend::glutin::DisplayCreationError),
    IncompatibleOpenGl(#[from] glium::IncompatibleOpenGl),
    ProgramCreationError(#[from] glium::ProgramCreationError),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum BeginRenderError {
    TextureCreationError(#[from] glium::texture::TextureCreationError),
    FrameBufferValidationError(#[from] glium::framebuffer::ValidationError),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum DrawLayoutError {
    BeginRenderError(#[from] BeginRenderError),
    DrawError(#[from] glium::DrawError),
    ReadError(#[from] glium::ReadError),
    QueryError(#[from] QueryError),
    LayoutError(#[from] LayoutError),
}

pub(in super) struct Shaders {
    /// The cel shader used for drawing the sprites
    pub cel: Program,
    /// The outline shader used for drawing an outline around the sprites
    pub outline: Program,
}

/// The data backing one of the Renderers
struct RenderData {
    color_texture: Texture2d,
    depth_texture: DepthTexture2d,
}

/// The ID of the data associated with a Renderer
///
/// This can be used to read the data after rendering has been performed. The struct is *not*
/// Clone or Copy, which helps enforce that the data is read once and then destroyed.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RenderId(usize);

// Having this alias allows us to change the Display struct later without changing the type name
// all throughout our code
//
// On Windows headless causes issues when compiled with electron, so we use a
// hidden display instead (see build_display).
#[cfg(not(windows))]
pub type Display = glium::backend::glutin::headless::Headless;
#[cfg(windows)]
pub type Display = glium::backend::glutin::Display;

/// A render context that is only allowed to be used on a single thread. Only *one* instance of
/// this struct should be created per thread.
pub struct ThreadRenderContext {
    /// We only use headless windows, so there should never be any events generated (probably?)
    ///
    /// We still need to keep it around though because dropping it may cause issues (maybe?)
    _event_loop: EventLoop<()>,
    /// The OpenGL context and display
    display: Display,
    /// The shader programs used during rendering
    shaders: Shaders,
    /// The data backing each Renderer
    render_data: Vec<RenderData>,
}

impl ThreadRenderContext {
    /// Creates a new thread renderer.
    ///
    /// No other OpenGL context should be made current on this thread while this value exists.
    pub fn new() -> Result<Self, ContextCreationError> {
        let event_loop = EventLoop::new();

        let display = Self::build_display(
            ContextBuilder::new().with_depth_buffer(24),
            &event_loop,
            // Size does not matter because we do not render to the screen
            PhysicalSize {width: 2, height: 2},
        )?;

        let cel_shader = Program::from_source(
            &display,
            include_str!("shader/cel.vs"),
            include_str!("shader/cel.fs"),
            None,
        )?;

        let outline_shader = Program::from_source(
            &display,
            include_str!("shader/screen_triangle.vs"),
            // include_str!("shader/cel.vs"),
            include_str!("shader/sobel.fs"),
            None,
        )?;

        Ok(Self {
            _event_loop: event_loop,
            display,
            shaders: Shaders {
                cel: cel_shader,
                outline: outline_shader,
            },
            render_data: Vec::new(),
        })
    }

    /// Builds a headless display on non-Windows platforms. This is the
    /// preferred method when headless is supported.
    #[cfg(not(windows))]
    fn build_display(
        ctx_builder: ContextBuilder<glium::glutin::NotCurrent>,
        event_loop: &EventLoop<()>,
        size: PhysicalSize<u32>,
    ) -> Result<Display, ContextCreationError> {
        let ctx = ctx_builder.build_headless(&event_loop, size)?;
        Ok(Display::new(ctx)?)
    }

    /// On Windows we build a hidden window instead of a headless display due
    /// to broken behaviour when headless is compiled by neon for electron.
    #[cfg(windows)]
    fn build_display(
        ctx_builder: ContextBuilder<glium::glutin::NotCurrent>,
        event_loop: &EventLoop<()>,
        size: PhysicalSize<u32>,
    ) -> Result<Display, ContextCreationError> {
        let win_builder = glium::glutin::window::WindowBuilder::new()
            .with_visible(false)
            .with_inner_size(size);

        Ok(Display::new(win_builder, ctx_builder, &event_loop)?)
    }

    /// Scales the given image up, with no anti-aliasing or other interpolation of any kind.
    pub fn scale(&mut self, image: &RgbaImage, scale: NonZeroU32) -> Result<RgbaImage, DrawLayoutError> {
        //TODO: Do this scaling using the GPU. Should the error type still be DrawLayoutError?

        //TODO: Could optimize the case of scale == 1
        let scale = scale.get();
        let (width, height) = image.dimensions();
        let mut scaled_image = RgbaImage::new(width * scale, height * scale);
        scale_to_fit(&image, &mut scaled_image);

        Ok(scaled_image)
    }

    /// Draws the given layout, returning the image that was rendered
    pub fn draw(&mut self, layout: LayoutNode) -> Result<RgbaImage, DrawLayoutError> {
        let Size {width, height} = layout.size();

        let mut final_image = RgbaImage::new(width.get(), height.get());
        for (offset, node) in layout.iter_targets() {
            use LayoutNode::*;
            match node {
                RenderedImage(image) => {
                    let image = self.draw_render(image)?;
                    copy(&image, &mut final_image, (offset.x, offset.y));
                },

                Grid(_) => {
                    let image = self.draw(node)?;
                    copy(&image, &mut final_image, (offset.x, offset.y));
                },

                Empty {..} => {
                    // Draw nothing
                },
            }
        }

        Ok(final_image)
    }

    fn draw_render(&mut self, image: RenderedImage) -> Result<RgbaImage, DrawLayoutError> {
        let RenderedImage {size, background, camera, lights, ambient_light, geometry, outline} = image;
        let FileQuery {query, file} = geometry;
        let Camera {view, projection} = *camera.fetch_camera()?;
        let lights = lights.fetch_lights()?;

        let Size {width, height} = size;
        let width = width.get();
        let height = height.get();

        let color_texture = Texture2d::empty(&self.display,
            width, height).unwrap();
        let depth_texture = DepthTexture2d::empty_with_format(&self.display, DepthFormat::F32,
            MipmapsOption::NoMipmap, width, height).unwrap();

        self.render_data.push(RenderData {
            color_texture,
            depth_texture,
        });
        // This unwrap() will never panic because we just pushed data into the Vec
        let data = &self.render_data.last().unwrap();

        let target = SimpleFrameBuffer::with_depth_buffer(&self.display, &data.color_texture,
            &data.depth_texture).unwrap();

        let render_id = RenderId(self.render_data.len() - 1);
        let mut renderer = Renderer {
            display: &self.display,
            shaders: &self.shaders,
            target,
        };

        renderer.clear(background);

        let mut file = file.lock().expect("bug: file lock was poisoned");
        let geos = file.query_geometry(&query, renderer.display())?;
        for geo in &*geos {
            renderer.render(&*geo, &lights, ambient_light, view, projection, &outline, &data.color_texture, &data.depth_texture)?;
        }

        let RenderId(id) = render_id;
        let data = self.render_data.remove(id);
        // This stops and performs the read synchronously. For max parallelism, we'd probably want
        // read_to_pixel_buffer().
        let image: RawImage2d<u8> = data.color_texture.read();
        let mut image = RgbaImage::from_raw(image.width, image.height, image.data.into_owned())
            .expect("bug: image data buffer did not match expected size for width and height");
        flip_vertical_in_place(&mut image);

        Ok(image)
    }
}
