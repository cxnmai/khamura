mod actions;
mod error_popup;
mod overlays;
mod recording;
mod stream;
mod view;

use super::{
    camera_capture::{CaptureMessage, CaptureRequest, capture_frames},
    preview_frame::render_image,
    settings::{Settings, SettingsDismissed},
    toolbar::{CaptureRequested, GalleryRequested, SettingsToggled, Toolbar},
};
use crate::{
    capture_settings::{CameraMode, CaptureSettings, PhotoAspect},
    config::Config,
    session_settings::{CameraFit, SessionSettings},
};
use async_channel::{Receiver, Sender};
use gpui::{
    AnyWindowHandle, Context, Entity, FocusHandle, Focusable, IntoElement, MouseButton, ObjectFit,
    Render, RenderImage, Size, Subscription, Task, Window, div, img, prelude::*, px, size,
};
use image::RgbaImage;
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
enum Activity {
    Idle,
    Countdown(Instant),
    SavingPhoto,
    StartingVideo,
    Recording(Instant),
    Finalizing,
}

pub struct Camera {
    frame: Option<Arc<RenderImage>>,
    source_frame: Option<Arc<RgbaImage>>,
    rendered_mirror: bool,
    status: String,
    settings: Entity<Settings>,
    gallery: Option<Entity<super::gallery::Gallery>>,
    settings_open: bool,
    focus_pending: bool,
    previous_focus: Option<FocusHandle>,
    requests: Sender<CaptureRequest>,
    request: CaptureRequest,
    capture_ready: bool,
    fps: u32,
    activity: Activity,
    recorder: Option<crate::media::Recorder>,
    media_task: Option<Task<()>>,
    error: Option<String>,
    notice: Option<(String, Instant)>,
    window_handle: AnyWindowHandle,
    root_focus: FocusHandle,
    close_requested: bool,
    _tick_task: Task<()>,
    toolbar: Entity<Toolbar>,
    _toolbar_subscription: Subscription,
    _settings_subscription: Subscription,
    _capture_subscription: Subscription,
    _capture_task: Task<()>,
}

impl Camera {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (sender, receiver) = async_channel::bounded(2);
        let (requests, request_receiver) = async_channel::unbounded();
        let prefs = cx.global::<CaptureSettings>();
        let request = CaptureRequest {
            revision: 1,
            device: prefs.camera_device.clone(),
            quality: (prefs.mode == CameraMode::Video)
                .then_some(prefs.quality)
                .flatten(),
        };
        let _ = requests.try_send(request.clone());
        let capture_task = Self::receive_frames(cx, receiver);
        let tick_task = Self::tick_task(cx);
        let root_focus = cx.focus_handle();
        root_focus.focus(window);
        let weak = cx.weak_entity();
        window.on_window_should_close(cx, move |_, cx| {
            weak.update(cx, |camera, cx| camera.request_close(cx))
                .unwrap_or(true)
        });
        let toolbar = cx.new(Toolbar::new);
        let settings = cx.new(Settings::new);
        let toolbar_subscription = cx.subscribe(&toolbar, |camera, _, _: &SettingsToggled, cx| {
            camera.set_settings_open(!camera.settings_open, cx);
        });
        let settings_subscription =
            cx.subscribe(&settings, |camera, _, _: &SettingsDismissed, cx| {
                camera.set_settings_open(false, cx);
            });
        let capture_subscription = cx.subscribe(&toolbar, |camera, _, _: &CaptureRequested, cx| {
            camera.capture(cx)
        });
        cx.subscribe(&toolbar, |camera, _, _: &GalleryRequested, cx| {
            if !matches!(camera.activity, Activity::Idle) {
                return;
            }
            camera.set_settings_open(false, cx);
            crate::gallery::GalleryStore::refresh(cx);
            let gallery = cx.new(super::gallery::Gallery::new);
            cx.subscribe(
                &gallery,
                |camera, _, _: &super::gallery::GalleryDismissed, cx| {
                    camera.gallery = None;
                    camera.previous_focus = None;
                    camera.focus_pending = true;
                    cx.notify();
                },
            )
            .detach();
            camera.gallery = Some(gallery);
            cx.notify();
        })
        .detach();
        cx.observe_global::<CaptureSettings>(|camera, cx| camera.preferences_changed(cx))
            .detach();
        cx.observe_global::<SessionSettings>(|camera, cx| {
            if camera.rendered_mirror != cx.global::<SessionSettings>().mirror {
                camera.refresh_preview(cx);
            }
            cx.notify();
        })
        .detach();
        thread::spawn(move || capture_frames(sender, request_receiver));

        Self {
            frame: None,
            source_frame: None,
            rendered_mirror: cx.global::<SessionSettings>().mirror,
            status: "Starting camera…".into(),
            settings,
            gallery: None,
            settings_open: false,
            focus_pending: false,
            previous_focus: None,
            requests,
            request,
            capture_ready: false,
            fps: 30,
            activity: Activity::Idle,
            recorder: None,
            media_task: None,
            error: None,
            notice: None,
            close_requested: false,
            window_handle: window.window_handle(),
            root_focus,
            _tick_task: tick_task,
            toolbar,
            _toolbar_subscription: toolbar_subscription,
            _settings_subscription: settings_subscription,
            _capture_subscription: capture_subscription,
            _capture_task: capture_task,
        }
    }

    fn settings_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let trigger = self.toolbar.read(cx).settings_bounds();
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
                gpui::anchored()
                    .anchor(gpui::Corner::BottomRight)
                    .position(gpui::point(trigger.right(), trigger.top() - px(12.)))
                    .snap_to_window_with_margin(px(8.))
                    .child(self.settings.clone()),
            )
    }

    fn refresh_preview(&mut self, cx: &mut Context<Self>) {
        self.rendered_mirror = cx.global::<SessionSettings>().mirror;
        if let Some(source) = &self.source_frame {
            let prefs = cx.global::<CaptureSettings>();
            let aspect = if prefs.mode == CameraMode::Photo {
                prefs.aspect
            } else {
                PhotoAspect::Native
            };
            let (x, y, w, h) = crate::media::crop_bounds(source.width(), source.height(), aspect);
            let cropped = image::imageops::crop_imm(source.as_ref(), x, y, w, h).to_image();
            let image = Arc::new(render_image(&cropped, self.rendered_mirror));
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
        self.requests.close();
        if let Some(recorder) = &self.recorder {
            recorder.stop();
        }
    }
}
