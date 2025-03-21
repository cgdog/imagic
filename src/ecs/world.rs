use log::info;

use crate::asset::asset_manager::AssetManager;

use super::{component::Components, entity::{Entities, Entity}, types::TupleTypesInfo};

#[derive(Default)]
pub struct World {
    entities: Entities,
    components: Components,
    asset_manager: AssetManager,
}

impl World {
    pub fn new() -> Self {
        World::default()
    }

    pub fn spawn(&mut self) -> Entity {
        let id = self.entities.free_list.pop().unwrap_or_else(|| {
            let id = self.entities.generations.len() as u32;
            self.entities.generations.push(0);
            id
        });
        let generation = self.entities.generations[id as usize];
        Entity { id, generation }
    }

    pub fn spawn_with_component<T: 'static>(&mut self, component: T) -> Entity {
        let entity = self.spawn();
        self.add_component(entity, component);
        entity
    }

    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        self.components.add(entity, component);
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        self.components.remove::<T>(entity);
    }

    pub fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.components.get(entity)
    }

    pub fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components.get_mut(entity)
    }

    pub fn query<T: 'static>(&self) -> impl Iterator<Item = (&Entity, & T)> {
        self.components.iter()
    }

    pub fn query_mut<T: 'static>(&mut self) -> impl Iterator<Item = (&mut Entity, &mut T)> {
        self.components.iter_mut()
    }

    pub fn query_all<T: TupleTypesInfo>(&self) {
        let type_ids = T::type_ids();
        let type_names = T::type_names();
        info!("type ids: {:?}", type_ids);
        info!("type names: {:?}", type_names);
    }

    pub fn get_asset_manager(&self) -> &AssetManager {
        &self.asset_manager
    }

    pub fn get_asset_manager_mut(&mut self) -> &mut AssetManager {
        &mut self.asset_manager
    }

}