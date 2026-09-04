use super::Gallery;
use crate::{icons, session_settings::SessionSettings};
use gpui::{Bounds, Context, Pixels, div, prelude::*, px, svg};
use std::{cell::Cell, rc::Rc};

#[path = "gallery_clipboard.rs"]
mod clipboard;

#[derive(Clone, Copy)]
enum Action {
    Image,
    Path,
    Folder,
}

impl Gallery {
    pub(super) fn actions(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.global::<SessionSettings>().theme_color;
        let ink = super::super::settings::foreground(theme);
        let trigger = Rc::new(Cell::new(Bounds::<Pixels>::default()));
        let measured = trigger.clone();
        let mut items = Vec::new();
        if let Some(path) = &self.selected {
            if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
            {
                items.push((
                    if self.copy_busy {
                        "Copying…"
                    } else {
                        "Copy image"
                    },
                    Action::Image,
                ));
            }
            items.push(("Copy path", Action::Path));
        }
        items.push(("Open gallery folder", Action::Folder));
        div().relative().child(
            div()
                .id("gallery-options")
                .tab_index(0)
                .size(px(32.))
                .rounded_full()
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .hover(|style| style.bg(ink.opacity(0.07)))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.options_open = !this.options_open;
                    cx.notify();
                }))
                .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        this.options_open = !this.options_open;
                        cx.stop_propagation();
                        cx.notify();
                    }
                }))
                .child(
                    gpui::canvas(move |bounds, _, _| measured.set(bounds), |_, _, _, _| {})
                        .absolute()
                        .size_full(),
                )
                .child(svg().path(icons::ELLIPSIS).size(px(18.)).text_color(ink))
                .when(self.options_open, |button| {
                    button.child(
                        gpui::deferred(
                            div()
                                .id("gallery-actions-menu")
                                .absolute()
                                .top(px(38.))
                                .right_0()
                                .w(px(174.))
                                .p(px(4.))
                                .rounded(px(12.))
                                .bg(theme.to_gpui(1.))
                                .text_color(ink)
                                .border_1()
                                .border_color(ink.opacity(0.08))
                                .occlude()
                                .on_mouse_down_out(cx.listener(
                                    move |this, event: &gpui::MouseDownEvent, _, cx| {
                                        if !trigger.get().contains(&event.position) {
                                            this.options_open = false;
                                            cx.notify();
                                        }
                                    },
                                ))
                                .on_click(|_, _, cx| cx.stop_propagation())
                                .children(items.into_iter().enumerate().map(
                                    |(index, (label, action))| {
                                        div()
                                            .id(("gallery-action", index))
                                            .tab_index(0)
                                            .h(px(30.))
                                            .px(px(10.))
                                            .rounded(px(8.))
                                            .flex()
                                            .items_center()
                                            .text_xs()
                                            .cursor_pointer()
                                            .text_color(ink.opacity(0.75))
                                            .hover(|style| {
                                                style.bg(ink.opacity(0.06)).text_color(ink)
                                            })
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.perform(action, cx)
                                            }))
                                            .on_key_down(cx.listener(
                                                move |this, event: &gpui::KeyDownEvent, _, cx| {
                                                    if matches!(
                                                        event.keystroke.key.as_str(),
                                                        "enter" | "space"
                                                    ) {
                                                        this.perform(action, cx);
                                                        cx.stop_propagation();
                                                    }
                                                },
                                            ))
                                            .child(label)
                                    },
                                )),
                        )
                        .with_priority(2),
                    )
                }),
        )
    }
}
