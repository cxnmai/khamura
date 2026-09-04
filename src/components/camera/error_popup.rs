use super::*;

impl Camera {
    pub(super) fn error_popup(&self, error: String, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .occlude()
            .bg(gpui::black().opacity(0.25))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .id("capture-error")
                    .occlude()
                    .w(px(340.))
                    .max_w_full()
                    .max_h_full()
                    .overflow_y_scroll()
                    .p(px(20.))
                    .rounded(px(16.))
                    .bg(gpui::rgb(0x252830))
                    .text_color(gpui::white())
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child("Could not complete action")
                    .child(div().text_sm().child(error))
                    .when(
                        !self.capture_ready && matches!(self.activity, Activity::Idle),
                        |popup| {
                            popup.child(
                                div()
                                    .id("retry-camera")
                                    .tab_index(0)
                                    .cursor_pointer()
                                    .rounded(px(6.))
                                    .p(px(8.))
                                    .bg(gpui::white().opacity(0.12))
                                    .on_click(
                                        cx.listener(|camera, _, _, cx| camera.retry_camera(cx)),
                                    )
                                    .on_key_down(cx.listener(
                                        |camera, event: &gpui::KeyDownEvent, _, cx| {
                                            if matches!(
                                                event.keystroke.key.as_str(),
                                                "enter" | "space"
                                            ) {
                                                camera.retry_camera(cx);
                                                cx.stop_propagation();
                                            }
                                        },
                                    ))
                                    .child("Retry camera"),
                            )
                        },
                    )
                    .child(
                        div()
                            .id("dismiss-error")
                            .tab_index(0)
                            .cursor_pointer()
                            .rounded(px(6.))
                            .p(px(8.))
                            .bg(gpui::white().opacity(0.12))
                            .on_click(cx.listener(|camera, _, _, cx| {
                                camera.error = None;
                                cx.notify();
                            }))
                            .on_key_down(cx.listener(
                                |camera, event: &gpui::KeyDownEvent, _, cx| {
                                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                        camera.error = None;
                                        cx.notify();
                                        cx.stop_propagation();
                                    }
                                },
                            ))
                            .child("Dismiss · Esc"),
                    ),
            )
    }
}
