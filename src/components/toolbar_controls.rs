use std::{cell::Cell, rc::Rc};
#[path = "toolbar_control_button.rs"]
mod button;
#[path = "toolbar_choices.rs"]
mod choices;
use crate::{
    capture_settings::{CameraMode, CaptureSettings, PhotoAspect},
    icons::{GRID, MIC, MIC_OFF, RATIO, TIMER, VIDEO_QUALITY},
    session_settings::SessionSettings,
};
pub(super) use button::Tooltip;
use button::button;
use choices::{Menu, aspect_label, choices, quality_label};
use gpui::{Context, FocusHandle, IntoElement, Render, Window, div, prelude::*, px};
pub(super) struct ToolbarControls {
    menu: Option<Menu>,
    focus: FocusHandle,
    trigger_bounds: Rc<Cell<gpui::Bounds<gpui::Pixels>>>,
}
impl ToolbarControls {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<CaptureSettings>(|this, cx| {
            this.dismiss(cx);
        })
        .detach();
        Self {
            menu: None,
            focus: cx.focus_handle(),
            trigger_bounds: Rc::new(Cell::new(gpui::Bounds::default())),
        }
    }
    pub(super) fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.menu = None;
        cx.notify();
    }
    fn toggle(&mut self, menu: Menu, window: &mut Window, cx: &mut Context<Self>) {
        if cx.global::<CaptureSettings>().busy {
            return;
        }
        self.menu = if self.menu == Some(menu) {
            None
        } else {
            Some(menu)
        };
        window.focus(&self.focus);
        cx.notify();
    }
}
impl Render for ToolbarControls {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = cx.global::<CaptureSettings>();
        let busy = settings.busy;
        let photo = settings.mode == CameraMode::Photo;
        let theme = cx.global::<SessionSettings>().theme_color;
        let color = super::super::settings::foreground(theme);
        let first = if photo {
            button(
                "timer",
                TIMER,
                (settings.timer_seconds != 0).then(|| settings.timer_seconds.to_string()),
                format!(
                    "Photo timer: {} · Escape cancels countdown",
                    if settings.timer_seconds == 0 {
                        "Off".into()
                    } else {
                        format!("{} seconds", settings.timer_seconds)
                    }
                ),
                busy,
                settings.timer_seconds != 0,
                color,
                cx.listener(|this, _, window, cx| this.toggle(Menu::Timer, window, cx)),
            )
            .into_any_element()
        } else {
            button(
                "microphone",
                if settings.microphone_on { MIC } else { MIC_OFF },
                None,
                format!(
                    "Microphone: {}",
                    if settings.microphone_on { "On" } else { "Off" }
                ),
                busy,
                settings.microphone_on,
                color,
                |_, _, cx| {
                    if !cx.global::<CaptureSettings>().busy {
                        CaptureSettings::change(cx, |s| s.microphone_on = !s.microphone_on);
                    }
                },
            )
            .into_any_element()
        };
        let (menu, icon, active, tooltip) = if photo {
            (
                Menu::Aspect,
                RATIO,
                settings.aspect != PhotoAspect::Native,
                format!(
                    "Photo aspect ratio: {} (capture crop)",
                    aspect_label(settings.aspect)
                ),
            )
        } else {
            (
                Menu::Quality,
                VIDEO_QUALITY,
                settings.quality.is_some(),
                format!("Video quality: {}", quality_label(settings.quality)),
            )
        };
        let second = button(
            "capture-format",
            icon,
            None,
            tooltip,
            busy,
            active,
            color,
            cx.listener(move |this, _, window, cx| this.toggle(menu, window, cx)),
        );
        let grid = button(
            "grid",
            GRID,
            None,
            format!(
                "Rule-of-thirds grid: {} (not saved in captures)",
                if settings.grid { "On" } else { "Off" }
            ),
            busy,
            settings.grid,
            color,
            |_, _, cx| {
                if !cx.global::<CaptureSettings>().busy {
                    CaptureSettings::change(cx, |s| s.grid = !s.grid);
                }
            },
        );
        let items = self.menu.map(|menu| choices(menu, settings));
        let (menu_width, menu_center) = match self.menu {
            Some(Menu::Timer) => (144., 18.),
            Some(Menu::Aspect) => (128., 56.),
            _ => (224., 56.),
        };
        let trigger_bounds = self.trigger_bounds.clone();
        div()
            .relative()
            .track_focus(&self.focus)
            .flex()
            .items_center()
            .gap(px(2.))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" && this.menu.take().is_some() {
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .child(
                gpui::canvas(
                    move |bounds, _, _| trigger_bounds.set(bounds),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .child(first)
            .child(second)
            .child(grid)
            .when_some(items, |view, items| {
                view.child(
                    div()
                        .id("capture-options")
                        .absolute()
                        .bottom(px(48.))
                        .left(px(menu_center - menu_width / 2.))
                        .w(px(menu_width))
                        .max_h(px(240.))
                        .overflow_y_scroll()
                        .occlude()
                        .rounded(px(12.))
                        .p(px(4.))
                        .bg(theme.to_gpui(1.))
                        .text_color(color)
                        .border_1()
                        .border_color(color.opacity(0.08))
                        .on_mouse_down_out(cx.listener(
                            |this, event: &gpui::MouseDownEvent, _, cx| {
                                // Let trigger clicks toggle the existing menu rather than
                                // dismissing on mouse-down and reopening on mouse-up.
                                if !this.trigger_bounds.get().contains(&event.position) {
                                    this.dismiss(cx);
                                }
                            },
                        ))
                        .children(items.into_iter().enumerate().map(
                            |(index, (label, choice, selected))| {
                                div()
                                    .id(("capture-choice", index))
                                    .px(px(10.))
                                    .h(px(30.))
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap(px(12.))
                                    .rounded(px(8.))
                                    .text_xs()
                                    .text_color(color.opacity(if selected { 1. } else { 0.7 }))
                                    .cursor_pointer()
                                    .hover(|row| row.bg(color.opacity(0.06)).text_color(color))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if !cx.global::<CaptureSettings>().busy {
                                            CaptureSettings::change(cx, |s| choice.apply(s));
                                        }
                                        this.menu = None;
                                        cx.notify();
                                    }))
                                    .child(label)
                                    .child(
                                        div()
                                            .size(px(4.))
                                            .flex_shrink_0()
                                            .rounded_full()
                                            .when(selected, |dot| dot.bg(color.opacity(0.8))),
                                    )
                            },
                        )),
                )
            })
    }
}
