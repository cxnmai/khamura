use super::*;

pub(super) fn elapsed_label(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    if seconds >= 3600 {
        format!(
            "{:02}:{:02}:{:02}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60
        )
    } else {
        format!("{:02}:{:02}", seconds / 60, seconds % 60)
    }
}

pub(super) fn grid() -> impl IntoElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .children([1., 2.].into_iter().flat_map(|third| {
            [
                div()
                    .absolute()
                    .left(gpui::relative(third / 3.))
                    .top_0()
                    .bottom_0()
                    .w(px(1.))
                    .bg(gpui::white().opacity(0.5))
                    .into_any_element(),
                div()
                    .absolute()
                    .top(gpui::relative(third / 3.))
                    .left_0()
                    .right_0()
                    .h(px(1.))
                    .bg(gpui::white().opacity(0.5))
                    .into_any_element(),
            ]
        }))
}

impl Camera {
    pub(super) fn overlays(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let recording = match self.activity {
            Activity::Recording(start) => Some(elapsed_label(start.elapsed())),
            Activity::Finalizing => Some("Saving video…".into()),
            _ => None,
        };
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .when_some(recording, |view, time| {
                view.child(
                    div()
                        .absolute()
                        .top(px(24.))
                        .left_0()
                        .right_0()
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .px(px(14.))
                                .py(px(7.))
                                .rounded_full()
                                .bg(gpui::black().opacity(0.75))
                                .text_color(gpui::white())
                                .text_sm()
                                .child(div().size(px(7.)).rounded_full().bg(gpui::red()))
                                .child(time),
                        ),
                )
            })
            .when_some(
                if let Activity::Countdown(deadline) = self.activity {
                    Some(deadline.saturating_duration_since(Instant::now()).as_secs() + 1)
                } else {
                    None
                },
                |view, seconds| {
                    view.child(
                        div()
                            .absolute()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(gpui::white())
                            .child(
                                div()
                                    .rounded_full()
                                    .bg(gpui::black().opacity(0.6))
                                    .px(px(28.))
                                    .py(px(12.))
                                    .text_size(px(56.))
                                    .child(seconds.to_string()),
                            ),
                    )
                },
            )
            .when_some(
                self.notice.as_ref().map(|(message, _)| message.clone()),
                |view, message| {
                    view.child(
                        div()
                            .absolute()
                            .top(px(64.))
                            .left_0()
                            .right_0()
                            .flex()
                            .justify_center()
                            .child(
                                div()
                                    .max_w(px(380.))
                                    .rounded(px(10.))
                                    .px(px(12.))
                                    .py(px(8.))
                                    .bg(gpui::black().opacity(0.75))
                                    .text_color(gpui::white())
                                    .text_xs()
                                    .child(message),
                            ),
                    )
                },
            )
            .when_some(self.error.clone(), |view, error| {
                view.child(self.error_popup(error, cx))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recording_timer_formats_minutes_and_hours() {
        assert_eq!(elapsed_label(Duration::from_secs(0)), "00:00");
        assert_eq!(elapsed_label(Duration::from_secs(65)), "01:05");
        assert_eq!(elapsed_label(Duration::from_secs(3661)), "01:01:01");
    }
}
