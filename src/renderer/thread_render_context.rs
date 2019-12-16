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
    texture::RawImage2d,
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

pub struct Shaders {
    /// The cel shader used for drawing the sprites
    pub cel: Program,
    /// The outline shader used for drawing an outline around the sprites
    pub outline: Program,
}

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
}

impl ThreadRenderContext {
    /// Creates a new thread renderer.
    ///
    /// # Safety
    ///
    /// By calling this function, you guarantee that no other OpenGL contexts will be made current
    /// on this thread.
    pub unsafe fn new() -> Result<Self, ContextCreationError> {
        // At this point, we don't know what size we'll need, so we just pick anything
        let size = PhysicalSize {
            //TODO: We'll want a smaller default size once we figure out how to properly resize the
            // context.
            width: 2048.0,
            height: 2048.0,
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
        })
    }

    /// Returns a new renderer that can be used for drawing
    pub fn begin_render(&mut self, (width, height): (u32, u32)) -> Renderer {
        //TODO: Figure out how to actually resize the context
        assert!(width <= 2048 && height <= 2048,
            "bug: images larger than 2048x2048 are not supported yet!");

        Renderer {
            display: &self.display,
            shaders: &self.shaders,
            target: self.display.draw(),
        }
    }

    /// Takes a screenshot of the current buffer and returns it as an image buffer
    pub fn finish_render(&mut self) -> Result<RgbaImage, glium::ReadError> {
        // Wait for any pending draw calls to finish
        self.display.finish();

        let image: RawImage2d<u8> = self.display.read_front_buffer()?;
        let image = RgbaImage::from_raw(image.width, image.height, image.data.into_owned())
            .expect("bug: provided buffer was not big enough");
        let image = imageops::flip_vertical(&image);

        Ok(image)
    }
}
