use crate::theme::CAMERA_LETTERBOX_COLOR;
use gpui::{App, IntoElement, RenderOnce, Window, div, prelude::*, px};

#[derive(IntoElement)]
pub struct Toolbar;

impl RenderOnce for Toolbar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .absolute()
            .bottom(px(24.))
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(
                div()
                    .w(px(280.))
                    .h(px(48.))
                    .rounded_full()
                    .bg(CAMERA_LETTERBOX_COLOR.to_gpui(1.0)),
            )
    }
}
