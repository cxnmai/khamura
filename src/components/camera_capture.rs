use async_channel::Sender;
use nokhwa::{
    Camera as NokhwaCamera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(super) struct CapturedFrame {
    pub(super) pixels: Vec<u8>,
    pub(super) width: u32,
    pub(super) height: u32,
}

pub(super) fn capture_frames(
    sender: Sender<Result<CapturedFrame, String>>,
    stop_capture: Arc<AtomicBool>,
) {
    let requested_format =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = match NokhwaCamera::new(CameraIndex::default(), requested_format) {
        Ok(camera) => camera,
        Err(error) => {
            let _ = sender.try_send(Err(format!("Could not open camera: {error}")));
            return;
        }
    };

    if let Err(error) = camera.open_stream() {
        let _ = sender.try_send(Err(format!("Could not start camera: {error}")));
        return;
    }

    while !stop_capture.load(Ordering::Relaxed) {
        let buffer = match camera.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                let _ = sender.try_send(Err(format!("Could not capture frame: {error}")));
                break;
            }
        };
        let decoded = match buffer.decode_image::<RgbFormat>() {
            Ok(decoded) => decoded,
            Err(error) => {
                let _ = sender.try_send(Err(format!("Could not decode frame: {error}")));
                break;
            }
        };

        let width = decoded.width();
        let height = decoded.height();
        let mut bgra = Vec::with_capacity(decoded.as_raw().len() / 3 * 4);
        for pixel in decoded.as_raw().chunks_exact(3) {
            // GPUI's RenderImage expects pixels in BGRA order.
            bgra.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
        }

        if sender
            .send_blocking(Ok(CapturedFrame {
                pixels: bgra,
                width,
                height,
            }))
            .is_err()
        {
            break;
        }
    }
}
