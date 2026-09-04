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
    let index = device
        .as_ref()
        .map(|id| CameraIndex::String(id.clone()))
        .unwrap_or_default();
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
            .set_camera_format(*format)
            .map_err(|e| e.to_string())?;
    }
    camera.open_stream().map_err(|e| e.to_string())?;
    let fps = camera.frame_rate().max(1);
    Ok((camera, choices, fps))
}
