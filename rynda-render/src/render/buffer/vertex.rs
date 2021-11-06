use gl::types::*;
use std::mem;
use std::marker::PhantomData;

/// A Verte Buffer Object (VBO)
pub struct VertexBuffer<T> {
    /// Makes compiler happy about T usage
    phantom: PhantomData<T>,
    /// OpenGL id of the buffer
    pub id: GLuint,
}

impl<T> VertexBuffer<T> {
    pub fn new(data: &[T]) -> Self {
        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                mem::transmute(&data[0]),
                gl::STATIC_DRAW,
            );
        }
        VertexBuffer { phantom: PhantomData, id: vbo }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
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
