use gl::types::*;
use std::{rc::Rc, str};

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
pub struct PlanecastPipeline {
    pub program: ShaderProgram,
    pub texture: Rc<Texture<{ TextureFormat::RGBA }>>,
    pub camera: Camera,
    pub planes_number: u32,
    pub segment: u32,
}

impl PlanecastPipeline {
    pub fn new(
        compute_shader: &str,
        texture: Rc<Texture<{ TextureFormat::RGBA }>>,
        camera: &Camera,
    ) -> Self {
        let cs = Shader::compile(ShaderType::Compute, compute_shader);
        let program = ShaderProgram::link(vec![cs]);

        PlanecastPipeline {
            program,
            texture,
            camera: camera.clone(),
            planes_number: 10,
            segment: 0,
        }
    }
}

impl Pipeline for PlanecastPipeline {
    fn bind(&mut self) {
        self.program.use_program();

        self.texture.bind_mut(0);

        let vp_screen = self.camera.vanishing_point_screenspace();
        let vp_world = self.camera.vanishing_point();
        self.program.set_uniform("vp", &vp_screen);
        self.program
            .set_uniform::<GLfloat>("np", &(self.planes_number as f32));
        self.program
            .set_uniform::<GLfloat>("segment", &(self.segment as f32));
        let mvp_inv = self.camera.matrix().inverse();
        // let vp_rev_project = mvp_inv.project_point3(vp_screen.extend(0.0));
        // println!("{vp_world} vs {vp_rev_project}");
        self.program.set_uniform("mvp_inv", &mvp_inv);
    }

    fn draw(&self) {
        unsafe {
            gl::DispatchCompute(self.planes_number, 1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }
    }
}
