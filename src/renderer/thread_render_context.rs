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
use image::{RgbaImage, imageops};
use thiserror::Error;

use super::Renderer;

#[derive(Debug, Error)]
pub enum ContextCreationError {
    #[error("{0}")]
    CreationError(#[from] glium::glutin::CreationError),
    #[error("{0}")]
    IncompatibleOpenGl(#[from] glium::IncompatibleOpenGl),
    #[error("{0}")]
    ProgramCreationError(#[from] glium::ProgramCreationError),
}

#[derive(Debug, Error)]
pub enum BeginRenderError {
    #[error("{0}")]
    TextureCreationError(#[from] glium::texture::TextureCreationError),
    #[error("{0}")]
    FrameBufferValidationError(#[from] glium::framebuffer::ValidationError),
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
/// This can be used ot read the data after rendering has been performed. The struct is *not*
/// Clone or Copy, which helps enforce that the data is read once and then destroyed.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RenderId(usize);

// Having this alias allows us to swap Headless with Display during debugging
pub type Display = glium::backend::glutin::headless::Headless;
//pub type Display = glium::backend::glutin::Display;

/// A render context that is only allowed to be used on a single thread. Only *one* instance of
/// this struct should be created per thread.
pub struct ThreadRenderContext {
    /// We only use headless windows, so there should never be any events generated (probably?)
    ///
    /// We still need to keep it around though because dropping it may cause issues (maybe?)
    _event_loop: EventLoop<()>,
    /// The OpenGL context and display. Assumes that the OpenGL context that is current on this
    /// thread will never change.
    display: Display,
    /// The shader programs used during rendering
    shaders: Shaders,
    /// The data backing each Renderer
    render_data: Vec<RenderData>,
}

impl ThreadRenderContext {
    /// Creates a new thread renderer.
    ///
    /// # Safety
    ///
    /// By calling this function, you guarantee that no other OpenGL contexts will be made current
    /// on this thread.
    pub unsafe fn new() -> Result<Self, ContextCreationError> {
        // This size does not matter because we do not render to the screen
        let size = PhysicalSize {
            width: 500.0,
            height: 500.0,
        };

        let event_loop = EventLoop::new();

        let ctx = ContextBuilder::new()
            // A 24-bit depth buffer is pretty typical for most OpenGL applications
            .with_depth_buffer(24)
            .build_headless(&event_loop, size)?;
        let display = Display::new(ctx)?;

        // This code is useful for debugging when `type Display = glium::backend::glutin::Display`
        //let window_builder = glium::glutin::window::WindowBuilder::new()
        //    .with_inner_size(glium::glutin::dpi::LogicalSize::from_physical(size, 4.0));
        //let context_builder = ContextBuilder::new()
        //    // A 24-bit depth buffer is pretty typical for most OpenGL applications
        //    .with_depth_buffer(24);
        //let display = Display::new(window_builder, context_builder, &event_loop).unwrap();

        let cel_shader = Program::from_source(
            &display,
            include_str!("shader/cel.vs"),
            include_str!("shader/cel.fs"),
            None,
        )?;

        let outline_shader = Program::from_source(
            &display,
            include_str!("shader/outline.vs"),
            include_str!("shader/outline.fs"),
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

    /// Returns a new renderer that can be used for drawing
    pub fn begin_render(&mut self, (width, height): (u32, u32)) -> Result<(RenderId, Renderer), BeginRenderError> {
        let color_texture = Texture2d::empty_with_format(&self.display,
            UncompressedFloatFormat::F32F32F32F32, MipmapsOption::NoMipmap, width, height)?;
        let depth_texture = DepthTexture2d::empty_with_format(&self.display, DepthFormat::F32,
            MipmapsOption::NoMipmap, width, height)?;

        self.render_data.push(RenderData {
            color_texture,
            depth_texture,
        });
        let data = &self.render_data.last().unwrap();

        let target = SimpleFrameBuffer::with_depth_buffer(&self.display, &data.color_texture,
            &data.depth_texture)?;

        let render_id = RenderId(self.render_data.len() - 1);
        Ok((render_id, Renderer {
            display: &self.display,
            shaders: &self.shaders,
            target,
        }))
    }

    /// Returns the image that was rendered
    pub fn finish_render(&mut self, render_id: RenderId) -> Result<RgbaImage, glium::ReadError> {
        // Wait for any pending draw calls to finish
        self.display.finish();

        let RenderId(id) = render_id;
        let data = self.render_data.remove(id);
        let image: RawImage2d<u8> = data.color_texture.read();
        let image = RgbaImage::from_raw(image.width, image.height, image.data.into_owned())
            .expect("bug: provided buffer was not big enough");
        let image = imageops::flip_vertical(&image);

        Ok(image)
    }
}
