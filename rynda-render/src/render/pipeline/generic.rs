/// Each drawing pipeline shares common pattern create-bind-draw call sequence.
pub trait Pipeline {
    /// Bind to OpenGL state and prepare for render
    fn bind(&self);

    /// Perform drawing call
    fn draw(&self);

    /// Unbind buffers if needed
    fn unbind(&self) {}

    /// Shorcut for bind and draw calls
    fn bind_draw(&self) {
        self.bind();
        self.draw();
        self.unbind();
    }
}
