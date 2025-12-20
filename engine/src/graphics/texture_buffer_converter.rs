use thiserror::Error;
use crate::{assets::{Texture, TextureFormat}, prelude::{buffer::BufferUsages, graphics_context::GraphicsContext}};

#[derive(Error, Debug)]
pub enum TextureReadError {
    #[error("Buffer mapping failed: {0}")]
    MappingFailed(String),
    #[error("Invalid data size for texture format")]
    InvalidDataSize,
    #[error("Unsupported texture format: {0:?}")]
    UnsupportedFormat(TextureFormat),
}

pub enum CubeTextureData {
    /// For cube texture with format of Rgba8Unorm, etc.
    U8(Vec<Vec<u8>>),
    /// For cube texture with format of Rgba32Float, etc.
    F32(Vec<Vec<f32>>),
    /// For cube texture with format of Rgba16Uint, etc.
    U16(Vec<Vec<u16>>),
    /// For cube texture with format of Rgba32Sint, etc.
    I32(Vec<Vec<i32>>),
}

pub fn read_cube_texture(
    graphics_context: & GraphicsContext,
    cube_texture: &Texture,
) -> Result<CubeTextureData, TextureReadError> {
    let bytes_per_pixel = cube_texture.bytes_per_pixel();
    let width = cube_texture.size.width;
    let height = cube_texture.size.height;
    let face_size = (width * height * bytes_per_pixel) as wgpu::BufferAddress;
    let total_size = face_size * 6;

    let device = &graphics_context.device;
    let queue = &graphics_context.queue;

    let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: total_size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        label: Some("cube_texture_read_buffer"),
        mapped_at_creation: false, // note here.
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("cube_texture_copy_encoder"),
    });

    if let Some(gpu_texture) = &cube_texture.gpu_texture {
        for face in 0..6 {
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: gpu_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face, // cube face index
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &read_buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: face_size * face as wgpu::BufferAddress,
                        bytes_per_row: Some(bytes_per_pixel * width),
                        rows_per_image: Some(height),
                    },
                },
                wgpu::Extent3d {
                    width: width,
                    height: height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    queue.submit(std::iter::once(encoder.finish()));

    read_and_convert_data(device, read_buffer, face_size, cube_texture.size.width, cube_texture.size.height, cube_texture.format)
}

fn read_and_convert_data(
    device: &wgpu::Device,
    read_buffer: wgpu::Buffer,
    face_size: wgpu::BufferAddress,
    width: u32,
    height: u32,
    format: TextureFormat,
) -> Result<CubeTextureData, TextureReadError> {
    let buffer_slice = read_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();

    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });

    let _= device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None});

    match receiver.recv() {
        Ok(Ok(())) => {
            let data = buffer_slice.get_mapped_range();
            convert_buffer_data(&data, face_size, width, height, format)
        }
        Ok(Err(e)) => Err(TextureReadError::MappingFailed(format!("{:?}", e))),
        Err(_) => Err(TextureReadError::MappingFailed("Channel error".to_string())),
    }
}

fn convert_buffer_data(
    data: &[u8],
    face_size: wgpu::BufferAddress,
    width: u32,
    height: u32,
    format: TextureFormat,
) -> Result<CubeTextureData, TextureReadError> {
    match format {
        TextureFormat::Rgba8Unorm
        | TextureFormat::Rgba8UnormSrgb
        | TextureFormat::Rgba8Snorm
        | TextureFormat::Rgba8Uint
        | TextureFormat::Rgba8Sint
        | TextureFormat::Bgra8Unorm
        | TextureFormat::Bgra8UnormSrgb => convert_to_u8(data, face_size),
        TextureFormat::Rgba32Float => convert_to_f32(data, face_size, width, height),
        TextureFormat::Rgba16Float => convert_to_f16(data, face_size, width, height),
        TextureFormat::Rgba32Uint => convert_to_u32(data, face_size, width, height),
        TextureFormat::Rgba32Sint => convert_to_i32(data, face_size, width, height),
        _ => Err(TextureReadError::UnsupportedFormat(format)),
    }
}

fn convert_to_u8(
    data: &[u8],
    face_size: wgpu::BufferAddress,
) -> Result<CubeTextureData, TextureReadError> {
    let mut faces_data = Vec::with_capacity(6);

    for face in 0..6 {
        let start = (face_size * face as wgpu::BufferAddress) as usize;
        let end = start + face_size as usize;

        if end > data.len() {
            return Err(TextureReadError::InvalidDataSize);
        }

        let face_data = data[start..end].to_vec();
        faces_data.push(face_data);
    }

    Ok(CubeTextureData::U8(faces_data))
}

