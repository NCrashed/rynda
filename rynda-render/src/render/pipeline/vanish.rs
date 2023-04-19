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
        texture::TextureFormat,
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
    pub texture_right: Texture<{ TextureFormat::RGBA }>,
    pub texture_top: Texture<{ TextureFormat::RGBA }>,
    pub texture_left: Texture<{ TextureFormat::RGBA }>,
    pub texture_bottom: Texture<{ TextureFormat::RGBA }>,
    pub framebuffer: FrameBuffer<()>,
    pub segment_program: ShaderProgram,
    pub collect_program: ShaderProgram,
    pub vao_vanish: VertexArray,
    pub vbo_vanish: VertexBuffer<GLfloat>,
    pub sbo_vanish: VertexBuffer<GLfloat>,
    pub ebo_vanish: IndexBuffer<GLshort>,
    pub camera: Camera,
}

/// Making triangles that converges in vanish point and ends on edges of screen. Simple
/// geometry is involved.
fn vanish_mesh(vp: Vec2, aspect: f32) -> (Vec<GLfloat>, Vec<GLshort>, Vec<GLfloat>) {
    let verts_num = 4 * 3;
    let mut verts = Vec::with_capacity(verts_num);
    let mut ids = Vec::with_capacity(verts_num);
    let mut segments = Vec::with_capacity(verts_num);
    let mut i = 0;
    // Right segment. Tan Pi/4 is 1, so segment vp-h is same as h-y02.
    //     /* y02
    //    / |
    //   /  |
    //vp*---* h
    //   \  |
    //    \ |
    //     \* y04
    //
    let y02 = vp.y - (aspect - vp.x);
    let y04 = vp.y + (aspect - vp.x);
    if vp.x < aspect && y04 > -1.0 && y02 < 1.0 {
        verts.extend_from_slice(&[vp.x, vp.y, aspect, y04, aspect, y02]);
        ids.extend_from_slice(&[i, i + 1, i + 2]);
        i += 3;
        segments.extend_from_slice(&[0.0, 0.0, 0.0]);
    }

    // Upper segment. Tan Pi/4 is 1, so segment vp-h is same as h-y02.
    //   x11  h    x12
    //   *----*----*
    //    \   |   /
    //     \  |  /
    //      \ | /
    //       \|/
    //        * vp
    let x11 = vp.x + (-1.0 - vp.y);
    let x12 = vp.x - (-1.0 - vp.y);
    if vp.y > -1.0 && x11 < aspect && x12 > -aspect {
        verts.extend_from_slice(&[vp.x, vp.y, x12, -1.0, x11, -1.0]);
        ids.extend_from_slice(&[i, i + 1, i + 2]);
        i += 3;
        segments.extend_from_slice(&[1.0, 1.0, 1.0]);
    }

    // Left segment. Tan Pi/4 is 1, so segment vp-h is same as h-y02.
    // y21*
    //    |\
    //    | \
    //  h *--* vp
    //    | /
    //    |/
    // y23*
    //
    let y21 = vp.y + (-aspect - vp.x);
    let y23 = vp.y - (-aspect - vp.x);
    if vp.x > -aspect && y21 < 1.0 && y23 > -1.0 {
        verts.extend_from_slice(&[vp.x, vp.y, -aspect, y21, -aspect, y23]);
        ids.extend_from_slice(&[i, i + 1, i + 2]);
        i += 3;
        segments.extend_from_slice(&[2.0, 2.0, 2.0]);
    }

    // Down segment. Tan Pi/4 is 1, so segment vp-h is same as h-y02.
    //        * vp
    //       /|\
    //      / | \
    //     /  |  \
    //    /   |   \
    //   *----*----*
    //   x33  h    x34
    let x33 = vp.x - (1.0 - vp.y);
    let x34 = vp.x + (1.0 - vp.y);
    if vp.y < 1.0 && x33 < aspect && x34 > -aspect {
        verts.extend_from_slice(&[vp.x, vp.y, x33, 1.0, x34, 1.0]);
        ids.extend_from_slice(&[i, i + 1, i + 2]);
        segments.extend_from_slice(&[3.0, 3.0, 3.0]);
    }
    (verts, ids, segments)
}

pub struct VanishPrograms<'a> {
    pub segment_compute_shader: &'a str,
    pub collect_vertex_shader: &'a str,
    pub collect_fragment_shader: &'a str,
}

