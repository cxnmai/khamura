#[path = "toolbar_control_button.rs"]
mod button;
#[path = "toolbar_choices.rs"]
mod choices;
use crate::{
    capture_settings::{CameraMode, CaptureSettings},
    session_settings::SessionSettings,
};
pub(super) use button::Tooltip;
use button::button;
use choices::{Menu, aspect_label, choices, quality_label};
use gpui::{Context, FocusHandle, IntoElement, Render, Window, div, prelude::*, px};
pub(super) struct ToolbarControls {
    menu: Option<Menu>,
    focus: FocusHandle,
}
impl ToolbarControls {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<CaptureSettings>(|this, cx| {
            if cx.global::<CaptureSettings>().busy {
                this.menu = None;
            }
            cx.notify();
        })
        .detach();
        Self {
            menu: None,
            focus: cx.focus_handle(),
        }
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
                format!(
                    "{}",
                    if settings.timer_seconds == 0 {
                        "Off".into()
                    } else {
                        format!("{}s", settings.timer_seconds)
                    }
                ),
                "Photo timer · Escape cancels countdown".into(),
                busy,
                settings.timer_seconds != 0,
                color,
                cx.listener(|this, _, window, cx| this.toggle(Menu::Timer, window, cx)),
            )
            .into_any_element()
        } else {
            button(
                "microphone",
                format!("{}", if settings.microphone_on { "On" } else { "Off" }),
                "Record microphone audio".into(),
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
        let (menu, label, tooltip) = if photo {
            (
                Menu::Aspect,
                aspect_label(settings.aspect).to_string(),
                "Photo aspect ratio (capture crop)".into(),
            )
        } else {
            (
                Menu::Quality,
                settings
                    .quality
                    .map(|q| format!("{}p", q.height))
                    .unwrap_or("Native".into()),
                format!("Video quality: {}", quality_label(settings.quality)),
            )
        };
        let second = button(
            "capture-format",
            label,
            tooltip,
            busy,
            false,
            color,
            cx.listener(move |this, _, window, cx| this.toggle(menu, window, cx)),
        );
        let grid = button(
            "grid",
            "".into(),
            "Rule-of-thirds grid (not saved in captures)".into(),
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
            .child(first)
            .child(second)
            .child(grid)
            .when_some(items, |view, items| {
                view.child(
                    div()
                        .id("capture-options")
                        .absolute()
                        .bottom(px(52.))
                        .left_0()
                        .w(px(220.))
                        .max_h(px(280.))
                        .overflow_y_scroll()
                        .occlude()
                        .rounded(px(12.))
                        .p(px(6.))
                        .bg(theme.to_gpui(1.))
                        .text_color(color)
                        .border_1()
                        .border_color(color.opacity(0.15))
                        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                            this.menu = None;
                            cx.notify();
                        }))
                        .children(items.into_iter().enumerate().map(
                            |(index, (label, choice, selected))| {
                                div()
                                    .id(("capture-choice", index))
                                    .px(px(10.))
                                    .py(px(7.))
                                    .rounded(px(6.))
                                    .text_xs()
                                    .cursor_pointer()
                                    .when(selected, |row| row.bg(color.opacity(0.14)))
                                    .hover(|row| row.bg(color.opacity(0.1)))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if !cx.global::<CaptureSettings>().busy {
                                            CaptureSettings::change(cx, |s| choice.apply(s));
                                        }
                                        this.menu = None;
                                        cx.notify();
                                    }))
                                    .child(label)
                            },
                        )),
                )
            })
    }
}
