use super::transform::{Transform, Transformable};
use glam::Vec3;
use std::ops::Mul;

/// Axis aligned bounding box. Used for defining rough occupied space, detecting
/// visibility of object and e.t.c.
#[derive(Debug)]
pub struct AABB {
    pub minv: Vec3,
    pub maxv: Vec3,
}

impl AABB {
    pub fn new() -> Self {
        AABB {
            minv: Vec3::ZERO,
            maxv: Vec3::ZERO,
        }
    }

    /// Ensures that minv vector less than maxv
    pub fn normalize(&mut self) {
        let newmin = Vec3::new(self.minv.x.min(self.maxv.x), self.minv.y.min(self.maxv.y), self.minv.z.min(self.maxv.z));
        let newmax = Vec3::new(self.maxv.x.max(self.minv.x), self.maxv.y.max(self.minv.y), self.maxv.z.max(self.minv.z));

        self.minv = newmin;
        self.maxv = newmax;
    }

    /// Get list of lines that shows boundaries of the volume
    pub fn lines(&self) -> Vec<(Vec3, Vec3)> {
        let x0 = self.minv.x;
        let y0 = self.minv.y;
        let z0 = self.minv.z;
        let x1 = self.maxv.x;
        let y1 = self.maxv.y;
        let z1 = self.maxv.z;

        let p0 = Vec3::new(x0, y0, z0);
        let p1 = Vec3::new(x1, y0, z0);
        let p2 = Vec3::new(x0, y1, z0);
        let p3 = Vec3::new(x1, y1, z0);
        let p4 = Vec3::new(x0, y0, z1);
        let p5 = Vec3::new(x1, y0, z1);
        let p6 = Vec3::new(x0, y1, z1);
        let p7 = Vec3::new(x1, y1, z1);

        vec![
            (p0, p1),
            (p0, p2),
            (p1, p3),
            (p2, p3),
            (p4, p5),
            (p4, p6),
            (p5, p7),
            (p6, p7),
            (p0, p4),
            (p1, p5),
            (p2, p6),
            (p3, p7),
        ]
    }
}

impl Default for AABB {
    fn default() -> Self {
        AABB::new()
    }
}

/// Things that has bounding volume
pub trait HasBounding {
    fn bounding_box(&self) -> AABB;
}

impl Transformable for AABB {
    fn transform(&mut self, t: &Transform) {
        let m = t.matrix();
        self.minv = m.project_point3(self.minv);
        self.maxv = m.project_point3(self.maxv);
        self.normalize();
    }
}

impl Mul<AABB> for Transform {
    type Output = AABB;

    fn mul(self, mut rhs: AABB) -> AABB {
        rhs.transform(&self);
        rhs
    }
}
