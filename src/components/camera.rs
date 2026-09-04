use super::{
    camera_capture::{CapturedFrame, capture_frames},
    settings::{Settings, SettingsDismissed},
    toolbar::{SettingsToggled, Toolbar},
};
use crate::session_settings::{CameraFit, SessionSettings};
use async_channel::Receiver;
use gpui::{
    Context, Entity, FocusHandle, Focusable, IntoElement, MouseButton, ObjectFit, Render,
    RenderImage, Size, Subscription, Task, WeakEntity, Window, div, img, prelude::*, px, size,
};
use image::{Frame, ImageBuffer, Rgba};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

pub struct Camera {
    frame: Option<Arc<RenderImage>>,
    status: String,
    settings: Entity<Settings>,
    settings_open: bool,
    focus_pending: bool,
    previous_focus: Option<FocusHandle>,
    stop_capture: Arc<AtomicBool>,
    toolbar: Entity<Toolbar>,
    _toolbar_subscription: Subscription,
    _settings_subscription: Subscription,
    _capture_task: Task<()>,
}

impl Camera {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (sender, receiver) = async_channel::bounded(1);
        let stop_capture = Arc::new(AtomicBool::new(false));

        let capture_task = Self::receive_frames(cx, receiver);
        let capture_stop = Arc::clone(&stop_capture);
        let toolbar = cx.new(Toolbar::new);
        let settings = cx.new(Settings::new);
        let toolbar_subscription = cx.subscribe(&toolbar, |camera, _, _: &SettingsToggled, cx| {
            camera.set_settings_open(!camera.settings_open, cx);
        });
        let settings_subscription =
            cx.subscribe(&settings, |camera, _, _: &SettingsDismissed, cx| {
                camera.set_settings_open(false, cx);
            });
        cx.observe_global::<SessionSettings>(|_, cx| cx.notify())
            .detach();
        thread::spawn(move || capture_frames(sender, capture_stop));

        Self {
            frame: None,
            status: "Starting camera…".into(),
            settings,
            settings_open: false,
            focus_pending: false,
            previous_focus: None,
            stop_capture,
            toolbar,
            _toolbar_subscription: toolbar_subscription,
            _settings_subscription: settings_subscription,
            _capture_task: capture_task,
        }
    }

    fn settings_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .size_full()
            .child(div().absolute().size_full().on_mouse_down(
                MouseButton::Left,
                cx.listener(|camera, _, _, cx| {
                    camera.set_settings_open(false, cx);
                    cx.stop_propagation();
                }),
            ))
            .child(
                div()
                    .absolute()
                    .top(px(16.))
                    .bottom(px(84.))
                    .left_0()
                    .right_0()
                    .flex()
                    .justify_center()
                    .items_end()
                    .child(self.settings.clone()),
            )
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

    fn set_settings_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.settings_open != open {
            self.settings_open = open;
            self.focus_pending = true;
            self.toolbar
                .update(cx, |toolbar, cx| toolbar.set_settings_open(open, cx));
            cx.notify();
        }
    }
}

impl Render for Camera {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_pending {
            self.focus_pending = false;
            if self.settings_open {
                self.previous_focus = window.focused(cx);
                self.settings.read(cx).focus_handle(cx).focus(window);
            } else if let Some(focus) = self.previous_focus.take() {
                focus.focus(window);
            } else {
                window.blur();
            }
        }
        let config = cx.global::<SessionSettings>();
        if let Some(frame) = self.frame.clone() {
            let image = match config.fit {
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
                .bg(config.theme_color.to_gpui(config.background_opacity))
                .child(image)
                .when(self.settings_open, |view| {
                    view.child(self.settings_overlay(cx))
                })
                .child(self.toolbar.clone())
        } else {
            div()
                .size_full()
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .bg(gpui::black())
                .text_color(gpui::white())
                .child(self.status.clone())
                .when(self.settings_open, |view| {
                    view.child(self.settings_overlay(cx))
                })
                .child(self.toolbar.clone())
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
