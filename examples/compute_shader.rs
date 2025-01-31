//! double nums by compute shader.
use std::num::NonZeroU64;

use imagic::{
    prelude::{ComputeShader, ImagicAppTrait, ImagicOption},
    window::WindowSize,
    Imagic,
};
use wgpu::util::DeviceExt;

pub struct App {}

impl ImagicAppTrait for App {
    fn init(&mut self, imagic: &mut imagic::prelude::ImagicContext) {
        // Corrected to call the specific init method for ComputeShader
        ComputeShader::execute(self, imagic);
    }

    fn get_imagic_option(&self) -> imagic::prelude::ImagicOption {
        ImagicOption {
            window_size: WindowSize::new(500.0, 500.0),
            window_title: "compute shader demo",
        }
    }
}

impl ComputeShader for App {
    fn execute(&mut self, imagic_context: &mut imagic::prelude::ImagicContext) {
        let arguments: [f32; 3] = [1.2, 2.3, 3.4];
        let device = imagic_context.graphics_context().get_device();
        let module =
            device.create_shader_module(wgpu::include_wgsl!("common/shaders/double_nums.wgsl"));
        let input_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("input data buffer"),
            contents: bytemuck::cast_slice(&arguments),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let output_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ouput data buffer"),
            size: input_data_buffer.size(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        // usage of `MAP_READ` can only be used with `COPY_DST`.
        let download_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("download buffer"),
            size: input_data_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("compute bind group layout"),
            entries: &[
                // Input buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        // This is the size of a single element in the buffer.
                        min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
                // Output buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        // This is the size of a single element in the buffer.
                        min_binding_size: Some(NonZeroU64::new(4).unwrap()),
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_data_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("doubleMe"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // A compute pass is a single series of compute operations. While we are recording a compute
        // pass, we cannot record to the encoder.
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });

        // Set the pipeline that we want to use
        compute_pass.set_pipeline(&pipeline);
        // Set the bind group that we want to use
        compute_pass.set_bind_group(0, &bind_group, &[]);

        let workgroup_count = arguments.len().div_ceil(64);
        compute_pass.dispatch_workgroups(workgroup_count as u32, 1, 1);

        // Now we drop the compute pass, giving us access to the encoder again.
        drop(compute_pass);

        // We add a copy operation to the encoder. This will copy the data from the output buffer on the
        // GPU to the download buffer on the CPU.
        encoder.copy_buffer_to_buffer(
            &output_data_buffer,
            0,
            &download_buffer,
            0,
            output_data_buffer.size(),
        );

        let command_buffer = encoder.finish();

        imagic_context
            .graphics_context()
            .get_queue()
            .submit([command_buffer]);

        let buffer_slice = download_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {
            // In this case we know exactly when the mapping will be finished,
            // so we don't need to do anything in the callback.
        });

        // Wait for the GPU to finish working on the submitted work. This doesn't work on WebGPU, so we would need
        // to rely on the callback to know when the buffer is mapped.
        device.poll(wgpu::Maintain::Wait);

        // We can now read the data from the buffer.
        let data = buffer_slice.get_mapped_range();
        // Convert the data back to a slice of f32.
        let result: &[f32] = bytemuck::cast_slice(&data);

        // Print out the result.
        println!("Result: {:?}", result);
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut imagic = Imagic::new(Box::new(App {}));
    imagic.run();
}
