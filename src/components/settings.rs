use super::opacity_slider::OpacitySlider;
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
}
impl Settings {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<SessionSettings>(|_, cx| cx.notify())
            .detach();
        Self {
            focus: cx.focus_handle(),
            opacity: cx.new(OpacitySlider::new),
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = cx.global::<SessionSettings>();
        let selected_fit = settings.fit;
        let selected_color = settings.theme_color;
        let opacity = settings.background_opacity;
        let foreground = foreground(selected_color);
        let fits = [
            ("preview-fit", "Fit", CameraFit::Contain),
            ("preview-fill", "Fill", CameraFit::Cover),
        ];
        let swatches = [
            ("color-black", Rgb::new(0, 0, 0)),
            ("color-slate", Rgb::new(45, 55, 72)),
            ("color-blue", Rgb::new(30, 58, 95)),
            ("color-plum", Rgb::new(70, 42, 65)),
            ("color-cream", Rgb::new(232, 227, 216)),
        ];
        div()
            .id("settings-panel")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, event, _, cx| {
                if event.keystroke.key == "escape" {
                    cx.emit(SettingsDismissed);
                    cx.stop_propagation();
                }
            }))
            .occlude()
            .w(px(280.))
            .max_w_full()
            .max_h_full()
            .overflow_y_scroll()
            .p(px(16.))
            .rounded(px(20.))
            .border_1()
            .border_color(foreground.opacity(0.16))
            .bg(selected_color.to_gpui(1.))
            .text_color(foreground)
            .text_sm()
            .flex()
            .flex_col()
            .gap(px(16.))
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
                            .p(px(3.))
                            .rounded_full()
                            .bg(foreground.opacity(0.08))
                            .children(fits.into_iter().map(|(id, label, fit)| {
                                let selected = selected_fit == fit;
                                div()
                                    .id(id)
                                    .px(px(12.))
                                    .py(px(5.))
                                    .rounded_full()
                                    .cursor_pointer()
                                    .when(selected, |button| button.bg(foreground.opacity(0.18)))
                                    .hover(|style| style.bg(foreground.opacity(0.12)))
                                    .on_click(move |_, _, cx| {
                                        cx.update_global::<SessionSettings, _>(|settings, _| {
                                            settings.fit = fit
                                        });
                                    })
                                    .child(label)
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child("Appearance")
                    .child(div().flex().gap(px(10.)).children(swatches.into_iter().map(
                        |(id, color)| {
                            div()
                                .id(id)
                                .size(px(30.))
                                .rounded_full()
                                .cursor_pointer()
                                .border_2()
                                .border_color(if selected_color == color {
                                    foreground
                                } else {
                                    foreground.opacity(0.25)
                                })
                                .bg(color.to_gpui(1.))
                                .hover(|style| style.border_color(foreground))
                                .on_click(move |_, _, cx| {
                                    cx.update_global::<SessionSettings, _>(|settings, _| {
                                        settings.theme_color = color
                                    });
                                })
                        },
                    ))),
            )
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
                            .child(format!("{:.0}%", opacity * 100.)),
                    )
                    .child(self.opacity.clone()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(foreground.opacity(0.6))
                    .child("Changes apply for this session"),
            )
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
