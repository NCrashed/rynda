use gl::types::*;
use std::marker::PhantomData;
use std::ptr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        let mut buffer = IndexBuffer {
            phantom: PhantomData,
            id: 0,
            length: 0,
            primitive,
        };
        unsafe {
            gl::GenBuffers(1, &mut buffer.id);
        }

        buffer.load(data);
        buffer
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
        }
    }

    pub fn load(&mut self, data: &[T]) {
        if !data.is_empty() {
            unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (std::mem::size_of_val(data)) as GLsizeiptr,
                    &data[0] as *const T as *const std::ffi::c_void,
                    gl::STATIC_DRAW,
                );
            }
            self.length = data.len();
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
        if self.length > 0 {
            unsafe {
                gl::DrawElements(
                    primitive_type_id(self.primitive),
                    self.length as GLint,
                    gl::UNSIGNED_SHORT,
                    ptr::null(),
                );
            }
        }
    }
}
