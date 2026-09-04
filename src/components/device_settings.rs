use crate::{
    capture_settings::CaptureSettings,
    device_catalog::{Device, DeviceCatalog},
    session_settings::SessionSettings,
};
use gpui::{Context, IntoElement, Render, Window, div, prelude::*, px};

pub struct DeviceSettings {
    catalog: DeviceCatalog,
    loading: bool,
    open: Option<bool>,
}
impl DeviceSettings {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<CaptureSettings>(|this, cx| {
            if cx.global::<CaptureSettings>().busy { this.open = None; }
            cx.notify();
        }).detach();
        let mut this = Self { catalog: DeviceCatalog::default(), loading: false, open: None };
        this.refresh(cx);
        this
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.loading || cx.global::<CaptureSettings>().busy { return; }
        self.loading = true;
        let query = cx.background_executor().spawn(async { DeviceCatalog::discover() });
        cx.spawn(async move |this, cx| {
            let catalog = query.await;
            let _ = this.update(cx, |this, cx| {
                this.catalog = catalog;
                this.loading = false;
                cx.notify();
            });
        }).detach();
    }

    fn selector(&self, camera: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = cx.global::<CaptureSettings>();
        let selected = if camera { &settings.camera_device } else { &settings.microphone_device };
        let devices = if camera { &self.catalog.cameras } else { &self.catalog.microphones };
        let label = selected.as_ref().map(|id| devices.iter().find(|device| &device.id == id)
            .map(|device| device.label.clone()).unwrap_or_else(|| format!("{id} (unavailable)")))
            .unwrap_or_else(|| "System default".into());
        let busy = settings.busy;
        let mut options = vec![(None, "System default".to_string())];
        options.extend(devices.iter().map(|Device { id, label }| (Some(id.clone()), label.clone())));
        if let Some(id) = selected {
            if !devices.iter().any(|device| &device.id == id) {
                options.push((Some(id.clone()), format!("{id} (unavailable)")));
            }
        }
        let ink = super::settings::foreground(cx.global::<SessionSettings>().theme_color);
        div().flex().flex_col().gap(px(4.))
            .child(if camera { "Camera" } else { "Microphone" })
            .child(div().id(if camera { "camera-device" } else { "microphone-device" })
                .tab_index(0).border_1().border_color(ink.opacity(0.2)).rounded(px(6.))
                .px(px(8.)).py(px(7.)).text_xs().truncate().child(format!("{label} ▾"))
                .when(busy, |field| field.opacity(0.5))
                .when(!busy, |field| field.cursor_pointer().hover(|style| style.bg(ink.opacity(0.1))))
                .on_click(cx.listener(move |this, _, _, cx| this.toggle(camera, cx)))
                .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, _, cx| {
                    if event.keystroke.key == "enter" || event.keystroke.key == "space" {
                        this.toggle(camera, cx);
                        cx.stop_propagation();
                    }
                })))
            .when(self.open == Some(camera) && !busy, |panel| panel.child(
                div().id(if camera { "camera-options" } else { "microphone-options" })
                    .max_h(px(160.)).overflow_y_scroll().rounded(px(6.)).border_1()
                    .border_color(ink.opacity(0.2)).children(options.into_iter().enumerate().map(|(index, (id, label))| {
                        let chosen = &id == selected;
                        div().id((if camera { "camera-option" } else { "microphone-option" }, index))
                            .tab_index(0).px(px(8.)).py(px(7.)).text_xs().truncate()
                            .cursor_pointer().when(chosen, |item| item.bg(ink.opacity(0.12)))
                            .hover(|style| style.bg(ink.opacity(0.08))).child(label)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.select(camera, id.clone(), cx);
                            }))
                    }))
            ))
    }

    fn toggle(&mut self, camera: bool, cx: &mut Context<Self>) {
        if cx.global::<CaptureSettings>().busy { return; }
        self.open = if self.open == Some(camera) { None } else { Some(camera) };
        cx.notify();
    }

    fn select(&mut self, camera: bool, id: Option<String>, cx: &mut Context<Self>) {
        CaptureSettings::change(cx, |settings| {
            if camera {
                settings.camera_device = id;
                settings.quality = None;
                settings.qualities.clear();
            } else { settings.microphone_device = id; }
        });
        self.open = None;
        cx.notify();
    }
}

impl Render for DeviceSettings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_col().gap(px(12.))
            .child(self.selector(true, cx))
            .child(self.selector(false, cx))
            .child(div().id("refresh-devices").text_xs().cursor_pointer()
                .child(if self.loading { "Finding devices…" } else { "Refresh devices" })
                .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))))
            .children(self.catalog.errors.iter().map(|error| div().text_xs().child(error.clone())))
    }
}
