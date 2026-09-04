use crate::session_settings::SessionSettings;
use gpui::{IntoElement, div, prelude::*, px};

pub(super) fn save_status(settings: &SessionSettings, ink: gpui::Hsla) -> impl IntoElement {
    let message = match &settings.save_error {
        Some(error) => format!("Could not save settings: {error}"),
        None if settings.dirty => "Release slider to save changes".into(),
        None => "Changes save automatically".into(),
    };
    div()
        .text_xs()
        .text_color(ink.opacity(0.7))
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(message)
        .when(settings.save_error.is_some(), |view| {
            view.child(
                div()
                    .id("retry-save")
                    .cursor_pointer()
                    .text_color(ink)
                    .on_click(|_, _, cx| SessionSettings::save(cx))
                    .child("Retry saving"),
            )
        })
}
