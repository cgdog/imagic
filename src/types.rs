pub type ID = usize;

pub type RR<T> = std::rc::Rc<std::cell::RefCell<T>>;
pub type RRB<T> = std::rc::Rc<std::cell::RefCell<Box<T>>>;
pub type ORR<T> = Option<std::rc::Rc<std::cell::RefCell<T>>>;

#[macro_export]
macro_rules! RRB_new {
    ($object:expr) => {
        std::rc::Rc::new(std::cell::RefCell::new(Box::new($object)))
    };
}
