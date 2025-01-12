use wgpu::{TextureFormat, TextureView, TextureViewDescriptor};

use super::GraphicsContext;

pub struct Texture {
    texture: wgpu::Texture,
    view: Option<TextureView>,
    size: wgpu::Extent3d,
}

impl Texture {
    pub fn create(
        graphics_context: &GraphicsContext,
        width: u32,
        height: u32,
        depth: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: depth,
        };
        let texture = graphics_context
            .get_device()
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("imagic_texture"),
                size,
                mip_level_count: 1,
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

    pub fn create_from_bytes(
        graphics_context: &GraphicsContext,
        buffer: &[u8],
        format: wgpu::TextureFormat,
    ) -> Self {
        let img = image::load_from_memory(buffer).unwrap();
        let img_rgba = img.to_rgba8();
        use image::GenericImageView;
        let dimensions = img.dimensions();
        let mut texture = Texture::create(
            graphics_context,
            dimensions.0,
            dimensions.1,
            1,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        texture.fill_content(graphics_context, &img_rgba, Some(dimensions.0 * 4));

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
    ) -> Self {
        let mut imgs_data = Vec::new();
        let mut width = u32::MAX;
        let mut height = u32::MAX;
        for buffer in &buffers {
            let img = image::load_from_memory(buffer).unwrap();
            if width == u32::MAX {
                use image::GenericImageView;
                (width, height) = img.dimensions();
            }
            let img_rgba = img.to_rgba8();
            imgs_data.push(img_rgba);
        }

        let mut texture = Texture::create(
            graphics_context,
            width,
            height,
            6,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );

        let face_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1, // 每次上传一个面
        };

        for (i, img) in imgs_data.iter().enumerate() {
            assert_eq!(img.width(), width);
            assert_eq!(img.height(), height);
            // info!("i: {i},width: {width}, height: {height}, size: {:?}", texture.size);

            graphics_context.get_queue().write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: i as u32 },
                    aspect: wgpu::TextureAspect::All,
                },
                img,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                face_size,
            );
        }

        let texture_view = texture.create_view(
            &wgpu::TextureViewDescriptor {
                label: Some("Cube Texture View"),
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..Default::default()
            }
        );
        texture.view = Some(texture_view);

        texture
    }

    pub fn create_hdr_texture(
        graphics_context: &GraphicsContext,
        width: u32,
        height: u32,
        pixels: &[u8],
        format: wgpu::TextureFormat,
    ) -> Self {
        let mut texture = Texture::create(
            graphics_context,
            width,
            height,
            1,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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
    ) -> Self {
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let mut dpeth_texture = Texture::create(graphics_context, width, height, 1, format, usage);

        let texture_view = dpeth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        dpeth_texture.view = Some(texture_view);
        dpeth_texture
    }

    fn fill_content(
        &self,
        graphics_context: &GraphicsContext,
        content: &[u8],
        bytes_per_row: Option<u32>,
    ) {
        graphics_context.get_queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &content,
            wgpu::ImageDataLayout {
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
}
