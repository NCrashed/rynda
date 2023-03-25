use gl::types::*;
use glam::Vec2;
use std::str;

use super::generic::Pipeline;
use crate::render::{
    array::vertex::VertexArray,
    buffer::{
        frame::FrameBuffer,
        index::{IndexBuffer, PrimitiveType},
        texture::Texture,
        vertex::VertexBuffer,
    },
    camera::Camera,
    shader::{
        compile::{Shader, ShaderType},
        program::ShaderProgram,
    },
};

/// Pipeline that renders first 4 framebuffers and blits them to resulting
/// framebuffer according to vanish point of the camera.
pub struct VanishPipeline {
    pub framebuffer_top: FrameBuffer<()>,
    pub framebuffer_bottom: FrameBuffer<()>,
    pub framebuffer_left: FrameBuffer<()>,
    pub framebuffer_right: FrameBuffer<()>,
    pub framebuffer: FrameBuffer<()>,
    pub segment_program: ShaderProgram,
    pub collect_program: ShaderProgram,
    pub vao_quad: VertexArray,
    pub vbo_quad: VertexBuffer<GLfloat>,
    pub ebo_quad: IndexBuffer<GLshort>,
    pub vao_vanish: VertexArray,
    pub vbo_vanish: VertexBuffer<GLfloat>,
    pub sbo_vanish: VertexBuffer<GLint>,
    pub ebo_vanish: IndexBuffer<GLshort>,
    pub camera: Camera,
}

/// In trangle fan ordering
fn vanish_mesh(vp: Vec2, aspect: f32) -> (Vec<GLfloat>, Vec<GLshort>) {
    let mut verts = Vec::with_capacity(5);
    let mut ids = Vec::with_capacity(12);
    let x3 = 1.0 - vp.y + vp.x;
    let x4 = vp.y - 1.0 + vp.x;
    if vp.y < 1.0 && x3 > -aspect && x4 < aspect {
        verts.extend_from_slice(&[
            vp.x, vp.y, 
            x3, 1.0,
            x4, 1.0,
        ]);
        ids.extend_from_slice(&[
            0, 1, 2
        ]);
    }
    (verts, ids)
    // let mut verts = vec![
    //      0.0, 0.0,
    //     -1.0, -1.0, 
    //      1.0, -1.0,  
    //      1.0,  1.0, 
    //     -1.0,  1.0
    // ];
    // dbg!(vp);
    // verts[0] = vp.x;
    // verts[1] = vp.y;
    // verts
}

// static VANISH_INDEX_DATA: [GLshort; 12] = [0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 1];
static VANISH_INDEX_DATA: [GLshort; 3] = [0, 1, 2];

static SEGMENT_DATA: [GLint; 12] = [1, 1, 1, 3, 3, 3, 0, 0, 0, 2, 2, 2];

static QUAD_POSITION_DATA: [GLfloat; 8] = [-1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0];
static QUAD_INDEX_DATA: [GLshort; 4] = [1, 2, 0, 3];

impl VanishPipeline {
    pub fn new(
        segment_vertex_shader: &str,
        segment_fragment_shader: &str,
        collect_vertex_shader: &str,
        collect_fragment_shader: &str,
        width: u32,
        height: u32,
        camera: &Camera,
    ) -> Self {
        let vs = Shader::compile(ShaderType::Vertex, segment_vertex_shader);
        let fs = Shader::compile(ShaderType::Fragment, segment_fragment_shader);
        let segment_program = ShaderProgram::link(vec![vs, fs]);

        let vs = Shader::compile(ShaderType::Vertex, collect_vertex_shader);
        let fs = Shader::compile(ShaderType::Fragment, collect_fragment_shader);
        let collect_program = ShaderProgram::link(vec![vs, fs]);

        let texture = Texture::new(gl::TEXTURE1, width, height, None);
        let framebuffer_top = FrameBuffer::new(texture);
        let texture = Texture::new(gl::TEXTURE2, width, height, None);
        let framebuffer_bottom = FrameBuffer::new(texture);
        let texture = Texture::new(gl::TEXTURE3, width, height, None);
        let framebuffer_left = FrameBuffer::new(texture);
        let texture = Texture::new(gl::TEXTURE4, width, height, None);
        let framebuffer_right = FrameBuffer::new(texture);

        let texture = Texture::new(gl::TEXTURE5, width, height, None);
        let framebuffer = FrameBuffer::new(texture);

        let vao_quad = VertexArray::new();
        let vbo_quad: VertexBuffer<GLfloat> = VertexBuffer::new(&QUAD_POSITION_DATA);
        let ebo_quad: IndexBuffer<GLshort> =
            IndexBuffer::new(PrimitiveType::TriangleStrip, &QUAD_INDEX_DATA);

        let vp_screen = camera.vanishing_point_window(width, height);
        let vp = vp_screen / Vec2::new(width as f32, height as f32)*2.0 - 1.0;
        let (mesh_vecs, mesh_ids) = vanish_mesh(vp, camera.aspect);
        let vao_vanish = VertexArray::new();
        let vbo_vanish: VertexBuffer<GLfloat> = VertexBuffer::new(&mesh_vecs);
        let sbo_vanish: VertexBuffer<GLint> = VertexBuffer::new(&SEGMENT_DATA);
        let ebo_vanish: IndexBuffer<GLshort> =
            IndexBuffer::new(PrimitiveType::Triangles, &mesh_ids);

        VanishPipeline {
            framebuffer_top,
            framebuffer_bottom,
            framebuffer_left,
            framebuffer_right,
            framebuffer,
            segment_program,
            collect_program,
            vao_quad,
            vbo_quad,
            ebo_quad,
            vao_vanish,
            vbo_vanish,
            sbo_vanish,
            ebo_vanish,
            camera: camera.clone(),
        }
    }
}

