use crate::render::buffer::vertex::VertexBuffer;
use gl::types::*;
use std::ffi::CString;
use std::{mem, ptr, str};

use super::{compile::Shader, uniform::UniformValue, vertex::VertexAttribute};

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
        let i = unsafe { gl::GetAttribLocation(self.id, name_cstr.as_ptr()) };
        assert!(i >= 0, "Failed to query attrib location {}", name);
        i
    }

    /// Get location of uniform in the program
    pub fn uniform_location(&self, name: &str) -> GLint {
        let name_cstr = CString::new(name).unwrap();
        let i = unsafe { gl::GetUniformLocation(self.id, name_cstr.as_ptr()) };
        assert!(i >= 0, "Failed to query uniform location {}", name);
        i
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

    pub fn set_uniform<T: UniformValue>(&self, attr_name: &str, value: &T) {
        let slot_id = self.uniform_location(attr_name);
        UniformValue::upload_uniform(slot_id, value);
    }

    pub fn print_uniforms(&self) {
        let mut count = 0;
        unsafe {
            gl::GetProgramiv(self.id, gl::ACTIVE_UNIFORMS, &mut count);
        }
        println!("Active Uniforms: {count}");
        let buf_size: usize = 64;
        let mut uniform_type: GLenum = 0;
        let mut length: GLsizei = 0;
        let mut name: Vec<u8> = vec![0; buf_size];
        let mut size: GLint = 0;

        for i in 0..count {
            unsafe {
                gl::GetActiveUniform(
                    self.id,
                    i as GLuint,
                    buf_size as i32,
                    &mut length,
                    &mut size,
                    &mut uniform_type,
                    mem::transmute(name.as_mut_ptr()),
                );
            }
            let name_utf8: &str =
                std::str::from_utf8(&name[0..length as usize]).expect("utf8 name");
            println!("Uniform type: {uniform_type} name: {name_utf8}");
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
