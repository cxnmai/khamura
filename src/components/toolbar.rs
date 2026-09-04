use crate::{
    icons::{APERTURE, CIRCLE_STOP, SLIDERS_HORIZONTAL, VIDEO},
    session_settings::SessionSettings,
};
use gpui::{
    App, ClickEvent, Context, EventEmitter, IntoElement, Render, Window, div, prelude::*, px, svg,
};

const BAR_WIDTH: f32 = 280.0;
const BAR_HEIGHT: f32 = 48.0;
const TOGGLE_HEIGHT: f32 = 40.0;
const END_SIZE: f32 = 40.0;
const ACTIVE_CIRCLE_SIZE: f32 = 32.0;
const ICON_SIZE: f32 = 24.0;
const SETTINGS_ICON_SIZE: f32 = 18.0;
const WELL_DARKEN_FACTOR: f32 = 0.75;
const WELL_INSET: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    Photo,
    Video,
}

pub struct SettingsToggled;

pub struct Toolbar {
    selected_mode: CameraMode,
    active: bool,
    settings_open: bool,
}

impl Toolbar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<SessionSettings>(|_, cx| cx.notify())
            .detach();
        Self {
            selected_mode: CameraMode::Photo,
            active: false,
            settings_open: false,
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

    pub fn set_settings_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.settings_open = open;
        cx.notify();
    }

    fn on_settings_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(SettingsToggled);
    }
}

impl EventEmitter<SettingsToggled> for Toolbar {}

impl Render for Toolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let photo_selected = self.selected_mode == CameraMode::Photo;
        let photo_active = photo_selected && self.active;
        let video_selected = self.selected_mode == CameraMode::Video;
        let video_active = video_selected && self.active;

        let bar_color = cx.global::<SessionSettings>().theme_color.to_gpui(1.0);
        let well_color = darken_color(bar_color);
        let theme_icon = contrasting_icon_color(well_color);
        let photo_circle = photo_selected.then(|| {
            if photo_active {
                gpui::white().opacity(0.65)
            } else {
                gpui::white()
            }
        });
        let video_circle = video_selected.then(|| {
            if video_active {
                gpui::red()
            } else {
                gpui::red().opacity(0.65)
            }
        });
        let photo_icon = icon_color(photo_circle.unwrap_or(well_color), photo_selected);
        let video_icon = icon_color(video_circle.unwrap_or(well_color), video_selected);

        let mode_buttons = vec![
            mode_button(
                "photo-mode",
                photo_circle,
                photo_icon,
                APERTURE,
                ICON_SIZE,
                cx.listener(Self::on_photo_click),
            )
            .into_any_element(),
            mode_button(
                "video-mode",
                video_circle,
                video_icon,
                if video_active { CIRCLE_STOP } else { VIDEO },
                ICON_SIZE,
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

        let settings_icon = if self.settings_open {
            theme_icon
        } else {
            theme_icon.opacity(0.55)
        };
        let settings_button = div()
            .size(px(TOGGLE_HEIGHT))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(well_color)
            .child(mode_button(
                "settings-toggle",
                None,
                settings_icon,
                SLIDERS_HORIZONTAL,
                SETTINGS_ICON_SIZE,
                cx.listener(Self::on_settings_click),
            ));

        // Add future controls to this list; the outer pill lays them out consistently.
        let bar_elements = vec![
            mode_toggle.into_any_element(),
            settings_button.into_any_element(),
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
                    .justify_between()
                    .px(px(8.))
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

fn icon_color(background: gpui::Hsla, selected: bool) -> gpui::Hsla {
    let color = contrasting_icon_color(background);
    if selected { color } else { color.opacity(0.55) }
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
    circle_color: Option<gpui::Hsla>,
    icon_color: gpui::Hsla,
    icon_path: &'static str,
    icon_size: f32,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let mut button = div()
        .id(id)
        .size(px(END_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .on_click(on_click);

    if let Some(circle_color) = circle_color {
        button = button.child(
            div()
                .size(px(ACTIVE_CIRCLE_SIZE))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(circle_color)
                .child(icon(icon_path, icon_color, icon_size)),
        );
    } else {
        button = button.child(icon(icon_path, icon_color, icon_size));
    }

    button
}

fn icon(path: &'static str, color: gpui::Hsla, icon_size: f32) -> impl IntoElement {
    svg().size(px(icon_size)).path(path).text_color(color)
}
