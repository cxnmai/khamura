#[path = "gallery_grid.rs"]
mod grid;

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
                        .image_cache(&self.photo_cache)
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
        grid::grid(self, window, cx)
    }
}
