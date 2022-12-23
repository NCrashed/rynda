use std::marker::PhantomData;
use gl::types::*;

use super::depth::DepthBuffer;
use super::texture::{Texture, TextureFormat};

/// It’s a container for textures and an optional depth buffer.
pub struct FrameBuffer<T> {
    /// Makes compiler happy about T usage
    phantom: PhantomData<T>,
    /// OpenGL id of the buffer
    pub id: GLuint,
    /// Depth buffer attached
    pub depth_buffer: DepthBuffer<T>,
    /// Binded color texture
    pub color_buffer: Texture<{TextureFormat::RGBA}>,
}

impl<T> FrameBuffer<T> {
    /// Allocates new empty frame buffer
    pub fn new(color_buffer: Texture<{TextureFormat::RGBA}>) -> Self {
        let mut id = 0;
        let width = color_buffer.width;
        let height = color_buffer.height;

        unsafe {
            gl::GenFramebuffers(1, &mut id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, id);
        }

        let depth_buffer = DepthBuffer::new(width, height);

        unsafe {
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, depth_buffer.id);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, color_buffer.id, 0);

            let draw_buffers: Vec<GLenum> = vec![gl::COLOR_ATTACHMENT0];
            gl::DrawBuffers(draw_buffers.len() as i32, draw_buffers.as_ptr());

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Failed to finish framebuffer");
            } 
        }

        Self {
            phantom: PhantomData,
            id,
            depth_buffer,
            color_buffer,
        }
    }

    /// Bind framebuffer buffer
    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            gl::Viewport(0, 0, self.color_buffer.width as i32, self.color_buffer.height as i32);
        }
    }
}

impl<T> Drop for FrameBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}
