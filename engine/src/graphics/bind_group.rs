use std::collections::HashMap;

pub(crate) type BindGroup = wgpu::BindGroup;


pub(crate) type BindGroupID = u32;
pub(crate) const INVALID_BINDGROUP_ID: BindGroupID = BindGroupID::MAX;

type BindGroups = HashMap<BindGroupID, BindGroup>;

/// The BindGroupManager manages all the bind groups in the engine.
/// It provides methods to add, get, and remove bind groups.
/// Each bind group is identified by a unique BindGroupID.
pub(crate) struct BindGroupManager {
    bind_groups: BindGroups,
}

impl BindGroupManager {
    pub(crate) fn new() -> Self {
        Self {
            bind_groups: BindGroups::new(),
        }
    }

    /// Add a bind group.
    /// # Arguments
    /// 
    /// * `bind_group` - The bind group to add.
    /// # Returns
    /// 
    /// * `BindGroupID` - The ID of the added bind group.
    pub(crate) fn add(&mut self, bind_group: BindGroup) -> BindGroupID{
        static mut ID: BindGroupID = 0;
        let cur_id: BindGroupID;
        unsafe {
            cur_id = ID;
            ID += 1;
        }
        self.bind_groups.insert(cur_id, bind_group);
        cur_id
    }

    pub(crate) fn get(&self, id: &BindGroupID) -> Option<&BindGroup> {
        self.bind_groups.get(id)
    }

    #[allow(unused)]
    pub(crate) fn get_mut(&mut self, id: &BindGroupID) -> Option<&mut BindGroup> {
        self.bind_groups.get_mut(id)
    }

    pub(crate) fn remove(&mut self, id: &BindGroupID) -> Option<BindGroup> {
        self.bind_groups.remove(&*id)
    }
}