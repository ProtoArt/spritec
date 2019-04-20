use crate::image_buffer::ImageBuffer;

/// The full state of the renderer. Stored in web assembly memory and outlives the function it is
/// created in.
#[derive(Debug, Clone)]
pub struct Renderer {
    image_data: ImageBuffer,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            image_data: ImageBuffer::new(64, 64, 12),
        }
    }

    /// Loads the Renderer from the given pointer. Leaks the value so it is not dropped.
    pub unsafe fn from_raw_leak(ctx_ptr: *mut Renderer) -> &'static mut Self {
        Box::leak(Box::from_raw(ctx_ptr))
    }

    pub fn image_data(&self) -> &ImageBuffer {
        &self.image_data
    }

    pub fn image_data_mut(&mut self) -> &mut ImageBuffer {
        &mut self.image_data
    }
}
