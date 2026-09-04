use async_channel::Sender;
use nokhwa::{
    Camera as NokhwaCamera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};


pub(super) struct CapturedFrame {
    pub(super) pixels: Vec<u8>,
    pub(super) width: u32,
    pub(super) height: u32,
}

