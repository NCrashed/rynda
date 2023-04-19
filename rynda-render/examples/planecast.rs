extern crate gl;
extern crate glfw;

use glam::UVec3;
use glfw::{Action, Context, CursorMode, Key};
use ndarray::Array3;
use std::str;

use rynda_format::types::{volume::RleVolume, voxel::RgbVoxel};
use rynda_render::render::{
    buffer::texture::Texture,
    camera::Camera,
    debug::enable_gl_debug,
    pipeline::{
        debug::{DebugLine, DebugPipeline},
        generic::Pipeline,
        planecast::PlanecastPipeline,
        quad::QuadPipeline,
        texture::TexturePipeline,
    },
};

use glam::{Mat4, Vec3};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(false));
    let width = 1024;
    let height = 1024;
    let (mut window, events) = glfw
        .create_window(
            width,
            height,
            "Rynda debug of planes casting in pointermap",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(CursorMode::Disabled);
    window.make_current();

    // Load the OpenGL function pointers3
    gl::load_with(|s| window.get_proc_address(s) as *const _);
    enable_gl_debug();

    let voxels: Array3<RgbVoxel> = Array3::from_shape_fn((32, 32, 32), |(x, y, z)| {
        let sx = (x as isize) - 16;
        let sz = (z as isize) - 16;
        let sy = (y as isize) - 32;
        if sx * sx + sz * sz + sy * sy < 16 * 16 {
            RgbVoxel::only_red(1)
        } else {
            RgbVoxel::empty()
        }
    });
    let volume: RleVolume = voxels.into();
    let pointmap_texture = Texture::from_pointermap(gl::TEXTURE0, &volume);

    let quad_vertex = str::from_utf8(include_bytes!("../shaders/quad.vert")).unwrap();
    let vertex_shader = str::from_utf8(include_bytes!("../shaders/quad_transform.vert")).unwrap();
    let fragment_shader = str::from_utf8(include_bytes!("../shaders/quad.frag")).unwrap();
    let compute_shader = str::from_utf8(include_bytes!("../shaders/pointermap.frag")).unwrap();
    let debug_vertex = str::from_utf8(include_bytes!("../shaders/debug.vert")).unwrap();
    let debug_fragment = str::from_utf8(include_bytes!("../shaders/debug.frag")).unwrap();
    let planecast_shader = str::from_utf8(include_bytes!("../shaders/planecast.comp")).unwrap();

    let mut camera = Camera::look_at(Vec3::new(0.0, 2.0, -5.0), Vec3::ZERO);
    camera.near = 1.0;
    camera.far = 10.0;
    let mut debug_camera = Camera::look_at(Vec3::new(-5.5, 0.0, -5.0), Vec3::ZERO);

    let mut texture_pipeline = TexturePipeline::new(quad_vertex, compute_shader, 64, 64);
    let mut planecast_pipelinne = PlanecastPipeline::new(
        planecast_shader,
        texture_pipeline.framebuffer.color_buffer.clone(),
        &camera,
    );
    let mut quad_pipeline = QuadPipeline::new(
        vertex_shader,
        fragment_shader,
        texture_pipeline.framebuffer.color_buffer.clone(),
        width,
        height,
    );
    let mut debug_pipeline = DebugPipeline::new(debug_vertex, debug_fragment);

    let (cx0, cy0) = window.get_cursor_pos();
    let mut events_ctx = EventContext::new(cx0, cy0, width, height);

    while !window.should_close() {
        let active_camera = if events_ctx.mode == 0 {
            &mut debug_camera
        } else {
            &mut camera
        };
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut events_ctx, active_camera);
        }
        events_ctx.move_camera(active_camera);
        quad_pipeline.width = events_ctx.width;
        quad_pipeline.height = events_ctx.height;

        texture_pipeline.bind();
        unsafe {
            // Clear the screen
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        texture_pipeline
            .program
            .set_uniform("mode", &(events_ctx.mode as i32));
        texture_pipeline.program.set_uniform(
            "volume_size",
            &UVec3::new(volume.xsize, volume.ysize, volume.zsize),
        );
        pointmap_texture.bind(0);
        texture_pipeline.program.set_uniform("pointermap", &0i32);
        texture_pipeline.draw();
        texture_pipeline.unbind();

        planecast_pipelinne.camera = active_camera.clone();
        planecast_pipelinne.bind_draw();

        quad_pipeline.bind();
        unsafe {
            // Clear the screen
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        let mvp = if events_ctx.mode == 0 {
            debug_camera.matrix()
        } else {
            Mat4::IDENTITY
        };
        quad_pipeline.program.set_uniform("MVP", &mvp);
        quad_pipeline.draw();

        debug_pipeline.bind();
        debug_pipeline.set_lines(&DebugLine::from_vec(
            camera.lines(),
            Vec3::new(1.0, 0.0, 0.0),
        ));
        debug_pipeline.set_mvp(&mvp);
        debug_pipeline.draw();

        window.swap_buffers();
    }
}

const MOUSE_ROTATION_SPEED: f64 = 0.001;
const CAMERA_MOVE_SPEED: f64 = 0.01;

struct EventContext {
    pub mode: u32,
    pub old_cx: f64,
    pub old_cy: f64,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub width: u32,
    pub height: u32,
}

impl EventContext {
    fn new(old_cx: f64, old_cy: f64, width: u32, height: u32) -> Self {
        EventContext {
            mode: 0,
            old_cx,
            old_cy,
            left: false,
            right: false,
            up: false,
            down: false,
            width,
            height,
        }
    }
}

impl EventContext {
    pub fn move_camera(&self, camera: &mut Camera) {
        if self.up {
            camera.move_forward(CAMERA_MOVE_SPEED);
        }
        if self.down {
            camera.move_forward(-CAMERA_MOVE_SPEED);
        }
        if self.left {
            camera.move_right(-CAMERA_MOVE_SPEED);
        }
        if self.right {
            camera.move_right(CAMERA_MOVE_SPEED);
        }
    }
}

fn handle_window_event(
    window: &mut glfw::Window,
    event: glfw::WindowEvent,
    ctx: &mut EventContext,
    camera: &mut Camera,
) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
            if ctx.mode == 0 {
                ctx.mode = 1;
            } else {
                ctx.mode = 0;
            }
        }
        glfw::WindowEvent::Key(Key::Up | Key::W, _, state, _) => {
            ctx.up = state != Action::Release;
        }
        glfw::WindowEvent::Key(Key::Down | Key::S, _, state, _) => {
            ctx.down = state != Action::Release;
        }
        glfw::WindowEvent::Key(Key::Left | Key::A, _, state, _) => {
            ctx.left = state != Action::Release;
        }
        glfw::WindowEvent::Key(Key::Right | Key::D, _, state, _) => {
            ctx.right = state != Action::Release;
        }
        glfw::WindowEvent::FramebufferSize(width, height) => {
            unsafe {
                gl::Viewport(0, 0, width, height);
            }
            camera.aspect = width as f32 / height as f32;
            ctx.width = width as u32;
            ctx.height = height as u32;
        }
        glfw::WindowEvent::CursorPos(cx, cy) => {
            let dx = (cx - ctx.old_cx) * MOUSE_ROTATION_SPEED;
            let dy = (cy - ctx.old_cy) * MOUSE_ROTATION_SPEED;
            camera.update_cursor(dx, dy);
            ctx.old_cx = cx;
            ctx.old_cy = cy;
        }
        _ => {}
    }
}
