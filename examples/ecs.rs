use std::any::TypeId;

use imagic::ecs::world::World;
use log::info;

#[derive(Debug)]
pub struct Name {
    name: &'static str
}

impl Name {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug)]
struct Velocity {
    speed: f32,
}

impl Velocity {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }
}
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut world = World::new();
    // Nearly any type can be used as a component with zero boilerplate
    let entity_a = world.spawn_with_component(Position::new(0.0, 0.0));
    world.add_component(entity_a, Name::new("entity a"));
    world.add_component(entity_a, Velocity::new(1.0));
    
    let entity_b = world.spawn();
    world.add_component(entity_b, Name::new("entitry b"));
    world.add_component(entity_b, Position::new(-1.0, 0.0));
    world.add_component(entity_b, Velocity::new(-1.0));

    if let Some(pos_a) = world.get::<Position>(entity_a) {
        info!("pos_a: {:?}", pos_a);
    }

    for (_entity, name) in world.query::<Name>() {
        info!("entity name: {:?}", name);
    }

    // world.query_all::<(i32, f32, &str, u32, &mut Position, Velocity)>();
    // world.query_all::<(Position, &Position, &mut Position, & mut Position, Position)>();
    // world.query_all::<(i32, &i32, &mut i32)>();
    println!("{:?}", TypeId::of::<i32>());
    println!("{:?}", TypeId::of::<& i32>());
    println!("{:?}", TypeId::of::<&mut i32>());
}
