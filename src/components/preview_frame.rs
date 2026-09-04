use gpui::RenderImage;
use image::{Frame, RgbaImage, imageops};

/// Always derive the preview from the original pixels so toggling cannot compound flips.
fn preview_pixels(source: &RgbaImage, mirror: bool) -> RgbaImage {
    let mut pixels = source.clone();
    if mirror {
        imageops::flip_horizontal_in_place(&mut pixels);
    }
    pixels
}

pub(super) fn render_image(source: &RgbaImage, mirror: bool) -> RenderImage {
    RenderImage::new(vec![Frame::new(preview_pixels(source, mirror))])
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn mirror_reverses_each_row_without_modifying_source() {
        let original = RgbaImage::from_fn(3, 2, |x, y| Rgba([(y * 3 + x) as u8, 0, 0, 255]));
        let first_channel =
            |image: RgbaImage| image.pixels().map(|pixel| pixel[0]).collect::<Vec<_>>();
        assert_eq!(
            first_channel(preview_pixels(&original, true)),
            [2, 1, 0, 5, 4, 3]
        );
        assert_eq!(
            first_channel(preview_pixels(&original, false)),
            [0, 1, 2, 3, 4, 5]
        );
        assert_eq!(
            first_channel(preview_pixels(&original, true)),
            [2, 1, 0, 5, 4, 3]
        );
        assert_eq!(first_channel(original), [0, 1, 2, 3, 4, 5]);
    }
}
