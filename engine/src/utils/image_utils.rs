/**
 * Saves a RGBA32 image represented as a Vec<f32> to an HDR image file.
 */
pub fn save_rgba32_array_to_hdr_image(pixel_data: &Vec<f32>, width: u32, height: u32, path: &str) {
    let mut bytes = Vec::<f32>::new();
    let mut i = 0;
    while i < pixel_data.len() {
        bytes.push(pixel_data[i]);
        bytes.push(pixel_data[i + 1]);
        bytes.push(pixel_data[i + 2]);
        i+= 4;
    }
    if let Some(image_buffer) = image::ImageBuffer::<image::Rgb<f32>, Vec<f32>>::from_vec(width, height, bytes) {
        let result = image_buffer.save(path);
        match result {
            Ok(_) => {
                log::info!("Saved an image to {}", path);
            },
            Err(e) => {
                log::error!("Error: failed to save image to {}: Reason:{}", path, e);
                return;
            }
        }
    } else {
        eprintln!("Error: provided Vec<f32> cannot be converted to ImageBuffer!");
    }
}