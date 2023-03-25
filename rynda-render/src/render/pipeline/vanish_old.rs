use gl::types::*;
use std::str;

use super::generic::Pipeline;
use crate::render::{
    buffer::texture::{Texture, TextureFormat},
    camera::Camera,
    shader::{
        compile::{Shader, ShaderType},
        program::ShaderProgram,
    },
};

/// Pipeline that renders raycast to a texture
pub struct VanishPointPipeline {
    pub program: ShaderProgram,
    pub texture: Texture<{ TextureFormat::RGBA }>,
    pub image_dimensions: (u32, u32),
    pub camera: Camera,
}

impl VanishPointPipeline {
    pub fn new(compute_shader: &str, xsize: u32, ysize: u32, camera: &Camera) -> Self {
        let cs = Shader::compile(ShaderType::Compute, compute_shader);
        let program = ShaderProgram::link(vec![cs]);

        let texture = Texture::new(gl::TEXTURE1, xsize, ysize, None);

        VanishPointPipeline {
            program,
            texture,
            image_dimensions: (xsize, ysize),
            camera: camera.clone(),
        }
    }
}

impl Pipeline for VanishPointPipeline {
    fn bind(&mut self) {
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
        self.texture.clear();
        let vp_screen = self
            .camera
            .vanishing_point_window(self.image_dimensions.0, self.image_dimensions.1);

        // Segment 1 (right)
        let np: u32 = if vp_screen.x as u32 >= self.image_dimensions.0 {
            0
        } else {
            2 * (self.image_dimensions.0 - (vp_screen.x as u32))
        };
        self.program.set_uniform("segment", &0u32);
        self.program.set_uniform("np", &np);
        unsafe {
            println!("Segments: {}", np);
            gl::DispatchCompute(np, 1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }

        // Segment 2 (top)
        let np = if vp_screen.y <= 0.0 {
            0
        } else {
            2 * (vp_screen.y as u32)
        };
        // println!("np = {}", np);
        self.program.set_uniform("segment", &1u32);
        self.program.set_uniform("np", &np);
        unsafe {
            println!("Segments: {}", np);
            gl::DispatchCompute(np, 1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }

        // Segment 3 (left)
        let np = if vp_screen.x <= 0.0 {
            0
        } else {
            2 * (vp_screen.x as u32)
        };
        self.program.set_uniform("segment", &2u32);
        self.program.set_uniform("np", &np);
        unsafe {
            println!("Segments: {}", np);
            gl::DispatchCompute(np, 1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }

        // Segment 4 (bottom)
        let np = if vp_screen.y >= self.image_dimensions.1 as f32 {
            0
        } else {
            2 * ((self.image_dimensions.1 as f32 - vp_screen.y) as u32)
        };
        self.program.set_uniform("segment", &3u32);
        self.program.set_uniform("np", &np);
        unsafe {
            println!("Segments: {}", np);
            gl::DispatchCompute(np, 1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }
    }
}
