use image::{DynamicImage, GenericImageView, ImageFormat, imageops};
use std::io::{Cursor};

pub fn make_square_with_blur(input_bytes: &[u8]) -> Result<Vec<u8>, image::ImageError> {

    let img = image::load_from_memory(input_bytes)?;
    let (width, height) = img.dimensions();
    let size = width.max(height);

    let bg_buf = imageops::resize(&img, size, size, imageops::FilterType::Gaussian);
    let mut bg_dyn = DynamicImage::ImageRgba8(bg_buf);
    bg_dyn = DynamicImage::ImageRgba8(imageops::blur(&bg_dyn, 20.0));

    let fg_buf = if width > height {
        imageops::resize(&img, size, (size as f32 * (height as f32 / width as f32)) as u32, imageops::FilterType::Lanczos3)
    } else {
        imageops::resize(&img, (size as f32 * (width as f32 / height as f32)) as u32, size, imageops::FilterType::Lanczos3)
    };
    let fg_dyn = DynamicImage::ImageRgba8(fg_buf);
    let (fg_w, fg_h) = fg_dyn.dimensions();

    let mut canvas = DynamicImage::new_rgba8(size, size);
    imageops::overlay(&mut canvas, &bg_dyn, 0, 0);
    imageops::overlay(&mut canvas, &fg_dyn, ((size - fg_w) / 2) as i64, ((size - fg_h) / 2) as i64);

    let mut buf = Vec::new();
    canvas.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
    Ok(buf)
}
