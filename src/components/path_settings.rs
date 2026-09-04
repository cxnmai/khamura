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

