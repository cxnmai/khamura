use std::{cell::Cell, rc::Rc};
#[path = "toolbar_controls.rs"]
mod controls;
#[path = "toolbar_gallery.rs"]
mod gallery;
#[path = "toolbar_style.rs"]
mod style;
use crate::{
    capture_settings::{CameraMode, CaptureSettings},
    icons::{APERTURE, CIRCLE_STOP, SLIDERS_HORIZONTAL, VIDEO},
    session_settings::SessionSettings,
};
use gpui::{ClickEvent, Context, EventEmitter, IntoElement, Render, Window, div, prelude::*, px};
use style::*;

pub const BAR_WIDTH: f32 = 344.0;
const BAR_HEIGHT: f32 = 48.0;
const TOGGLE_HEIGHT: f32 = 40.0;
const END_SIZE: f32 = 40.0;
const ICON_SIZE: f32 = 24.0;
const SETTINGS_ICON_SIZE: f32 = 18.0;
const WELL_INSET: f32 = 4.0;

pub struct CaptureRequested;

pub struct SettingsToggled;
pub struct GalleryRequested;

pub struct Toolbar {
    controls: gpui::Entity<controls::ToolbarControls>,
    settings_open: bool,
    settings_bounds: Rc<Cell<gpui::Bounds<gpui::Pixels>>>,
}

impl Toolbar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<SessionSettings>(|_, cx| cx.notify())
            .detach();
        cx.observe_global::<CaptureSettings>(|_, cx| cx.notify())
            .detach();
        cx.observe_global::<crate::gallery::GalleryStore>(|_, cx| cx.notify())
            .detach();
        Self {
            controls: cx.new(controls::ToolbarControls::new),
            settings_open: false,
            settings_bounds: Rc::new(Cell::new(gpui::Bounds::default())),
        }
    }

    fn select_or_toggle(&mut self, mode: CameraMode, cx: &mut Context<Self>) {
        let state = cx.global::<CaptureSettings>();
        if state.mode == mode {
            if !state.busy || state.recording {
                cx.emit(CaptureRequested);
            }
        } else if !state.busy {
            CaptureSettings::change(cx, |settings| settings.mode = mode);
        }
    }

    fn on_photo_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.select_or_toggle(CameraMode::Photo, cx);
    }

    fn on_video_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.select_or_toggle(CameraMode::Video, cx);
    }

    pub fn settings_bounds(&self) -> gpui::Bounds<gpui::Pixels> {
        self.settings_bounds.get()
    }

    pub fn set_settings_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.settings_open = open;
        if open {
            self.controls
                .update(cx, |controls, cx| controls.dismiss(cx));
        }
        cx.notify();
    }

    fn on_settings_click(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(SettingsToggled);
    }
}

impl EventEmitter<SettingsToggled> for Toolbar {}
impl EventEmitter<CaptureRequested> for Toolbar {}
impl EventEmitter<GalleryRequested> for Toolbar {}

impl Render for Toolbar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let capture = cx.global::<CaptureSettings>();
        let photo_selected = capture.mode == CameraMode::Photo;
        let video_selected = capture.mode == CameraMode::Video;
        let video_active = capture.recording;

        let bar_color = cx.global::<SessionSettings>().theme_color.to_gpui(1.0);
        let well_color = darken_color(bar_color);
        let theme_icon = contrasting_icon_color(well_color);
        let photo_circle = photo_selected.then(gpui::white);
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
        let settings_bounds = self.settings_bounds.clone();
        let settings_button = div()
            .relative()
            .size(px(TOGGLE_HEIGHT))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(well_color)
            .child(
                gpui::canvas(
                    move |bounds, _, _| settings_bounds.set(bounds),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
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
            self.controls.clone().into_any_element(),
            gallery::preview(cx).into_any_element(),
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
                    .occlude()
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
