use gl::types::*;
use std::{mem, ptr};
use std::marker::PhantomData;

pub enum PrimitiveType {
    Points, 
    LineStrip, 
    LineLoop, 
    Lines, 
    LineStripAdjacency, 
    LinesAdjacency, 
    Triangles, 
    TrianglesAdjacency, 
    TriangleFan, 
    TriangleStrip, 
    TriangleStripAdjacency,
    Patches,
}

pub fn primitive_type_id(value: PrimitiveType) -> GLenum {
    match value {
        PrimitiveType::Points => gl::POINTS, 
        PrimitiveType::LineStrip => gl::LINE_STRIP, 
        PrimitiveType::LineLoop => gl::LINE_LOOP, 
        PrimitiveType::Lines => gl::LINES, 
        PrimitiveType::LineStripAdjacency => gl::LINE_STRIP_ADJACENCY, 
        PrimitiveType::LinesAdjacency => gl::LINES_ADJACENCY, 
        PrimitiveType::Triangles => gl::TRIANGLES, 
        PrimitiveType::TrianglesAdjacency => gl::TRIANGLES_ADJACENCY, 
        PrimitiveType::TriangleFan => gl::TRIANGLE_FAN, 
        PrimitiveType::TriangleStrip => gl::TRIANGLE_STRIP, 
        PrimitiveType::TriangleStripAdjacency => gl::TRIANGLE_STRIP_ADJACENCY,
        PrimitiveType::Patches => gl::PATCHES,
    }
}

/// A Index Buffer Objects (IBO)
pub struct IndexBuffer<T> {
    /// Makes compiler happy about T usage
    phantom: PhantomData<T>,
    /// OpenGL id of the buffer
    pub id: GLuint,
    /// Number of elements in the index buffer
    pub length: usize, 
    /// Which primivitive to draw
    pub primitive: PrimitiveType,
}

impl<T> IndexBuffer<T> {
    pub fn new(primitive: PrimitiveType, data: &[T]) -> Self {
        let mut ebo = 0;
        unsafe {
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                mem::transmute(&data[0]),
                gl::STATIC_DRAW,
            );
        }
        IndexBuffer { phantom: PhantomData, id: ebo, length: data.len(), primitive }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }
}

impl<T> Drop for IndexBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

impl IndexBuffer<GLshort> {
    pub fn draw(&self) {
        unsafe {
            gl::DrawElements(
                gl::TRIANGLE_STRIP,
                self.length as GLint,
                gl::UNSIGNED_SHORT,
                ptr::null(),
            );
        }
    }
}
