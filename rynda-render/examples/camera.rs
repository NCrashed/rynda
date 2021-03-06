extern crate gl;
extern crate glfw;

use glfw::{Action, Context, CursorMode, Key};
use ndarray::Array3;
use std::str;

use rynda_format::types::{volume::RleVolume, voxel::RgbVoxel};
use rynda_render::render::{
    camera::Camera,
    debug::enable_gl_debug,
    pipeline::{generic::Pipeline, quad::QuadPipeline, raycast::RaycastPipeline},
};

use glam::Vec3;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    let (mut window, events) = glfw
        .create_window(
            1024,
            1024,
            "Rynda camera matrix test",
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

    // let z = RgbVoxel::empty();
    // let r = RgbVoxel::only_red(1);
    // let voxels: Array3<RgbVoxel> = arr3(&[[[z, r], [z, r]], [[z, z], [z, z]]]);
    let voxels: Array3<RgbVoxel> = Array3::from_shape_fn((256, 256, 256), |(x, y, z)| {
        let sx = (x as isize) - 128;
        let sz = (z as isize) - 128;
        let sy = (y as isize) - 256;
        if sx * sx + sz * sz + sy * sy < 64 * 64 {
            RgbVoxel::only_red(1)
        } else {
            RgbVoxel::empty()
        }
    });
    let volume: RleVolume = voxels.into();

    let vertex_shader =
        str::from_utf8(include_bytes!("../shaders/quad_vertex_transform.glsl")).unwrap();
    let fragment_shader = str::from_utf8(include_bytes!("../shaders/quad_fragment.glsl")).unwrap();
    let compute_shader =
        str::from_utf8(include_bytes!("../shaders/pointermap_compute.glsl")).unwrap();
    let mut raycast_pipeline = RaycastPipeline::new(compute_shader, &volume);
    let mut quad_pipeline =
        QuadPipeline::new(vertex_shader, fragment_shader, &raycast_pipeline.texture);

    let mut camera = Camera::look_at(Vec3::new(-5.5, 0.0, -5.0), Vec3::ZERO);

    let (cx0, cy0) = window.get_cursor_pos();
    let mut events_ctx = EventContext::default();
    events_ctx.old_cx = cx0;
    events_ctx.old_cy = cy0;

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut events_ctx, &mut camera);
        }
        events_ctx.move_camera(&mut camera);

        unsafe {
            // Clear the screen
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        raycast_pipeline.bind();
        raycast_pipeline
            .program
            .set_uniform("mode", &(events_ctx.mode as i32));
        raycast_pipeline.draw();

        quad_pipeline.bind();
        quad_pipeline.program.set_uniform("MVP", &camera.matrix());
        quad_pipeline.draw();

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
}

impl Default for EventContext {
    fn default() -> Self {
        EventContext {
            mode: 0,
            old_cx: 0.,
            old_cy: 0.,
            left: false,
            right: false,
            up: false,
            down: false,
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
