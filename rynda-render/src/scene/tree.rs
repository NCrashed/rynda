use super::node::SceneNode;

/// Root of scene hierarchy that defines how the rendering objects are located
/// relatively to each other.
pub struct SceneTree {
    /// Root of hierarchy. Effectively defines transformation of whole world.
    pub root: SceneNode,
}

impl SceneTree {
    pub fn new() -> Self {
        SceneTree {
            root: SceneNode::new(),
        }
    }
}

impl Default for SceneTree {
    fn default() -> Self {
        SceneTree::new()
    }
}