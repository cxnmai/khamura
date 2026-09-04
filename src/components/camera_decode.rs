use super::CapturedFrame;
use nokhwa::{Buffer, pixel_format::RgbAFormat, utils::FrameFormat};

pub(super) fn decode(buffer: &Buffer) -> Result<CapturedFrame, String> {
    let resolution = buffer.resolution();
    let pixels = if buffer.source_frame_format() == FrameFormat::MJPEG {
        decode_jpeg(buffer.buffer(), resolution.width(), resolution.height())
            .map_err(|error| format!("Camera decoding failed: {error}"))?
    } else {
        let decoded = buffer
            .decode_image::<RgbAFormat>()
            .map_err(|error| format!("Camera decoding failed: {error}"))?;
        let mut pixels = decoded.into_raw();
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        pixels
    };
    Ok(CapturedFrame {
        pixels,
        width: resolution.width(),
        height: resolution.height(),
    })
}

fn decode_jpeg(data: &[u8], width: u32, height: u32) -> std::io::Result<Vec<u8>> {
    let decoder = mozjpeg::Decompress::new_mem(data)?;
    if decoder.width() != width as usize || decoder.height() != height as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "JPEG dimensions do not match camera format",
        ));
    }
    // Use the existing JPEG library's SIMD color conversion directly into GPUI's
    // BGRA layout, avoiding an intermediate RGB allocation and channel copy.
    let mut decoder = decoder.to_colorspace(mozjpeg::ColorSpace::JCS_EXT_BGRA)?;
    let pixels = decoder.read_scanlines::<u8>()?;
    decoder.finish()?;
    Ok(pixels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nokhwa::{pixel_format::RgbFormat, utils::Resolution};

    #[test]
    fn raw_decoder_preserves_bgra_layout() {
        let buffer = Buffer::new(
            Resolution::new(2, 1),
            &[10, 20, 30, 40, 50, 60],
            FrameFormat::RAWRGB,
        );
        let decoded = decode(&buffer).unwrap();
        assert_eq!((decoded.width, decoded.height), (2, 1));
        assert_eq!(decoded.pixels, [30, 20, 10, 255, 60, 50, 40, 255]);
    }

    #[test]
    fn fallback_formats_match_previous_rgb_conversion() {
        for (format, data) in [
            (FrameFormat::RAWRGB, vec![10, 20, 30, 40, 50, 60].repeat(2)),
            (FrameFormat::RAWBGR, vec![30, 20, 10, 60, 50, 40].repeat(2)),
            (FrameFormat::GRAY, vec![16, 80, 140, 235]),
            (FrameFormat::YUYV, vec![16, 100, 235, 140].repeat(2)),
            (FrameFormat::NV12, vec![16, 80, 140, 235, 100, 140]),
        ] {
            let buffer = Buffer::new(Resolution::new(2, 2), &data, format);
            let rgb = buffer.decode_image::<RgbFormat>().unwrap();
            let expected: Vec<u8> = rgb
                .as_raw()
                .chunks_exact(3)
                .flat_map(|p| [p[2], p[1], p[0], 255])
                .collect();
            assert_eq!(decode(&buffer).unwrap().pixels, expected, "{format}");
        }
    }

    #[test]
    fn jpeg_fast_path_matches_rgb_decoder_without_resampling() {
        let mut encoder = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        encoder.set_size(9, 6);
        let mut encoder = encoder.start_compress(Vec::new()).unwrap();
        let source: Vec<u8> = (0..9 * 6 * 3).map(|i| (i * 17) as u8).collect();
        encoder.write_scanlines(&source).unwrap();
        let jpeg = encoder.finish().unwrap();
        let buffer = Buffer::new(Resolution::new(9, 6), &jpeg, FrameFormat::MJPEG);
        let previous = buffer.decode_image::<RgbFormat>().unwrap();
        let expected: Vec<u8> = previous
            .as_raw()
            .chunks_exact(3)
            .flat_map(|p| [p[2], p[1], p[0], 255])
            .collect();
        assert_eq!(decode(&buffer).unwrap().pixels, expected);
        assert!(decode_jpeg(&jpeg, 8, 6).is_err());
        assert!(decode_jpeg(b"not a jpeg", 9, 6).is_err());
    }
}
