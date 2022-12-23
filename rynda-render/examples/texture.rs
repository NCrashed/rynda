extern crate gl;
extern crate glfw;

use glfw::{Action, Context, Key};
use ndarray::Array3;
use std::str;

use rynda_format::types::{volume::RleVolume, voxel::RgbVoxel};
use rynda_render::render::{
    debug::enable_gl_debug,
    pipeline::{generic::Pipeline, quad::QuadPipeline, texture::TexturePipeline},
    texture::Texture,
};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(false));
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

    let quad_vertex = str::from_utf8(include_bytes!("../shaders/quad_vertex.glsl")).unwrap();
    let quad_fragment = str::from_utf8(include_bytes!("../shaders/quad_fragment.glsl")).unwrap();
    let texture_fragment = str::from_utf8(include_bytes!("../shaders/pointermap_fragment.glsl")).unwrap();

    let texture_pipeline = TexturePipeline::new(quad_vertex, texture_fragment, width, height);
    texture_pipeline.program.print_uniforms();

    let quad_pipeline =
        QuadPipeline::new(quad_vertex, quad_fragment, &texture_pipeline.framebuffer.color_buffer);
    

    let mut mode: u32 = 0;
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut mode);
        }

        texture_pipeline.bind();
        unsafe {
            // Clear the screen
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        texture_pipeline.program.set_uniform("mode", &(mode as i32));

        let slot: i32 = 0;
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot as u32);
            gl::BindTexture(gl::TEXTURE_2D, pointmap_texture.id);

            // Bind input buffer
            let volume_size_id = texture_pipeline.program.uniform_location("volume_size");
            gl::Uniform3ui(
                volume_size_id,
                volume.xsize,
                volume.ysize,
                volume.zsize,
            );
        }
        // pointmap_texture.bind();
        texture_pipeline.program.set_uniform("pointermap", &slot);
        texture_pipeline.draw();

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        quad_pipeline.bind_draw();

        window.swap_buffers();
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, mode: &mut u32) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
            if *mode == 0 {
                *mode = 1;
            } else {
                *mode = 0;
            }
        }
        glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, width, height);
        },
        _ => {}
    }
}
