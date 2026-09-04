use crate::capture_settings::PhotoAspect;
use image::{ImageEncoder, RgbaImage};
use std::path::{Path, PathBuf};

/// Input is the BGRA byte layout consumed by GPUI; output is conventional RGBA.
pub fn photo_pixels(source: &RgbaImage, mirror: bool, aspect: PhotoAspect) -> RgbaImage {
    let (width, height) = source.dimensions();
    let ratio = match aspect {
        PhotoAspect::Native => None,
        PhotoAspect::FourThree => Some((4, 3)),
        PhotoAspect::SixteenNine => Some((16, 9)),
        PhotoAspect::Square => Some((1, 1)),
    };
    let (w, h) = ratio.map_or((width, height), |(x, y)| {
        if u64::from(width) * y > u64::from(height) * x {
            (((u64::from(height) * x / y) as u32).max(1), height)
        } else {
            (width, ((u64::from(width) * y / x) as u32).max(1))
        }
    });
    let mut pixels = image::imageops::crop_imm(source, (width - w) / 2, (height - h) / 2, w, h).to_image();
    if mirror {
        image::imageops::flip_horizontal_in_place(&mut pixels);
    }
    for pixel in pixels.pixels_mut() {
        pixel.0.swap(0, 2);
    }
    pixels
}

pub fn save_photo(output: &Path, source: &RgbaImage, mirror: bool, aspect: PhotoAspect) -> Result<PathBuf, String> {
    if source.width() == 0 || source.height() == 0 {
        return Err("Camera returned an empty image".into());
    }
    let pixels = photo_pixels(source, mirror, aspect);
    let (mut temporary, destination) = super::destination(output, "png")?;
    image::codecs::png::PngEncoder::new(temporary.as_file_mut())
        .write_image(pixels.as_raw(), pixels.width(), pixels.height(), image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("Cannot encode photo: {e}"))?;
    super::publish(temporary, destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn photo_is_png_with_correct_channels_mirror_and_crop() {
        let source = RgbaImage::from_fn(4, 2, |x, _| image::Rgba([x as u8, 20, 30, 255]));
        let output = tempfile::tempdir().unwrap();
        let path = save_photo(output.path(), &source, true, PhotoAspect::Square).unwrap();
        let actual = image::open(path).unwrap().to_rgba8();
        assert_eq!(actual.dimensions(), (2, 2));
        assert_eq!(actual.get_pixel(0, 0).0, [30, 20, 2, 255]);
        assert_eq!(actual.get_pixel(1, 0).0, [30, 20, 1, 255]);
        assert_eq!(std::fs::read_dir(output.path()).unwrap().count(), 1);
    }
}
