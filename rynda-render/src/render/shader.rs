use gl::types::*;
use std::ffi::CString;
use std::{ptr, str};
use super::buffer::vertex::VertexBuffer;
use glam::{Vec2, Vec3};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

pub fn shader_type_id(v: ShaderType) -> GLuint {
    match v {
        ShaderType::Vertex => gl::VERTEX_SHADER,
        ShaderType::Fragment => gl::FRAGMENT_SHADER,
        ShaderType::Compute => gl::COMPUTE_SHADER,
    }
}

#[derive(Debug)]
pub struct Shader {
    pub shader_type: ShaderType,
    pub id: GLuint,
}

impl Shader {
    pub fn compile(shader_type: ShaderType, src: &str) -> Self {
        let shader;
        unsafe {
            shader = gl::CreateShader(shader_type_id(shader_type));
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
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                panic!(
                    "{}",
                    str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8")
                );
            }
        }
        Shader {
            shader_type,
            id: shader,
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

/// Trait that maps rust types into OpenGL vertex attributes types
pub trait VertexAttribute {
    /// Primitive component type of the attribute. E.x. GLfloat
    type Element;

    /// Count of primitive elements in one attribute value. E.x. 3 for vec3
    fn elements_count() -> GLint;

    /// ID of type in GL format. E.x. GL_FLOAT
    fn element_type_id() -> GLuint;
}

impl VertexAttribute for Vec2 {
    type Element = GLfloat;

    fn elements_count() -> GLint {
        2
    }

    fn element_type_id() -> GLuint {
        gl::FLOAT
    }
}

impl VertexAttribute for Vec3 {
    type Element = GLfloat;

    fn elements_count() -> GLint {
        3
    }

    fn element_type_id() -> GLuint {
        gl::FLOAT
    }
}

#[derive(Debug)]
pub struct ShaderProgram {
    pub id: GLuint,
    pub shaders: Vec<Shader>,
}

impl ShaderProgram {
    pub fn link(shaders: Vec<Shader>) -> Self {
        let program;
        unsafe {
            program = gl::CreateProgram();
            for s in shaders.iter() {
                gl::AttachShader(program, s.id);
            }
            gl::LinkProgram(program);
            // Get the link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                panic!(
                    "{}",
                    str::from_utf8(&buf).expect("ProgramInfoLog not valid utf8")
                );
            }
        }
        ShaderProgram {
            id: program,
            shaders,
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    /// Get location of vertex attribute in the program
    pub fn attr_location(&self, name: &str) -> GLint {
        let name_cstr = CString::new(name).unwrap();
        unsafe { gl::GetAttribLocation(self.id, name_cstr.as_ptr()) }
    }

    /// Get location of uniform in the program
    pub fn uniform_location(&self, name: &str) -> GLint {
        let name_cstr = CString::new(name).unwrap();
        unsafe { gl::GetUniformLocation(self.id, name_cstr.as_ptr()) }
    }

    /// Binding vertex buffer to given attribute in the program
    pub fn bind_attribute<T: VertexAttribute>(&self, attr_name: &str, buffer: &VertexBuffer<T::Element>) {
        let attr = self.attr_location(attr_name);
        unsafe {
            gl::EnableVertexAttribArray(attr as GLuint);
            buffer.bind();
            gl::VertexAttribPointer(
                attr as GLuint,
                T::elements_count(),
                T::element_type_id(),
                gl::FALSE as GLboolean,
                0,
                ptr::null(),
            );
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

