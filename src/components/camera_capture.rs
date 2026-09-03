use async_channel::Sender;
use image::{ColorType, ImageEncoder, codecs::jpeg::JpegEncoder};
use nokhwa::{
    Camera as NokhwaCamera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(super) enum CaptureMessage {
    Frame(Vec<u8>),
    Error(String),
}

pub(super) fn capture_frames(sender: Sender<CaptureMessage>, stop_capture: Arc<AtomicBool>) {
    let requested_format =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = match NokhwaCamera::new(CameraIndex::default(), requested_format) {
        Ok(camera) => camera,
        Err(error) => {
            let _ = sender.try_send(CaptureMessage::Error(format!(
                "Could not open camera: {error}"
            )));
            return;
        }
    };

    if let Err(error) = camera.open_stream() {
        let _ = sender.try_send(CaptureMessage::Error(format!(
            "Could not start camera: {error}"
        )));
        return;
    }

    while !stop_capture.load(Ordering::Relaxed) {
        let buffer = match camera.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                let _ = sender.try_send(CaptureMessage::Error(format!(
                    "Could not capture frame: {error}"
                )));
                break;
            }
        };
        let decoded = match buffer.decode_image::<RgbFormat>() {
            Ok(decoded) => decoded,
            Err(error) => {
                let _ = sender.try_send(CaptureMessage::Error(format!(
                    "Could not decode frame: {error}"
                )));
                break;
            }
        };

        let mut jpeg = Vec::new();
        let encoder = JpegEncoder::new_with_quality(&mut jpeg, 80);
        if let Err(error) = encoder.write_image(
            decoded.as_raw(),
            decoded.width(),
            decoded.height(),
            ColorType::Rgb8.into(),
        ) {
            let _ = sender.try_send(CaptureMessage::Error(format!(
                "Could not encode frame: {error}"
            )));
            break;
        }

        if sender.send_blocking(CaptureMessage::Frame(jpeg)).is_err() {
            break;
        }
    }
}
