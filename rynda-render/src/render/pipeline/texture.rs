use gl::types::*;
use std::str;
use glam::Vec2;

use super::generic::Pipeline;
use crate::render::{
    array::vertex::VertexArray,
    buffer::{
        frame::FrameBuffer,
        index::{IndexBuffer, PrimitiveType},
        vertex::VertexBuffer,
        texture::Texture,
    },
    shader::{
        compile::{Shader, ShaderType},
        program::ShaderProgram,
    },
};

/// Pipeline that renders to a texture
pub struct TexturePipeline {
    pub framebuffer: FrameBuffer<()>,
    pub program: ShaderProgram,
    pub vao: VertexArray,
    pub vbo: VertexBuffer<GLfloat>,
    pub ebo: IndexBuffer<GLshort>,
}

static QUAD_POSITION_DATA: [GLfloat; 8] = [-1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0];
static QUAD_INDEX_DATA: [GLshort; 4] = [1, 2, 0, 3];

impl TexturePipeline {
    pub fn new(vertex_shader: &str, fragment_shader: &str, width: u32, height: u32) -> Self {
        let vs = Shader::compile(ShaderType::Vertex, vertex_shader);
        let fs = Shader::compile(ShaderType::Fragment, fragment_shader);
        let program = ShaderProgram::link(vec![vs, fs]);

        let texture = Texture::new(gl::TEXTURE1, width, height, None);
        let framebuffer = FrameBuffer::new(texture);

        let vao = VertexArray::new();
        let vbo: VertexBuffer<GLfloat> = VertexBuffer::new(&QUAD_POSITION_DATA);
        let ebo: IndexBuffer<GLshort> =
            IndexBuffer::new(PrimitiveType::TriangleStrip, &QUAD_INDEX_DATA);

        TexturePipeline {
            framebuffer,
            program,
            vao,
            vbo,
            ebo,
        }
    }
}

impl Pipeline for TexturePipeline {
    fn bind(&self) {
        // Bind framebuffer 
        self.framebuffer.bind();

        // Bind vertex array
        self.vao.bind();

        // Use quad program
        self.program.use_program();
        // let width = self.framebuffer.color_buffer.width;
        // let height = self.framebuffer.color_buffer.height;
        // self.program.set_uniform("aspect", &(width as f32 / height as f32 ));
        self.program.bind_attribute::<Vec2>("position", &self.vbo);
    }

    fn draw(&self) {
        self.ebo.draw();
    }

    fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
}
