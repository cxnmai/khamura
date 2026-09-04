use gpui::{prelude::*, div, px, App, Context, IntoElement, Render, Window};
pub(super) struct Tooltip(pub String);
impl Render for Tooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().px(px(8.)).py(px(5.)).rounded(px(6.)).bg(gpui::black()).text_color(gpui::white()).text_xs().child(self.0.clone())
    }
}
pub(super) fn button(id: &'static str, label: String, tooltip: String, busy: bool, active: bool, color: gpui::Hsla, click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    div().id(id).h(px(32.)).px(px(7.)).rounded(px(8.)).flex().items_center().text_xs()
        .text_color(color.opacity(if busy { 0.3 } else { 0.9 }))
        .when(active, |b| b.bg(color.opacity(0.12)))
        .when(!busy, |b| b.cursor_pointer().hover(|s| s.bg(color.opacity(0.1))))
        .tooltip(move |_, cx| cx.new(|_| Tooltip(tooltip.clone())).into()).on_click(click)
        .gap(px(4.))
        .when_some(match id { "timer" => Some(crate::icons::TIMER), "microphone" => Some(crate::icons::MIC), "grid" => Some(crate::icons::GRID), _ => None }, |b, path| b.child(gpui::svg().path(path).size(px(15.))))
        .child(label)
}
