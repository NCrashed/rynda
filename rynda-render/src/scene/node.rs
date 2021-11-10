use crate::math::transform::Transform;

/// Single node in scene tree
pub struct SceneNode {
    pub transform: Transform,
}

impl SceneNode {
    pub fn new() -> Self {
        SceneNode {
            transform: Transform::new(),
        }
    }
}

impl Default for SceneNode {
    fn default() -> Self {
        SceneNode::new()
    }
}