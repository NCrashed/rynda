use glam::{Mat4, Quat, Vec3, Vec4};
use std::ops::Mul;

/// Contains scale-rotation-translation information
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub scale: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub translation: Vec3,
}

impl Transform {
    /// Identity matrix transformation
    pub fn new() -> Self {
        Transform {
            scale: Vec3::ONE,
            forward: -Vec3::Z,
            up: Vec3::Y,
            translation: Vec3::ZERO,
        }
    }

    /// Transformation that defines position and target the rotation will face to
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        Transform {
            scale: Vec3::ONE,
            forward: (target - eye).normalize(),
            translation: eye,
            up,
        }
    }

    /// Construct only offset translation
    pub fn translation(translation: Vec3) -> Self {
        Transform {
            translation: -translation,
            ..Transform::default()
        }
    }

    /// Get rotation matrix for the transformation
    pub fn rotation(&self) -> Mat4 {
        Mat4::look_at_rh(self.translation, self.translation + self.forward, self.up)
    }

    /// Get matrix that transforms vectors with the given scale-rotation-translation
    pub fn matrix(&self) -> Mat4 {
        self.rotation() * Mat4::from_scale(self.scale)
    }

    /// Get right vector direction normalized
    pub fn right(&self) -> Vec3 {
        self.forward.cross(Vec3::Y).normalize()
    }

    /// Get left vector direction normalized
    pub fn left(&self) -> Vec3 {
        Vec3::Y.cross(self.forward).normalize()
    }

    /// Get down vector direction normalized
    pub fn down(&self) -> Vec3 {
        -self.up
    }

    /// Rotate forward vector to aply rotation in the quaternion
    pub fn rotate(&mut self, quat: Quat) {
        self.forward = quat * self.forward;
    }

    /// Add translation to the transformation
    pub fn translate(&mut self, offset: Vec3) {
        self.translation += offset;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform::new()
    }
}

/// Things that has transformation inside
pub trait HasTransform {
    fn transformation(&self) -> Transform;
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Transform {
        Transform {
            scale: self.scale * rhs.scale,
            forward: rhs.rotation().transform_vector3(self.forward),
            up: rhs.rotation().transform_vector3(self.up),
            translation: self.translation + rhs.translation,
        }
    }
}

impl Mul<Vec3> for Transform {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        self.matrix().project_point3(rhs)
    }
}

impl Mul<Vec4> for Transform {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        self.matrix() * rhs
    }
}

/// Things that can be mutually transformed
pub trait Transformable {
    fn transform(&mut self, t: &Transform);
}

impl Transformable for Vec3 {
    fn transform(&mut self, t: &Transform) {
        *self = t.matrix().project_point3(*self);
    }
}

impl Transformable for Vec4 {
    fn transform(&mut self, t: &Transform) {
        *self = t.matrix() * *self;
    }
}
