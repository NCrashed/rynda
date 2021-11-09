
use gl::types::*;
use std::str;

use super::generic::Pipeline;
use crate::render::{
    array::vertex::VertexArray,
    buffer::{
        index::{IndexBuffer, PrimitiveType},
        vertex::VertexBuffer,
    },
    shader::{Shader, ShaderProgram, ShaderType},
};
use glam::Vec3;

pub struct DebugLine {
    pub start: Vec3, 
    pub end: Vec3, 
    pub color: Vec3,
}

/// Drawing pipeline that draws a single quad with given texture
pub struct DebugPipeline {
    pub program: ShaderProgram,
    pub vao: VertexArray,
    pub pos_buffer: VertexBuffer<GLfloat>,
    pub col_buffer: VertexBuffer<GLfloat>,
    pub ebo: IndexBuffer<GLshort>,
    pub lines: Vec<DebugLine>,
}

impl DebugPipeline {
    pub fn new(vertex_shader: &str, fragment_shader: &str) -> Self {
        let vs = Shader::compile(ShaderType::Vertex, vertex_shader);
        let fs = Shader::compile(ShaderType::Fragment, fragment_shader);
        let program = ShaderProgram::link(vec![vs, fs]);

        let vao = VertexArray::new();
        let pos_buffer: VertexBuffer<GLfloat> = VertexBuffer::new(&[]);
        let col_buffer: VertexBuffer<GLfloat> = VertexBuffer::new(&[]);
        let ebo: IndexBuffer<GLshort> = IndexBuffer::new(PrimitiveType::Lines, &[]);

        DebugPipeline {
            program,
            vao,
            pos_buffer,
            col_buffer,
            ebo,
            lines: vec![],
        }
    }
}

impl Pipeline for DebugPipeline {
    fn bind(&self) {
        // Bind vertex array
        self.vao.bind();

        // Use quad program
        self.program.use_program();
        
        // Bind vector attributes
        self.program.bind_attribute::<Vec3>("position", &self.pos_buffer);
        self.program.bind_attribute::<Vec3>("color", &self.col_buffer);
    }

    fn draw(&self) {
        self.ebo.draw();
    }
}
