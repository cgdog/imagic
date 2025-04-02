use image::{ImageBuffer, Rgba};
use wgpu::{TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};

use crate::{asset::{asset::{Asset, Handle}, asset_manager::AssetManager}, prelude::Mipmaps2DGenerator};

use super::GraphicsContext;

pub struct Texture {
    texture: wgpu::Texture,
    view: Option<TextureView>,
    size: wgpu::Extent3d,
}

impl Asset for Texture {}

impl Texture {
    pub fn get(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn get_size(&self) -> wgpu::Extent3d {
        self.size
    }

    pub fn set_view(&mut self, view: TextureView) {
        self.view = Some(view);
    }

    pub fn get_view(&self) -> &Option<TextureView> {
        &self.view
    }

    pub fn create(
        graphics_context: &GraphicsContext,
        width: u32,
        height: u32,
        depth: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        mip_level_count: u32,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: depth,
        };
        // let mut view_formats: &[wgpu::TextureFormat] = &[];
        // if mip_level_count > 1 && format == wgpu::TextureFormat::Rgba8UnormSrgb {
        //     // This view format is used to generate mipmaps by compute shader.
        //     view_formats = &[wgpu::TextureFormat::Rgba8Unorm];
        // }
        let texture = graphics_context
            .get_device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("imagic_texture"),
                size,
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            });

        Self {
            texture,
            view: None,
            size,
        }
    }

    /// Create a 2d texture from bytes of an image file.
    pub fn create_from_bytes(
        graphics_context: &GraphicsContext,
        buffer: &[u8],
        format: wgpu::TextureFormat,
        is_flip_y: bool,
        is_generate_mipmaps: bool,
    ) -> Self {
        let mut img = image::load_from_memory(buffer).unwrap();
        if is_flip_y {
            img = img.flipv();
        }
        let img_rgba = img.to_rgba8();
        use image::GenericImageView;
        let dimensions = img.dimensions();
        let mut mip_level_count = 1;
        let mut usage: TextureUsages = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
        if is_generate_mipmaps {
            mip_level_count = dimensions.0.ilog2().min(dimensions.1.ilog2()) + 1;
            usage |= TextureUsages::COPY_SRC
        }
        let mut texture = Texture::create(
            graphics_context,
            dimensions.0,
            dimensions.1,
            1,
            format,
            usage,
            mip_level_count,
        );
        texture.fill_content(graphics_context, &img_rgba, Some(dimensions.0 * 4));
        
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        texture.view = Some(texture_view);
        if is_generate_mipmaps {
            Mipmaps2DGenerator::generate_mipmaps(graphics_context, &mut texture, mip_level_count);
        }
        texture
    }

    /// Create a 2d texture from raw bytes (custom buffer, not from any image file).
    pub fn create_from_raw_bytes(
        graphics_context: &GraphicsContext,
        buffer: &[u8],
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        mip_level_count: u32,
    ) -> Self {
        let mut texture = Texture::create(
            graphics_context,
            width,
            height,
            1,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            mip_level_count,
        );
        texture.fill_content(graphics_context, &buffer, Some(width * 4));

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        texture.view = Some(texture_view);
        texture
    }

    /// Creates a cube texture (CubeTexture).
    ///
    /// This function creates a cube texture using six image buffers, each corresponding to one face of the cube.
    /// These images will be loaded and used to create a cube texture, suitable for environment mapping and other 3D graphics effects.
    ///
    /// # Parameters
    ///
    /// * `graphics_context` - The graphics context used to create the texture.
    /// * `buffers` - An array of six byte slices, each corresponding to one face of the cube.
    /// * `format` - The format of the texture, specifying how the image data is stored and processed.
    ///
    /// # Returns
    ///
    /// Returns a `Texture` instance representing the created cube texture.
    pub fn create_cube_texture_from_bytes(
        graphics_context: &GraphicsContext,
        buffers: [&[u8]; 6],
        format: wgpu::TextureFormat,
        mip_level_count: u32,
    ) -> Self {
        let (width, height, imgs_data) = Self::create_cube_image_buffers_from_bytes(buffers);
        let mut texture = Texture::create_cube_texture(
            graphics_context,
            format,
            width,
            height,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            mip_level_count,
        );

        texture.fill_cube_texture_with_bytes(graphics_context, imgs_data);
        texture
    }

    /// Create DynamicImage from image file bytes and return array of ImageBuffer.
    fn create_cube_image_buffers_from_bytes(
        buffers: [&[u8]; 6],
    ) -> (u32, u32, Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>) {
        let mut imgs_data = Vec::new();
        let mut width = u32::MAX;
        let mut height = u32::MAX;
        for buffer in &buffers {
            let img = image::load_from_memory(buffer).unwrap();
            if width == u32::MAX {
                use image::GenericImageView;
                (width, height) = img.dimensions();
            }
            // Note: we can call img.to_rgba8 for .jpeg image file.
            let img_rgba = img.to_rgba8();
            imgs_data.push(img_rgba);
        }
        (width, height, imgs_data)
    }

    /// Fill a cube texture with 6 ImageBuffer.
    pub fn fill_cube_texture_with_bytes(
        &mut self,
        graphics_context: &GraphicsContext,
        imgs_data: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    ) {
        let face_size = wgpu::Extent3d {
            width: self.size.width,
            height: self.size.height,
            depth_or_array_layers: 1, // upload one face data each time.
        };

        for (i, img) in imgs_data.iter().enumerate() {
            graphics_context.get_queue().write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
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
                    bytes_per_row: Some(4 * self.size.width),
                    rows_per_image: Some(self.size.height),
                },
                face_size,
            );
        }
    }

    /// Create a cube texture without filling content.
    pub fn create_cube_texture(
        graphics_context: &GraphicsContext,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        usage: wgpu::TextureUsages,
        mip_level_count: u32,
    ) -> Self {
        let mut texture = Texture::create(
            graphics_context,
            width,
            height,
            6,
            format,
            usage,
            mip_level_count,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Cube Texture View"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            format: Some(format),
            base_mip_level: 0,
            mip_level_count: Some(mip_level_count),
            base_array_layer: 0,
            array_layer_count: Some(6),

            ..Default::default()
        });
        texture.view = Some(texture_view);

        texture
    }

    pub fn create_hdr_texture(
        graphics_context: &GraphicsContext,
        width: u32,
        height: u32,
        pixels: &[u8],
        format: wgpu::TextureFormat,
        mip_level_count: u32,
    ) -> Self {
        let mut texture = Texture::create(
            graphics_context,
            width,
            height,
            1,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            mip_level_count,
        );
        texture.fill_content(
            graphics_context,
            pixels,
            Some(width * std::mem::size_of::<[f32; 4]>() as u32),
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        texture.view = Some(texture_view);
        texture
    }

    pub fn create_depth_texture(
        graphics_context: &GraphicsContext,
        width: u32,
        height: u32,
        format: TextureFormat,
        is_create_view: bool,
    ) -> Self {
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let mut dpeth_texture =
            Texture::create(graphics_context, width, height, 1, format, usage, 1);

        if is_create_view {
            let texture_view = dpeth_texture.create_view(&wgpu::TextureViewDescriptor::default());
            dpeth_texture.view = Some(texture_view);
        }
        dpeth_texture
    }

    /// Fill 2d texture content.
    fn fill_content(
        &self,
        graphics_context: &GraphicsContext,
        content: &[u8],
        bytes_per_row: Option<u32>,
    ) {
        graphics_context.get_queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &content,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row,
                rows_per_image: Some(self.size.height),
            },
            self.size,
        );
    }

    pub fn create_view(&self, desc: &TextureViewDescriptor<'_>) -> TextureView {
        self.texture.create_view(desc)
    }

    pub fn get_texture_view(&self) -> &TextureView {
        &self.view.as_ref().expect("Texture view is not created!")
    }

    /////// default textures ///////
    /// Get default 2x2 white texture.
    pub fn white() -> &'static Handle<Texture> {
        Self::_internal_white(None, None)
    }

    pub fn cube_texture_placeholder() -> &'static Handle<Texture> {
        Self::_internal_cube_placeholder(None, None)
    }

    /// Get default 2x2 black texture.
    // pub fn black() -> ID {
    //     Self::_internal_black(None, &mut None)
    // }

    /// Get default 2x2 blue texture, which can be used as default normal texture in tangent space.
    // pub fn blue() -> ID {
    //     Self::_internal_blue(None, &mut None)
    // }

    /// Create all default textures.
    pub(crate) fn _internal_create_default_textures(
        graphics_context: Option<&GraphicsContext>,
        asset_manager: &mut AssetManager,
    ) {
        // At preset, all missing textures will use a default white image and PBR will disable related features.
        Self::_internal_white(graphics_context, Some(asset_manager));
        // Self::_internal_black(graphics_context, texture_manager);
        // Self::_internal_blue(graphics_context, texture_manager);
        Self::_internal_cube_placeholder(graphics_context, Some(asset_manager));
    }

    /// Get or create default a 2x2 white texture.
    #[allow(static_mut_refs)]
    pub(crate) fn _internal_white(
        graphics_context: Option<&GraphicsContext>,
        asset_manager: Option<&mut AssetManager>,
    ) -> &'static Handle<Texture> {
        static mut WHITE_TEXTURE_ID: Handle<Texture> = Handle::INVALID;
        unsafe {
            if WHITE_TEXTURE_ID == Handle::INVALID {
                if let (Some(graphics_context), Some(asset_manager)) =
                    (graphics_context, asset_manager)
                {
                    let white_image_data: Vec<u8> = vec![
                        255, 255, 255, 255, // (R, G, B, A)
                        255, 255, 255, 255, //
                        255, 255, 255, 255, //
                        255, 255, 255, 255, //
                    ];

                    let white_texture = Texture::create_from_raw_bytes(
                        graphics_context,
                        &white_image_data,
                        2,
                        2,
                        wgpu::TextureFormat::Rgba8UnormSrgb,
                        1,
                    );
                    WHITE_TEXTURE_ID = asset_manager.add(white_texture);
                }
            }
            &WHITE_TEXTURE_ID
        }
    }

    /// Get or create default a 2x2 black texture.
    #[allow(static_mut_refs)]
    pub(crate) fn _internal_black(
        graphics_context: Option<&GraphicsContext>,
        asset_manager: Option<&mut AssetManager>,
    ) -> &'static Handle<Texture> {
        static mut BLACK_TEXTURE_ID: Handle<Texture> = Handle::INVALID;
        unsafe {
            if BLACK_TEXTURE_ID == Handle::INVALID {
                if let (Some(graphics_context), Some(asset_manager)) =
                    (graphics_context, asset_manager)
                {
                    let black_image_data: Vec<u8> = vec![
                        0, 0, 0, 255, // (R, G, B, A)
                        0, 0, 0, 255, //
                        0, 0, 0, 255, //
                        0, 0, 0, 255, //
                    ];

                    let white_texture = Texture::create_from_raw_bytes(
                        graphics_context,
                        &black_image_data,
                        2,
                        2,
                        wgpu::TextureFormat::Rgba8UnormSrgb,
                        1,
                    );
                    BLACK_TEXTURE_ID = asset_manager.add(white_texture);
                }
            }
            &BLACK_TEXTURE_ID
        }
    }

    #[allow(static_mut_refs)]
    pub(crate) fn _internal_cube_placeholder(
        graphics_context: Option<&GraphicsContext>,
        asset_manager: Option<&mut AssetManager>,
    ) -> &'static Handle<Texture> {
        static mut CUBE_PLACE_HOLDER: Handle<Texture> = Handle::INVALID;
        unsafe {
            if CUBE_PLACE_HOLDER == Handle::INVALID {
                if let (Some(graphics_context), Some(asset_manager)) =
                    (graphics_context, asset_manager)
                {
                    let cube_texture = Self::create_cube_texture(
                        graphics_context,
                        wgpu::TextureFormat::Rgba8UnormSrgb,
                        1,
                        1,
                        wgpu::TextureUsages::TEXTURE_BINDING,
                        1,
                    );
                    CUBE_PLACE_HOLDER = asset_manager.add(cube_texture);
                }
            }
            &CUBE_PLACE_HOLDER
        }
    }
}
