use wgpu::{BindGroup, BindGroupLayout, ComputePipeline, TextureAspect, TextureFormat};

use crate::prelude::{GraphicsContext, Texture};

pub struct Mipmaps2DGenerator {}

impl Mipmaps2DGenerator {
    const TEXTURE_USAGE: wgpu::TextureUsages = wgpu::TextureUsages::TEXTURE_BINDING.union(
        wgpu::TextureUsages::STORAGE_BINDING
            .union(wgpu::TextureUsages::COPY_DST.union(wgpu::TextureUsages::COPY_SRC)),
    );

    pub fn generate_mipmaps(
        graphics_context: &GraphicsContext,
        original_texture: &mut Texture,
        mip_level_count: u32,
    ) {
        let texture_size = original_texture.get_size();
        let width = texture_size.width;
        let height = texture_size.height;
        let mut format = original_texture.get().format();
        if format == wgpu::TextureFormat::Rgba8UnormSrgb {
            format = wgpu::TextureFormat::Rgba8Unorm;
        }

        let mut non_srgb_texture = Texture::create(graphics_context, width, height, 1, wgpu::TextureFormat::Rgba8Unorm, Self::TEXTURE_USAGE, mip_level_count);
        
        // let mut non_srgb_texture = graphics_context.get_device().create_texture(&wgpu::TextureDescriptor {
        //     label: Some("Destination Texture"),
        //     size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        //     mip_level_count,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Rgba8Unorm,
        //     usage: Self::TEXTURE_USAGE,
        //     view_formats: &[]
        // });

        let bind_group_layout = Self::create_bind_group_layout(graphics_context, format);
        let compute_pipeline = Self::create_compute_pipeline(graphics_context, &bind_group_layout);

        let mut encoder = graphics_context
            .get_device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: original_texture.get(),
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: non_srgb_texture.get(),
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            );

        // info!("mip_level_count: {}", mip_level_count);
        
        for mip in 1..mip_level_count {
            // info!("mip: {}", mip);
            let bind_group =
                Self::create_bind_group(graphics_context, mip, &bind_group_layout, &mut non_srgb_texture, format);

            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let workgroup_size_x = 8;
            let workgroup_size_y = 8;
            // note: Parentheses that encapsule "self.face_size >> mip" below is essential.
            let workgroup_count_x = ((width >> mip) + workgroup_size_x - 1) / workgroup_size_x;
            let workgroup_count_y = ((height >> mip) + workgroup_size_y - 1) / workgroup_size_y;
            // info!("workgroup_count: {}", workgroup_count);
            compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
        }

        for mip_level in 0..mip_level_count {
            let mip_width = width >> mip_level;
            let mip_height = height >> mip_level;
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: non_srgb_texture.get(),
                    mip_level,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: original_texture.get(),
                    mip_level,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d { width: mip_width, height: mip_height, depth_or_array_layers: 1 },
            );
        }

        graphics_context.get_queue().submit(Some(encoder.finish()));
    }

    fn create_bind_group(
        graphics_context: &GraphicsContext,
        mip: u32,
        bind_group_layout: &BindGroupLayout,
        texture: &mut Texture,
        format: TextureFormat,
    ) -> BindGroup {
        let input_texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Input Mipmap Texture View"),
            base_mip_level: mip - 1,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: None,
            dimension: Some(wgpu::TextureViewDimension::D2),
            format: Some(format),
            aspect: TextureAspect::All,
            usage: Some(Self::TEXTURE_USAGE),
        });

        let output_texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Output Mipmap Texture View"),
            base_mip_level: mip,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: None,
            dimension: Some(wgpu::TextureViewDimension::D2),
            format: Some(format),
            aspect: TextureAspect::All,
            usage: Some(Self::TEXTURE_USAGE),
        });

        let bind_group =
            graphics_context
                .get_device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("2D Mipmap Generation Bind Group"),
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&input_texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&output_texture_view),
                        },
                    ],
                });

        bind_group
    }

    fn create_bind_group_layout(
        graphics_context: &GraphicsContext,
        format: TextureFormat,
    ) -> BindGroupLayout {
        let bind_group_layout = graphics_context.get_device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Mipmap Generation Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: format, //wgpu::TextureFormat::Rgba32Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            },
        );
        bind_group_layout
    }

    fn create_compute_pipeline(
        graphics_context: &GraphicsContext,
        bind_group_layout: &BindGroupLayout,
    ) -> ComputePipeline {
        let device = graphics_context.get_device();
        let compute_shader = device.create_shader_module(wgpu::include_wgsl!(
            "../../shaders/compute/gen_2d_mipmaps.wgsl"
        ));

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("2d Mipmap Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let entry_point = Some("main");
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("2d Mipmap Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: entry_point,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        compute_pipeline
    }
}
