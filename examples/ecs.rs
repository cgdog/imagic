/// Test hecs example
use hecs::World;

fn main() {
    let mut world = World::new();
    // Nearly any type can be used as a component with zero boilerplate
    let a = world.spawn((123, true, "abc"));
    let b = world.spawn((42, false));
    // Systems can be simple for loops
    for (_id, (number, &flag)) in world.query_mut::<(&mut i32, &bool)>() {
        if flag {
            *number *= 2;
        }
    }
    // Random access is simple and safe
    assert_eq!(*world.get::<&i32>(a).unwrap(), 246);
    assert_eq!(*world.get::<&i32>(b).unwrap(), 42);
}
