use crate::{
    icons::{APERTURE, CIRCLE_STOP, VIDEO},
    theme::CAMERA_LETTERBOX_COLOR,
};
use gpui::{App, ClickEvent, Context, IntoElement, Render, Window, div, prelude::*, px, svg};

const TOOLBAR_WIDTH: f32 = 224.0;
const TOOLBAR_HEIGHT: f32 = 56.0;
const RAIL_HEIGHT: f32 = 28.0;
const END_SIZE: f32 = 56.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    Photo,
    Video,
}

pub struct Toolbar {
    selected_mode: CameraMode,
    active: bool,
}

impl Toolbar {
    pub fn new(_: &mut Context<Self>) -> Self {
        Self {
            selected_mode: CameraMode::Photo,
            active: false,
        }
    }

    fn select_or_toggle(&mut self, mode: CameraMode, cx: &mut Context<Self>) {
        if self.selected_mode == mode {
            self.active = !self.active;
        } else {
            self.selected_mode = mode;
            self.active = false;
        }
        cx.notify();
    }

    fn on_photo_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.select_or_toggle(CameraMode::Photo, cx);
    }

    fn on_video_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.select_or_toggle(CameraMode::Video, cx);
    }
}

impl Render for Toolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let photo_selected = self.selected_mode == CameraMode::Photo;
        let photo_active = photo_selected && self.active;
        let video_selected = self.selected_mode == CameraMode::Video;
        let video_active = video_selected && self.active;

        let rail_color = CAMERA_LETTERBOX_COLOR.to_gpui(1.0);
        let rail_hsla = gpui::Hsla::from(rail_color);
        let photo_color = if photo_active {
            gpui::white().opacity(0.65)
        } else if photo_selected {
            gpui::white()
        } else {
            rail_hsla
        };
        let video_color = if video_active {
            gpui::red()
        } else if video_selected {
            gpui::red().opacity(0.65)
        } else {
            rail_hsla
        };
        let muted_icon = gpui::white().opacity(0.55);

        div()
            .absolute()
            .bottom(px(24.))
            .left_0()
            .right_0()
            .h(px(TOOLBAR_HEIGHT))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .relative()
                    .w(px(TOOLBAR_WIDTH))
                    .h(px(TOOLBAR_HEIGHT))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .absolute()
                            .left(px(END_SIZE / 2.0))
                            .right(px(END_SIZE / 2.0))
                            .top(px((TOOLBAR_HEIGHT - RAIL_HEIGHT) / 2.0))
                            .h(px(RAIL_HEIGHT))
                            .rounded_full()
                            .bg(rail_color),
                    )
                    .child(mode_button(
                        "photo-mode",
                        photo_color,
                        if photo_selected {
                            gpui::black()
                        } else {
                            muted_icon
                        },
                        APERTURE,
                        cx.listener(Self::on_photo_click),
                    ))
                    .child(mode_button(
                        "video-mode",
                        video_color,
                        if video_selected {
                            gpui::white()
                        } else {
                            muted_icon
                        },
                        if video_active { CIRCLE_STOP } else { VIDEO },
                        cx.listener(Self::on_video_click),
                    )),
            )
    }
}

fn mode_button(
    id: &'static str,
    background: gpui::Hsla,
    icon_color: gpui::Hsla,
    icon_path: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(END_SIZE))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(background)
        .on_click(on_click)
        .child(svg().size(px(24.)).path(icon_path).text_color(icon_color))
}
