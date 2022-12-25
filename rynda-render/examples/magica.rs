extern crate dot_vox;
extern crate gl;
extern crate glfw;

use dot_vox::load_bytes;
use glam::UVec3;
use glfw::{Action, Context, Key};
use std::str;

use rynda_format::types::volume::RleVolume;
use rynda_render::render::{
    buffer::texture::Texture,
    debug::enable_gl_debug,
    pipeline::{generic::Pipeline, quad::QuadPipeline, texture::TexturePipeline},
};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    let mut width = 1024;
    let mut height = 1024;
    let (mut window, events) = glfw
        .create_window(
            width,
            height,
            "Rynda VoxelMagica test",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();

    // Load the OpenGL function pointers3
    gl::load_with(|s| window.get_proc_address(s) as *const _);
    enable_gl_debug();

    let volume: RleVolume = load_bytes(include_bytes!("../../assets/test_model.vox"))
        .unwrap()
        .into();
    let pointmap_texture = Texture::from_pointermap(gl::TEXTURE0, &volume);

    let quad_vertex = str::from_utf8(include_bytes!("../shaders/quad_vertex.glsl")).unwrap();
    let quad_fragment = str::from_utf8(include_bytes!("../shaders/quad_fragment.glsl")).unwrap();
    let texture_fragment =
        str::from_utf8(include_bytes!("../shaders/pointermap_fragment.glsl")).unwrap();

    let texture_pipeline = TexturePipeline::new(quad_vertex, texture_fragment, width, height);
    texture_pipeline.program.print_uniforms();

    let mut quad_pipeline = QuadPipeline::new(
        quad_vertex,
        quad_fragment,
        &texture_pipeline.framebuffer.color_buffer,
        width, height
    );

    let mut mode: u32 = 0;
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut mode, &mut width, &mut height);
        }
        quad_pipeline.width = width; 
        quad_pipeline.height = height; 
        
        texture_pipeline.bind();
        texture_pipeline.program.set_uniform("mode", &(mode as i32));
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
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        quad_pipeline.draw();

        window.swap_buffers();
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, mode: &mut u32, cwidth: &mut u32, cheight: &mut u32,) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
            if *mode == 0 {
                *mode = 1;
            } else {
                *mode = 0;
            }
        }
        glfw::WindowEvent::FramebufferSize(width, height) => {
            unsafe {
                gl::Viewport(0, 0, width, height);
            }
            *cwidth = width as u32;
            *cheight = height as u32;
        },
        _ => {}
    }
}
