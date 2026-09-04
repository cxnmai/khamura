use std::{cell::Cell, rc::Rc};
#[path = "device_options.rs"]
mod options;
use crate::{
    capture_settings::CaptureSettings, device_catalog::DeviceCatalog,
    session_settings::SessionSettings,
};
use gpui::{Context, IntoElement, Render, Window, div, prelude::*, px};

pub struct DeviceSettings {
    catalog: DeviceCatalog,
    loading: bool,
    open: Option<bool>,
    focus: gpui::FocusHandle,
    bounds: [Rc<Cell<gpui::Bounds<gpui::Pixels>>>; 2],
}
impl DeviceSettings {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<CaptureSettings>(|this, cx| {
            if cx.global::<CaptureSettings>().busy {
                this.open = None;
            }
            cx.notify();
        })
        .detach();
        let mut this = Self {
            catalog: DeviceCatalog::default(),
            loading: false,
            open: None,
            focus: cx.focus_handle(),
            bounds: std::array::from_fn(|_| Rc::new(Cell::new(gpui::Bounds::default()))),
        };
        this.refresh(cx);
        this
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.loading || cx.global::<CaptureSettings>().busy {
            return;
        }
        self.loading = true;
        let query = cx
            .background_executor()
            .spawn(async { DeviceCatalog::discover() });
        cx.spawn(async move |this, cx| {
            let catalog = query.await;
            let _ = this.update(cx, |this, cx| {
                this.catalog = catalog;
                this.loading = false;
                cx.notify();
            });
        })
        .detach();
    }

    fn selector(&self, camera: bool, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = cx.global::<CaptureSettings>();
        let selected = if camera {
            settings.camera_device.clone()
        } else {
            settings.microphone_device.clone()
        };
        let devices = if camera {
            &self.catalog.cameras
        } else {
            &self.catalog.microphones
        };
        let label = selected
            .as_ref()
            .map(|id| {
                devices
                    .iter()
                    .find(|device| &device.id == id)
                    .map(|device| options::label(device, camera))
                    .unwrap_or_else(|| format!("{id} (unavailable)"))
            })
            .unwrap_or_else(|| "System default".into());
        let busy = settings.busy;
        let mut options = vec![(None, "System default".to_string())];
        options.extend(
            devices
                .iter()
                .map(|device| (Some(device.id.clone()), options::label(device, camera))),
        );
        if let Some(id) = &selected {
            if !devices.iter().any(|device| &device.id == id) {
                options.push((Some(id.clone()), format!("{id} (unavailable)")));
            }
        }
        let bounds = self.bounds[usize::from(camera)].clone();
        let ink = super::settings::foreground(cx.global::<SessionSettings>().theme_color);
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                div()
                    .text_xs()
                    .text_color(ink.opacity(0.65))
                    .child(if camera { "Camera" } else { "Microphone" }),
            )
            .child(
                div()
                    .id(if camera {
                        "camera-device"
                    } else {
                        "microphone-device"
                    })
                    .relative()
                    .child(
                        gpui::canvas(move |rect, _, _| bounds.set(rect), |_, _, _, _| {})
                            .absolute()
                            .size_full(),
                    )
                    .tab_index(0)
                    .border_1()
                    .border_color(ink.opacity(0.1))
                    .bg(ink.opacity(0.025))
                    .rounded(px(8.))
                    .px(px(8.))
                    .py(px(6.))
                    .text_xs()
                    .truncate()
                    .child(format!("{label} ▾"))
                    .when(busy, |field| field.opacity(0.5))
                    .when(!busy, |field| {
                        field
                            .cursor_pointer()
                            .hover(|style| style.bg(ink.opacity(0.055)))
                    })
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.toggle(camera, window, cx)),
                    )
                    .on_key_down(cx.listener(
                        move |this, event: &gpui::KeyDownEvent, window, cx| {
                            if event.keystroke.key == "enter" || event.keystroke.key == "space" {
                                this.toggle(camera, window, cx);
                                cx.stop_propagation();
                            }
                        },
                    )),
            )
            .when(self.open == Some(camera) && !busy, |panel| {
                panel.child(options::popup(
                    self,
                    camera,
                    options,
                    selected.clone(),
                    window,
                    cx,
                ))
            })
    }

    fn toggle(&mut self, camera: bool, window: &mut Window, cx: &mut Context<Self>) {
        if cx.global::<CaptureSettings>().busy {
            return;
        }
        self.open = if self.open == Some(camera) {
            None
        } else {
            Some(camera)
        };
        window.focus(&self.focus);
        cx.notify();
    }

    fn select(&mut self, camera: bool, id: Option<String>, cx: &mut Context<Self>) {
        CaptureSettings::change(cx, |settings| {
            if camera {
                settings.camera_device = id;
                settings.quality = None;
                settings.qualities.clear();
            } else {
                settings.microphone_device = id;
            }
        });
        self.open = None;
        cx.notify();
    }
}

impl Render for DeviceSettings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ink = super::settings::foreground(cx.global::<SessionSettings>().theme_color);
        div()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" && this.open.take().is_some() {
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(self.selector(true, window, cx))
            .child(self.selector(false, window, cx))
            .child(
                div()
                    .id("refresh-devices")
                    .text_xs()
                    .text_color(ink.opacity(0.55))
                    .hover(|style| style.text_color(ink.opacity(0.85)))
                    .cursor_pointer()
                    .child(if self.loading {
                        "Finding devices…"
                    } else {
                        "Refresh devices"
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
            )
            .children(
                self.catalog
                    .errors
                    .iter()
                    .map(|error| div().text_xs().child(error.clone())),
            )
    }
}
