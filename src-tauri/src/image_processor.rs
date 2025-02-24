use image::{io::Reader as ImageReader, imageops::FilterType, DynamicImage, GenericImageView};
use std::{fs::File, io::Write, path::Path, path::PathBuf};
use webp::Encoder;
use crate::initialise::EnvPaths;

pub fn process_thumbnail(image_path: &str) -> String {
    // Load image
    let img = ImageReader::open(image_path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image");

    // Setup paths
    let paths = EnvPaths::new();
    let output_path = paths.thumbnail_path.join(
        Path::new(image_path)
            .file_stem()
            .expect("Invalid filename")
            .to_string_lossy()
            .to_string() + ".webp"
    );

    if !output_path.exists() {

    // Calculate new dimensions
    let max_width = 512;
    let (width, height) = img.dimensions();
    let (new_width, new_height) = if width > max_width {
        (max_width, (height * max_width) / width)
    } else {
        (width, height)
    };

    // Resize with faster filter
    let resized_img = img.resize(new_width, new_height, FilterType::Triangle);
    
    // Convert to RGBA buffer for direct processing
    let mut rgba_buffer = resized_img.to_rgba8();

    // Combined contrast (15%) and brightness (+10) lookup table
    let mut lookup_table = [0u8; 256];
    for i in 0..256 {
        let contrast_factor = 1.15; // 15% contrast increase
        let brighten_value = 10;
        
        let contrast_adjusted = ((i as f32 / 255.0 - 0.5) * contrast_factor + 0.5);
        let brightened = (contrast_adjusted * 255.0).clamp(0.0, 255.0) as i32 + brighten_value;
        lookup_table[i] = brightened.clamp(0, 255) as u8;
    }

    // Apply lookup table directly to pixel buffer
    for pixel in rgba_buffer.chunks_exact_mut(4) {
        pixel[0] = lookup_table[pixel[0] as usize];
        pixel[1] = lookup_table[pixel[1] as usize];
        pixel[2] = lookup_table[pixel[2] as usize];
        // Alpha channel (pixel[3]) remains unchanged
    }

    // Create WebP encoder from raw buffer
    let (width, height) = rgba_buffer.dimensions();
    let encoder = Encoder::from_rgba(rgba_buffer.as_ref(), width, height);
    
    // Encode and save
    let webp_data = encoder.encode(75.0).to_vec();
    File::create(&output_path)
        .and_then(|mut f| f.write_all(&webp_data))
        .expect("Failed to write thumbnail");
    }

    output_path.to_string_lossy().into_owned()
}