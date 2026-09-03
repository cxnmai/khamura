use crate::{
    icons::{APERTURE, CIRCLE_STOP, VIDEO},
    theme::CAMERA_LETTERBOX_COLOR,
};
use gpui::{App, ClickEvent, Context, IntoElement, Render, Window, div, prelude::*, px, svg};

const BAR_WIDTH: f32 = 280.0;
const BAR_HEIGHT: f32 = 48.0;
const TOGGLE_WIDTH: f32 = END_SIZE * 2.0;
const TOGGLE_HEIGHT: f32 = 40.0;
const RAIL_HEIGHT: f32 = 20.0;
const END_SIZE: f32 = 40.0;
const ICON_SIZE: f32 = 20.0;

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

        let bar_color = CAMERA_LETTERBOX_COLOR.to_gpui(1.0);
        let rail_color = gpui::Hsla::from(bar_color);
        let photo_color = if photo_active {
            gpui::white().opacity(0.65)
        } else if photo_selected {
            gpui::white()
        } else {
            rail_color
        };
        let video_color = if video_active {
            gpui::red()
        } else if video_selected {
            gpui::red().opacity(0.65)
        } else {
            rail_color
        };
        let photo_icon = contrasting_icon_color(photo_color);
        let video_icon = contrasting_icon_color(video_color);

        div()
            .absolute()
            .bottom(px(24.))
            .left_0()
            .right_0()
            .h(px(BAR_HEIGHT))
            .flex()
            .items_center()
            .justify_center()
            // The toolbar remains the outer pill; the photo/video toggle is its first element.
            .child(
                div()
                    .w(px(BAR_WIDTH))
                    .h(px(BAR_HEIGHT))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_start()
                    .pl(px(8.))
                    .bg(bar_color)
                    .child(
                        div()
                            .relative()
                            .w(px(TOGGLE_WIDTH))
                            .h(px(TOGGLE_HEIGHT))
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .absolute()
                                    .left(px(END_SIZE / 2.0))
                                    .right(px(END_SIZE / 2.0))
                                    .top(px((TOGGLE_HEIGHT - RAIL_HEIGHT) / 2.0))
                                    .h(px(RAIL_HEIGHT))
                                    .rounded_full()
                                    .bg(rail_color),
                            )
                            .child(mode_button(
                                "photo-mode",
                                photo_color,
                                photo_icon,
                                APERTURE,
                                cx.listener(Self::on_photo_click),
                            ))
                            .child(mode_button(
                                "video-mode",
                                video_color,
                                video_icon,
                                if video_active { CIRCLE_STOP } else { VIDEO },
                                cx.listener(Self::on_video_click),
                            )),
                    ),
            )
    }
}

fn contrasting_icon_color(background: gpui::Hsla) -> gpui::Hsla {
    let color = gpui::Rgba::from(background);
    let brightness = 0.299 * color.r + 0.587 * color.g + 0.114 * color.b;

    if brightness >= 0.5 {
        gpui::black()
    } else {
        gpui::white()
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
        .child(
            svg()
                .size(px(ICON_SIZE))
                .path(icon_path)
                .text_color(icon_color),
        )
}
