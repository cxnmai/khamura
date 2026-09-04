use gpui::{App, Context, IntoElement, Render, Window, div, prelude::*, px};
pub(crate) struct Tooltip(pub String);
impl Render for Tooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(8.))
            .py(px(5.))
            .rounded(px(6.))
            .bg(gpui::black())
            .text_color(gpui::white())
            .text_xs()
            .child(self.0.clone())
    }
}
pub(super) fn button(
    id: &'static str,
    icon: &'static str,
    badge: Option<String>,
    tooltip: String,
    busy: bool,
    active: bool,
    color: gpui::Hsla,
    click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .relative()
        .size(px(36.))
        .flex_shrink_0()
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .text_color(color.opacity(if busy { 0.3 } else { 0.9 }))
        .when(active, |b| b.bg(color.opacity(0.12)))
        .when(!busy, |b| {
            b.cursor_pointer().hover(|s| s.bg(color.opacity(0.1)))
        })
        .tooltip(move |_, cx| cx.new(|_| Tooltip(tooltip.clone())).into())
        .on_click(click)
        .child(
            gpui::svg()
                .path(icon)
                .size(px(19.))
                .text_color(color.opacity(if busy {
                    0.3
                } else if active {
                    1.0
                } else {
                    0.65
                })),
        )
        .when_some(badge, |button, badge| {
            button.child(
                div()
                    .absolute()
                    .right(px(1.))
                    .bottom_0()
                    .text_size(px(9.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(badge),
            )
        })
}
