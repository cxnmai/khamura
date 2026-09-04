use super::*;

pub(super) fn tile(
    item: &crate::gallery::GalleryItem,
    index: usize,
    edge: f32,
    ink: gpui::Hsla,
    cx: &Context<Gallery>,
) -> gpui::Stateful<gpui::Div> {
    let path = item.path.clone();
    let keyboard_path = path.clone();
    div()
        .id(("gallery-photo", index))
        .tab_index(0)
        .on_key_down(
            cx.listener(move |gallery, event: &gpui::KeyDownEvent, _, cx| {
                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    gallery.open_item(keyboard_path.clone(), cx);
                    cx.stop_propagation();
                }
            }),
        )
        .relative()
        .size(px(edge))
        .rounded(px(6.))
        .overflow_hidden()
        .bg(ink.opacity(0.04))
        .cursor_pointer()
        .hover(|style| style.opacity(0.8))
        .on_click(cx.listener(move |gallery, _, _, cx| {
            gallery.open_item(path.clone(), cx);
        }))
        .when_some(item.thumbnail.clone(), |view, thumbnail| {
            view.child(img(thumbnail).size_full().object_fit(ObjectFit::Cover))
        })
        .when(item.is_video, |view| {
            view.child(
                div()
                    .absolute()
                    .bottom(px(8.))
                    .right(px(8.))
                    .size(px(24.))
                    .rounded_full()
                    .bg(gpui::black().opacity(0.5))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path(icons::PLAY)
                            .size(px(12.))
                            .text_color(gpui::white()),
                    ),
            )
        })
}
