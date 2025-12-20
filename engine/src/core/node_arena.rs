use crate::core::{Node, NodeId};

/// A container for managing nodes in a scene.
pub struct NodeArena {
    nodes: Vec<Option<Node>>,
    free_list: Vec<u32>,
}
impl NodeArena {
    /// Create a new empty NodeArena.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Create a new node with the given name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the node.
    /// 
    /// # Returns
    /// 
    /// * `NodeID` - The ID of the newly created node.
    pub fn create_node(&mut self, name: impl Into<String>) -> NodeId {
        let mut node = Node {
            parent: None,
            children: None,
            name: name.into(),
            ..Default::default()
        };

        if let Some(index) = self.free_list.pop() {
            let node_id = NodeId::new(index);
            node.id = node_id;
            self.nodes[index as usize] = Some(node);
            node_id
        } else {
            let id = self.nodes.len() as u32;
            let node_id = NodeId::new(id);
            node.id = node_id;
            self.nodes.push(Some(node));
            node_id
        }
    }

    /// Get a reference to the node with the given ID.
    /// 
    /// # Panics
    /// 
    /// Panics if the given ID does not correspond to a valid node.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the node to retrieve.
    /// 
    /// # Returns
    /// 
    /// * `&Node` - A reference to the node with the given ID.
    pub fn get_forcely(&self, id: &NodeId) -> &Node {
        self.nodes[id.0 as usize]
            .as_ref()
            .expect("Invalid NodeID when NodeArena get()")
    }

    /// Get a mutable reference to the node with the given ID.
    /// 
    /// # Panics
    /// 
    /// Panics if the given ID does not correspond to a valid node.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the node to retrieve.
    /// 
    /// # Returns
    /// 
    /// * `&mut Node` - A mutable reference to the node with the given ID.
    pub fn get_mut_forcely(&mut self, id: &NodeId) -> &mut Node {
        self.nodes[id.0 as usize]
            .as_mut()
            .expect("Invalid NodeID when NodeArena get_mut()")
    }

    /// Get a reference to the node with the given ID, if it exists.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the node to retrieve.
    /// 
    /// # Returns
    /// 
    /// * `Option<&Node>` - A reference to the node with the given ID, if it exists.
    pub fn get(&self, id: &NodeId) -> Option<&Node> {
        self.nodes[id.0 as usize].as_ref()
    }

    /// Get a mutable reference to the node with the given ID, if it exists.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the node to retrieve.
    /// 
    /// # Returns
    /// 
    /// * `Option<&mut Node>` - A mutable reference to the node with the given ID, if it exists.
    pub fn get_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.nodes[id.0 as usize].as_mut()
    }

    /// Attach a child node to a parent node.
    /// 
    /// # Arguments
    /// 
    /// * `parent` - The ID of the parent node.
    /// * `child` - The ID of the child node to attach.
    /// 
    /// # Returns
    /// 
    /// * `bool` - Returns `true` if the attachment was successful, `false` otherwise.
    pub fn attach_to_parent(&mut self, parent: NodeId, child: &NodeId) -> bool {
        if self.detach_from_parent(child) {
            self.get_mut_forcely(child).parent = Some(parent);
            if let Some(parent) = self.get_mut(&parent) {
                if let Some(children) = &mut parent.children {
                    children.push(*child);
                } else {
                    parent.children = Some(vec![*child]);
                }
                true
            } else {
                log::warn!("Invalid parent NodeID ({}) when NodeArena attach()", parent);
                false
            }
        } else {
            false
        }
    }

    /// Detach a child node from its parent.
    /// 
    /// # Arguments
    /// 
    /// * `child_id` - The ID of the child node to detach.
    /// 
    /// # Returns
    /// 
    /// * `bool` - Returns `true` if the detachment was successful or no parent, `false` otherwise (the child_id is invalid).
    pub fn detach_from_parent(&mut self, child_id: &NodeId) -> bool {
        if let Some(child) = self.get(child_id) && let Some(parent_id) = child.parent
            && let Some(parent) = self.get_mut(&parent_id)
            && let Some(siblings) = &mut parent.children {
            siblings.retain(|&c| c != *child_id);
        }
        if let Some(child) = self.get_mut(child_id) {
            child.parent = None;
            true
        } else {
            log::warn!("Invalid NodeID ({}) when NodeArena detach()", child_id);
            false
        }
    }

    /// Destroy a node and its children.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The ID of the node to destroy.
    /// 
    /// # Returns
    /// 
    /// * `bool` - Returns `true` if the destruction was successful, `false` otherwise.
    pub fn destroy_node(&mut self, id: &NodeId) -> bool {
        if let Some(node) = self.get_mut(id) {
            if let Some(children) = node.children.take() {
                for c in &children {
                    self.destroy_node(c);
                }
            }

            self.detach_from_parent(id);
            self.nodes[id.0 as usize] = None;
            self.free_list.push(id.0);
            true
        } else {
            log::warn!("Invalid NodeID ({}) when NodeArena destroy()", id);
            false
        }
        
    }
}
