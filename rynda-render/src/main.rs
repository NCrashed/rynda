extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::{Action, Context, Key};
use std::ffi::CString;
use std::{mem, ptr, str};

use rynda_render::render::{
    shader::{compile_shader, link_program},
    texture::create_texture,
};

// Vertex data
static POSITION_DATA: [GLfloat; 8] = [-1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0];
static INDEX_DATA: [u16; 4] = [1, 2, 0, 3];

// Compute shader sources
static COMPUTE_SRC: &str = "
#version 440 
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

layout (binding = 0) uniform sampler2D img_input;
layout (rgba8, binding = 1) uniform image2D img_output;

void main() {
    ivec2 cell_coord = ivec2(gl_GlobalInvocationID.xy);
    vec4 cell_sample = texelFetch(img_input, cell_coord, 0); 

    vec4 pixel = vec4(0.0, 0.0, 0.0, 1.0);
    bool alive = cell_sample.r > 0;
    int count = 0;
    for(int i = -1; i <= 1; ++i)
    {
        for(int j = -1; j <= 1; ++j)
        {
            if(i == 0 && j == 0)
             continue;
            float tex = texelFetch(img_input, cell_coord + ivec2(i,j), 0).r;
            if(tex > 0)
                ++count;
        }
    }
    float new_cell = cell_sample.r;
    if(count < 2)                                   new_cell = 0.0f;
    else if(alive && (count == 2 || count == 3))    new_cell = 1.0f;
    else if(alive && count > 3)                     new_cell = 0.0f;
    else if(!alive && count == 3)                   new_cell = 1.0f;
    pixel = vec4(new_cell,new_cell,new_cell,1.0f);
 
    imageStore(img_output, cell_coord, pixel);
}
";

// Shader sources
static VS_SRC: &str = "
#version 150
in vec2 position;
out vec2 tex_coords;

const vec2 madd=vec2(0.5,0.5);

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vec2 tex_pos = position.xy;
    tex_pos.y *= -1;
    tex_coords = tex_pos.xy*madd+madd;
}
";

static FS_SRC: &str = "
#version 150
uniform sampler2D img_output;
in vec2 tex_coords;
out vec4 f_color;

void main() {
    f_color = texture(img_output, tex_coords);
}";

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

    // Create GLSL shaders
    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let cs = compile_shader(COMPUTE_SRC, gl::COMPUTE_SHADER);
    let program = link_program(&[vs, fs]);
    let compute_program = link_program(&[cs]);
    // building a texture with "OpenGL" drawn on it
    // let image = image::load(
    //     Cursor::new(&include_bytes!("../assets/life.png")[..]),
    //     image::ImageFormat::Png,
    // )
    // .unwrap()
    // .to_rgba8();
    let image_dimensions = (1024, 1024); // image.dimensions();
    let input_tex_id;
    let output_tex_id;

    let mut vao = 0;
    let mut eab = 0;
    let mut vbo = 0;

    unsafe {
        // Create input texture
        // input_tex_id = create_texture(
        //     gl::TEXTURE0,
        //     image_dimensions.0,
        //     image_dimensions.1,
        //     Some(&image),
        // );
        input_tex_id = create_texture(gl::TEXTURE0, image_dimensions.0, image_dimensions.1, None);
        output_tex_id = create_texture(gl::TEXTURE1, image_dimensions.0, image_dimensions.1, None);

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

        // Bind input texture in Texture Unit 0
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, input_tex_id as GLuint);
        // Bind output texture in Texture Unit 1
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, output_tex_id as GLuint);

        // Set our "texture" sampler to use Texture Unit 1
        let img_output = CString::new("img_output").unwrap();
        let output_tex_id = gl::GetUniformLocation(program, img_output.as_ptr());
        gl::Uniform1i(output_tex_id, 1);

        // Set "img_input" sampler to use Texture Unit 0
        let img_input = CString::new("img_input").unwrap();
        let input_tex_id = gl::GetUniformLocation(compute_program, img_input.as_ptr());
        gl::Uniform1i(input_tex_id, 0);
    }

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }

        unsafe {
            // Compute next state of game
            gl::UseProgram(compute_program);
            gl::BindImageTexture(
                1,
                output_tex_id as GLuint,
                0,
                gl::FALSE,
                0,
                gl::WRITE_ONLY,
                gl::RGBA8,
            );
            gl::DispatchCompute(image_dimensions.0, image_dimensions.1, 1);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);

            // Clear the screen to black
            gl::UseProgram(program);
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw a quad from the two triangles
            gl::DrawElements(
                gl::TRIANGLE_STRIP,
                INDEX_DATA.len() as GLint,
                gl::UNSIGNED_SHORT,
                ptr::null(),
            );

            // Copy to the next step
            gl::CopyImageSubData(
                output_tex_id,
                gl::TEXTURE_2D,
                0,
                0,
                0,
                0,
                input_tex_id,
                gl::TEXTURE_2D,
                0,
                0,
                0,
                0,
                image_dimensions.0 as GLint,
                image_dimensions.1 as GLint,
                1,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        window.swap_buffers();
    }

    unsafe {
        gl::DeleteProgram(program);
        gl::DeleteProgram(compute_program);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        gl::DeleteShader(cs);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &eab);
        gl::DeleteTextures(1, &input_tex_id);
        gl::DeleteTextures(1, &output_tex_id);
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
