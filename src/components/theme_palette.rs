use crate::{session_settings::SessionSettings, theme::Rgb};
use gpui::{Context, Hsla, IntoElement, Render, Window, div, prelude::*, px};

// Backgrounds from https://github.com/helix-editor/helix/tree/master/runtime/themes
// Names identify the corresponding TOML files; Nord Snow uses nord.toml's nord6.
const DARK: [(&str, u32); 10] = [
    ("Gruvbox", 0x282828),
    ("Nord", 0x2e3440),
    ("Catppuccin Mocha", 0x1e1e2e),
    ("Dracula", 0x282a36),
    ("One Dark", 0x282c34),
    ("Tokyo Night", 0x1a1b26),
    ("Rosé Pine", 0x191724),
    ("Everforest Dark", 0x2d353b),
    ("Solarized Dark", 0x002b36),
    ("Ayu Dark", 0x0f1419),
];
const LIGHT: [(&str, u32); 10] = [
    ("Gruvbox Light", 0xfbf1c7),
    ("Nord Snow", 0xeceff4),
    ("Catppuccin Latte", 0xeff1f5),
    ("Solarized Light", 0xfdf6e3),
    ("Rosé Pine Dawn", 0xfaf4ed),
    ("GitHub Light", 0xffffff),
    ("Ayu Light", 0xfafafa),
    ("Tokyo Night Day", 0xe1e2e7),
    ("Base16 Default Light", 0xf8f8f8),
    ("Flexoki Light", 0xfffcf0),
];

fn rgb(hex: u32) -> Rgb {
    Rgb::new((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

pub(super) fn theme_palette(selected: Rgb, foreground: Hsla) -> impl IntoElement {
    let name = DARK
        .iter()
        .chain(LIGHT.iter())
        .find(|(_, hex)| rgb(*hex) == selected)
        .map(|(name, _)| *name)
        .unwrap_or("Custom");
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child("Appearance")
                .child(div().text_color(foreground.opacity(0.5)).child(name)),
        )
        .children([DARK, LIGHT].into_iter().map(move |row| {
            div()
                .flex()
                .justify_between()
                .children(row.into_iter().map(move |(name, hex)| {
                    let color = rgb(hex);
                    div()
                        .id(name)
                        .size(px(22.))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .border_1()
                        .border_color(if color == selected {
                            foreground
                        } else {
                            foreground.opacity(0.)
                        })
                        .cursor_pointer()
                        .hover(|style| style.border_color(foreground.opacity(0.6)))
                        .tooltip(move |_, cx| cx.new(|_| SwatchTooltip(name)).into())
                        .on_click(move |_, _, cx| {
                            SessionSettings::change(cx, |settings| settings.theme_color = color);
                        })
                        .child(
                            div()
                                .size(px(16.))
                                .rounded_full()
                                .border_1()
                                .border_color(foreground.opacity(0.25))
                                .bg(color.to_gpui(1.)),
                        )
                }))
        }))
}

struct SwatchTooltip(&'static str);

impl Render for SwatchTooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(8.))
            .py(px(4.))
            .rounded(px(6.))
            .bg(gpui::black())
            .text_color(gpui::white())
            .text_xs()
            .child(self.0)
    }
}
