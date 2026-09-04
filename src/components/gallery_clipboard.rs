use super::{Action, Gallery};
use crate::config::Config;
use gpui::{ClipboardItem, Context, Image, ImageFormat};

impl Gallery {
    pub(super) fn perform(&mut self, action: Action, cx: &mut Context<Self>) {
        self.options_open = false;
        self.action_notice = None;
        match action {
            Action::Folder => self.open_folder(cx),
            Action::Path => {
                if let Some(path) = &self.selected {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        path.to_string_lossy().into_owned(),
                    ));
                    self.action_notice = Some("Path copied".into());
                }
            }
            Action::Image => self.copy_image(cx),
        }
        cx.notify();
    }

    fn open_folder(&mut self, cx: &mut Context<Self>) {
        let path = cx.global::<Config>().photo_directory.clone();
        let prepare = cx.background_executor().spawn(async move {
            std::fs::create_dir_all(&path)
                .map(|_| path)
                .map_err(|error| format!("Could not open gallery folder: {error}"))
        });
        cx.spawn(async move |this, cx| {
            let result = prepare.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(path) => cx.open_with_system(&path),
                    Err(error) => this.action_notice = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn copy_image(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.selected.clone().filter(|_| !self.copy_busy) else {
            return;
        };
        self.copy_busy = true;
        let read = cx.background_executor().spawn(async move {
            std::fs::read(path).map_err(|error| format!("Could not copy image: {error}"))
        });
        cx.spawn(async move |this, cx| {
            let result = read.await;
            let _ = this.update(cx, |this, cx| {
                this.copy_busy = false;
                this.action_notice = Some(match result {
                    Ok(bytes) => {
                        cx.write_to_clipboard(ClipboardItem::new_image(&Image::from_bytes(
                            ImageFormat::Png,
                            bytes,
                        )));
                        "Image copied".into()
                    }
                    Err(error) => error,
                });
                cx.notify();
            });
        })
        .detach();
    }
}
