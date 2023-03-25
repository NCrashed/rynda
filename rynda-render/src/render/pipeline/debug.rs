use gl::types::*;
use std::str;

use super::generic::Pipeline;
use crate::render::{
    array::vertex::VertexArray,
    buffer::{
        index::{IndexBuffer, PrimitiveType},
        vertex::VertexBuffer,
    },
    shader::{
        compile::{Shader, ShaderType},
        program::ShaderProgram,
    },
};
use glam::{Mat4, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct DebugLine {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Vec3,
}

impl DebugLine {
    pub fn new(start: Vec3, end: Vec3, color: Vec3) -> Self {
        DebugLine { start, end, color }
    }

    /// Attach color to given lines and make them [DebugLine]
    pub fn from_vec(lines: Vec<(Vec3, Vec3)>, color: Vec3) -> Vec<DebugLine> {
        lines
            .iter()
            .map(|(v1, v2)| DebugLine::new(*v1, *v2, color))
            .collect()
    }
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

    pub fn set_lines(&mut self, new_lines: &[DebugLine]) {
        self.lines = new_lines.to_vec();
        self.load();
    }

    pub fn load(&mut self) {
        let mut positions = vec![0.0; self.lines.len() * 6];
        let mut colors = vec![0.0; self.lines.len() * 6];
        let mut indecies = vec![0; self.lines.len() * 2];

        for (i, line) in self.lines.iter().enumerate() {
            positions[i * 6] = line.start.x;
            positions[i * 6 + 1] = line.start.y;
            positions[i * 6 + 2] = line.start.z;
            positions[i * 6 + 3] = line.end.x;
            positions[i * 6 + 4] = line.end.y;
            positions[i * 6 + 5] = line.end.z;

            colors[i * 6] = line.color.x;
            colors[i * 6 + 1] = line.color.y;
            colors[i * 6 + 2] = line.color.z;
            colors[i * 6 + 3] = line.color.x;
            colors[i * 6 + 4] = line.color.y;
            colors[i * 6 + 5] = line.color.z;

            indecies[i * 2] = (i * 2) as i16;
            indecies[i * 2 + 1] = (i * 2 + 1) as i16;
        }

        self.pos_buffer.load(&positions);
        self.col_buffer.load(&colors);
        self.ebo.load(&indecies);
    }

    pub fn set_mvp(&mut self, mvp: &Mat4) {
        self.program.set_uniform("MVP", mvp);
    }
}

impl Pipeline for DebugPipeline {
    fn bind(&mut self) {
        // Bind vertex array
        self.vao.bind();

        // Use quad program
        self.program.use_program();

        // Bind vector attributes
        self.program
            .bind_attribute::<Vec3>("position", &self.pos_buffer);
        self.program
            .bind_attribute::<Vec3>("color", &self.col_buffer);
    }

    fn draw(&self) {
        self.ebo.draw();
    }
}
