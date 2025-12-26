use crate::core::{Node, NodeHandle};

struct NodeSlot {
    node: Option<Node>,
    generation: u32,
}

/// A container for managing nodes in a scene.
pub struct NodeArena {
    nodes: Vec<NodeSlot>,
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
    pub fn create_node(&mut self, name: impl Into<String>) -> NodeHandle {
        let mut node = Node {
            parent: None,
            children: None,
            name: name.into(),
            ..Default::default()
        };

        if let Some(index) = self.free_list.pop() {
            let node_mut_ref = &mut self.nodes[index as usize];
            let node_handle = NodeHandle::new_with_generation(index as u64, node_mut_ref.generation);
            node.id = node_handle;
            node_mut_ref.node = Some(node);
            node_handle
        } else {
            let id = self.nodes.len() as u64;
            let node_handle = NodeHandle::new(id);
            node.id = node_handle;
            self.nodes.push(NodeSlot { node: Some(node), generation: 0 });
            node_handle
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
    pub fn get_forcely(&self, node_handle: &NodeHandle) -> &Node {
        let slot = self.nodes.get(node_handle.id as usize)
            .expect("Invalid NodeID when NodeArena get()");
        if slot.generation != node_handle.generation {
            panic!("Invalid NodeID generation ({}) when NodeArena get()", node_handle);
        } else {
            slot.node.as_ref()
                .expect("Invalid NodeID when NodeArena get()")
        }
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
    pub fn get_mut_forcely(&mut self, node_handle: &NodeHandle) -> &mut Node {
        let slot = self.nodes.get_mut(node_handle.id as usize)
            .expect("Invalid NodeID when NodeArena get_mut()");
        if slot.generation != node_handle.generation {
            panic!("Invalid NodeID generation ({}) when NodeArena get_mut()", node_handle);
        } else {
            slot.node.as_mut()
                .expect("Invalid NodeID when NodeArena get_mut()")
        }
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
    pub fn get(&self, node_handle: &NodeHandle) -> Option<&Node> {
        let slot = self.nodes.get(node_handle.id as usize)?;
        if slot.generation != node_handle.generation {
            None
        } else {
            slot.node.as_ref()
        }
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
    pub fn get_mut(&mut self, node_handle: &NodeHandle) -> Option<&mut Node> {
        let slot = self.nodes.get_mut(node_handle.id as usize)?;
        if slot.generation != node_handle.generation {
            None
        } else {
            slot.node.as_mut()
        }
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
    pub fn attach_to_parent(&mut self, parent: NodeHandle, child: &NodeHandle) -> bool {
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
    pub fn detach_from_parent(&mut self, child_id: &NodeHandle) -> bool {
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
    pub fn destroy_node(&mut self, node_handle: &NodeHandle) -> bool {
        self.detach_from_parent(node_handle);
        if node_handle.id >= self.nodes.len() as u64 || self.nodes[node_handle.id as usize].generation != node_handle.generation {
            #[cfg(debug_assertions)]
            {
                log::warn!("Invalid NodeID generation ({}) when NodeArena destroy()", node_handle);
            }
            return false;
        }

        if let Some(node) = self.nodes[node_handle.id as usize].node.take() {
            if let Some(children) = node.children {
                for c in &children {
                    self.destroy_node(c);
                }
            }
            self.nodes[node_handle.id as usize].generation += 1;
            self.free_list.push(node_handle.id as u32);
            true
        } else {
            #[cfg(debug_assertions)]
            {
                log::warn!("Invalid NodeID ({}) when NodeArena destroy(). Node of slot is None.", node_handle);
            }
            false
        }        
    }
}
