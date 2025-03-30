use std::{collections::HashMap, hash::Hash};

use crate::ecs::world::World;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ScheduleStage {
    Init,
    Update,
    LateUpdate,
    FixedUpdate,
}

pub trait System {
    fn run(&mut self, schedule_stage: &ScheduleStage, world: &mut World);
}

type PlainFuncSystemType = fn(world: &mut World);

impl System for PlainFuncSystemType {
    fn run(&mut self, _schedule_stage: &ScheduleStage, world: &mut World) {
        self(world);
    }
}

type ObjectSystemFuncType<T> = fn (instance: &mut T, world: &mut World);
struct ObjectSystem<T> {
    object: T,
    systems: HashMap<ScheduleStage, Vec<ObjectSystemFuncType<T>>>,
}

impl<T> ObjectSystem<T> {
    pub fn new(object: T, systems_vec: Vec<(ScheduleStage, ObjectSystemFuncType<T>)>) -> Self {
        let mut systems: HashMap<ScheduleStage, Vec<ObjectSystemFuncType<T>>> = HashMap::new();
        for (schedule_stage, system) in systems_vec {
            if let Some(cur_systems) = systems.get_mut(&schedule_stage) {
                cur_systems.push(system);
            } else {
                systems.insert(schedule_stage, vec![system]);
            }
        }
        Self { object, systems }
    }
}

impl<T> System for ObjectSystem<T> {
    fn run(&mut self, schedule_stage: &ScheduleStage, world: &mut World) {
        if let Some(cur_systems) = self.systems.get_mut(schedule_stage) {
            for system in cur_systems {
                system(&mut self.object, world);
            }
        }
    }
}

pub struct Systems {
    systems: HashMap<ScheduleStage, Vec<Box<dyn System>>>,
    object_systems: Vec<Box<dyn System>>,
}

impl Systems {
    pub fn new() -> Self {
        Self { systems: HashMap::new(), object_systems: Vec::new() }
    }

    fn register_system_internal(&mut self, system_schedule: ScheduleStage, system: Box<dyn System>) {
        if let Some(systems) = self.systems.get_mut(&system_schedule) {
            systems.push(system);
        } else {
            self.systems.insert(system_schedule, vec![system]);
        }
    }
    pub fn register_system(&mut self, system_schedule: ScheduleStage, system: PlainFuncSystemType) {
        self.register_system_internal(system_schedule, Box::new(system));
    }

    pub fn register_object_system<T: 'static>(&mut self, object: T, systems: Vec<(ScheduleStage, ObjectSystemFuncType<T>)>) {
        self.object_systems.push(Box::new(ObjectSystem::<T>::new(object, systems)));
    }


    pub fn run_schedule(&mut self, schedule_stage: &ScheduleStage, world: &mut World) {
        if let Some(systems) = self.systems.get_mut(schedule_stage) {
            for system in systems.iter_mut() {
                system.run(schedule_stage, world);
            }
        }

        for object_system in self.object_systems.iter_mut() {
            object_system.run(schedule_stage, world);
        }
    }
}