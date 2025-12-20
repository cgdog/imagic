use std::{io::Cursor, rc::Rc, u32};

use ahash::AHashMap;
use image::{DynamicImage, ImageReader};
use wgpu::{Device, Queue};

use crate::
    assets::{mipmap_generator::MipmapGenerator, textures::{
        sampler::{AddressMode, FilterMode, Sampler, SamplerHandle},
        texture::{
            Extent3d, SourceType, Texture, TextureDimension, TextureFormat, TextureHandle,
            TextureUsages,
        },
    }}
;

pub struct TextureSamplerManager {
    device: Option<Rc<Device>>,
    queue: Option<Rc<Queue>>,
    textures: AHashMap<TextureHandle, Texture>,
    samplers: AHashMap<SamplerHandle, Sampler>,
    mipmap_generator: Option<MipmapGenerator>,
}

impl TextureSamplerManager {
    pub(crate) fn new() -> Self {
        let texture_manager = Self {
            device: None,
            queue: None,
            textures: AHashMap::new(),
            samplers: AHashMap::new(),
            mipmap_generator: None,
        };
        texture_manager
    }

    pub(crate) fn init(&mut self, device: Rc<Device>, queue: Rc<Queue>) {
        self.mipmap_generator = Some(MipmapGenerator::new(device.clone(), queue.clone()));
        self.device = Some(device);
        self.queue = Some(queue);
        self._try_crate_default_texture();
        self._try_create_default_sampler();
    }

    pub fn get_texture(&self, handle: &TextureHandle) -> Option<&Texture> {
        self.textures.get(handle)
    }

    pub fn get_texture_mut(&mut self, handle: &TextureHandle) -> Option<&mut Texture> {
        self.textures.get_mut(handle)
    }

    pub fn remove_texture(&mut self, handle: &TextureHandle) -> Option<Texture> {
        self.textures.remove(handle)
    }

