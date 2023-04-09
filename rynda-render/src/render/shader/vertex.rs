use gl::types::*;
use glam::{Vec2, Vec3};

/// Trait that maps rust types into OpenGL vertex attributes types
pub trait VertexAttribute {
    /// Primitive component type of the attribute. E.x. GLfloat
    type Element;

    /// Count of primitive elements in one attribute value. E.x. 3 for vec3
    fn elements_count() -> GLint;

    /// ID of type in GL format. E.x. GL_FLOAT
    fn element_type_id() -> GLuint;
}

impl VertexAttribute for GLint {
    type Element = GLint;

    fn elements_count() -> GLint {
        1
    }

    fn element_type_id() -> GLuint {
        gl::INT
    }
}

impl VertexAttribute for GLshort {
    type Element = GLshort;

    fn elements_count() -> GLint {
        1
    }

    fn element_type_id() -> GLuint {
        gl::SHORT
    }
}

impl VertexAttribute for GLfloat {
    type Element = GLfloat;

    fn elements_count() -> GLint {
        1
    }

    fn element_type_id() -> GLuint {
        gl::FLOAT
    }
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
