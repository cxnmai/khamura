use crate::{icons, session_settings::SessionSettings};
use gpui::{Context, RenderImage, Task, Window, div, img, prelude::*, px, svg};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
};
#[path = "video_decode.rs"]
mod decode;
#[path = "video_process.rs"]
mod process;

pub struct VideoPlayer {
    path: PathBuf,
    playback: Arc<decode::Playback>,
    task: Task<()>,
    frame: Option<Arc<RenderImage>>,
    ended: bool,
    error: Option<String>,
}
impl VideoPlayer {
    pub fn new(path: PathBuf, cx: &mut Context<Self>) -> Self {
        let (playback, task) = Self::start(path.clone(), cx);
        Self {
            path,
            playback,
            task,
            frame: None,
            ended: false,
            error: None,
        }
    }
    fn start(path: PathBuf, cx: &mut Context<Self>) -> (Arc<decode::Playback>, Task<()>) {
        let (tx, rx) = async_channel::bounded(2);
        let playback = decode::Playback::start(path, tx);
        let task = cx.spawn(async move |this, cx| {
            while let Ok(event) = rx.recv().await {
                if this
                    .update(cx, |player, cx| {
                        match event {
                            decode::Event::Frame(pixels) => {
                                let pixels = image::RgbaImage::from_raw(
                                    decode::WIDTH,
                                    decode::HEIGHT,
                                    pixels,
                                )
                                .unwrap();
                                let frame =
                                    Arc::new(RenderImage::new(vec![image::Frame::new(pixels)]));
                                if let Some(old) = player.frame.replace(frame) {
                                    cx.drop_image(old, None);
                                }
                            }
                            decode::Event::End => player.ended = true,
                            decode::Event::Error(error) => {
                                player.error = Some(error);
                                player.ended = true;
                            }
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        (playback, task)
    }
    pub(super) fn toggle(&mut self, cx: &mut Context<Self>) {
        if self.ended {
            self.restart(cx);
        } else {
            self.playback.paused.fetch_xor(true, Ordering::Relaxed);
            cx.notify();
        }
    }
    fn restart(&mut self, cx: &mut Context<Self>) {
        self.playback.stop();
        let (playback, task) = Self::start(self.path.clone(), cx);
        self.playback = playback;
        self.task = task;
        self.ended = false;
        self.error = None;
        cx.notify();
    }
}
impl Drop for VideoPlayer {
    fn drop(&mut self) {
        self.playback.stop();
    }
}
impl Render for VideoPlayer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<SessionSettings>().theme_color;
        let ink = super::settings::foreground(theme);
        let paused = self.playback.paused.load(Ordering::Relaxed);
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.))
            .child(div().flex_1().min_h(px(0.)).w_full().when_some(
                self.frame.clone(),
                |view, frame| {
                    view.child(img(frame).size_full().object_fit(gpui::ObjectFit::Contain))
                },
            ))
            .when_some(self.error.clone(), |view, error| {
                view.child(div().text_xs().child(error))
            })
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .items_center()
                    .pb(px(10.))
                    .child(
                        div()
                            .id("video-play-pause")
                            .cursor_pointer()
                            .p(px(8.))
                            .rounded_full()
                            .hover(move |s| s.bg(ink.opacity(0.08)))
                            .tab_index(0)
                            .tooltip(|_, cx| {
                                cx.new(|_| PlayerTooltip("Play / pause · Space")).into()
                            })
                            .on_key_down(cx.listener(
                                |player, event: &gpui::KeyDownEvent, _, cx| {
                                    if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                                        player.toggle(cx);
                                        cx.stop_propagation();
                                    }
                                },
                            ))
                            .on_click(cx.listener(|player, _, _, cx| player.toggle(cx)))
                            .child(
                                svg()
                                    .path(if self.ended || paused {
                                        icons::PLAY
                                    } else {
                                        icons::PAUSE
                                    })
                                    .size(px(18.))
                                    .text_color(ink),
                            ),
                    )
                    .child(
                        div()
                            .id("video-restart")
                            .tab_index(0)
                            .tooltip(|_, cx| cx.new(|_| PlayerTooltip("Restart")).into())
                            .on_key_down(cx.listener(
                                |player, event: &gpui::KeyDownEvent, _, cx| {
                                    if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                                        player.restart(cx);
                                        cx.stop_propagation();
                                    }
                                },
                            ))
                            .cursor_pointer()
                            .p(px(8.))
                            .rounded_full()
                            .hover(move |s| s.bg(ink.opacity(0.08)))
                            .on_click(cx.listener(|player, _, _, cx| player.restart(cx)))
                            .child(svg().path(icons::ROTATE_CCW).size(px(18.)).text_color(ink)),
                    ),
            )
    }
}

struct PlayerTooltip(&'static str);
impl Render for PlayerTooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(8.))
            .py(px(5.))
            .rounded(px(6.))
            .bg(gpui::rgb(0x202126))
            .text_color(gpui::white())
            .text_xs()
            .child(self.0)
    }
}