    /// Create an attachment, e.g, depth attachment, or color attachment.
    pub fn create_attachment(
        &mut self,
        width: u32,
        height: u32,
        depth_or_array_layers: u32,
        dimension: TextureDimension,
        mip_level_count: u32,
        format: TextureFormat,
    ) -> TextureHandle {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers,
        };
        // TODO: make COPY_SRC be optional.
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC;
        // TODO: is mip_level_count required for depth buffer?
        let gpu_texture = Self::create_gpu_texture_safely(&self.device, size, format, usage, mip_level_count, dimension);
        if let Some(gpu_texture) = gpu_texture {
            // let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let view_dimension = if depth_or_array_layers == 1 {
                Some(wgpu::TextureViewDimension::D2)
            } else {
                Some(wgpu::TextureViewDimension::Cube)
            };
            let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Image Texture View"),
                dimension: view_dimension,
                format: Some(format),
                base_mip_level: 0,
                mip_level_count: Some(mip_level_count),
                base_array_layer: 0,
                array_layer_count: Some(depth_or_array_layers),
                ..Default::default()
            });
            let texture = Texture::create_pure_gpu_texture(
                gpu_texture,
                view.into(),
                mip_level_count,
                size.into(),
                dimension,
                usage,
                format,
            );
            let texture_handle = texture.handle;
            self.textures.insert(texture_handle, texture);
            texture_handle
        } else {
            // depth texture without gpu initialized
            let texture = Texture::create_pure_gpu_texture_without_init(
                mip_level_count,
                size,
                dimension,
                usage,
                format,
            );
            let texture_handle = texture.handle;
            self.textures.insert(texture_handle, texture);
            texture_handle
        }
    }

    /// Create a texture from an image file binary bytes.
    pub fn create_texture_from_image(
        &mut self,
        data: Vec<Vec<u8>>,
        dimension: TextureDimension,
        format: TextureFormat,
        is_flip_y: bool,
        is_generate_mipmaps: bool,
    ) -> TextureHandle {
        self.create_texture_from_image_with_usages(
            data,
            dimension,
            format,
            is_flip_y,
            is_generate_mipmaps,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        )
    }

    /// Create a texture with given usages from an image file binary bytes.
    pub fn create_texture_from_image_with_usages(
        &mut self,
        data: Vec<Vec<u8>>,
        dimension: TextureDimension,
        format: TextureFormat,
        is_flip_y: bool,
        is_generate_mipmaps: bool,
        texture_usage: TextureUsages,
    ) -> TextureHandle {
        let texture = Texture::from_image(data, dimension, texture_usage, format, is_flip_y, is_generate_mipmaps);
        let texture_handle = texture.handle;
        self.textures.insert(texture_handle, texture);
        texture_handle
    }

    /// Create a texture from raw color bytes.
    pub fn create_texture_from_raw_bytes(
        &mut self,
        data: Vec<Vec<u8>>,
        dimension: TextureDimension,
        width: u32,
        height: u32,
        depth_or_array_layers: u32,
        format: TextureFormat,
        is_flip_y: bool,
        is_generate_mipmaps: bool,
    ) -> TextureHandle {
        let texture_usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
        let texture = Texture::from_raw_bytes(
            data,
            dimension,
            texture_usage,
            width,
            height,
            depth_or_array_layers,
            format,
            is_flip_y,
            is_generate_mipmaps,
        );
        let texture_handle = texture.handle;
        self.textures.insert(texture_handle, texture);
        texture_handle
    }

    /// Create DynamicImage from image file bytes and return array of ImageBuffer.
    fn create_image_buffers_from_bytes(
        buffers: &Vec<Vec<u8>>,
        is_flip_y: bool,
    ) -> (u32, u32, Vec<Vec<u8>>, u32) {
        let mut imgs_data = Vec::new();
        let mut width = u32::MAX;
        let mut height = u32::MAX;
        let mut bytes_per_row = u32::MAX;
        for buffer in buffers {
            // let mut img = image::load_from_memory(buffer).unwrap();
            let mut img = ImageReader::new(Cursor::new(buffer))
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            if is_flip_y {
                img = img.flipv();
            }
            if width == u32::MAX {
                use image::GenericImageView;
                (width, height) = img.dimensions();
            }
            // Note: we can call img.to_rgba8 for .jpeg image file.
            // let img_rgba = img.to_rgba8();
            // imgs_data.push(img_rgba);
            
            match img {
                DynamicImage::ImageRgb32F(img_buffer) => { // e.g., .hdr
                    let pixels: Vec<f32> = img_buffer
                        .pixels()
                        .flat_map(|pixel| vec![pixel[0], pixel[1], pixel[2], 1.0])
                        .collect();
                    let flat_data: &[u8] = bytemuck::cast_slice(&pixels);
                    imgs_data.push(flat_data.to_vec());
                    if bytes_per_row == u32::MAX {
                        bytes_per_row = width * 4 * 4;
                    }
                }
                DynamicImage::ImageRgba8(img_buffer) => { // e.g., png
                    let pixels: Vec<u8> = img_buffer
                        .pixels()
                        .flat_map(|pixel| vec![pixel[0], pixel[1], pixel[2], pixel[3]])
                        .collect();
                    // let flat_data: &[u8] = bytemuck::cast_slice(&pixels);
                    imgs_data.push(pixels);
                    if bytes_per_row == u32::MAX {
                        bytes_per_row = width * 4;
                    }
                }
                DynamicImage::ImageRgb8(img_buffer) => { // e.g., jpeg
                    let pixels: Vec<u8> = img_buffer
                        .pixels()
                        .flat_map(|pixel| vec![pixel[0], pixel[1], pixel[2], 255])
                        .collect();
                    // let flat_data: &[u8] = bytemuck::cast_slice(&pixels);
                    imgs_data.push(pixels);
                    if bytes_per_row == u32::MAX {
                        bytes_per_row = width * 4;
                    }
                }
                _=> {
                    log::error!("texture format is not implemented!");
                }
            }
            
        }
        (width, height, imgs_data, bytes_per_row)
    }

    fn fill_texture_with_image_bytes(
        gpu_texture: &wgpu::Texture,
        queue: &Option<Rc<Queue>>,
        // imgs_data: &Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>,
        imgs_data: &Vec<Vec<u8>>,
        bytes_per_row: Option<u32>,
        rows_per_image: Option<u32>,
        size: Extent3d,
    ) {
        if let Some(queue) = queue {
            let face_size = wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1, // upload one face data each time.
            };
            for (i, img) in imgs_data.iter().enumerate() {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: gpu_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: i as u32,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    img,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row,
                        rows_per_image,
                    },
                    face_size,
                );
            }
        }
    }

    pub fn generate_mipmaps(&mut self, texture_handle: &TextureHandle) {
        if let Some(mipmap_generator) = &mut self.mipmap_generator
            && let Some(texture) = self.textures.get_mut(texture_handle)
            && texture.mip_level_count > 1 {
            mipmap_generator.generate_mipmap(texture);
        }
    }

    /// Create GPU texture(& mipmaps) for a given texture handle.
    pub(crate) fn ensure_gpu_texture_valid(&mut self, texture_handle: &TextureHandle) {
        if let Some(texture) = self.textures.get_mut(texture_handle) {
            if texture.gpu_texture.is_some() {
                return;
            }
            if texture.is_generate_mipmaps {
                texture.usage |= TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT;
            }

            match texture.source_type {
                SourceType::ImageFileBytes => {
                    let (width, height, imgs, bytes_per_row) = Self::create_image_buffers_from_bytes(&texture.data, texture.is_flip_y);
                    // TODO: implement depth_or_array_layers according to Texture.dimension.
                    texture.size = Extent3d::new(width, height, texture.data.len() as u32);
                    texture.refresh_mip_level_count();
                    let gpu_texture = Self::create_gpu_texture(
                        &self.device,
                        texture.size.width,
                        texture.size.height,
                        texture.size.depth_or_array_layers,
                        texture.format,
                        texture.usage,
                        texture.mip_level_count,
                    );
                    let view_dimension = if texture.size.depth_or_array_layers == 1 {
                        Some(wgpu::TextureViewDimension::D2)
                    } else {
                        // TODO: support D2Array & D3
                        Some(wgpu::TextureViewDimension::Cube)
                    };
                    let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some("Image Texture View"),
                        dimension: view_dimension,
                        format: Some(texture.format),
                        base_mip_level: 0,
                        mip_level_count: Some(texture.mip_level_count),
                        base_array_layer: 0,
                        array_layer_count: Some(texture.size.depth_or_array_layers),
                        ..Default::default()
                    });
                    texture.view = Some(view.into());
                    Self::fill_texture_with_image_bytes(
                        &gpu_texture,
                        &self.queue,
                        &imgs,
                        Some(bytes_per_row),
                        Some(height),
                        texture.size,
                    );
                    texture.gpu_texture = Some(gpu_texture);
                    if texture.mip_level_count > 1 && let Some(mipmap_generator) = &mut self.mipmap_generator {
                        mipmap_generator.generate_mipmap(texture);
                    }
                }
                SourceType::RawColorBytes => {
                    let gpu_texture = Self::create_gpu_texture(
                        &self.device,
                        texture.size.width,
                        texture.size.height,
                        texture.size.depth_or_array_layers,
                        texture.format,
                        texture.usage,
                        texture.mip_level_count,
                    );
                    // let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let view_dimension = if texture.size.depth_or_array_layers == 1 {
                        Some(wgpu::TextureViewDimension::D2)
                    } else {
                        // TODO: support D2Array & D3
                        Some(wgpu::TextureViewDimension::Cube)
                    };
                    let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some("RawColor Texture View"),
                        dimension: view_dimension,
                        format: Some(texture.format),
                        base_mip_level: 0,
                        mip_level_count: Some(texture.mip_level_count),
                        base_array_layer: 0,
                        array_layer_count: Some(texture.size.depth_or_array_layers),
                        ..Default::default()
                    });
                    texture.view = Some(view.into());
                    Self::write_to_gpu_texture(
                        &self.queue,
                        &gpu_texture,
                        &texture.data,
                        Some(texture.size.width * 4),
                        Some(texture.size.height),
                        texture.size,
                    );
                    texture.gpu_texture = Some(gpu_texture);
                    if texture.mip_level_count > 1 && let Some(mipmap_generator) = &mut self.mipmap_generator {
                        mipmap_generator.generate_mipmap(texture);
                    }
                }
                SourceType::GPUBuffer => {
                    let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
                    let gpu_texture = Self::create_gpu_texture(
                        &self.device,
                        texture.size.width,
                        texture.size.height,
                        texture.size.depth_or_array_layers,
                        texture.format,
                        usage,
                        texture.mip_level_count,
                    );
                    let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
                    texture.view = Some(view.into());
                }
            }
        }
    }

    /// Create a GPU texture.
    fn create_gpu_texture(
        device: &Option<Rc<Device>>,
        width: u32,
        height: u32,
        depth: u32,
        format: TextureFormat,
        usage: TextureUsages,
        mip_level_count: u32,
    ) -> wgpu::Texture {
        if let Some(device) = device {
            let size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: depth,
            };

            let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("imagic_texture"),
                size,
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            });

            gpu_texture
        } else {
            panic!("TextureManager is not inited with device");
        }
    }

    pub(crate) fn ensure_depth_texture_valid(
        &mut self,
        texture_handle: TextureHandle,
    ) -> &wgpu::TextureView {
        if let Some(texture) = self.textures.get_mut(&texture_handle) {
            Self::ensure_attachment_valid(&self.device, texture)
        } else {
            panic!(
                "{}",
                format!("depth texture {} is not created", texture_handle)
            );
        }
    }

    pub(crate) fn ensure_color_attachment_valid(
        &mut self,
        texture_handle: TextureHandle,
    ) -> &wgpu::TextureView {
        if let Some(texture) = self.textures.get_mut(&texture_handle) {
            Self::ensure_attachment_valid(&self.device, texture)
        } else {
            panic!(
                "{}",
                format!("color attachment {} is not created", texture_handle)
            );
        }
    }

    fn ensure_attachment_valid<'a>(
        device: &Option<Rc<Device>>,
        texture: &'a mut Texture,
    ) -> &'a wgpu::TextureView {
        if let Some(_view) = &texture.view {
            &texture.view.as_ref().unwrap().view
        } else {
            // this branch is runned just before draw call but without a render pass created.
            let usage =
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
            let gpu_texture = Self::create_gpu_texture_safely(
                device,
                texture.size,
                texture.format,
                usage,
                texture.mip_level_count,
                texture.dimension,
            );
            // maybe need to generate mipmaps here.
            if let Some(gpu_texture) = &gpu_texture {
                let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
                texture.view = Some(view.into());
            }
            texture.gpu_texture = gpu_texture;
            &texture.view.as_ref().unwrap().view
        }
    }

    pub(crate) fn get_texture_view_forcely(
        &self,
        texture_handle: &TextureHandle,
    ) -> &wgpu::TextureView {
        &self
            .textures
            .get(texture_handle)
            .as_ref()
            .unwrap()
            .view
            .as_ref()
            .unwrap()
            .view
    }

    /// Create a GPU texture safely.
    pub(crate) fn create_gpu_texture_safely(
        device: &Option<Rc<Device>>,
        size: Extent3d,
        format: TextureFormat,
        usage: TextureUsages,
        mip_level_count: u32,
        dimension: TextureDimension,
    ) -> Option<wgpu::Texture> {
        if let Some(device) = device {
            let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("imagic_texture"),
                size: size.into(),
                mip_level_count,
                sample_count: 1,
                dimension,
                format,
                usage,
                view_formats: &[],
            });

            Some(gpu_texture)
        } else {
            None
        }
    }

    /// Upload texture content to GPU.
    pub(crate) fn write_to_gpu_texture(
        queue: &Option<Rc<Queue>>,
        gpu_texture: &wgpu::Texture,
        content: &Vec<Vec<u8>>,
        bytes_per_row: Option<u32>,
        rows_per_image: Option<u32>,
        size: Extent3d,
    ) {
        if let Some(queue) = queue {
            for (i, data) in content.iter().enumerate() {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: gpu_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: i as u32,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    data,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row,
                        rows_per_image,
                    },
                    wgpu::Extent3d {
                        width: size.width,
                        height: size.height,
                        depth_or_array_layers: 1, // Note: cube texture should also use 1.
                    },
                );
            }
        } else {
            panic!("TextureManager is not inited with queue");
        }
    }

    pub fn create_sampler(
        &mut self,
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        address_mode_w: AddressMode,
        mag_filter: FilterMode,
        min_filter: FilterMode,
        mipmap_filter: FilterMode,
    ) -> SamplerHandle {
        let handle = Sampler::compute_sampler_handle(address_mode_u, address_mode_v, address_mode_w, mag_filter, min_filter, mipmap_filter);
        if !self.samplers.contains_key(&handle) {
            let sampler = Sampler::new_with_handle(
                address_mode_u,
                address_mode_v,
                address_mode_w,
                mag_filter,
                min_filter,
                mipmap_filter,
                handle,
            );
            self.samplers.insert(handle, sampler);
        }
        handle
    }

    pub(crate) fn create_gpu_sampler(&mut self, sampler_handle: &SamplerHandle) {
        if let Some(device) = &self.device
            && let Some(sampler) = self.samplers.get_mut(sampler_handle)
            && sampler.gpu_sampler.is_none()
        {
            let gpu_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("lxy create sampler"),
                address_mode_u: sampler.address_mode_u,
                address_mode_v: sampler.address_mode_u,
                address_mode_w: sampler.address_mode_w,
                mag_filter: sampler.mag_filter,
                min_filter: sampler.min_filter,
                mipmap_filter: sampler.mipmap_filter,
                ..Default::default()
            });
            sampler.gpu_sampler = Some(gpu_sampler);
        }
    }

    fn _try_create_default_sampler(&mut self) {
        let default_sampler_handle = Sampler::default_sampler();
        // User may have created a sampler whose values are equal to default sampler (so handles are same) but without create the gpu_sampler.
        if !self.samplers.contains_key(&default_sampler_handle) {
            self.create_sampler(
                AddressMode::ClampToEdge,
                AddressMode::ClampToEdge,
                AddressMode::ClampToEdge,
                FilterMode::Linear,
                FilterMode::Linear,
                FilterMode::Linear,
            );
        }

        self.create_gpu_sampler(&default_sampler_handle);
    }

    pub fn get_sampler(&self, sampler_handle: &SamplerHandle) -> Option<&Sampler> {
        self.samplers.get(sampler_handle)
    }

    pub fn get_sampler_mut(&mut self, sampler_handle: &SamplerHandle) -> Option<&mut Sampler> {
        self.samplers.get_mut(sampler_handle)
    }

    pub fn _try_crate_default_texture(&mut self) {
        let default_2d_texture_handle = Texture::white();
        if !self.textures.contains_key(&default_2d_texture_handle) {
            let image_data: Vec<Vec<u8>> = vec![vec![
                255, 255, 255, 255, // (R, G, B, A)
                255, 255, 255, 255, //
                255, 255, 255, 255, //
                255, 255, 255, 255, //
            ]];
            self.create_texture_from_raw_bytes(
                image_data,
                TextureDimension::D2,
                2,
                2,
                1,
                TextureFormat::Rgba8UnormSrgb,
                false,
                true,
            );
        }
        self.ensure_gpu_texture_valid(&default_2d_texture_handle);

        let default_cube_texture_handle = Texture::default_cube_texture();
        if !self.textures.contains_key(&default_cube_texture_handle) {
            let image_data: Vec<Vec<u8>> = vec![
                vec![0, 0, 0, 1],
                vec![0, 0, 0, 1],
                vec![0, 0, 0, 1],
                vec![0, 0, 0, 1],
                vec![0, 0, 0, 1],
                vec![0, 0, 0, 1],
            ];
            self.create_texture_from_raw_bytes(
                image_data,
                TextureDimension::D2,
                1,
                1,
                6,
                TextureFormat::Rgba8UnormSrgb,
                false,
                true,
            );
        }
        self.ensure_gpu_texture_valid(&default_cube_texture_handle);
    }
}
