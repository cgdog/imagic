
pub type RR<T> = std::rc::Rc<std::cell::RefCell<T>>;
pub type RRB<T> = std::rc::Rc<std::cell::RefCell<Box<T>>>;

/// Option Rc RefCell type.
pub type ORR<T> = Option<std::rc::Rc<std::cell::RefCell<T>>>;
pub type ORRB<T> = Option<std::rc::Rc<std::cell::RefCell<Box<T>>>>;

pub type WR<T> = std::rc::Weak<std::cell::RefCell<T>>;
pub type OWR<T> = Option<std::rc::Weak<std::cell::RefCell<T>>>;

pub type HashID = u64;

#[macro_export]
macro_rules! RRB_new {
    ($object:expr) => {
        std::rc::Rc::new(std::cell::RefCell::new(Box::new($object)))
    };
}

#[macro_export]
macro_rules! RR_new {
    ($object:expr) => {
        std::rc::Rc::new(std::cell::RefCell::new($object))
    };
}

#[macro_export]
macro_rules! ORR_new {
    ($object:expr) => {
        Some(std::rc::Rc::new(std::cell::RefCell::new($object)))
    };
}

#[macro_export]
macro_rules! impl_as_any {
    () => {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    };
}

#[macro_export]
macro_rules! impl_component {
    ($struct_name:ident) => {
        impl $crate::components::component::Component for $struct_name {
            // $crate::impl_as_any!();
        }
    };
}

pub use impl_as_any;
pub use impl_component;
pub use ORR_new;
pub use RR_new;
pub use RRB_new;