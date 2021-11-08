use gl::types::*;

/// A Vertex Array Object (VAO)
pub struct VertexArray {
    /// OpenGL id of the buffer
    pub id: GLuint,
}

impl VertexArray {
    pub fn new() -> Self {
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }
        VertexArray { id: vao }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

impl Default for VertexArray {
    fn default() -> Self {
        Self::new()
    }
}
