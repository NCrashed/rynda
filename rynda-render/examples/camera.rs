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
    pipeline::{generic::Pipeline, quad::QuadPipeline, texture::TexturePipeline},
};

use glam::Vec3;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    let width = 1024;
    let height = 1024;
    let (mut window, events) = glfw
        .create_window(
            width,
            height,
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

    let pointmap_texture = Texture::from_pointermap(gl::TEXTURE0, &volume);

    let quad_vertex = str::from_utf8(include_bytes!("../shaders/quad.vert")).unwrap();
    let quad_mvp_vertex = str::from_utf8(include_bytes!("../shaders/quad_transform.vert")).unwrap();
    let quad_fragment = str::from_utf8(include_bytes!("../shaders/quad.frag")).unwrap();
    let texture_fragment = str::from_utf8(include_bytes!("../shaders/pointermap.frag")).unwrap();

    let mut texture_pipeline = TexturePipeline::new(quad_vertex, texture_fragment, width, height);
    texture_pipeline.program.print_uniforms();

    let mut quad_pipeline = QuadPipeline::new(
        quad_mvp_vertex,
        quad_fragment,
        &texture_pipeline.framebuffer.color_buffer,
        width,
        height,
    );

    let mut camera = Camera::look_at(Vec3::new(-5.5, 0.0, -5.0), Vec3::ZERO);

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

        quad_pipeline.bind();
        unsafe {
            // Clear the screen
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
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
