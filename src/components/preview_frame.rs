use crate::{capture_settings::PhotoAspect, media::crop_bounds};
use gpui::RenderImage;
use image::{Frame, RgbaImage, imageops};

/// Always derive the preview from the original pixels so toggling cannot compound flips.
fn preview_pixels(source: &RgbaImage, mirror: bool, aspect: PhotoAspect) -> RgbaImage {
    let (x, y, width, height) = crop_bounds(source.width(), source.height(), aspect);
    let cropped = imageops::crop_imm(source, x, y, width, height);
    // The crop is a borrowed view. Copy directly into the final preview allocation,
    // reversing pixels during the copy when mirrored rather than cloning twice.
    if mirror {
        imageops::flip_horizontal(&*cropped)
    } else {
        cropped.to_image()
    }
}

pub(super) fn render_image(source: &RgbaImage, mirror: bool, aspect: PhotoAspect) -> RenderImage {
    RenderImage::new(vec![Frame::new(preview_pixels(source, mirror, aspect))])
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
            first_channel(preview_pixels(&original, true, PhotoAspect::Native)),
            [2, 1, 0, 5, 4, 3]
        );
        assert_eq!(
            first_channel(preview_pixels(&original, false, PhotoAspect::Native)),
            [0, 1, 2, 3, 4, 5]
        );
        assert_eq!(
            first_channel(preview_pixels(&original, true, PhotoAspect::Native)),
            [2, 1, 0, 5, 4, 3]
        );
        assert_eq!(first_channel(original), [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn combined_crop_and_mirror_match_previous_pipeline() {
        for (width, height) in [(9, 6), (6, 9), (1, 1)] {
            let original = RgbaImage::from_fn(width, height, |x, y| {
                Rgba([x as u8, y as u8, (x + y) as u8, 255])
            });
            for aspect in [
                PhotoAspect::Native,
                PhotoAspect::FourThree,
                PhotoAspect::SixteenNine,
                PhotoAspect::Square,
            ] {
                for mirror in [false, true] {
                    let (x, y, w, h) = crop_bounds(width, height, aspect);
                    let mut expected = imageops::crop_imm(&original, x, y, w, h).to_image();
                    if mirror {
                        imageops::flip_horizontal_in_place(&mut expected);
                    }
                    assert_eq!(preview_pixels(&original, mirror, aspect), expected);
                }
            }
        }
    }
}
