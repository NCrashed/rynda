extern crate gl;
extern crate glfw;

use glfw::{Action, Context, CursorMode, Key};
use ndarray::Array3;
use std::str;

use rynda_format::types::{volume::RleVolume, voxel::RgbVoxel};
use rynda_render::render::{
    camera::Camera,
    debug::enable_gl_debug,
    pipeline::{
        debug::{DebugLine, DebugPipeline},
        generic::Pipeline,
        quad::QuadPipeline,
        raycast::RaycastPipeline,
    },
};

use glam::{Mat4, Vec3, Vec4};

fn make_debug_lines(matrix: &Mat4) -> Vec<DebugLine> {
    let p0 = Vec3::new(-1.0, -1.0, -1.0);
    let p1 = Vec3::new(1.0, -1.0, -1.0);
    let p2 = Vec3::new(-1.0, 1.0, -1.0);
    let p3 = Vec3::new(1.0, 1.0, -1.0);
    let p4 = Vec3::new(-1.0, -1.0, 1.0);
    let p5 = Vec3::new(1.0, -1.0, 1.0);
    let p6 = Vec3::new(-1.0, 1.0, 1.0);
    let p7 = Vec3::new(1.0, 1.0, 1.0);

    fn transform(v: Vec3, m: &Mat4) -> Vec3 {
        let v1 = *m * Vec4::new(v.x, v.y, v.z, 1.0);
        Vec3::new(v1.x / v1.w, v1.y / v1.w, v1.z / v1.w)
    }

    fn line(v1: Vec3, v2: Vec3, m: &Mat4) -> DebugLine {
        DebugLine {
            start: transform(v1, m),
            end: transform(v2, m),
            color: Vec3::new(1.0, 0.0, 0.0),
        }
    }

    vec![
        line(p0, p1, matrix),
        line(p0, p2, matrix),
        line(p1, p3, matrix),
        line(p2, p3, matrix),
        line(p4, p5, matrix),
        line(p4, p6, matrix),
        line(p5, p7, matrix),
        line(p6, p7, matrix),
        line(p0, p4, matrix),
        line(p1, p5, matrix),
        line(p2, p6, matrix),
        line(p3, p7, matrix),
    ]
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    let (mut window, events) = glfw
        .create_window(
            1024,
            1024,
            "Rynda debug boundaries test",
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
    // let voxels = rynda_format::from_vox::vox_to_rle_volume("assets/test_model.vox").unwrap();
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
    let debug_vertex = str::from_utf8(include_bytes!("../shaders/debug_vertex.glsl")).unwrap();
    let debug_fragment = str::from_utf8(include_bytes!("../shaders/debug_fragment.glsl")).unwrap();

    let mut raycast_pipeline = RaycastPipeline::new(compute_shader, &volume);
    let mut quad_pipeline =
        QuadPipeline::new(vertex_shader, fragment_shader, &raycast_pipeline.texture);
    let mut debug_pipeline = DebugPipeline::new(debug_vertex, debug_fragment);
    
    let mut camera = Camera::look_at(Vec3::new(0.0, 2.0, -5.0), Vec3::ZERO);
    camera.near = 1.0;
    camera.far = 10.0;
    let mut debug_camera = Camera::look_at(Vec3::new(-5.5, 0.0, -5.0), Vec3::ZERO);

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

        raycast_pipeline.bind();
        raycast_pipeline
            .program
            .set_uniform("mode", &(events_ctx.mode as i32));
        raycast_pipeline.draw();

        quad_pipeline.bind();
        let mvp = if events_ctx.mode == 0 {
            debug_camera.matrix()
        } else {
            Mat4::IDENTITY
        };
        quad_pipeline.program.set_uniform("MVP", &mvp);
        quad_pipeline.draw();

        debug_pipeline.bind();
        let cam_inverse = camera.matrix().inverse();
        debug_pipeline.set_lines(&make_debug_lines(&cam_inverse));
        debug_pipeline.set_mvp(&mvp);
        debug_pipeline.draw();

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
