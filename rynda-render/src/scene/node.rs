use crate::math::aabb::AABB;
use crate::math::transform::Transform;

/// Single node in scene tree
pub struct SceneNode {
    pub transform: Transform,
    pub boundaries: AABB,
    pub children: Vec<SceneNode>,
}

impl SceneNode {
    pub fn new() -> Self {
        SceneNode {
            transform: Transform::new(),
            boundaries: AABB::new(),
            children: vec![],
        }
    }
}

impl Default for SceneNode {
    fn default() -> Self {
        SceneNode::new()
    }
}
