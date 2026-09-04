use crate::session_settings::SessionSettings;
use gpui::{IntoElement, div, prelude::*, px};

pub(super) fn mirror_control(enabled: bool, ink: gpui::Hsla) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child("Mirror")
        .child(
            div()
                .id("mirror-preview")
                .tab_index(0)
                .px(px(12.))
                .py(px(5.))
                .rounded_full()
                .bg(ink.opacity(if enabled { 0.18 } else { 0.08 }))
                .cursor_pointer()
                .hover(|style| style.bg(ink.opacity(0.25)))
                .on_click(|_, _, cx| {
                    SessionSettings::change(cx, |settings| settings.mirror = !settings.mirror);
                })
                .on_key_down(|event, _, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        SessionSettings::change(cx, |settings| settings.mirror = !settings.mirror);
                        cx.stop_propagation();
                    }
                })
                .child(if enabled { "On" } else { "Off" }),
        )
}
