extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::{Action, Context, Key};
use ndarray::{Array3};
use std::os::raw::c_void;
use std::{ptr, str};

use rynda_format::types::{volume::RleVolume, voxel::RgbVoxel};
use rynda_render::render::{
    buffer::{
        shader::ShaderBuffer,
        vertex::VertexBuffer,
        index::{PrimitiveType, IndexBuffer},
    },
    array::vertex::VertexArray,
    shader::{ShaderType, Shader, ShaderProgram},
    texture::Texture,
};

// Vertex data
static POSITION_DATA: [GLfloat; 8] = [-1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0];
static INDEX_DATA: [GLshort; 4] = [1, 2, 0, 3];

extern "system" fn debug_print(
    _source: GLenum,
    gltype: GLenum,
    _id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    _user_param: *mut c_void,
) {
    let msg: &str = unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(
            message as *const u8,
            length as usize,
        ))
        .unwrap()
    };
    let iserror = if gltype == gl::DEBUG_TYPE_ERROR {
        "** GL ERROR **"
    } else {
        ""
    };
    println!(
        "GL CALLBACK: {} type = {:#01x}, severity = {:#01x}, message = {}",
        iserror, gltype, severity, msg
    );
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    let (mut window, events) = glfw
        .create_window(
            1024,
            1024,
            "Rynda pointmap test",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();

    // Load the OpenGL function pointers3
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(debug_print, std::ptr::null());
    }

    // Create GLSL shaders
    let vs = Shader::compile(ShaderType::Vertex, str::from_utf8(include_bytes!("../shaders/quad_vertex.glsl")).unwrap());
    let fs = Shader::compile(ShaderType::Fragment, str::from_utf8(include_bytes!("../shaders/quad_fragment.glsl")).unwrap());
    let cs = Shader::compile(ShaderType::Compute, str::from_utf8(include_bytes!("../shaders/pointermap_compute.glsl")).unwrap());
    let program = ShaderProgram::link(vec![vs, fs]);
    let compute_program = ShaderProgram::link(vec![cs]);

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

    let pointmap_buffer = ShaderBuffer::from_pointermap(&volume);
    let image_dimensions = (volume.xsize, volume.zsize);
    let output_tex = Texture::new(gl::TEXTURE1, image_dimensions.0, image_dimensions.1, None);

    // Create Vertex Array Object
    let _vao = VertexArray::new();
    // Create a Vertex Buffer Object and copy the vertex data to it
    let _vbo: VertexBuffer<GLfloat> = VertexBuffer::new(&POSITION_DATA);
    // Create buffer for indecies and fill data to it
    let ebo: IndexBuffer<GLshort> = IndexBuffer::new(PrimitiveType::TriangleStrip, &INDEX_DATA);

    let mode_id;

    unsafe {
        // Use shader program
        program.use_program();

        // Specify the layout of the vertex data
        let pos_attr = program.attr_location("position");
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(
            pos_attr as GLuint,
            2,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            0,
            ptr::null(),
        );
        
        // Bind output texture in Texture Unit 1
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, output_tex.id);

        // Set our "texture" sampler to use Texture Unit 1
        let output_tex_id = program.uniform_location("img_output");
        gl::Uniform1i(output_tex_id, 1);

        mode_id = compute_program.uniform_location("mode");

        // Bind input buffer
        compute_program.use_program();
        let volume_size_id = compute_program.uniform_location("volume_size");
        gl::Uniform3ui(volume_size_id, volume.xsize, volume.ysize, volume.zsize);
    }

    let mut mode: u32 = 0;
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut mode);
        }

        unsafe {
            // Clear the screen to black
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Compute next state of game
            compute_program.use_program();
            pointmap_buffer.bind(0);
            gl::Uniform1i(mode_id, mode as i32);
            gl::BindImageTexture(
                1,
                output_tex.id as GLuint,
                0,
                gl::FALSE,
                0,
                gl::WRITE_ONLY,
                gl::RGBA8,
            );
            gl::DispatchCompute(image_dimensions.0, image_dimensions.1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);

            // Draw a quad from the two triangles
            program.use_program();
            ebo.draw();
        }
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
