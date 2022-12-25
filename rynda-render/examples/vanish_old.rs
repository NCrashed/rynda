extern crate gl;
extern crate glfw;

use glfw::{Action, Context, CursorMode, Key};
use ndarray::Array3;
use std::str;

use rynda_format::types::{volume::RleVolume, voxel::RgbVoxel};
use rynda_render::{
    math::{aabb::HasBounding, transform::Transform},
    render::{
        camera::Camera,
        debug::enable_gl_debug,
        pipeline::{
            debug::{DebugLine, DebugPipeline},
            generic::Pipeline,
            quad::QuadPipeline,
            // raycast::RaycastPipeline,
            vanish_old::VanishPointPipeline,
        },
    },
    scene::chunk::ChunkedModel,
};

use glam::{IVec3, Mat4, Vec3};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    let screen_size = (1024, 1024);
    let (mut window, events) = glfw
        .create_window(
            screen_size.0,
            screen_size.1,
            "Rynda vanishing point test",
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
    let mut model = ChunkedModel::new();
    model.transform = Transform::translation(Vec3::new(-1.0, -1.0, -1.0));
    model.add_chunk(IVec3::new(0, 0, 0), volume);

    let vertex_shader =
        str::from_utf8(include_bytes!("../shaders/quad_vertex_transform.glsl")).unwrap();
    let fragment_shader = str::from_utf8(include_bytes!("../shaders/quad_fragment.glsl")).unwrap();
    let vanish_shader =
        str::from_utf8(include_bytes!("../shaders/vanishpoint_compute.glsl")).unwrap();
    let debug_vertex = str::from_utf8(include_bytes!("../shaders/debug_vertex.glsl")).unwrap();
    let debug_fragment = str::from_utf8(include_bytes!("../shaders/debug_fragment.glsl")).unwrap();

    let mut camera = Camera::look_at(Vec3::new(0.0, 2.0, -5.0), Vec3::ZERO);
    camera.near = 1.0;
    camera.far = 10.0;
    let mut debug_camera = Camera::look_at(Vec3::new(-5.5, 0.0, -5.0), Vec3::ZERO);

    let mut vanish_pipeline =
        VanishPointPipeline::new(vanish_shader, screen_size.0, screen_size.1, &camera);
    let quad_pipeline = QuadPipeline::new(vertex_shader, fragment_shader, &vanish_pipeline.texture);
    let mut debug_pipeline = DebugPipeline::new(debug_vertex, debug_fragment);

    let (cx0, cy0) = window.get_cursor_pos();
    let mut events_ctx = EventContext {
        old_cx: cx0,
        old_cy: cy0,
        ..Default::default()
    };

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

        unsafe {
            // Clear the screen
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        if events_ctx.mode == 0 {
            debug_pipeline.bind();
            let mvp = debug_camera.matrix();
            let camera_lines = DebugLine::from_vec(camera.lines(), Vec3::new(1.0, 0.0, 0.0));
            let volume_lines =
                DebugLine::from_vec(model.bounding_box().lines(), Vec3::new(0.0, 1.0, 0.0));

            debug_pipeline.set_lines(&[camera_lines, volume_lines].concat());
            debug_pipeline.set_mvp(&mvp);
            debug_pipeline.draw();
        } else {
            vanish_pipeline.bind();
            vanish_pipeline.camera = camera.clone();

            let vp_screen = camera.vanishing_point_window(screen_size.0, screen_size.1);

            vanish_pipeline
                .program
                .set_uniform("vanish_point", &vp_screen);
            vanish_pipeline.draw();

            quad_pipeline.bind();
            quad_pipeline.program.set_uniform("MVP", &Mat4::IDENTITY);
            quad_pipeline.draw();
        }

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
