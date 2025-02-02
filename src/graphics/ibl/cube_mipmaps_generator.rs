use wgpu::{BindGroup, BindGroupLayout, ComputePipeline, TextureAspect};

use crate::{
    prelude::{ComputeShader, ImagicContext, Texture, INVALID_ID},
    types::ID,
};

pub enum MipmapGeneratorType {
    BilinearFilter,
    GaussianFilter4x4,
    /// not implemented, faded to BilinearFiler.
    BoxAndMultiFilter,
}

pub struct CubeMipmapsGenerator {
    cube_without_mipmaps: ID,
    cube_with_mipmaps: ID,
    face_size: u32,
    format: wgpu::TextureFormat,
    mipmap_generator_type: MipmapGeneratorType,
}

impl ComputeShader for CubeMipmapsGenerator {
    fn execute(&mut self, imagic_context: &mut ImagicContext) {
        let mip_level_count = self.create_cube_texture_with_mipmaps(imagic_context);
        let bind_group_layout = self.create_bind_group_layout(imagic_context);
        let compute_pipeline = self.create_compute_pipeline(imagic_context, &bind_group_layout);

        let mut encoder = imagic_context
            .graphics_context()
            .get_device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // info!("mip_level_count: {}", mip_level_count);
        for mip in 1..mip_level_count {
            // info!("mip: {}", mip);
            let bind_group = self.create_bind_group(imagic_context, mip, &bind_group_layout);

            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let workgroup_size = 8;
            // note: Parentheses that encapsule "self.face_size >> mip" below is essential.
            let workgroup_count = ((self.face_size >> mip) + workgroup_size - 1) / workgroup_size;
            // info!("workgroup_count: {}", workgroup_count);
            compute_pass.dispatch_workgroups(workgroup_count, workgroup_count, 6);
        }

        imagic_context
            .graphics_context()
            .get_queue()
            .submit(Some(encoder.finish()));

        imagic_context
            .graphics_context()
            .get_device()
            .poll(wgpu::Maintain::Wait);
    }
}

impl CubeMipmapsGenerator {
    pub fn new(cube_without_mipmaps: ID, face_size: u32, format: wgpu::TextureFormat, mipmap_generator_type: MipmapGeneratorType) -> Self {
        Self {
            cube_without_mipmaps,
            face_size,
            cube_with_mipmaps: INVALID_ID,
            format,
            mipmap_generator_type
        }
    }

    pub fn get_cube_with_mipmap(&self) -> ID {
        self.cube_with_mipmaps
    }

    fn create_cube_texture_with_mipmaps(&mut self, imagic_context: &mut ImagicContext) -> u32 {
        let mip_level_count = self.face_size.ilog2() + 1;
        let cube_map_with_mipmap = Texture::create_cube_texture(
            imagic_context.graphics_context(),
            self.format,
            self.face_size,
            self.face_size,
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            mip_level_count,
        );

        let original_cube_texture = imagic_context
            .texture_manager()
            .get_texture(self.cube_without_mipmaps);

        let mut encoder = imagic_context
            .graphics_context()
            .get_device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mipmap Copy Encoder"),
            });
        for layer in 0..6 {
            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: original_cube_texture.get(),
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: cube_map_with_mipmap.get(),
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1,
                },
            );
        }

        imagic_context
            .graphics_context()
            .get_queue()
            .submit(Some(encoder.finish()));
        imagic_context
            .graphics_context()
            .get_device()
            .poll(wgpu::Maintain::Wait);

        self.cube_with_mipmaps = imagic_context
            .texture_manager_mut()
            .add_texture(cube_map_with_mipmap);
        // info!("cube_with_mipmaps: {}", self.cube_with_mipmaps);

        mip_level_count
    }

    fn create_bind_group_layout(&mut self, imagic_context: &mut ImagicContext) -> BindGroupLayout {
        let bind_group_layout = imagic_context
            .graphics_context()
            .get_device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Mipmap Generation Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba32Float,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                ],
            });
        bind_group_layout
    }

    fn create_bind_group(
        &mut self,
        imagic_context: &mut ImagicContext,
        mip: u32,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        let cube_texture_with_mipmaps = imagic_context
            .texture_manager()
            .get_texture(self.cube_with_mipmaps);
        let input_texture_view =
            cube_texture_with_mipmaps.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Input Mipmap Texture View"),
                base_mip_level: mip - 1,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(6),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                format: Some(self.format),
                aspect: TextureAspect::All,
            });

        let output_texture_view =
            cube_texture_with_mipmaps.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Output Mipmap Texture View"),
                base_mip_level: mip,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(6),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                format: Some(self.format),
                aspect: TextureAspect::All,
            });

        let bind_group = imagic_context
            .graphics_context()
            .get_device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Mipmap Generation Bind Group"),
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

    fn create_compute_pipeline(
        &mut self,
        imagic_context: &mut ImagicContext,
        bind_group_layout: &BindGroupLayout,
    ) -> ComputePipeline {
        let device = imagic_context.graphics_context().get_device();
        let compute_shader = device.create_shader_module(wgpu::include_wgsl!(
            "../../shaders/compute/gen_mipmaps.wgsl"
        ));

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Mipmap Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let entry_point = match self.mipmap_generator_type {
            MipmapGeneratorType::BilinearFilter => {
                Some("main_bilinear")
            }
            MipmapGeneratorType::GaussianFilter4x4 => {
                Some("main_gaussian")
            }
            _ => Some("main_bilinear")
        };
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Mipmap Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: entry_point,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        compute_pipeline
    }
}
