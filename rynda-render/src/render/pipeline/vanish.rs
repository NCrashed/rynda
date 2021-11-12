use gl::types::*;
use std::str;

use super::generic::Pipeline;
use crate::render::{
    shader::{
        compile::{Shader, ShaderType},
        program::ShaderProgram,
    },
    texture::Texture,
};

/// Pipeline that renders raycast to a texture
pub struct VanishPointPipeline {
    pub program: ShaderProgram,
    pub texture: Texture,
    pub image_dimensions: (u32, u32),
}

impl VanishPointPipeline {
    pub fn new(compute_shader: &str, xsize: u32, ysize: u32) -> Self {
        let cs = Shader::compile(ShaderType::Compute, compute_shader);
        let program = ShaderProgram::link(vec![cs]);

        let texture = Texture::new(gl::TEXTURE1, xsize, ysize, None);

        VanishPointPipeline {
            program,
            texture,
            image_dimensions: (xsize, ysize),
        }
    }
}

impl Pipeline for VanishPointPipeline {
    fn bind(&self) {
        self.program.use_program();

        unsafe {
            // Bind output texture in Texture Unit 1
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.texture.id);

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
        }
    }

    fn draw(&self) {
        unsafe {
            gl::DispatchCompute(self.image_dimensions.0, self.image_dimensions.1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }
    }
}
