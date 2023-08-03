extern crate gl;
extern crate glfw;

use glam::UVec3;
use glfw::{Action, Context, Key};
use ndarray::Array3;
use std::str;

use rynda_format::types::{volume::RleVolume, voxel::RgbVoxel};
use rynda_render::render::{
    buffer::{texture::Texture, shader::ShaderBuffer},
    debug::enable_gl_debug,
    pipeline::{generic::Pipeline, quad::QuadPipeline, texture::TexturePipeline},
};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    let width = 1024;
    let height = 1024;
    let (mut window, events) = glfw
        .create_window(
            width,
            height,
            "Rynda texture target test",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();

    // Load the OpenGL function pointers3
    gl::load_with(|s| window.get_proc_address(s) as *const _);
    enable_gl_debug();

    // let z = RgbVoxel::empty();
    // let r = RgbVoxel::only_red(1);
    // let voxels: Array3<RgbVoxel> = ndarray::arr3(&[[[z, r], [z, r]], [[z, z], [z, z]]]);

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
    let pointmap_buffer = ShaderBuffer::from_pointermap(&volume);

    let quad_vertex = str::from_utf8(include_bytes!("../shaders/quad.vert")).unwrap();
    let quad_vertex_transform =
        str::from_utf8(include_bytes!("../shaders/quad_transform.vert")).unwrap();
    let quad_fragment = str::from_utf8(include_bytes!("../shaders/quad.frag")).unwrap();
    let texture_fragment = str::from_utf8(include_bytes!("../shaders/pointermap.frag")).unwrap();

    let mut texture_pipeline =
        TexturePipeline::new(quad_vertex_transform, texture_fragment, width, height);

    let mut quad_pipeline = QuadPipeline::new(
        quad_vertex,
        quad_fragment,
        texture_pipeline.framebuffer.color_buffer.clone(),
        width,
        height,
    );

    let mut ctx = EventContext::new(width, height);
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut ctx);
        }
        quad_pipeline.width = ctx.width;
        quad_pipeline.height = ctx.height;
        let aspect = ctx.width as f32 / ctx.height as f32;
        let aspect_mvp =
            glam::Mat4::orthographic_rh_gl(-1.0 * aspect, 1.0 * aspect, -1.0, 1.0, -1.0, 1.0);

        texture_pipeline.bind();
        unsafe {
            // Clear the screen
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        texture_pipeline
            .program
            .set_uniform("mode", &(ctx.mode as i32));
        texture_pipeline.program.set_uniform(
            "volume_size",
            &UVec3::new(volume.xsize, volume.ysize, volume.zsize),
        );
        texture_pipeline.program.set_uniform("MVP", &aspect_mvp);
        pointmap_buffer.bind(1);
        pointmap_texture.bind(0);

        texture_pipeline.program.set_uniform("pointermap", &0i32);
        texture_pipeline.draw();
        texture_pipeline.unbind();

        quad_pipeline.bind_draw();

        window.swap_buffers();
    }
}

struct EventContext {
    pub mode: u32,
    pub width: u32,
    pub height: u32,
}

impl EventContext {
    pub fn new(width: u32, height: u32) -> Self {
        EventContext {
            mode: 0,
            width,
            height,
        }
    }
}

fn handle_window_event(
    window: &mut glfw::Window,
    event: glfw::WindowEvent,
    ctx: &mut EventContext,
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
        glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, width, height);
            ctx.width = width as u32;
            ctx.height = height as u32;
        },
        _ => {}
    }
}
