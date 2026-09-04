use super::{
    camera_capture::{CapturedFrame, capture_frames},
    preview_frame::render_image,
    settings::{Settings, SettingsDismissed},
    toolbar::{SettingsToggled, Toolbar},
};
use crate::session_settings::{CameraFit, SessionSettings};
use async_channel::Receiver;
use gpui::{
    Context, Entity, FocusHandle, Focusable, IntoElement, MouseButton, ObjectFit, Render,
    RenderImage, Size, Subscription, Task, WeakEntity, Window, div, img, prelude::*, px, size,
};
use image::RgbaImage;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

pub struct Camera {
    frame: Option<Arc<RenderImage>>,
    source_frame: Option<RgbaImage>,
    rendered_mirror: bool,
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
        cx.observe_global::<SessionSettings>(|camera, cx| {
            if camera.rendered_mirror != cx.global::<SessionSettings>().mirror {
                camera.refresh_preview(cx);
            }
            cx.notify();
        })
        .detach();
        thread::spawn(move || capture_frames(sender, capture_stop));

        Self {
            frame: None,
            source_frame: None,
            rendered_mirror: cx.global::<SessionSettings>().mirror,
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
                                if let Some(source) =
                                    RgbaImage::from_raw(frame.width, frame.height, frame.pixels)
                                {
                                    camera.source_frame = Some(source);
                                    camera.refresh_preview(cx);
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

    fn refresh_preview(&mut self, cx: &mut Context<Self>) {
        self.rendered_mirror = cx.global::<SessionSettings>().mirror;
        if let Some(source) = &self.source_frame {
            let image = Arc::new(render_image(source, self.rendered_mirror));
            if let Some(previous_frame) = self.frame.replace(image) {
                cx.drop_image(previous_frame, None);
            }
        }
    }

    fn set_settings_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.settings_open != open {
            if !open {
                SessionSettings::save(cx);
            }
            self.settings_open = open;
            self.focus_pending = true;
            self.toolbar
                .update(cx, |toolbar, cx| toolbar.set_settings_open(open, cx));
            cx.notify();
        }
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        self.stop_capture.store(true, Ordering::Relaxed);
    }
}
