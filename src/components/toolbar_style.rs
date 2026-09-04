use gpui::{prelude::*, App, ClickEvent, IntoElement, Window, div, px, svg};
const WELL_DARKEN_FACTOR: f32 = 0.75;
const END_SIZE: f32 = 40.;
const ACTIVE_CIRCLE_SIZE: f32 = 32.;
pub(super) fn darken_color(color: gpui::Rgba) -> gpui::Hsla {
    gpui::Hsla::from(gpui::Rgba {
        r: color.r * WELL_DARKEN_FACTOR,
        g: color.g * WELL_DARKEN_FACTOR,
        b: color.b * WELL_DARKEN_FACTOR,
        a: 1.0,
    })
}

pub(super) fn icon_color(background: gpui::Hsla, selected: bool) -> gpui::Hsla {
    let color = contrasting_icon_color(background);
    if selected { color } else { color.opacity(0.55) }
}

pub(super) fn contrasting_icon_color(background: gpui::Hsla) -> gpui::Hsla {
    let color = gpui::Rgba::from(background);
    let brightness = 0.299 * color.r + 0.587 * color.g + 0.114 * color.b;

    if brightness >= 0.5 {
        gpui::black()
    } else {
        gpui::white()
    }
}

pub(super) fn mode_button(
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
        .on_click(on_click)
        .tooltip(move |_, cx| {
            let label = match id {
                "photo-mode" => "Photo · Space to capture",
                "video-mode" => "Video · Space to start / stop recording",
                _ => "Settings",
            };
            cx.new(|_| super::controls::Tooltip(label.into())).into()
        });

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
