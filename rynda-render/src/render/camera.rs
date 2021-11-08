use glam::f32::{Mat4, Quat, Vec3};

/// Contains information that required for conversion from world to screen space coordinates.
pub struct Camera {
    pub scale: Vec3,
    pub forward: Vec3,
    pub translation: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    /// Create default camera
    pub fn new() -> Self {
        Camera {
            scale: Vec3::ONE,
            forward: Vec3::Z,
            translation: Vec3::ZERO,
            fov: std::f32::consts::PI / 6.0,
            aspect: 1.0,
            near: 0.01,
            far: 100.0,
        }
    }

    /// Create camera at position looking given direction
    pub fn look_at(eye: Vec3, target: Vec3) -> Self {
        Camera {
            scale: Vec3::ONE,
            forward: (target - eye).normalize(),
            translation: eye,
            fov: std::f32::consts::PI / 6.0,
            aspect: 1.0,
            near: 0.01,
            far: 100.0,
        }
    }

    /// Return view matrix that transforms from world space to camera space
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from_scale(self.scale)
            * Mat4::look_at_rh(self.translation, self.translation + self.forward, Vec3::Y)
    }

    /// Return projection matrix (perspective) that transforms from camera space to screen space
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov, self.aspect, self.near, self.far)
    }

    /// Return projection-view matrix that transforms from world coords to screen space
    pub fn matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Get right vector direction normalized
    pub fn right(&self) -> Vec3 {
        self.forward.cross(Vec3::Y).normalize()
    }

    /// Update rotation of camera according to dx and dy displacment of cursor
    pub fn update_cursor(&mut self, dx: f64, dy: f64) {
        if dx.abs() >= std::f64::EPSILON {
            self.forward = Quat::from_rotation_y(-dx as f32) * self.forward;
        }
        if dy.abs() >= std::f64::EPSILON {
            self.forward = Quat::from_axis_angle(-self.right(), dy as f32) * self.forward;
        }
    }

    /// Move camera forward by given amount
    pub fn move_forward(&mut self, dv: f64) {
        self.translation += self.forward * (dv as f32);
    }

    /// Move camera forward by given amount
    pub fn move_right(&mut self, dv: f64) {
        self.translation += self.right() * (dv as f32);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
