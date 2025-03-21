use std::u32;

use super::entity::Entity;

pub struct SparseSet<T> {
    sparse: Vec<u32>,       // 稀疏数组：实体 ID -> 密集索引
    pub(crate) dense: Vec<Entity>,     // 密集数组实体列表
    pub(crate) data: Vec<T>,           // 密集存储的组件数据
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            data: Vec::new()
        }
    }
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert(&mut self, entity: Entity, component: T) -> Option<T> {
        let entity_id = entity.id as usize;
        if entity_id >= self.sparse.len() {
            self.sparse.resize(entity_id + 1, u32::MAX);
        }

        let dense_index = self.sparse[entity_id];
        if dense_index != u32::MAX {
            let old = std::mem::replace(&mut self.data[dense_index as usize], component);
            return Some(old);
        }

        self.sparse[entity_id] = self.dense.len() as u32;
        self.dense.push(entity);
        self.data.push(component);

        None
    }

    pub fn get(& self, entity: Entity) -> Option<& T> {
        let dense_index = *self.sparse.get(entity.id as usize)?;
        if dense_index == u32::MAX || self.dense[dense_index as usize] != entity {
            return None;
        }
        Some(& self.data[dense_index as usize])
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let dense_index = *self.sparse.get(entity.id as usize)?;
        if dense_index == u32::MAX || self.dense[dense_index as usize] != entity {
            return None;
        }
        Some(&mut self.data[dense_index as usize])
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Entity, &T)> {
        self.dense.iter().zip(self.data.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut Entity, &mut T)> {
        self.dense.iter_mut().zip(self.data.iter_mut())
    }

    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let dense_index = self.sparse.get(entity.id as usize)?;
        if *dense_index == u32::MAX || self.dense[*dense_index as usize] != entity {
            return None;
        }

        // 将最后一个元素移到被删除的位置
        let last_index = self.dense.len() - 1;
        let last_entity_id = self.dense[last_index].id;

        // Perform the swap and remove operations
        self.dense.swap_remove(*dense_index as usize);
        let removed_data = self.data.swap_remove(*dense_index as usize);

        // Update the sparse array after the swap
        self.sparse[last_entity_id as usize] = *dense_index;
        self.sparse[entity.id as usize] = u32::MAX;

        Some(removed_data)
    }
}