use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

#[derive(Debug, Clone, Copy)]
pub enum Layer {
    Default = 1 << 0,
    UI = 1 << 1,
    RenderTarget = 1 << 3,
    Custom1 = 1 << 4,
    Custom2 = 1 << 5,
    Custom3 = 1 << 6,
    Custom4 = 1 << 7,
    All = 0xffffffff,
}

impl Default for Layer {
    fn default() -> Self {
        Layer::Default
    }
}

impl From<Layer> for u32 {
    fn from(value: Layer) -> Self {
        value as u32
    }
}

impl BitOr for Layer {
    type Output = LayerMask;

    fn bitor(self, rhs: Self) -> Self::Output {
        LayerMask(self as u32 | rhs as u32)
    }
}

/// LayerMask
#[derive(Debug, Clone, Copy)]
pub struct LayerMask(u32);

impl Default for LayerMask {
    fn default() -> Self {
        LayerMask(Layer::Default.into())
    }
}

impl From<LayerMask> for u32 {
    fn from(value: LayerMask) -> Self {
        value.0
    }
}

impl LayerMask {

    pub fn new(layer_mask: u32) -> Self {
        LayerMask(layer_mask)
    }

    pub fn get(&self) -> u32 {
        self.0
    }

    pub fn set(&mut self, layer_mask: u32) {
        self.0 = layer_mask;
    }

    pub fn contains(&self, layer: Layer) -> bool {
        self.0 & (layer as u32) != 0
    }

    pub fn insert(&mut self, layer: Layer) {
        self.0 |= layer as u32;
    }

    pub fn remove(&mut self, layer: Layer) {
        self.0 &= !(layer as u32);
    }
}

impl BitOr for LayerMask {
    type Output = LayerMask;

    fn bitor(self, rhs: Self) -> Self::Output {
        LayerMask(self.0 | rhs.0)
    }
}

impl BitOrAssign for LayerMask {
    fn bitor_assign(&mut self, rhs: LayerMask) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for LayerMask {
    type Output = LayerMask;

    fn bitand(self, rhs: LayerMask) -> Self::Output {
        LayerMask(self.0 & rhs.0)
    }
}

impl BitAndAssign for LayerMask {
    fn bitand_assign(&mut self, rhs: LayerMask) {
        self.0 &= rhs.0;
    }
}