use std::collections::HashMap;

pub type BindGroup = wgpu::BindGroup;


pub type BindGroupID = u32;
pub const INVALID_BINDGROUP_ID: BindGroupID = BindGroupID::MAX;

type BindGroups = HashMap<BindGroupID, BindGroup>;

/// The BindGroupManager manages all the bind groups in the engine.
/// It provides methods to add, get, and remove bind groups.
/// Each bind group is identified by a unique BindGroupID.
pub struct BindGroupManager {
    bind_groups: BindGroups,
}

impl BindGroupManager {
    pub fn new() -> Self {
        Self {
            bind_groups: BindGroups::new(),
        }
    }

    /// Add a bind group.
    /// TODO: add remove bind group method. use arena to manage bind groups.
    /// # Arguments
    /// 
    /// * `bind_group` - The bind group to add.
    /// # Returns
    /// 
    /// * `BindGroupID` - The ID of the added bind group.
    pub fn add(&mut self, bind_group: BindGroup) -> BindGroupID{
        static mut ID: BindGroupID = 0;
        let cur_id: BindGroupID;
        unsafe {
            cur_id = ID;
            ID += 1;
        }
        self.bind_groups.insert(cur_id, bind_group);
        cur_id
    }

    pub fn get(&self, id: &BindGroupID) -> Option<&BindGroup> {
        self.bind_groups.get(id)
    }

    pub fn get_mut(&mut self, id: &BindGroupID) -> Option<&mut BindGroup> {
        self.bind_groups.get_mut(id)
    }

    pub fn remove(&mut self, id: &BindGroupID) -> Option<BindGroup> {
        self.bind_groups.remove(&*id)
    }
}