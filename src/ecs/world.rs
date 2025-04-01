use crate::{asset::asset_manager::AssetManager, prelude::ImagicContext};

use super::{component::Components, entity::{Entities, Entity}, types::TupleTypesInfo};

#[derive(Default)]
pub struct World {
    entities: Entities,
    components: Components,
    context: ImagicContext,
}

impl World {
    pub fn new() -> Self {
        World::default()
    }

    pub fn context(&self) -> &ImagicContext {
        &self.context
    }
    pub fn context_mut(&mut self) -> &mut ImagicContext {
        &mut self.context
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

    pub fn despawn(&mut self, entity: Entity) {
        if self.entities.is_valid(&entity){
            self.components.remove_all(entity);
            self.entities.generations[entity.id as usize] += 1;
            self.entities.free_list.push(entity.id);
        }
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

    /// Get all entities with all given components.
    /// 
    /// Usage: let entities = get_all<(i32, Transform, u32)>();
    pub fn get_all<T: TupleTypesInfo>(&mut self) -> Option<Vec<Entity>> {
        self.components.get_all::<T>()
    }

    pub fn query<T: 'static>(&self) -> impl Iterator<Item = (&Entity, & T)> {
        self.components.iter()
    }

    pub fn query_mut<T: 'static>(&mut self) -> impl Iterator<Item = (&mut Entity, &mut T)> {
        self.components.iter_mut()
    }

    pub fn asset_manager(&self) -> &AssetManager {
        self.context.asset_manager()
    }

    pub fn asset_manager_mut(&mut self) -> &mut AssetManager {
        self.context_mut().asset_manager_mut()
    }

}