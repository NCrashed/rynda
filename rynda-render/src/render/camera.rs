use crate::math::transform::Transform;
use glam::f32::{Mat4, Quat, Vec3};

/// Contains information that required for conversion from world to screen space coordinates.
pub struct Camera {
    pub transform: Transform,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    /// Create default camera
    pub fn new() -> Self {
        Camera {
            transform: Transform::new(),
            fov: std::f32::consts::PI / 6.0,
            aspect: 1.0,
            near: 0.01,
            far: 100.0,
        }
    }

    /// Create camera at position looking given direction
    pub fn look_at(eye: Vec3, target: Vec3) -> Self {
        Camera {
            transform: Transform::look_at(eye, target, Vec3::Y),
            fov: std::f32::consts::PI / 6.0,
            aspect: 1.0,
            near: 0.01,
            far: 100.0,
        }
    }

    /// Return view matrix that transforms from world space to camera space
    pub fn view_matrix(&self) -> Mat4 {
        self.transform.matrix()
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
        self.transform.right()
    }

    /// Update rotation of camera according to dx and dy displacment of cursor
    pub fn update_cursor(&mut self, dx: f64, dy: f64) {
        if dx.abs() >= std::f64::EPSILON {
            self.transform.rotate(Quat::from_rotation_y(-dx as f32));
        }
        if dy.abs() >= std::f64::EPSILON {
            self.transform.rotate(Quat::from_axis_angle(-self.right(), dy as f32));
        }
    }

    /// Move camera forward by given amount
    pub fn move_forward(&mut self, dv: f64) {
        self.transform.translate(self.transform.forward * (dv as f32));
    }

    /// Move camera forward by given amount
    pub fn move_right(&mut self, dv: f64) {
        self.transform.translate(self.right() * (dv as f32));
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
