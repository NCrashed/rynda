use glam::f32::{Vec3, Quat, Mat4};

/// Contains information that required for conversion from world to screen space coordinates. 
pub struct Camera {
    pub scale: Vec3, 
    pub rotation: Quat, 
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
            rotation: Quat::from_rotation_y(0.0),
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
            rotation: Quat::from_rotation_arc((target-eye).normalize(), Vec3::Z),
            translation: eye, 
            fov: std::f32::consts::PI / 6.0,
            aspect: 1.0,
            near: 0.01,
            far: 100.0,
        }
    }

    /// Return view matrix that transforms from world space to camera space
    pub fn view_matrix(&self) -> Mat4 {
        // Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
        Mat4::from_scale(self.scale) * Mat4::from_quat(self.rotation) * Mat4::from_translation(self.translation)
    }

    /// Return projection matrix (perspective) that transforms from camera space to screen space 
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov, self.aspect, self.near, self.far)
    }

    /// Return projection-view matrix that transforms from world coords to screen space
    pub fn matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}