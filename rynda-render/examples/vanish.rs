extern crate gl;
extern crate glfw;

use glam::Vec3;
use glfw::{Action, Context, CursorMode, Key};
use std::str;

use rynda_render::render::{
    camera::Camera,
    debug::enable_gl_debug,
    pipeline::{
        generic::Pipeline,
        quad::QuadPipeline,
        vanish::{VanishPipeline, VanishPrograms},
    },
};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(false));
    let width = 2048;
    let height = 1024;
    let (mut window, events) = glfw
        .create_window(
            width,
            height,
            "Rynda vanish point remap test",
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

    let mut camera = Camera::look_at(Vec3::new(-5.5, 0.0, -5.0), Vec3::ZERO);

    let quad_vertex = str::from_utf8(include_bytes!("../shaders/quad.vert")).unwrap();
    let vanish_vertex = str::from_utf8(include_bytes!("../shaders/vanish.vert")).unwrap();
    let quad_fragment = str::from_utf8(include_bytes!("../shaders/quad.frag")).unwrap();
    let segment_compute = str::from_utf8(include_bytes!("../shaders/segment.comp")).unwrap();
    let vanish_fragment = str::from_utf8(include_bytes!("../shaders/vanish.frag")).unwrap();

    let programs = VanishPrograms {
        segment_compute_shader: segment_compute,
        collect_vertex_shader: vanish_vertex,
        collect_fragment_shader: vanish_fragment,
    };
    let mut vanish_pipeline = VanishPipeline::new(programs, (512, 512), (width, height), &camera);

    let mut quad_pipeline = QuadPipeline::new(
        quad_vertex,
        quad_fragment,
        vanish_pipeline.framebuffer.color_buffer.clone(),
        width,
        height,
    );

    let (cx0, cy0) = window.get_cursor_pos();
    let mut events_ctx = EventContext::new(width, height);
    events_ctx.old_cx = cx0;
    events_ctx.old_cy = cy0;

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut events_ctx, &mut camera);
        }
        events_ctx.move_camera(&mut camera);
        vanish_pipeline.camera = camera.clone();
        quad_pipeline.width = events_ctx.width;
        quad_pipeline.height = events_ctx.height;

        vanish_pipeline.bind_draw();
        quad_pipeline.bind_draw();

        window.swap_buffers();
    }
}

const MOUSE_ROTATION_SPEED: f64 = 0.001;
const CAMERA_MOVE_SPEED: f64 = 0.1;

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
    fn new(width: u32, height: u32) -> Self {
        EventContext {
            mode: 0,
            old_cx: 0.,
            old_cy: 0.,
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
        glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, width, height);
            camera.aspect = width as f32 / height as f32;
            ctx.width = width as u32;
            ctx.height = height as u32;
        },
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
