
/// Component is a trait that all components must implement.
pub trait Component : 'static {
}

pub(crate) type ComponentTypeId = std::any::TypeId;

type ComponentIdType = u32;
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ComponentId(pub(crate) ComponentIdType);

impl ComponentId {
    pub fn new(id: ComponentIdType) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) const INVALID_COMPONENT_ID: ComponentId = ComponentId(u32::MAX);