impl Pipeline for VanishPipeline {
    // Bind actually binds all for segment rendering, not the collecting phase
    fn bind(&mut self) {
        // Shared between all segments
        self.vao_quad.bind();
        self.vbo_quad.bind();
        self.ebo_quad.bind();
        self.segment_program.use_program();
        self.segment_program
            .bind_attribute::<Vec2>("position", &self.vbo_quad);

        // Vanishing point of the camera defines which segments are visible
        let width = self.framebuffer.color_buffer.width;
        let height = self.framebuffer.color_buffer.height;
        let vp_screen = self.camera.vanishing_point_window(width, height);

        // First render all segments
        // Top segment
        if vp_screen.y > 0.0 {
            self.segment_program.set_uniform("segment", &0i32);
            self.framebuffer_top.bind();
            unsafe {
                gl::Viewport(0, 0, width as i32, height as i32);
            }
            self.ebo_quad.draw();
        }
        // Bottom segment
        if vp_screen.y < height as f32 {
            self.segment_program.set_uniform("segment", &1i32);
            self.framebuffer_bottom.bind();
            unsafe {
                gl::Viewport(0, 0, width as i32, height as i32);
            }
            self.ebo_quad.draw();
        }
        // Left segment
        if vp_screen.x > 0.0 {
            self.segment_program.set_uniform("segment", &2i32);
            self.framebuffer_left.bind();
            unsafe {
                gl::Viewport(0, 0, width as i32, height as i32);
            }
            self.ebo_quad.draw();
        }
        // Right segment
        if vp_screen.x < width as f32 {
            self.segment_program.set_uniform("segment", &3i32);
            self.framebuffer_right.bind();
            unsafe {
                gl::Viewport(0, 0, width as i32, height as i32);
            }
            self.ebo_quad.draw();
        }

        // Bind resulting framebuffer
        self.vao_vanish.bind();
        self.ebo_vanish.bind();
        let vp = vp_screen / Vec2::new(width as f32, height as f32)*2.0 - 1.0;
        let (mesh_vecs, mesh_ids) = vanish_mesh(vp, self.camera.aspect);
        self.vbo_vanish.load(&mesh_vecs);
        self.ebo_vanish.load(&mesh_ids);
        self.collect_program.use_program();
        self.collect_program.print_attributes();
        self.collect_program
            .bind_attribute::<Vec2>("position", &self.vbo_vanish);
        self.collect_program.bind_attribute::<GLint>("segment", &self.sbo_vanish);
        self.collect_program.set_uniform("vp_point", &vp);
        let aspect_mvp =
            glam::Mat4::orthographic_rh_gl(-1.0 * self.camera.aspect, 1.0 * self.camera.aspect, -1.0, 1.0, -1.0, 1.0);
        self.collect_program.set_uniform("MVP", &aspect_mvp);

        self.framebuffer.bind();
        self.framebuffer_top.color_buffer.bind(0);
        self.framebuffer_bottom.color_buffer.bind(1);
        self.framebuffer_left.color_buffer.bind(2);
        self.framebuffer_right.color_buffer.bind(3);
    }

    fn draw(&self) {
        let width = self.framebuffer.color_buffer.width;
        let height = self.framebuffer.color_buffer.height;

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        self.ebo_vanish.draw();
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
}
