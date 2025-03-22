type EntityID = u32;

#[derive(PartialEq, Clone, Copy, Hash, Eq)]
pub struct Entity {
    pub id: EntityID,
    pub generation: u32,
}

impl Entity {

}

pub struct Entities {
    pub generations: Vec<u32>,
    pub free_list: Vec<EntityID>,
}

impl Default for Entities {
    fn default() -> Self {
        Self { generations: Vec::new(), free_list: Vec::new() }
    }
}