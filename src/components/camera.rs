use super::{
    camera_capture::{CapturedFrame, capture_frames},
    toolbar::Toolbar,
};
use crate::theme::{CAMERA_LETTERBOX_COLOR, CAMERA_LETTERBOX_OPACITY};
use async_channel::Receiver;
use gpui::{
    Context, IntoElement, ObjectFit, Render, RenderImage, Size, Task, WeakEntity, Window, div, img,
    prelude::*, size,
};
use image::{Frame, ImageBuffer, Rgba};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraFit {
    Contain,
    Cover,
}

impl Default for CameraFit {
    fn default() -> Self {
        Self::Contain
    }
}

pub struct Camera {
    frame: Option<Arc<RenderImage>>,
    status: String,
    fit: CameraFit,
    stop_capture: Arc<AtomicBool>,
    _capture_task: Task<()>,
}

impl Camera {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (sender, receiver) = async_channel::bounded(1);
        let stop_capture = Arc::new(AtomicBool::new(false));

        let capture_task = Self::receive_frames(cx, receiver);
        let capture_stop = Arc::clone(&stop_capture);
        thread::spawn(move || capture_frames(sender, capture_stop));

        Self {
            frame: None,
            status: "Starting camera…".into(),
            fit: CameraFit::default(),
            stop_capture,
            _capture_task: capture_task,
        }
    }

    fn receive_frames(
        cx: &mut Context<Self>,
        receiver: Receiver<Result<CapturedFrame, String>>,
    ) -> Task<()> {
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            while let Ok(message) = receiver.recv().await {
                let update_succeeded = this
                    .update(&mut *cx, |camera, cx| {
                        match message {
                            Ok(frame) => {
                                if let Some(image) = render_image(frame) {
                                    let previous_frame = camera.frame.replace(Arc::new(image));
                                    if let Some(previous_frame) = previous_frame {
                                        cx.drop_image(previous_frame, None);
                                    }
                                    camera.status.clear();
                                }
                            }
                            Err(error) => {
                                camera.status = error;
                            }
                        }
                        cx.notify();
                    })
                    .is_ok();

                if !update_succeeded {
                    break;
                }
            }
        })
    }

    pub fn set_fit(&mut self, fit: CameraFit, cx: &mut Context<Self>) {
        if self.fit != fit {
            self.fit = fit;
            cx.notify();
        }
    }

    pub fn toggle_fit(&mut self, cx: &mut Context<Self>) {
        let fit = match self.fit {
            CameraFit::Contain => CameraFit::Cover,
            CameraFit::Cover => CameraFit::Contain,
        };
        self.set_fit(fit, cx);
    }
}

impl Render for Camera {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(frame) = self.frame.clone() {
            let image = match self.fit {
                CameraFit::Contain => {
                    // Size the painted image explicitly so resizing the window always leaves
                    // the unused area outside the image as letterbox space.
                    let image_size = contain_size(window.viewport_size(), frame.size(0));
                    img(frame)
                        .w(image_size.width)
                        .h(image_size.height)
                        .object_fit(ObjectFit::Fill)
                }
                CameraFit::Cover => img(frame).size_full().object_fit(ObjectFit::Cover),
            };

            div()
                .size_full()
                .relative()
                .flex()
                .items_center()
                .justify_center()
                // The image is opaque; only the letterbox area uses this alpha.
                .bg(CAMERA_LETTERBOX_COLOR.alpha(CAMERA_LETTERBOX_OPACITY))
                .child(image)
                .child(Toolbar)
        } else {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(gpui::black())
                .text_color(gpui::white())
                .child(self.status.clone())
                .child(Toolbar)
        }
    }
}

fn contain_size(
    viewport: Size<gpui::Pixels>,
    image: Size<gpui::DevicePixels>,
) -> Size<gpui::Pixels> {
    let image_ratio = image.width.0 as f32 / image.height.0 as f32;
    let viewport_ratio = viewport.width / viewport.height;

    if viewport_ratio > image_ratio {
        size(viewport.height * image_ratio, viewport.height)
    } else {
        size(viewport.width, viewport.width / image_ratio)
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        self.stop_capture.store(true, Ordering::Relaxed);
    }
}

fn render_image(frame: CapturedFrame) -> Option<RenderImage> {
    let buffer =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(frame.width, frame.height, frame.pixels)?;
    Some(RenderImage::new(vec![Frame::new(buffer)]))
}
