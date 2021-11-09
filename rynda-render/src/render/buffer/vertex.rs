use gl::types::*;
use std::marker::PhantomData;
use std::mem;

/// A Vertex Buffer Object (VBO)
pub struct VertexBuffer<T> {
    /// Makes compiler happy about T usage
    phantom: PhantomData<T>,
    /// OpenGL id of the buffer
    pub id: GLuint,
}

impl<T> VertexBuffer<T> {
    pub fn new(data: &[T]) -> Self {
        let mut buffer = VertexBuffer {
            phantom: PhantomData,
            id: 0,
        };
        unsafe {
            gl::GenBuffers(1, &mut buffer.id);
        }

        buffer.load(data);
        buffer
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    pub fn load(&mut self, data: &[T]) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                mem::transmute(&data[0]),
                gl::STATIC_DRAW,
            );
        }
    }
}

impl<T> Drop for VertexBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}
