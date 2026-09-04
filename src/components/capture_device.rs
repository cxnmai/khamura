use crate::capture_settings::VideoQuality;
use nokhwa::{
    Camera,
    pixel_format::{FormatDecoder, RgbFormat},
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};

pub(super) fn open(
    device: &Option<String>,
    quality: Option<VideoQuality>,
) -> Result<(Camera, Vec<VideoQuality>, u32), String> {
    let index = camera_index(device.as_deref())?;
    let request = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = Camera::new(index, request).map_err(|e| e.to_string())?;
    let formats = camera
        .compatible_camera_formats()
        .map_err(|e| e.to_string())?;
    let formats: Vec<_> = formats
        .into_iter()
        .filter(|f| RgbFormat::FORMATS.contains(&f.format()) && f.frame_rate() > 0)
        .collect();
    let mut choices: Vec<_> = formats
        .iter()
        .map(|f| VideoQuality {
            width: f.resolution().width(),
            height: f.resolution().height(),
            fps: f.frame_rate(),
        })
        .collect();
    choices.sort_by_key(|q| (q.width, q.height, q.fps));
    choices.dedup();
    if let Some(wanted) = quality {
        let format = formats.iter().find(|f| f.resolution().width() == wanted.width && f.resolution().height() == wanted.height && f.frame_rate() == wanted.fps)
            .ok_or("Selected video quality is not supported by this camera; choose Native or another preset")?;
        camera
            .set_camera_requset(RequestedFormat::new::<RgbFormat>(
                RequestedFormatType::Exact(*format),
            ))
            .map_err(|e| e.to_string())?;
    }
    camera.open_stream().map_err(|e| e.to_string())?;
    let fps = camera.frame_rate().max(1);
    Ok((camera, choices, fps))
}

// Nokhwa's V4L backend needs a numeric index, even when the UI stores device paths.
fn camera_index(device: Option<&str>) -> Result<CameraIndex, String> {
    let Some(device) = device else {
        return Ok(CameraIndex::default());
    };
    let resolved = std::fs::canonicalize(device).ok();
    let device = resolved
        .as_deref()
        .and_then(|p| p.to_str())
        .unwrap_or(device);
    let index = device
        .strip_prefix("/dev/video")
        .unwrap_or(device)
        .parse::<u32>()
        .map_err(|_| format!("Unsupported camera device: {device}"))?;
    Ok(CameraIndex::Index(index))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_device_paths_to_v4l_indices() {
        assert_eq!(
            camera_index(Some("/dev/video123")).unwrap(),
            CameraIndex::Index(123)
        );
        assert_eq!(camera_index(Some("2")).unwrap(), CameraIndex::Index(2));
        assert!(camera_index(Some("/tmp/not-a-camera")).is_err());
    }
}
