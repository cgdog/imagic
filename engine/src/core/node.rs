use ahash::AHashMap;

use crate::{
    components::transform::Transform, core::layer::Layer, math::Mat4, prelude::{ComponentId, ComponentTypeId}
};

/// Tag type for NodeHandle.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum NodeTag {}

/// Handle type for Node.
pub type NodeHandle = crate::types::Handle<NodeTag>;

/// Scene node
pub struct Node {
    /// The name of the node.
    pub name: String,
    /// The unique identifier of the node.
    pub id: NodeHandle,

    /// The parent node handle.
    pub parent: Option<NodeHandle>,
    /// The child node handles.
    pub children: Option<Vec<NodeHandle>>,

    /// Whether the node is enabled.
    pub enabled: bool,
    /// Whether the node is enabled in hierarchy.
    pub enabled_in_hierarchy: bool,

    /// The layer of the node.
    pub layer: Layer,

    // built in components
    /// A node always has a Transform component.
    pub transform: Transform,

    /// Other components attached to the node.
    pub(crate) components: AHashMap<ComponentTypeId, ComponentId>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            name: String::from("Node"),
            id: NodeHandle::INVALID,
            transform: Transform::default(),
            parent: None,
            children: None,
            enabled: true,
            enabled_in_hierarchy: true,
            layer: Layer::default(),
            components: AHashMap::new(),
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{name: {}, id: {}}}", self.name, self.id)
    }
}

impl Node {

    /// Check if the node has a specific child node.
    /// # Arguments
    /// 
    /// * `child` - The child node handle to check.
    /// 
    /// # Returns
    /// 
    /// * `bool` - Returns true if the node has the child node, otherwise false.
    pub fn has_child(&self, child: &NodeHandle) -> bool {
        if let Some(children) = &self.children {
            children.iter().any(|node_id| node_id == child)
        } else {
            false
        }
    }

    /// Lifecycle method called when the node is updated.
    /// # Arguments
    /// 
    /// * `parent_model_matrix` - The model matrix of the parent node, if any.
    pub(crate) fn on_update(&mut self, parent_model_matrix: Option<Mat4>) {
        self.transform.update_model_matrix(parent_model_matrix);
    }
}
