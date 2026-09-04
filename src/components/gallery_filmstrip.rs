use super::*;
use gpui::{ObjectFit, img};

impl Gallery {
    pub(super) fn filmstrip(&self, cx: &Context<Self>) -> impl IntoElement {
        let ink = super::super::settings::foreground(cx.global::<SessionSettings>().theme_color);
        div()
            .id("gallery-filmstrip")
            .h(px(76.))
            .w_full()
            .flex_shrink_0()
            .min_w_0()
            .flex()
            .items_center()
            .gap(px(6.))
            .px(px(20.))
            .py(px(8.))
            .overflow_x_scroll()
            .track_scroll(&self.filmstrip_scroll)
            .children(
                cx.global::<GalleryStore>()
                    .items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        let selected = self.selected.as_ref() == Some(&item.path);
                        let path = item.path.clone();
                        let keyboard_path = path.clone();
                        div()
                            .id(("gallery-filmstrip-item", index))
                            .tab_index(0)
                            .relative()
                            .size(px(52.))
                            .flex_shrink_0()
                            .rounded(px(5.))
                            .overflow_hidden()
                            .border_1()
                            .border_color(ink.opacity(if selected { 0.8 } else { 0.08 }))
                            .bg(ink.opacity(0.04))
                            .cursor_pointer()
                            .opacity(if selected { 1. } else { 0.6 })
                            .hover(|style| style.opacity(1.))
                            .on_click(cx.listener(move |gallery, _, _, cx| {
                                gallery.open_item(path.clone(), cx);
                            }))
                            .on_key_down(cx.listener(
                                move |gallery, event: &gpui::KeyDownEvent, _, cx| {
                                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                        gallery.open_item(keyboard_path.clone(), cx);
                                        cx.stop_propagation();
                                    }
                                },
                            ))
                            .when_some(item.thumbnail.clone(), |view, thumbnail| {
                                view.child(img(thumbnail).size_full().object_fit(ObjectFit::Cover))
                            })
                            .when(item.is_video, |view| {
                                view.child(
                                    div()
                                        .absolute()
                                        .bottom(px(3.))
                                        .right(px(3.))
                                        .size(px(16.))
                                        .rounded_full()
                                        .bg(gpui::black().opacity(0.55))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            svg()
                                                .path(icons::PLAY)
                                                .size(px(9.))
                                                .text_color(gpui::white()),
                                        ),
                                )
                            })
                    }),
            )
    }
}
