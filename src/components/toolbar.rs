use crate::{
    icons::{APERTURE, CIRCLE_STOP, MAXIMIZE_2, MINIMIZE_2, VIDEO},
    theme::CAMERA_LETTERBOX_COLOR,
};
use gpui::{
    App, ClickEvent, Context, EventEmitter, IntoElement, Render, Window, div, prelude::*, px, svg,
};

const BAR_WIDTH: f32 = 280.0;
const BAR_HEIGHT: f32 = 48.0;
const TOGGLE_HEIGHT: f32 = 40.0;
const END_SIZE: f32 = 40.0;
const ICON_SIZE: f32 = 20.0;
const WELL_DARKEN_FACTOR: f32 = 0.75;
const WELL_INSET: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    Photo,
    Video,
}

pub struct FitModeChanged {
    pub cover: bool,
}

pub struct Toolbar {
    selected_mode: CameraMode,
    active: bool,
    cover: bool,
}

impl Toolbar {
    pub fn new(_: &mut Context<Self>) -> Self {
        Self {
            selected_mode: CameraMode::Photo,
            active: false,
            cover: false,
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

    fn on_fit_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.cover = !self.cover;
        cx.emit(FitModeChanged { cover: self.cover });
        cx.notify();
    }
}

impl EventEmitter<FitModeChanged> for Toolbar {}

impl Render for Toolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let photo_selected = self.selected_mode == CameraMode::Photo;
        let photo_active = photo_selected && self.active;
        let video_selected = self.selected_mode == CameraMode::Video;
        let video_active = video_selected && self.active;

        let bar_color = CAMERA_LETTERBOX_COLOR.to_gpui(1.0);
        let rail_color = gpui::Hsla::from(bar_color);
        let well_color = darken_color(bar_color);
        let theme_icon = contrasting_icon_color(rail_color);
        let photo_icon = if photo_active {
            theme_icon.opacity(0.65)
        } else if photo_selected {
            theme_icon
        } else {
            theme_icon.opacity(0.55)
        };
        let video_icon = if video_active {
            theme_icon
        } else if video_selected {
            theme_icon.opacity(0.65)
        } else {
            theme_icon.opacity(0.55)
        };

        let mode_buttons = vec![
            mode_button(
                "photo-mode",
                photo_icon,
                APERTURE,
                cx.listener(Self::on_photo_click),
            )
            .into_any_element(),
            mode_button(
                "video-mode",
                video_icon,
                if video_active { CIRCLE_STOP } else { VIDEO },
                cx.listener(Self::on_video_click),
            )
            .into_any_element(),
        ];
        let toggle_width = END_SIZE * mode_buttons.len() as f32;
        let mode_toggle = div()
            .w(px(toggle_width + WELL_INSET * 2.0))
            .h(px(TOGGLE_HEIGHT))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(well_color)
            .child(
                div()
                    .w(px(toggle_width))
                    .h(px(TOGGLE_HEIGHT))
                    .flex()
                    .items_center()
                    .justify_between()
                    .children(mode_buttons),
            );

        let fit_button = div()
            .size(px(TOGGLE_HEIGHT))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(well_color)
            .child(mode_button(
                "fit-window",
                theme_icon,
                if self.cover { MINIMIZE_2 } else { MAXIMIZE_2 },
                cx.listener(Self::on_fit_click),
            ));

        // Add future controls to this list; the outer pill lays them out consistently.
        let bar_elements = vec![
            mode_toggle.into_any_element(),
            fit_button.into_any_element(),
        ];

        div()
            .absolute()
            .bottom(px(24.))
            .left_0()
            .right_0()
            .h(px(BAR_HEIGHT))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(BAR_WIDTH))
                    .h(px(BAR_HEIGHT))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_start()
                    .gap(px(8.))
                    .pl(px(8.))
                    .bg(bar_color)
                    .children(bar_elements),
            )
    }
}

fn darken_color(color: gpui::Rgba) -> gpui::Hsla {
    gpui::Hsla::from(gpui::Rgba {
        r: color.r * WELL_DARKEN_FACTOR,
        g: color.g * WELL_DARKEN_FACTOR,
        b: color.b * WELL_DARKEN_FACTOR,
        a: 1.0,
    })
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
    icon_color: gpui::Hsla,
    icon_path: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(END_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .on_click(on_click)
        .child(
            svg()
                .size(px(ICON_SIZE))
                .path(icon_path)
                .text_color(icon_color),
        )
}
