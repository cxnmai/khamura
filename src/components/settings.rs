use super::opacity_slider::OpacitySlider;
use super::path_settings::PathSettings;
use crate::{
    session_settings::{CameraFit, SessionSettings},
    theme::Rgb,
};
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, Render, Window, div,
    prelude::*, px,
};
pub struct SettingsDismissed;
pub struct Settings {
    focus: FocusHandle,
    opacity: Entity<OpacitySlider>,
    paths: Entity<PathSettings>,
    devices: Entity<super::device_settings::DeviceSettings>,
}
impl Settings {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<SessionSettings>(|_, cx| cx.notify())
            .detach();
        cx.observe_global::<crate::capture_settings::CaptureSettings>(|_, cx| cx.notify())
            .detach();
        Self {
            focus: cx.focus_handle(),
            opacity: cx.new(OpacitySlider::new),
            paths: cx.new(PathSettings::new),
            devices: cx.new(super::device_settings::DeviceSettings::new),
        }
    }
}
impl Focusable for Settings {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl EventEmitter<SettingsDismissed> for Settings {}
impl Render for Settings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = cx.global::<SessionSettings>();
        let selected_fit = settings.fit;
        let selected_color = settings.theme_color;
        let opacity = settings.background_opacity;
        let foreground = foreground(selected_color);
        let fits = [
            ("preview-fit", "Fit", CameraFit::Contain),
            ("preview-fill", "Fill", CameraFit::Cover),
        ];
        div()
            .id("settings-panel")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, event: &gpui::KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" {
                    cx.emit(SettingsDismissed);
                    cx.stop_propagation();
                }
            }))
            .occlude()
            .w(px(280.))
            .max_w_full()
            .max_h((window.viewport_size().height - px(100.)).max(px(0.)))
            .overflow_y_scroll()
            .p(px(16.))
            .rounded(px(14.))
            .border_1()
            .border_color(foreground.opacity(0.1))
            .bg(selected_color.to_gpui(1.))
            .text_color(foreground)
            .text_xs()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Settings"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child("Preview fit")
                    .child(
                        div()
                            .flex()
                            .p(px(2.))
                            .rounded(px(7.))
                            .bg(foreground.opacity(0.04))
                            .children(fits.into_iter().map(|(id, label, fit)| {
                                let selected = selected_fit == fit;
                                div()
                                    .id(id)
                                    .px(px(10.))
                                    .py(px(5.))
                                    .rounded(px(7.))
                                    .cursor_pointer()
                                    .when(selected, |button| button.bg(foreground.opacity(0.1)))
                                    .hover(|style| style.bg(foreground.opacity(0.07)))
                                    .on_click(move |_, _, cx| {
                                        SessionSettings::change(cx, |settings| settings.fit = fit);
                                    })
                                    .child(label)
                            })),
                    ),
            )
            .child(super::mirror_control::mirror_control(
                settings.mirror,
                foreground,
                cx.global::<crate::capture_settings::CaptureSettings>().busy,
            ))
            .child(super::theme_palette::theme_palette(
                selected_color,
                foreground,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child("Background opacity")
                            .child(
                                div()
                                    .text_color(foreground.opacity(0.55))
                                    .child(format!("{:.0}%", opacity * 100.)),
                            ),
                    )
                    .child(self.opacity.clone()),
            )
            .child(self.devices.clone())
            .child(self.paths.clone())
            .when(settings.save_error.is_some(), |view| {
                view.child(super::save_status::save_status(settings, foreground))
            })
    }
}
pub(super) fn foreground(color: Rgb) -> gpui::Hsla {
    let brightness = 0.299 * color.r as f32 + 0.587 * color.g as f32 + 0.114 * color.b as f32;
    if brightness >= 127.5 {
        gpui::black()
    } else {
        gpui::white()
    }
}
