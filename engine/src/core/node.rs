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
    pub name: String,
    pub id: NodeHandle,

    pub parent: Option<NodeHandle>,
    pub children: Option<Vec<NodeHandle>>,

    pub enabled: bool,
    pub enabled_in_hierarchy: bool,

    pub layer: Layer,

    // built in components
    /// A node always has a Transform component.
    pub transform: Transform,

    // custom components
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

    pub fn has_child(&self, child: &NodeHandle) -> bool {
        if let Some(children) = &self.children {
            children.iter().any(|node_id| node_id == child)
        } else {
            false
        }
    }

    pub(crate) fn on_update(&mut self, parent_model_matrix: Option<Mat4>) {
        self.transform.update_model_matrix(parent_model_matrix);
    }
}
