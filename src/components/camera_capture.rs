use crate::capture_settings::VideoQuality;
use async_channel::{Receiver, Sender};
use nokhwa::pixel_format::RgbFormat;

pub(super) struct CapturedFrame {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct CaptureRequest {
    pub revision: u64,
    pub device: Option<String>,
    pub quality: Option<VideoQuality>,
}

pub(super) enum CaptureMessage {
    Ready(u64, Vec<VideoQuality>, u32),
    Frame(u64, CapturedFrame),
    Error(u64, String),
}

// One device owner: switching closes the old stream before opening the new one.
// Preview frames are droppable, keeping capture independent of UI/encoding speed.
pub(super) fn capture_frames(sender: Sender<CaptureMessage>, requests: Receiver<CaptureRequest>) {
    let mut next = requests.recv_blocking().ok();
    while let Some(mut request) = next.take() {
        while let Ok(newer) = requests.try_recv() {
            request = newer;
        }
        let result = stream(&sender, &requests, &request);
        match result {
            Ok(replacement) => next = replacement,
            Err(error) => {
                if sender
                    .send_blocking(CaptureMessage::Error(request.revision, error))
                    .is_err()
                {
                    return;
                }
                next = requests.recv_blocking().ok();
            }
        }
    }
}

fn stream(
    sender: &Sender<CaptureMessage>,
    requests: &Receiver<CaptureRequest>,
    request: &CaptureRequest,
) -> Result<Option<CaptureRequest>, String> {
    let (mut camera, choices, fps) = super::capture_device::open(&request.device, request.quality)?;
    if sender
        .send_blocking(CaptureMessage::Ready(request.revision, choices, fps))
        .is_err()
    {
        return Ok(None);
    }
    loop {
        if let Ok(next) = requests.try_recv() {
            return Ok(Some(next));
        }
        if requests.is_closed() || sender.is_closed() {
            return Ok(None);
        }
        let buffer = camera
            .frame()
            .map_err(|e| format!("Camera capture failed: {e}"))?;
        let decoded = buffer
            .decode_image::<RgbFormat>()
            .map_err(|e| format!("Camera decoding failed: {e}"))?;
        let mut pixels = Vec::with_capacity(decoded.as_raw().len() / 3 * 4);
        for pixel in decoded.as_raw().chunks_exact(3) {
            pixels.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
        }
        let _ = sender.try_send(CaptureMessage::Frame(
            request.revision,
            CapturedFrame {
                pixels,
                width: decoded.width(),
                height: decoded.height(),
            },
        ));
    }
}
