use gl::types::*;
use std::str;

use super::generic::Pipeline;
use crate::render::{
    buffer::shader::ShaderBuffer,
    shader::{Shader, ShaderProgram, ShaderType},
    texture::Texture,
};
use rynda_format::types::{pointermap::PointerColumn, volume::RleVolume};

/// Pipeline that renders raycast to a texture
pub struct RaycastPipeline<'a> {
    pub program: ShaderProgram,
    pub texture: Texture,
    pub image_dimensions: (u32, u32),
    pub pointmap_buffer: ShaderBuffer<PointerColumn>,
    pub volume: &'a RleVolume,
}

impl<'a> RaycastPipeline<'a> {
    pub fn new(compute_shader: &str, volume: &'a RleVolume) -> Self {
        let cs = Shader::compile(ShaderType::Compute, compute_shader);
        let program = ShaderProgram::link(vec![cs]);

        let texture = Texture::new(gl::TEXTURE1, volume.xsize, volume.zsize, None);
        let pointmap_buffer = ShaderBuffer::from_pointermap(volume);

        RaycastPipeline {
            program,
            texture,
            image_dimensions: (volume.xsize, volume.zsize),
            pointmap_buffer,
            volume,
        }
    }
}

impl<'a> Pipeline for RaycastPipeline<'a> {
    fn bind(&self) {
        self.program.use_program();

        unsafe {
            // Bind output texture in Texture Unit 1
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.texture.id);

            // Bind input buffer
            let volume_size_id = self.program.uniform_location("volume_size");
            gl::Uniform3ui(
                volume_size_id,
                self.volume.xsize,
                self.volume.ysize,
                self.volume.zsize,
            );

            // Bind texture as mutable
            gl::BindImageTexture(
                1,
                self.texture.id as GLuint,
                0,
                gl::FALSE,
                0,
                gl::WRITE_ONLY,
                gl::RGBA8,
            );
            self.pointmap_buffer.bind(0);
        }
    }

    fn draw(&self) {
        unsafe {
            gl::DispatchCompute(self.image_dimensions.0, self.image_dimensions.1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }
    }
}
