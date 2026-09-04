use crate::{gallery::GalleryStore, icons, session_settings::SessionSettings};
use gpui::{App, Context, EventEmitter, FocusHandle, Focusable, Window, div, prelude::*, px, svg};
use std::path::PathBuf;

#[path = "gallery_actions.rs"]
mod actions;
#[path = "gallery_content.rs"]
mod content;

pub struct GalleryDismissed;
pub struct Gallery {
    focus: FocusHandle,
    focus_pending: bool,
    selected: Option<PathBuf>,
    player: Option<gpui::Entity<super::video_player::VideoPlayer>>,
    options_open: bool,
    action_notice: Option<String>,
    copy_busy: bool,
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
            player: None,
            options_open: false,
            action_notice: None,
            copy_busy: false,
        }
    }

    fn open_item(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.player = None;
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("mp4"))
        {
            self.player =
                Some(cx.new(|cx| super::video_player::VideoPlayer::new(path.clone(), cx)));
        }
        self.selected = Some(path);
        self.options_open = false;
        self.action_notice = None;
        cx.notify();
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        if self.options_open {
            self.options_open = false;
            cx.notify();
            return;
        }
        self.player = None;
        self.action_notice = None;
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
            .bg(theme.to_gpui(cx.global::<SessionSettings>().background_opacity))
            .text_color(ink)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|gallery, event: &gpui::KeyDownEvent, _, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => gallery.back(cx),
                    "space" if !event.is_held && !gallery.options_open => {
                        if let Some(player) = &gallery.player {
                            player.update(cx, |player, cx| player.toggle(cx));
                        }
                    }
                    "left" | "right" => gallery.navigate(event.keystroke.key == "right", cx),
                    _ => return,
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
                            .tab_index(0)
                            .tooltip(|_, cx| cx.new(|_| BackTooltip).into())
                            .on_key_down(cx.listener(
                                |gallery, event: &gpui::KeyDownEvent, _, cx| {
                                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                        gallery.back(cx);
                                        cx.stop_propagation();
                                    }
                                },
                            ))
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
                    .child(div().flex_1().min_w_0().text_sm().truncate().child(title))
                    .child(self.actions(cx)),
            )
            .when_some(self.action_notice.clone(), |view, notice| {
                view.child(
                    div()
                        .px(px(24.))
                        .pb(px(8.))
                        .text_xs()
                        .text_color(ink.opacity(0.7))
                        .child(notice),
                )
            })
            .child(self.content(window, cx))
    }
}

struct BackTooltip;
impl Render for BackTooltip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<SessionSettings>().theme_color;
        div()
            .px(px(8.))
            .py(px(4.))
            .rounded(px(5.))
            .bg(theme.to_gpui(1.))
            .text_color(super::settings::foreground(theme))
            .text_xs()
            .child("Back · Esc")
    }
}
