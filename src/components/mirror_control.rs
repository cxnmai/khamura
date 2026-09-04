use crate::session_settings::SessionSettings;
use gpui::{IntoElement, div, prelude::*, px};

pub(super) fn mirror_control(enabled: bool, ink: gpui::Hsla, busy: bool) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child("Mirror")
        .child(
            div()
                .id("mirror-preview")
                .tab_index(0)
                .min_w(px(42.))
                .text_center()
                .px(px(10.))
                .py(px(5.))
                .rounded(px(7.))
                .bg(ink.opacity(if enabled { 0.1 } else { 0.04 }))
                .when(busy, |button| button.opacity(0.4))
                .when(!busy, |button| button.cursor_pointer())
                .hover(|style| style.bg(ink.opacity(0.14)))
                .on_click(|_, _, cx| {
                    if !cx.global::<crate::capture_settings::CaptureSettings>().busy {
                        SessionSettings::change(cx, |settings| settings.mirror = !settings.mirror);
                    }
                })
                .on_key_down(|event, _, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        if !cx.global::<crate::capture_settings::CaptureSettings>().busy {
                            SessionSettings::change(cx, |settings| {
                                settings.mirror = !settings.mirror
                            });
                        }
                        cx.stop_propagation();
                    }
                })
                .child(if enabled { "On" } else { "Off" }),
        )
}
