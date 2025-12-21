use std::hash::{Hash, Hasher};

use ahash::AHasher;

use crate::assets::textures::texture_view::TextureView;

pub type TextureFormat = wgpu::TextureFormat;
pub type TextureUsages = wgpu::TextureUsages;
pub type TextureDimension = wgpu::TextureDimension;
pub type TextureViewDimension = wgpu::TextureViewDimension;
pub type TextureViewDescriptor<'a> = wgpu::TextureViewDescriptor<'a>;
pub type TextureAspect = wgpu::TextureAspect;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TextureTag {}
pub type TextureHandle = crate::types::Handle<TextureTag>;

#[derive(PartialEq, Clone)]
pub enum SourceType {
    /// Texture constructed from an image file binary bytes.
    ImageFileBytes,
    /// Texture constructed from custom raw color bytes.
    RawColorBytes,
    /// Texture is pure GPU texture.
    GPUBuffer,
}

#[derive(PartialEq, Clone)]
pub struct Texture {
    /// Texture bytes data on cpu.
    pub data: Vec<Vec<u8>>,
    pub dimension: TextureDimension,
    pub source_type: SourceType,
    pub usage: TextureUsages,
    pub format: TextureFormat,
    pub is_flip_y: bool,
    pub is_generate_mipmaps: bool,
    pub size: Extent3d,
    pub handle: TextureHandle,
    pub mip_level_count: u32,

    pub view: Option<TextureView>,
    pub(crate) gpu_texture: Option<wgpu::Texture>,
}

impl Texture {
    fn compute_texture_handle(
        data: &Vec<Vec<u8>>,
        usage: &TextureUsages,
        format: &TextureFormat,
        is_flip_y: bool,
        is_generate_mipmaps: bool,
        auto_increment: bool, // at present, only be true when create pure gpu texture.
    ) -> TextureHandle {
        static mut AUTO_INCREMENT_FACTOR: u64 = 0;
        let mut hasher = AHasher::default();
        let hasher_mut_ref = &mut hasher;
        if auto_increment {
            unsafe {
                let auto_increment_factor = AUTO_INCREMENT_FACTOR;
                auto_increment_factor.hash(hasher_mut_ref);
                AUTO_INCREMENT_FACTOR += 1;
            }
        }
        for data_element in data {
            data_element.hash(hasher_mut_ref);
        }
        usage.hash(hasher_mut_ref);
        format.hash(hasher_mut_ref);
        is_flip_y.hash(hasher_mut_ref);
        is_generate_mipmaps.hash(hasher_mut_ref);
        let texture_handle = hasher.finish();
        TextureHandle::new(texture_handle)
    }

    pub(crate) fn from_image(
        data: Vec<Vec<u8>>,
        dimension: TextureDimension,
        texture_usage: TextureUsages,
        format: TextureFormat,
        is_flip_y: bool,
        is_generate_mipmaps: bool,
    ) -> Self {
        let handle = Self::compute_texture_handle(&data, &texture_usage, &format, is_flip_y, is_generate_mipmaps, false);
        Self {
            data,
            dimension,
            usage: texture_usage,
            source_type: SourceType::ImageFileBytes,
            format,
            is_flip_y,
            is_generate_mipmaps,
            size: Extent3d::default(),
            handle,
            mip_level_count: 1,
            gpu_texture: None,
            view: None,
        }
    }

    pub(crate) fn from_raw_bytes(
        data: Vec<Vec<u8>>,
        dimension: TextureDimension,
        texture_usage: TextureUsages,
        width: u32,
        height: u32,
        depth_or_array_layers: u32,
        format: TextureFormat,
        is_flip_y: bool,
        is_generate_mipmaps: bool,
    ) -> Self {
        let handle = Self::compute_texture_handle(&data, &texture_usage, &format, is_flip_y, is_generate_mipmaps, false);
        let size = Extent3d::new(width, height, depth_or_array_layers);
        let mip_level_count = Self::compute_mip_level_count(is_generate_mipmaps, width, height);
        Self {
            data,
            dimension,
            usage: texture_usage,
            source_type: SourceType::RawColorBytes,
            format,
            is_flip_y,
            is_generate_mipmaps,
            handle,
            size,
            mip_level_count,
            gpu_texture: None,
            view: None,
        }
    }

    /// The default white texture with size 2x2.
    pub fn white() -> TextureHandle {
        static mut HANDLE: TextureHandle = TextureHandle::INVALID;
        if unsafe { HANDLE } == TextureHandle::INVALID {
            // Note: here we just compute its handle. The init function of [`TextureSamplerManager`] will really create this texture.
            let texture_usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
            let image_data: Vec<Vec<u8>> = vec![vec![
                    255, 255, 255, 255, // (R, G, B, A)
                    255, 255, 255, 255, //
                    255, 255, 255, 255, //
                    255, 255, 255, 255, //
                ]];
    
            let handle = Self::compute_texture_handle(&image_data, &texture_usage, &TextureFormat::Rgba8UnormSrgb, false, true, false);
            unsafe { HANDLE = handle };
        }

        unsafe { HANDLE }
    }

