use glam::{Mat4, Quat, Vec3};

/// Contains scale-rotation-translation information
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
            forward: Vec3::Z,
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

    /// Get matrix that transforms vectors with the given scale-rotation-translation
    pub fn matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.translation, self.translation + self.forward, self.up)
            * Mat4::from_scale(self.scale)
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