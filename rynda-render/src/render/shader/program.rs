use crate::render::buffer::vertex::VertexBuffer;
use gl::types::*;
use std::ffi::CString;
use std::{ptr, str};

use super::{shader::Shader, uniform::UniformValue, vertex::VertexAttribute};

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
    pub fn bind_attribute<T: VertexAttribute>(
        &self,
        attr_name: &str,
        buffer: &VertexBuffer<T::Element>,
    ) {
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

    pub fn set_uniform<T: UniformValue>(&mut self, attr_name: &str, value: &T) {
        let slot_id = self.uniform_location(attr_name);
        UniformValue::upload_uniform(slot_id, value);
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}
