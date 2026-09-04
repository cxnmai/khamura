use super::*;
use gpui::{AnyElement, ObjectFit, img};

impl Gallery {
    pub(super) fn navigate(&mut self, forward: bool, cx: &mut Context<Self>) {
        let items = &cx.global::<GalleryStore>().items;
        if let Some(index) = items
            .iter()
            .position(|item| Some(&item.path) == self.selected.as_ref())
        {
            let next = if forward {
                index.saturating_add(1)
            } else {
                index.saturating_sub(1)
            };
            if let Some(item) = items.get(next) {
                let path = item.path.clone();
                self.open_item(path, cx);
            }
        }
    }

    pub(super) fn content(&self, window: &Window, cx: &Context<Self>) -> AnyElement {
        let ink = super::super::settings::foreground(cx.global::<SessionSettings>().theme_color);
        if let Some(player) = &self.player {
            return div()
                .flex_1()
                .min_h_0()
                .child(player.clone())
                .into_any_element();
        }
        if let Some(path) = &self.selected {
            return div()
                .flex_1()
                .min_h_0()
                .p(px(20.))
                .child(
                    img(path.clone())
                        .size_full()
                        .object_fit(ObjectFit::Contain)
                        .with_fallback(move || {
                            div()
                                .size_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(ink.opacity(0.5))
                                .child("Photo unavailable")
                                .into_any_element()
                        }),
                )
                .into_any_element();
        }
        let store = cx.global::<GalleryStore>();
        if store.items.is_empty() {
            let message = if store.loading {
                "Loading captures…"
            } else if let Some(error) = &store.error {
                error
            } else {
                "Your photos and videos will appear here"
            };
            return div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .p(px(24.))
                .text_sm()
                .text_color(ink.opacity(0.5))
                .child(message.to_owned())
                .into_any_element();
        }
        let width = (f32::from(window.viewport_size().width) - 40.).max(1.);
        let columns = ((width + 8.) / 168.).floor().max(1.) as usize;
        let edge = ((width - (columns - 1) as f32 * 8.) / columns as f32).max(1.);
        div()
            .id("gallery-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .px(px(20.))
            .pb(px(20.))
            .when_some(store.error.clone(), |view, error| {
                view.child(
                    div()
                        .text_xs()
                        .text_color(ink.opacity(0.5))
                        .pb(px(12.))
                        .child(error),
                )
            })
            .child(div().flex().flex_wrap().gap(px(8.)).children(
                store.items.iter().enumerate().map(|(index, item)| {
                    let path = item.path.clone();
                    let keyboard_path = path.clone();
                    div()
                        .id(("gallery-photo", index))
                        .tab_index(0)
                        .on_key_down(cx.listener(
                            move |gallery, event: &gpui::KeyDownEvent, _, cx| {
                                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                    gallery.open_item(keyboard_path.clone(), cx);
                                    cx.stop_propagation();
                                }
                            },
                        ))
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
                }),
            ))
            .into_any_element()
    }
}
