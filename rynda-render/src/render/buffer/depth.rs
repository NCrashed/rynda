use gl::types::*;
use std::marker::PhantomData;

/// Contains depth information for frame buffer
pub struct DepthBuffer<T> {
    /// Makes compiler happy about T usage
    phantom: PhantomData<T>,
    /// OpenGL id of the buffer
    pub id: GLuint,
    pub width: u32,
    pub height: u32,
}

impl<T> DepthBuffer<T> {
    /// Allocates new empty frame buffer
    pub fn new(width: u32, height: u32) -> Self {
        let mut id = 0;

        unsafe {
            gl::GenRenderbuffers(1, &mut id);
            gl::BindRenderbuffer(gl::RENDERBUFFER, id);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH_COMPONENT,
                width as i32,
                height as i32,
            );
        }

        Self {
            phantom: PhantomData,
            id,
            width,
            height,
        }
    }
}

impl<T> Drop for DepthBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteRenderbuffers(1, &self.id);
        }
    }
}
