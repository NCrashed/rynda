use gl::types::*;
use std::{ptr, str};

use super::generic::Pipeline;
use crate::render::{
    array::vertex::VertexArray,
    buffer::{
        index::{IndexBuffer, PrimitiveType},
        vertex::VertexBuffer,
    },
    shader::{Shader, ShaderProgram, ShaderType},
    texture::Texture,
};

/// Drawing pipeline that draws a single quad with given texture
pub struct QuadPipeline<'a> {
    pub program: ShaderProgram,
    pub vao: VertexArray,
    pub texture: &'a Texture,
    pub vbo: VertexBuffer<GLfloat>,
    pub ebo: IndexBuffer<GLshort>,
}

static QUAD_POSITION_DATA: [GLfloat; 8] = [-1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0];
static QUAD_INDEX_DATA: [GLshort; 4] = [1, 2, 0, 3];

impl<'a> QuadPipeline<'a> {
    pub fn new(vertex_shader: &str, fragment_shader: &str, texture: &'a Texture) -> Self {
        let vs = Shader::compile(ShaderType::Vertex, vertex_shader);
        let fs = Shader::compile(ShaderType::Fragment, fragment_shader);
        let program = ShaderProgram::link(vec![vs, fs]);

        let vao = VertexArray::new();
        let vbo: VertexBuffer<GLfloat> = VertexBuffer::new(&QUAD_POSITION_DATA);
        let ebo: IndexBuffer<GLshort> =
            IndexBuffer::new(PrimitiveType::TriangleStrip, &QUAD_INDEX_DATA);

        QuadPipeline {
            program,
            vao,
            texture,
            vbo,
            ebo,
        }
    }
}

impl<'a> Pipeline for QuadPipeline<'a> {
    fn bind(&self) {
        // Bind vertex array
        self.vao.bind();

        // Use quad program
        self.program.use_program();
        let pos_attr = self.program.attr_location("position");
        unsafe {
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
            gl::BindTexture(gl::TEXTURE_2D, self.texture.id);

            // Set our "texture" sampler to use Texture Unit 1
            let tex_id = self.program.uniform_location("img_output");
            gl::Uniform1i(tex_id, 1);
        }
    }

    fn draw(&self) {
        self.ebo.draw();
    }
}
