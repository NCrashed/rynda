use gl::types::*;
use std::ffi::CString;
use std::{ptr, str};

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
