use imagic::ecs::{system::systems::{ScheduleStage, Systems}, world::World};
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

#[allow(unused)]
#[derive(Debug)]
struct Velocity {
    speed: f32,
}

impl Velocity {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }
}

fn system_1(world: &mut World) {
    info!("test system 1");
    let entities_with_name = world.get_all::<(Name, Position)>();
    if let Some(entities) = entities_with_name {
        for entity in entities.iter() {
            let name = world.get::<Name>(*entity);
            let position = world.get::<Position>(*entity);
            info!("name: {:?} , position: {:?}", name.unwrap(), position.unwrap());
        }
    }
}

fn system_2(world: &mut World) {
    info!("test system 2");
    let entities_with_velocity = world.query::<Velocity>();
    for (_, (_, velocity)) in entities_with_velocity.enumerate() {
        info!("velocity: {:?}", velocity);
    }
}

struct ObjectSystem {
    data: i32,
}

impl ObjectSystem {
    pub fn new(data: i32) -> Self {
        Self { data }
    }

    pub fn object_system_1(&mut self, world: &mut World) {
        info!("object_system_1, data: {}", self.data);
        self.data += 1;
        for (_, name) in world.query::<Name>() {
            info!("object_system_1, name: {:?}", name);
        }
    }

    pub fn object_system_2(&mut self, world: &mut World) {
        info!("object_system_1, data: {}", self.data);
        for (_, pos) in world.query::<Position>() {
            info!("object_system_1, position: {:?}", pos);
        }
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

    if let Some(pos_a) = world.get_mut::<Position>(entity_a) {
        info!("pos_a: {:?}", pos_a);
        pos_a.x = 10.0;
    }
    if let Some(pos_a) = world.get::<Position>(entity_a) {
        info!("pos_a: {:?}", pos_a);
    }

    for (_entity, name) in world.query::<Name>() {
        info!("entity name: {:?}", name);
    }

    let tmp = world.get_all::<(Name, Position)>();
    info!("tmp: {:?}", tmp.is_none());
    info!("test world.get_all::<(Name, Position)>() :");
    if let Some(entities) = world.get_all::<(Name, Position)>() {
        for entity in entities {
            if let Some(name) = world.get::<Name>(entity) {
                info!("name: {}", name.name);
            }
            if let Some(pos) = world.get_mut::<Position>(entity) {
                info!("position: {:?}", *pos);
                pos.x += 100.0;
                pos.y += 1001.0;
            }
        }
    }

    info!("test world.query::<Position>() :");
    for (_, pos) in world.query::<Position>() {
        info!("pos: {:?}", pos);
    }

    info!("test world.query_mut::<Position>() :");
    for (_, pos) in world.query_mut::<Position>() {
        pos.x = -pos.x;
        pos.y = -pos.y;
    }

    info!("test world.query::<Position>() :");
    for (_, pos) in world.query::<Position>() {
        info!("pos: {:?}", pos);
    }


    // world.query_all::<(i32, f32, &str, u32, &mut Position, Velocity)>();
    // world.query_all::<(Position, &Position, &mut Position, & mut Position, Position)>();
    // world.query_all::<(i32, &i32, &mut i32)>();
    // println!("{:?}", TypeId::of::<i32>());
    // println!("{:?}", TypeId::of::<& i32>());
    // println!("{:?}", TypeId::of::<&mut i32>());

    let mut systems = Systems::new();
    systems.register_system(ScheduleStage::Update, system_1);
    systems.register_system(ScheduleStage::Update, system_2);
    let object_system = ObjectSystem::new(100);
    systems.register_object_system::<ObjectSystem>(object_system, 
        vec![(ScheduleStage::Update, ObjectSystem::object_system_1), (ScheduleStage::Update, ObjectSystem::object_system_2)]);
    systems.run_schedule(&ScheduleStage::Update, &mut world);

    info!("despawn entity_b");
    // TODO:despawn has bug.
    world.despawn(entity_b);
    systems.run_schedule(&ScheduleStage::Update, &mut world);
    info!("end");
}