use gl::types::*;
use std::os::raw::c_void;

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

/// Enables debug printing of OpenGL errors
pub fn enable_gl_debug() {
    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(debug_print, std::ptr::null());
    }
}