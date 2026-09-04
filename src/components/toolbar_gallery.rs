use super::{GalleryRequested, Toolbar, controls::Tooltip};
use crate::{
    capture_settings::CaptureSettings, gallery::GalleryStore, session_settings::SessionSettings,
};
use gpui::{Context, IntoElement, ObjectFit, div, img, prelude::*, px, svg};

pub(super) fn preview(cx: &mut Context<Toolbar>) -> impl IntoElement {
    let thumbnail = cx
        .global::<GalleryStore>()
        .items
        .first()
        .and_then(|item| item.thumbnail.clone());
    let video = cx
        .global::<GalleryStore>()
        .items
        .first()
        .is_some_and(|item| item.is_video);
    let busy = cx.global::<CaptureSettings>().busy;
    let ink = super::super::settings::foreground(cx.global::<SessionSettings>().theme_color);
    div()
        .id("gallery-preview")
        .tab_index(0)
        .relative()
        .size(px(34.))
        .flex_shrink_0()
        .rounded(px(7.))
        .overflow_hidden()
        .border_1()
        .border_color(ink.opacity(0.12))
        .bg(ink.opacity(0.04))
        .flex()
        .items_center()
        .justify_center()
        .when(busy, |view| view.opacity(0.4))
        .when(!busy, |view| {
            view.cursor_pointer()
                .hover(|s| s.border_color(ink.opacity(0.4)))
        })
        .tooltip(|_, cx| cx.new(|_| Tooltip("Open gallery".into())).into())
        .on_click(cx.listener(|_, _, _, cx| {
            if !cx.global::<CaptureSettings>().busy {
                cx.emit(GalleryRequested);
            }
        }))
        .on_key_down(cx.listener(|_, event: &gpui::KeyDownEvent, _, cx| {
            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                if !cx.global::<CaptureSettings>().busy {
                    cx.emit(GalleryRequested);
                }
                cx.stop_propagation();
            }
        }))
        .when(thumbnail.is_none(), |view| {
            view.child(
                svg()
                    .path(crate::icons::IMAGE)
                    .size(px(18.))
                    .text_color(ink.opacity(0.5)),
            )
        })
        .when_some(thumbnail, |view, image| {
            view.child(img(image).size_full().object_fit(ObjectFit::Cover))
        })
        .when(video, |view| {
            view.child(
                div()
                    .absolute()
                    .bottom(px(2.))
                    .right(px(2.))
                    .size(px(13.))
                    .rounded_full()
                    .bg(gpui::black().opacity(0.55))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path(crate::icons::PLAY)
                            .size(px(8.))
                            .text_color(gpui::white()),
                    ),
            )
        })
}
