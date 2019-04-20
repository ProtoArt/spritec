use crate::image_buffer::ImageBuffer;

/// A shared context to store in the web assembly memory that outlives the function it is
/// created in.
#[derive(Debug, Clone)]
pub struct Context {
    image_data: ImageBuffer,
}

impl Context {
    pub fn new() -> Self {
        Self {
            image_data: ImageBuffer::new(64, 64, 12),
        }
    }

    /// Loads the Context from the given pointer. Leaks the value so it is not dropped.
    pub unsafe fn from_raw_leak(ctx_ptr: *mut Context) -> &'static mut Self {
        Box::leak(Box::from_raw(ctx_ptr))
    }

    pub fn image_data(&self) -> &ImageBuffer {
        &self.image_data
    }

    pub fn image_data_mut(&mut self) -> &mut ImageBuffer {
        &mut self.image_data
    }
}
