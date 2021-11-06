extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::{Action, Context, Key};
use ndarray::{arr3, Array3};
use std::ffi::CString;
use std::os::raw::c_void;
use std::{mem, ptr, str};

use rynda_format::types::{volume::RleVolume, voxel::RgbVoxel};
use rynda_render::render::{
    buffer::ShaderBuffer,
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

uniform uvec3 volume_size;
uniform int mode;

struct PointerColumn{
    uint pointer;
    uint fields; // unpacked fields
};

layout (shared, binding = 0) readonly buffer InputData {
    PointerColumn columns[];
};

layout (rgba8, binding = 1) uniform image2D img_output;

uint rle_count(uint fields) {
    return fields & 0xFFFF;
}

uint skipped(uint fields) {
    return (fields >> 16) & 0x3FF;
}

uint drawn(uint fields) {
    return (fields >> 26) & 0x3F;
}

uint flat_index(uvec2 pos)
{
    return pos.x + pos.y * volume_size.x;
}

void main() {
    uvec2 cell_coord = uvec2(gl_GlobalInvocationID.xy);

    vec4 pixel = vec4(0.0, 0.0, 0.0, 1.0);
    
    PointerColumn pcol = columns[flat_index(cell_coord)];
    float height = 0.0; 
    
    if (mode == 0) {
        height = float(drawn(pcol.fields)) / float(volume_size.y);
    } else {
        height = float(skipped(pcol.fields)) / float(volume_size.y);
    }
    
    pixel.r = height;
    pixel.g = height;
    pixel.b = height;

    imageStore(img_output, ivec2(cell_coord), pixel);
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

extern "system" fn debug_print(
    source: GLenum,
    gltype: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    userParam: *mut c_void,
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
    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let cs = compile_shader(COMPUTE_SRC, gl::COMPUTE_SHADER);
    let program = link_program(&[vs, fs]);
    let compute_program = link_program(&[cs]);

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

    // building a texture with "OpenGL" drawn on it
    // let image = image::load(
    //     Cursor::new(&include_bytes!("../assets/life.png")[..]),
    //     image::ImageFormat::Png,
    // )
    // .unwrap()
    // .to_rgba8();
    let pointmap_buffer;
    let image_dimensions = (volume.xsize, volume.zsize); // image.dimensions();
    let output_tex_id;

    let mut vao = 0;
    let mut eab = 0;
    let mut vbo = 0;

    let mode_id;

    unsafe {
        // Create input texture
        // input_tex_id = create_texture(
        //     gl::TEXTURE0,
        //     image_dimensions.0,
        //     image_dimensions.1,
        //     Some(&image),
        // );
        // input_tex_id = create_texture(gl::TEXTURE0, image_dimensions.0, image_dimensions.1, None);

        pointmap_buffer = ShaderBuffer::from_pointermap(&volume);
        // println!("{:?}", std::slice::from_raw_parts(volume.pointers, (volume.xsize*volume.zsize) as usize) );

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
        // gl::ActiveTexture(gl::TEXTURE0);
        // gl::BindTexture(gl::TEXTURE_2D, input_tex_id as GLuint);

        // Bind input buffer
        gl::UseProgram(compute_program);
        let volume_size = CString::new("volume_size").unwrap();
        let volume_size_id = gl::GetUniformLocation(compute_program, volume_size.as_ptr());
        gl::Uniform3ui(volume_size_id, volume.xsize, volume.ysize, volume.zsize);
        gl::UseProgram(program);
        // Bind output texture in Texture Unit 1
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, output_tex_id as GLuint);

        // Set our "texture" sampler to use Texture Unit 1
        let img_output = CString::new("img_output").unwrap();
        let output_tex_id = gl::GetUniformLocation(program, img_output.as_ptr());
        gl::Uniform1i(output_tex_id, 1);

        let mode_str = CString::new("mode").unwrap();
        mode_id = gl::GetUniformLocation(compute_program, mode_str.as_ptr());

        // // Set "img_input" sampler to use Texture Unit 0
        // let img_input = CString::new("img_input").unwrap();
        // let input_tex_id = gl::GetUniformLocation(compute_program, img_input.as_ptr());
        // gl::Uniform1i(input_tex_id, 0);
    }

    let mut mode: u32 = 0;
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut mode);
        }

        unsafe {
            // Compute next state of game
            gl::UseProgram(compute_program);
            pointmap_buffer.bind(0);
            gl::Uniform1i(mode_id, mode as i32);
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
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
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
            // gl::CopyImageSubData(
            //     output_tex_id,
            //     gl::TEXTURE_2D,
            //     0,
            //     0,
            //     0,
            //     0,
            //     input_tex_id,
            //     gl::TEXTURE_2D,
            //     0,
            //     0,
            //     0,
            //     0,
            //     image_dimensions.0 as GLint,
            //     image_dimensions.1 as GLint,
            //     1,
            // );
            // gl::GenerateMipmap(gl::TEXTURE_2D);
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
        gl::DeleteTextures(1, &output_tex_id);
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
