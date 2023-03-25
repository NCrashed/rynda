/// Each drawing pipeline shares common pattern create-bind-draw call sequence.
pub trait Pipeline {
    /// Bind to OpenGL state and prepare for render
    fn bind(&mut self);

    /// Perform drawing call
    fn draw(&self);

    /// Unbind buffers if needed
    fn unbind(&mut self) {}

    /// Shorcut for bind and draw calls
    fn bind_draw(&mut self) {
        self.bind();
        self.draw();
        self.unbind();
    }
}
