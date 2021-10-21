extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::{Action, Context, Key};
use std::ffi::CString;
use std::{io::Cursor, mem, ptr, str};

// Vertex data
static POSITION_DATA: [GLfloat; 8] = [-1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0];
static INDEX_DATA: [u16; 4] = [1, 2, 0, 3];

// Shader sources
static VS_SRC: &'static str = "
#version 150
in vec2 position;
out vec2 tex_coords;

const vec2 madd=vec2(0.5,0.5);

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    tex_coords = position.xy*madd+madd;
}
";

static FS_SRC: &'static str = "
#version 150
uniform sampler2D tex;
in vec2 tex_coords;
out vec4 f_color;

void main() {
    f_color = texture(tex, tex_coords);
}";

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("ProgramInfoLog not valid utf8")
            );
        }
        program
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    let (mut window, events) = glfw
        .create_window(1024, 1024, "Game of Life", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.make_current();
    window.set_framebuffer_size_polling(true);

    // Load the OpenGL function pointers3
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    // Create GLSL shaders
    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);

    // building a texture with "OpenGL" drawn on it
    let image = image::load(
        Cursor::new(&include_bytes!("../assets/life.png")[..]),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let mut texture_id = 0;

    let mut vao = 0;
    let mut eab = 0;
    let mut vbo = 0;

    unsafe {
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        let image_dimensions = image.dimensions();
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as GLint,
            image_dimensions.0 as GLint,
            image_dimensions.1 as GLint,
            0,
            gl::BGRA,
            gl::UNSIGNED_BYTE,
            mem::transmute(image.as_ptr()),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
        // ... which requires mipmaps. Generate them automatically.
        gl::GenerateMipmap(gl::TEXTURE_2D);

        // Create Vertex Array Object
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // Create a Vertex Buffer Object and copy the vertex data to it
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (POSITION_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&POSITION_DATA[0]),
            gl::STATIC_DRAW,
        );

        // Create buffer for indecies and fill data to it
        gl::GenBuffers(1, &mut eab);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, eab);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (INDEX_DATA.len() * mem::size_of::<GLshort>()) as GLsizeiptr,
            mem::transmute(&INDEX_DATA[0]),
            gl::STATIC_DRAW,
        );

        // Use shader program
        gl::UseProgram(program);
        let out_color = CString::new("out_color").unwrap();
        gl::BindFragDataLocation(program, 0, out_color.as_ptr());

        // Specify the layout of the vertex data
        let position = CString::new("position").unwrap();
        let pos_attr = gl::GetAttribLocation(program, position.as_ptr());
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(
            pos_attr as GLuint,
            2,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            0,
            ptr::null(),
        );

        // Bind our texture in Texture Unit 0
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        // Set our "texture" sampler to use Texture Unit 0
        let texture_pos = CString::new("texture").unwrap();
        let texture_id = gl::GetUniformLocation(program, texture_pos.as_ptr());
        gl::Uniform1i(texture_id, 0);
    }

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }

        unsafe {
            // Clear the screen to black
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw a quad from the two triangles
            gl::DrawElements(
                gl::TRIANGLE_STRIP,
                INDEX_DATA.len() as GLint,
                gl::UNSIGNED_SHORT,
                ptr::null(),
            );
        }
        window.swap_buffers();
    }

    unsafe {
        gl::DeleteProgram(program);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &eab);
        gl::DeleteTextures(1, &texture_id);
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, width, height);
        },
        _ => {}
    }
}
