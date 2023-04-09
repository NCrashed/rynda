use gl::types::*;
use std::{str, ptr}; 

use rynda_format::types::{volume::RleVolume, pointermap::PointerColumn};
use super::{
    buffer::{
        shader::ShaderBuffer,
        vertex::VertexBuffer,
        index::{PrimitiveType, IndexBuffer},
    },
    array::vertex::VertexArray,
    shader::{ShaderType, Shader, ShaderProgram},
    texture::Texture,
};
use glam::Vec3;

pub trait Pipeline {
    /// Bind to OpenGL state and prepare for render
    fn bind(&self);

    /// Perform drawing call
    fn draw(&self);
}



pub struct Pipeline {
    pub compute_program: ShaderProgram,
    pub quad_program: ShaderProgram,
    pub debug_program: ShaderProgram,
    pub output_tex: Texture,

    pub vao: VertexArray,
    pub vbo: VertexBuffer<GLfloat>,
    pub ebo: IndexBuffer<GLshort>,
    pub image_dimensions: (u32, u32),

    pub pointmap_buffer: ShaderBuffer<PointerColumn>,

    pub debug_lines: Vec<(Vec3, Vec3, Vec3)>,
}

impl Pipeline {
    pub fn new(vertex_shader: &str, fragment_shader: &str, compute_shader: &str, volume: &RleVolume) -> Self {

        let vs = Shader::compile(ShaderType::Vertex, vertex_shader);
        let fs = Shader::compile(ShaderType::Fragment, fragment_shader);
        let cs = Shader::compile(ShaderType::Compute, compute_shader);
        let dvs = Shader::compile(ShaderType::Vertex, str::from_utf8(include_bytes!("../../shaders/debug.vert")).unwrap());
        let dfs = Shader::compile(ShaderType::Fragment, str::from_utf8(include_bytes!("../../shaders/debug.frag")).unwrap());
        let program = ShaderProgram::link(vec![vs, fs]);
        let compute_program = ShaderProgram::link(vec![cs]);
        let debug_program = ShaderProgram::link(vec![dvs, dfs]);

        let output_tex = Texture::new(gl::TEXTURE1, volume.xsize, volume.zsize, None);

        let vao = VertexArray::new();
        let vbo: VertexBuffer<GLfloat> = VertexBuffer::new(&QUAD_POSITION_DATA);
        let ebo: IndexBuffer<GLshort> = IndexBuffer::new(PrimitiveType::TriangleStrip, &QUAD_INDEX_DATA);

        let pointmap_buffer = ShaderBuffer::from_pointermap(&volume);

        unsafe {
            // Use quad program
            program.use_program();
            let pos_attr = program.attr_location("position");
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(
                pos_attr as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );
            
            // Bind output texture in Texture Unit 1
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, output_tex.id);

            // Set our "texture" sampler to use Texture Unit 1
            let output_tex_id = program.uniform_location("img_output");
            gl::Uniform1i(output_tex_id, 1);

            // Bind input buffer
            compute_program.use_program();
            let volume_size_id = compute_program.uniform_location("volume_size");
            gl::Uniform3ui(volume_size_id, volume.xsize, volume.ysize, volume.zsize);
        }

        Pipeline {
            compute_program,
            quad_program: program, 
            debug_program,
            output_tex,
            vao, vbo, ebo, 
            image_dimensions: (volume.xsize, volume.zsize),
            pointmap_buffer,
            debug_lines: vec![],
        }
    }

    /// Draw each frame and call prepare callback before the compute shader call
    pub fn draw<F>(&self, mut prepare: F) 
        where F: FnMut()
    {
        unsafe {
            // Clear the screen
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Compute next state of game
            self.compute_program.use_program();
            gl::BindImageTexture(
                1,
                self.output_tex.id as GLuint,
                0,
                gl::FALSE,
                0,
                gl::WRITE_ONLY,
                gl::RGBA8,
            );
            self.pointmap_buffer.bind(0);
            prepare();
            gl::DispatchCompute(self.image_dimensions.0, self.image_dimensions.1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);

            // Draw a quad from the two triangles
            self.quad_program.use_program();
            self.ebo.draw();
        }
    }
}