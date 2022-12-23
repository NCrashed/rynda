use gl::types::*;
use glam::{IVec2, IVec3, IVec4, UVec2, UVec3, UVec4, Mat4, Vec2, Vec3, Vec4};

/// Trait that allows to upload value to shader uniform
pub trait UniformValue {
    fn upload_uniform(slot_id: GLint, value: &Self);
}

impl UniformValue for i32 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform1i(slot_id, *value);
        }
    }
}

impl UniformValue for u32 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform1ui(slot_id, *value);
        }
    }
}

impl UniformValue for f32 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform1f(slot_id, *value);
        }
    }
}

impl UniformValue for Mat4 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::UniformMatrix4fv(slot_id, 1, gl::FALSE, value.as_ref().as_ptr());
        }
    }
}

impl UniformValue for Vec4 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform4f(slot_id, value.x, value.y, value.z, value.w);
        }
    }
}

impl UniformValue for Vec3 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform3f(slot_id, value.x, value.y, value.z);
        }
    }
}

impl UniformValue for Vec2 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform2f(slot_id, value.x, value.y);
        }
    }
}

impl UniformValue for IVec4 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform4i(slot_id, value.x, value.y, value.z, value.w);
        }
    }
}

impl UniformValue for IVec3 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform3i(slot_id, value.x, value.y, value.z);
        }
    }
}

impl UniformValue for IVec2 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform2i(slot_id, value.x, value.y);
        }
    }
}

impl UniformValue for UVec4 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform4ui(slot_id, value.x, value.y, value.z, value.w);
        }
    }
}

impl UniformValue for UVec3 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform3ui(slot_id, value.x, value.y, value.z);
        }
    }
}

impl UniformValue for UVec2 {
    fn upload_uniform(slot_id: GLint, value: &Self) {
        unsafe {
            gl::Uniform2ui(slot_id, value.x, value.y);
        }
    }
}