    /// The default cube texture. Each face is 1x1. Each pixel is (0, 0, 0, 1).
    pub fn default_cube_texture() -> TextureHandle {
        static mut HANDLE: TextureHandle = TextureHandle::INVALID;
        if unsafe { HANDLE } == TextureHandle::INVALID {
            // Note: here we just compute its handle. The init function of [`TextureSamplerManager`] will really create this texture.
            let texture_usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
            let image_data: Vec<Vec<u8>> = vec![
                vec![0, 0, 0, 1],
                vec![0, 0, 0, 1],
                vec![0, 0, 0, 1],
                vec![0, 0, 0, 1],
                vec![0, 0, 0, 1],
                vec![0, 0, 0, 1],
            ];
    
            let handle = Self::compute_texture_handle(&image_data, &texture_usage, &TextureFormat::Rgba8UnormSrgb, false, true, false);
            unsafe { HANDLE = handle };
        }

        unsafe { HANDLE }
    }

    pub(crate) fn create_pure_gpu_texture(
        gpu_texture: wgpu::Texture,
        view: TextureView,
        mip_level_count: u32,
        size: Extent3d,
        dimension: TextureDimension,
        texture_usage: TextureUsages,
        format: TextureFormat,
    ) -> Self {
        let data = Vec::new();
        let is_flip_y = false;
        let is_generate_mipmaps = mip_level_count > 0;
        let handle = Self::compute_texture_handle(&data, &texture_usage, &format, is_flip_y, is_generate_mipmaps, true);

        Self {
            data,
            dimension,
            usage: texture_usage,
            source_type: SourceType::GPUBuffer,
            format,
            is_flip_y: false,
            is_generate_mipmaps,
            mip_level_count,
            gpu_texture: Some(gpu_texture),
            view: Some(view),
            size,
            handle,
        }
    }

    pub(crate) fn create_pure_gpu_texture_without_init(
        mipmap_level_count: u32,
        size: Extent3d,
        dimension: TextureDimension,
        texture_usage: TextureUsages,
        format: TextureFormat,
    ) -> Self {
        let data = Vec::new();
        let is_flip_y = false;
        let is_generate_mipmaps = mipmap_level_count > 0;
        let handle = Self::compute_texture_handle(&data, &texture_usage, &format, is_flip_y, is_generate_mipmaps, true);
        Self {
            data,
            dimension,
            usage: texture_usage,
            source_type: SourceType::GPUBuffer,
            format,
            is_flip_y: false,
            is_generate_mipmaps,
            mip_level_count: mipmap_level_count,
            gpu_texture: None,
            view: None,
            size,
            handle,
        }
    }

    pub fn refresh_mip_level_count(&mut self) {
        self.mip_level_count = Self::compute_mip_level_count(self.is_generate_mipmaps, self.size.width, self.size.height);
    }

    fn compute_mip_level_count(is_generate_mipmaps: bool, width: u32, height: u32) -> u32 {
        let mip_level_count = if is_generate_mipmaps {
            width.ilog2().min(height.ilog2()) + 1
        } else {
            1
        };
        mip_level_count
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        match self.format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba32Float => 16,
            TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => 4,
            TextureFormat::Rgba8Snorm => 4,
            TextureFormat::Rgba8Uint => 4,
            TextureFormat::Rgba8Sint => 4,
            _ => {
                log::warn!("Warning: Unknown texture format {:?}, defaulting to 4 bytes per pixel", self.format);
                4
            }
        }
    }

}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Extent3d {
    /// Width of the extent
    pub width: u32,
    /// Height of the extent
    pub height: u32,
    /// The depth of the extent or the number of array layers
    pub depth_or_array_layers: u32,
}

impl Default for Extent3d {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        }
    }
}

impl From<wgpu::Extent3d> for Extent3d {
    fn from(value: wgpu::Extent3d) -> Self {
        Self {
            width: value.width,
            height: value.height,
            depth_or_array_layers: value.depth_or_array_layers,
        }
    }
}

impl From<Extent3d> for wgpu::Extent3d {
    fn from(value: Extent3d) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: value.width,
            height: value.height,
            depth_or_array_layers: value.depth_or_array_layers,
        }
    }
}

impl Extent3d {
    pub fn new(width: u32, height: u32, depth_or_array_layers: u32) -> Self {
        Self {
            width,
            height,
            depth_or_array_layers,
        }
    }
}