fn convert_to_f32(
    data: &[u8],
    face_size: wgpu::BufferAddress,
    width: u32,
    height: u32,
) -> Result<CubeTextureData, TextureReadError> {
    let mut faces_data = Vec::with_capacity(6);
    let pixels_per_face = (width * height) as usize;
    let floats_per_face = pixels_per_face * 4; // RGBA

    for face in 0..6 {
        let start = (face_size * face as wgpu::BufferAddress) as usize;
        let end = start + face_size as usize;

        if end > data.len() {
            return Err(TextureReadError::InvalidDataSize);
        }

        let face_bytes = &data[start..end];
        let mut face_floats = Vec::with_capacity(floats_per_face);

        // 将字节转换为 f32
        for chunk in face_bytes.chunks_exact(4) {
            // 将字节转换为 f32，wgpu 缓冲区数据使用小端序，所以使用 from_le_bytes
            let float_val = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            face_floats.push(float_val);
        }

        if face_floats.len() != floats_per_face {
            return Err(TextureReadError::InvalidDataSize);
        }

        faces_data.push(face_floats);
    }

    Ok(CubeTextureData::F32(faces_data))
}

fn convert_to_f16(
    data: &[u8],
    face_size: wgpu::BufferAddress,
    width: u32,
    height: u32,
) -> Result<CubeTextureData, TextureReadError> {
    // convert f16 data to f32
    let mut faces_data = Vec::with_capacity(6);
    let pixels_per_face = (width * height) as usize;
    let floats_per_face = pixels_per_face * 4; // RGBA

    for face in 0..6 {
        let start = (face_size * face as wgpu::BufferAddress) as usize;
        let end = start + face_size as usize;

        if end > data.len() {
            return Err(TextureReadError::InvalidDataSize);
        }

        let face_bytes = &data[start..end];
        let mut face_floats = Vec::with_capacity(floats_per_face);

        for chunk in face_bytes.chunks_exact(2) {
            let half_val = u16::from_le_bytes([chunk[0], chunk[1]]);
            let float_val = half_to_float_simple(half_val);
            face_floats.push(float_val);
        }

        if face_floats.len() != floats_per_face {
            return Err(TextureReadError::InvalidDataSize);
        }

        faces_data.push(face_floats);
    }

    Ok(CubeTextureData::F32(faces_data))
}

fn convert_to_u32(
    data: &[u8],
    face_size: wgpu::BufferAddress,
    width: u32,
    height: u32,
) -> Result<CubeTextureData, TextureReadError> {
    let mut faces_data = Vec::with_capacity(6);
    let pixels_per_face = (width * height) as usize;
    let values_per_face = pixels_per_face * 4; // RGBA

    for face in 0..6 {
        let start = (face_size * face as wgpu::BufferAddress) as usize;
        let end = start + face_size as usize;

        if end > data.len() {
            return Err(TextureReadError::InvalidDataSize);
        }

        let face_bytes = &data[start..end];
        let mut face_u32s = Vec::with_capacity(values_per_face);

        for chunk in face_bytes.chunks_exact(4) {
            let u32_val = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            face_u32s.push(u32_val);
        }

        faces_data.push(face_u32s);
    }

    Ok(CubeTextureData::U16(
        faces_data
            .into_iter()
            .map(|vec| vec.into_iter().map(|v| v as u16).collect())
            .collect(),
    ))
}

fn convert_to_i32(
    data: &[u8],
    face_size: wgpu::BufferAddress,
    width: u32,
    height: u32,
) -> Result<CubeTextureData, TextureReadError> {
    let mut faces_data = Vec::with_capacity(6);
    let pixels_per_face = (width * height) as usize;
    let values_per_face = pixels_per_face * 4; // RGBA

    for face in 0..6 {
        let start = (face_size * face as wgpu::BufferAddress) as usize;
        let end = start + face_size as usize;

        if end > data.len() {
            return Err(TextureReadError::InvalidDataSize);
        }

        let face_bytes = &data[start..end];
        let mut face_i32s = Vec::with_capacity(values_per_face);

        for chunk in face_bytes.chunks_exact(4) {
            let i32_val = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            face_i32s.push(i32_val);
        }

        faces_data.push(face_i32s);
    }

    Ok(CubeTextureData::I32(faces_data))
}

fn half_to_float_simple(half: u16) -> f32 {
    // 这是一个简化的转换，实际应用中建议使用 half 库
    let sign = (half & 0x8000) as u32;
    let exponent = (half & 0x7C00) as u32;
    let fraction = (half & 0x03FF) as u32;
    
    if exponent == 0 {
        // 零或次正规数
        if fraction == 0 {
            f32::from_bits(sign << 16)
        } else {
            // 简化的次正规数处理
            let f = fraction as f32 * 2.0_f32.powi(-24);
            if sign != 0 { -f } else { f }
        }
    } else if exponent == 0x7C00 {
        // 无穷大或 NaN
        if fraction == 0 {
            f32::from_bits((sign << 16) | 0x7F800000)
        } else {
            f32::from_bits((sign << 16) | 0x7FC00000)
        }
    } else {
        // 正规数
        let exp_f32 = ((exponent >> 10) as i32 + (127 - 15)) as u32;
        f32::from_bits((sign << 16) | (exp_f32 << 23) | (fraction << 13))
    }
}