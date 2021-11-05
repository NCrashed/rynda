
use gl::types::*;
use std::{ptr, mem};

pub unsafe fn create_texture(
    unit: GLenum,
    width: u32,
    height: u32,
    image: Option<&image::RgbaImage>,
) -> GLuint {
    let mut tex_id = 0;
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
    tex_id
}