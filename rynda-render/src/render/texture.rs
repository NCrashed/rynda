use gl::types::*;
use std::{mem, ptr};

pub struct Texture {
    pub id: GLuint,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(unit: GLenum, width: u32, height: u32, image: Option<&image::RgbaImage>) -> Self {
        let mut tex_id = 0;
        unsafe {
            gl::GenTextures(1, &mut tex_id);
            gl::ActiveTexture(unit);
            gl::BindTexture(gl::TEXTURE_2D, tex_id);
            let datum = match image {
                None => ptr::null(),
                Some(i) => mem::transmute(i.as_ptr()),
            };
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                width as GLint,
                height as GLint,
                0,
                gl::BGRA,
                gl::UNSIGNED_BYTE,
                datum,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST_MIPMAP_NEAREST as GLint,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        Texture {
            id: tex_id,
            width,
            height,
        }
    }

    pub fn clear(&self) {
        unsafe {
            gl::ClearTexImage(self.id, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
