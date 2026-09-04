use crate::capture_settings::VideoQuality;
use async_channel::{Receiver, Sender};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[path = "camera_decode.rs"]
mod decode;

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
pub(super) fn capture_frames(
    sender: Sender<CaptureMessage>,
    requests: Receiver<CaptureRequest>,
    decode_enabled: Arc<AtomicBool>,
) {
    let mut next = requests.recv_blocking().ok();
    while let Some(mut request) = next.take() {
        while let Ok(newer) = requests.try_recv() {
            request = newer;
        }
        let result = stream(&sender, &requests, &request, &decode_enabled);
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
    decode_enabled: &AtomicBool,
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
        // Keep draining the device, but do not decode pixels the UI cannot accept.
        // This worker is the sole sender, so a free slot cannot fill during decoding.
        if sender.is_full() || !decode_enabled.load(Ordering::Relaxed) {
            continue;
        }
        let decoded = decode::decode(&buffer)?;
        let _ = sender.try_send(CaptureMessage::Frame(request.revision, decoded));
    }
}
