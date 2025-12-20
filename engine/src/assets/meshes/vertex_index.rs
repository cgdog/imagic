#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq)]
pub enum IndexFormat {
    Uint16 = 0,
    #[default]
    Uint32 = 1,
}

impl From<wgpu::IndexFormat> for IndexFormat {
    fn from(value: wgpu::IndexFormat) -> Self {
        match value {
            wgpu::IndexFormat::Uint16 => Self::Uint16,
            wgpu::IndexFormat::Uint32 => Self::Uint32,
        }
    }
}

impl From<IndexFormat> for wgpu::IndexFormat {
    fn from(value: IndexFormat) -> Self {
        match value {
            IndexFormat::Uint16 => wgpu::IndexFormat::Uint16,
            IndexFormat::Uint32 => wgpu::IndexFormat::Uint32,
        }
    }
}

pub enum IndexData {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl IndexData {
    pub fn new_u16(indices: Vec<u16>) -> Self {
        IndexData::U16(indices)
    }

    pub fn new_u32(indices: Vec<u32>) -> Self {
        IndexData::U32(indices)
    }

    pub fn index_format(&self) -> IndexFormat {
        match self {
            IndexData::U32(_items) => IndexFormat::Uint32,
            IndexData::U16(_items) => IndexFormat::Uint16,
        }
    }

    pub fn index_count(&self) -> u32 {
        match self {
            IndexData::U32(items) => items.len() as u32,
            IndexData::U16(items) => items.len() as u32,
        }
    }

    pub fn content(&self) -> &[u8] {
         match self {
            IndexData::U32(items) => bytemuck::cast_slice(items),
            IndexData::U16(items) => bytemuck::cast_slice(items),
        }
    }
}