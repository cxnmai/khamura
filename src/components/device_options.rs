use super::DeviceSettings;
use crate::{device_catalog::Device, session_settings::SessionSettings};
use gpui::{Context, IntoElement, Window, div, prelude::*, px};

pub(super) fn label(device: &Device, camera: bool) -> String {
    if camera {
        format!("{} ({})", device.label, device.id)
    } else {
        device.label.clone()
    }
}

pub(super) fn popup(
    owner: &DeviceSettings,
    camera: bool,
    options: Vec<(Option<String>, String)>,
    selected: Option<String>,
    window: &Window,
    cx: &mut Context<DeviceSettings>,
) -> impl IntoElement {
    let bounds = owner.bounds[usize::from(camera)].get();
    let theme = cx.global::<SessionSettings>().theme_color;
    let ink = super::super::settings::foreground(theme);
    let below = window.viewport_size().height - bounds.bottom() - px(8.);
    let above = bounds.top() - px(8.);
    let flip = below < px(200.) && above > below;
    let height = if flip { above } else { below }.min(px(200.)).max(px(32.));
    let anchor = if flip {
        gpui::Corner::BottomLeft
    } else {
        gpui::Corner::TopLeft
    };
    let position = gpui::point(
        bounds.left(),
        if flip {
            bounds.top() - px(4.)
        } else {
            bounds.bottom() + px(4.)
        },
    );
    gpui::deferred(
        gpui::anchored()
            .anchor(anchor)
            .position(position)
            .snap_to_window()
            .child(
                div()
                    .id(if camera {
                        "camera-options"
                    } else {
                        "microphone-options"
                    })
                    .w(bounds.size.width)
                    .max_h(height)
                    .overflow_y_scroll()
                    .bg(theme)
                    .text_color(ink)
                    .rounded(px(6.))
                    .border_1()
                    .border_color(ink.opacity(0.2))
                    .occlude()
                    .on_mouse_down_out(cx.listener(
                        move |this, event: &gpui::MouseDownEvent, _, cx| {
                            // Let the trigger's click handler toggle without reopening it.
                            if !this.bounds[usize::from(camera)]
                                .get()
                                .contains(&event.position)
                            {
                                this.open = None;
                                cx.notify();
                            }
                        },
                    ))
                    .children(options.into_iter().enumerate().map(|(index, (id, label))| {
                        let chosen = id == selected;
                        let keyboard_id = id.clone();
                        div()
                            .id((
                                if camera {
                                    "camera-option"
                                } else {
                                    "microphone-option"
                                },
                                index,
                            ))
                            .tab_index(0)
                            .px(px(8.))
                            .py(px(7.))
                            .text_xs()
                            .whitespace_normal()
                            .cursor_pointer()
                            .when(chosen, |item| item.bg(ink.opacity(0.12)))
                            .hover(|style| style.bg(ink.opacity(0.08)))
                            .child(label)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.select(camera, id.clone(), cx)
                            }))
                            .on_key_down(cx.listener(
                                move |this, event: &gpui::KeyDownEvent, _, cx| {
                                    if event.keystroke.key == "enter"
                                        || event.keystroke.key == "space"
                                    {
                                        this.select(camera, keyboard_id.clone(), cx);
                                        cx.stop_propagation();
                                    }
                                },
                            ))
                    })),
            ),
    )
    .with_priority(2)
}
