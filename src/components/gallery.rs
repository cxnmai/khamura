use crate::{gallery::GalleryStore, icons, session_settings::SessionSettings};
use gpui::{App, Context, EventEmitter, FocusHandle, Focusable, Window, div, prelude::*, px, svg};
use std::path::PathBuf;

#[path = "gallery_content.rs"]
mod content;

pub struct GalleryDismissed;
pub struct Gallery {
    focus: FocusHandle,
    focus_pending: bool,
    selected: Option<PathBuf>,
}

impl Gallery {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<GalleryStore>(|_, cx| cx.notify())
            .detach();
        cx.observe_global::<SessionSettings>(|_, cx| cx.notify())
            .detach();
        Self {
            focus: cx.focus_handle(),
            focus_pending: true,
            selected: None,
        }
    }

    pub fn focus_on_open(&mut self, cx: &mut Context<Self>) {
        self.focus_pending = true;
        self.selected = None;
        cx.notify();
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        if self.selected.take().is_none() {
            cx.emit(GalleryDismissed);
        }
        cx.notify();
    }
}

impl Focusable for Gallery {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl EventEmitter<GalleryDismissed> for Gallery {}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_pending {
            self.focus_pending = false;
            self.focus.focus(window);
        }
        let theme = cx.global::<SessionSettings>().theme_color;
        let ink = super::settings::foreground(theme);
        let title = self
            .selected
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Gallery".into());
        div()
            .id("gallery-page")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.to_gpui(1.))
            .text_color(ink)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|gallery, event: &gpui::KeyDownEvent, _, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => gallery.back(cx),
                    "left" | "right" => gallery.navigate(event.keystroke.key == "right", cx),
                    _ => {}
                }
                cx.stop_propagation();
            }))
            .child(
                div()
                    .h(px(64.))
                    .flex_shrink_0()
                    .px(px(20.))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .id("gallery-back")
                            .size(px(32.))
                            .rounded_full()
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|style| style.bg(ink.opacity(0.07)))
                            .on_click(cx.listener(|gallery, _, _, cx| gallery.back(cx)))
                            .child(svg().path(icons::ARROW_LEFT).size(px(18.)).text_color(ink)),
                    )
                    .child(div().text_sm().truncate().child(title)),
            )
            .child(self.content(window, cx))
    }
}
