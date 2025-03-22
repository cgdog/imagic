use std::any::TypeId;

use super::{types::TupleTypesInfo, world::World};

#[derive(Default)]
pub struct Query {
    all_components: Option<Vec<TypeId>>,
    any_components: Option<Vec<TypeId>>,
    exclude_components: Option<Vec<TypeId>>,
}

impl Query {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn with<T: TupleTypesInfo>(&mut self) -> &mut Self {
        let type_ids = T::type_ids();
        self.all_components = Some(type_ids);
        self
    }

    pub fn any<T: TupleTypesInfo>(&mut self) -> &mut Self {
        let type_ids = T::type_ids();
        self.any_components = Some(type_ids);
        self
    }

    pub fn exclude<T: TupleTypesInfo>(&mut self) -> &mut Self {
        let type_ids = T::type_ids();
        self.exclude_components = Some(type_ids);
        self
    }

    pub fn query(_world: &World) {

    }
}