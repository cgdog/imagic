
pub type RR<T> = std::rc::Rc<std::cell::RefCell<T>>;
pub type RRB<T> = std::rc::Rc<std::cell::RefCell<Box<T>>>;

/// Option Rc RefCell type.
pub type ORR<T> = Option<std::rc::Rc<std::cell::RefCell<T>>>;
pub type ORRB<T> = Option<std::rc::Rc<std::cell::RefCell<Box<T>>>>;

pub type WR<T> = std::rc::Weak<std::cell::RefCell<T>>;
pub type OWR<T> = Option<std::rc::Weak<std::cell::RefCell<T>>>;

pub type HashID = u64;

/// Handle type for resource.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Handle<T> {
    pub(crate) id: u64,
    pub(crate) generation: u32,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Handle<T> {
    pub const INVALID: Self = Self {
        id: u64::MAX,
        generation: 0,
        _marker: std::marker::PhantomData,
    };

    pub(crate) fn new(id: u64) -> Self {
        Self {
            id,
            generation: 0,
            _marker: std::marker::PhantomData,
        }
    }

    pub(crate) fn new_with_generation(id: u64, generation: u32) -> Self {
        Self {
            id,
            generation,
            _marker: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn raw(self) -> u64 {
        self.id
    }
}

impl<T> std::fmt::Display for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle({}) of {}", self.id, std::any::type_name::<T>())
    }
}

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