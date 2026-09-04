use crate::{config::Config, session_settings::SessionSettings};
use gpui::{Context, IntoElement, PathPromptOptions, Render, Window, div, prelude::*, px};
use std::path::{Path, PathBuf};

pub struct PathSettings {
    pending: bool,
    error: Option<String>,
}

impl PathSettings {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Config>(|_, cx| cx.notify()).detach();
        cx.observe_global::<SessionSettings>(|_, cx| cx.notify())
            .detach();
        Self {
            pending: false,
            error: None,
        }
    }

    fn choose(&mut self, config: bool, cx: &mut Context<Self>) {
        if self.pending {
            return;
        }
        self.pending = true;
        self.error = None;
        cx.notify();
        let file = config.then(|| {
            let path = cx.global::<Config>().path();
            cx.prompt_for_new_path(
                path.parent().unwrap_or(Path::new("/")),
                path.file_name().and_then(|name| name.to_str()),
            )
        });
        let directory = (!config).then(|| {
            cx.prompt_for_paths(PathPromptOptions {
                files: false,
                directories: true,
                multiple: false,
                prompt: Some("Select output directory".into()),
            })
        });
        cx.spawn(async move |this, cx| {
            let selected: Result<Option<PathBuf>, String> = if let Some(file) = file {
                file.await
                    .map_err(|error| error.to_string())
                    .and_then(|result| result.map_err(|error| error.to_string()))
            } else if let Some(directory) = directory {
                directory
                    .await
                    .map_err(|error| error.to_string())
                    .and_then(|result| result.map_err(|error| error.to_string()))
                    .map(|paths| paths.and_then(|paths| paths.into_iter().next()))
            } else {
                Ok(None)
            };
            let _ = this.update(cx, |this, cx| {
                this.pending = false;
                this.error = selected
                    .and_then(|path| match path {
                        Some(path) if config => SessionSettings::relocate_config(cx, &path),
                        Some(path) => SessionSettings::set_output_path(cx, &path),
                        None => Ok(()),
                    })
                    .err();
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for PathSettings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let config = cx.global::<Config>();
        let paths = [
            ("Config path", config.path().display().to_string(), true),
            (
                "Output path",
                config.photo_directory.display().to_string(),
                false,
            ),
        ];
        let ink = super::settings::foreground(cx.global::<SessionSettings>().theme_color);
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .children(paths.into_iter().map(|(label, path, config)| {
                let tooltip = path.clone();
                div().flex().flex_col().gap(px(4.)).child(label).child(
                    div()
                        .id(if config {
                            "config-path-value"
                        } else {
                            "output-path-value"
                        })
                        .tab_index(0)
                        .w_full()
                        .px(px(8.))
                        .py(px(7.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(ink.opacity(0.2))
                        .bg(ink.opacity(0.04))
                        .when(!self.pending, |field| {
                            field.cursor_pointer().hover(|style| {
                                style.bg(ink.opacity(0.1)).border_color(ink.opacity(0.4))
                            })
                        })
                        .when(self.pending, |field| field.opacity(0.5))
                        .on_click(cx.listener(move |this, _, _, cx| this.choose(config, cx)))
                        .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, _, cx| {
                            if event.keystroke.key == "enter" || event.keystroke.key == "space" {
                                this.choose(config, cx);
                                cx.stop_propagation();
                            }
                        }))
                        .text_xs()
                        .text_color(ink.opacity(0.75))
                        .truncate()
                        .tooltip(move |_, cx| cx.new(|_| PathTooltip(tooltip.clone())).into())
                        .child(super::path_label::path_label(&path)),
                )
            }))
            .when_some(self.error.clone(), |panel, error| {
                panel.child(
                    div()
                        .text_xs()
                        .child(format!("Could not change path: {error}")),
                )
            })
    }
}

struct PathTooltip(String);

impl Render for PathTooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(8.))
            .py(px(4.))
            .rounded(px(6.))
            .max_w(px(480.))
            .bg(gpui::black())
            .text_color(gpui::white())
            .text_xs()
            .child(self.0.clone())
    }
}