impl VanishPipeline {
    pub fn new(
        programs: VanishPrograms<'_>,
        segment_dims: (u32, u32),
        target_dims: (u32, u32),
        camera: &Camera,
    ) -> Self {
        let cs = Shader::compile(ShaderType::Compute, programs.segment_compute_shader);
        let segment_program = ShaderProgram::link(vec![cs]);

        let vs = Shader::compile(ShaderType::Vertex, programs.collect_vertex_shader);
        let fs = Shader::compile(ShaderType::Fragment, programs.collect_fragment_shader);
        let collect_program = ShaderProgram::link(vec![vs, fs]);

        let texture_top = Texture::new(gl::TEXTURE1, segment_dims.0, segment_dims.1, None);
        let texture_bottom = Texture::new(gl::TEXTURE2, segment_dims.0, segment_dims.1, None);
        let texture_left = Texture::new(gl::TEXTURE3, segment_dims.0, segment_dims.1, None);
        let texture_right = Texture::new(gl::TEXTURE4, segment_dims.0, segment_dims.1, None);

        let width = target_dims.0;
        let height = target_dims.1;
        let texture = Texture::new(gl::TEXTURE5, width, height, None);
        let framebuffer = FrameBuffer::new(texture);

        let vp_screen = camera.vanishing_point_window(width, height);
        let vp = vp_screen / Vec2::new(width as f32, height as f32) * 2.0 - 1.0;
        let (mesh_vecs, mesh_ids, segments) = vanish_mesh(vp, camera.aspect);
        let vao_vanish = VertexArray::new();
        let vbo_vanish: VertexBuffer<GLfloat> = VertexBuffer::new(&mesh_vecs);
        let sbo_vanish: VertexBuffer<GLfloat> = VertexBuffer::new(&segments);
        let ebo_vanish: IndexBuffer<GLshort> =
            IndexBuffer::new(PrimitiveType::Triangles, &mesh_ids);

        VanishPipeline {
            texture_top,
            texture_bottom,
            texture_left,
            texture_right,
            framebuffer,
            segment_program,
            collect_program,
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
        self.segment_program.use_program();

        // Vanishing point of the camera defines which segments are visible
        let width = self.framebuffer.color_buffer.width;
        let height = self.framebuffer.color_buffer.height;
        let vp_screen = self.camera.vanishing_point_window(width, height);

        let np = 10.0;
        self.segment_program.set_uniform::<GLfloat>("np", &np);

        // First render all segments
        // Top segment
        if vp_screen.y > 0.0 {
            self.segment_program.set_uniform::<GLfloat>("segment", &0.0);
            self.texture_top.bind_mut(0);
            unsafe {
                gl::DispatchCompute(self.texture_top.width, 1, 1);
            }
        }
        // Bottom segment
        if vp_screen.y < height as f32 {
            self.segment_program.set_uniform::<GLfloat>("segment", &1.0);
            self.texture_bottom.bind_mut(0);
            unsafe {
                gl::DispatchCompute(self.texture_bottom.width, 1, 1);
            }
        }
        // Left segment
        if vp_screen.x > 0.0 {
            self.segment_program.set_uniform::<GLfloat>("segment", &2.0);
            self.texture_left.bind_mut(0);
            unsafe {
                gl::DispatchCompute(self.texture_left.width, 1, 1);
            }
        }
        // Right segment
        if vp_screen.x < width as f32 {
            self.segment_program.set_uniform::<GLfloat>("segment", &3.0);
            self.texture_right.bind_mut(0);
            unsafe {
                gl::DispatchCompute(self.texture_right.width, 1, 1);
            }
        }
        unsafe {
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }

        // Bind resulting framebuffer
        self.vao_vanish.bind();
        self.ebo_vanish.bind();
        let mut vp = vp_screen / Vec2::new(width as f32, height as f32) * 2.0 - 1.0;
        vp.x = 0.0;
        let (mesh_vecs, mesh_ids, segments) = vanish_mesh(vp, self.camera.aspect);
        self.vbo_vanish.load(&mesh_vecs);
        self.ebo_vanish.load(&mesh_ids);
        self.sbo_vanish.load(&segments);
        self.collect_program.use_program();
        // self.collect_program.print_attributes();
        self.collect_program
            .bind_attribute::<Vec2>("position", &self.vbo_vanish);
        self.collect_program
            .bind_attribute::<GLfloat>("segment", &self.sbo_vanish);
        self.collect_program.set_uniform("vp_point", &vp);
        let aspect_mvp = glam::Mat4::orthographic_rh_gl(
            -1.0 * self.camera.aspect,
            1.0 * self.camera.aspect,
            -1.0,
            1.0,
            -1.0,
            1.0,
        );
        self.collect_program.set_uniform("MVP", &aspect_mvp);

        self.framebuffer.bind();
        self.texture_top.bind(0);
        self.texture_bottom.bind(1);
        self.texture_left.bind(2);
        self.texture_right.bind(3);
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
