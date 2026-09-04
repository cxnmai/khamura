use super::*;

impl Render for Camera {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_pending {
            self.focus_pending = false;
            if self.settings_open {
                self.previous_focus = window.focused(cx);
                self.settings.read(cx).focus_handle(cx).focus(window);
            } else if let Some(focus) = self.previous_focus.take() {
                focus.focus(window);
            } else {
                window.blur();
            }
        }
        let config = cx.global::<SessionSettings>();
        if let Some(frame) = self.frame.clone() {
            let image = match config.fit {
                CameraFit::Contain => {
                    // Size the painted image explicitly so resizing the window always leaves
                    // the unused area outside the image as letterbox space.
                    let image_size = contain_size(window.viewport_size(), frame.size(0));
                    img(frame)
                        .w(image_size.width)
                        .h(image_size.height)
                        .object_fit(ObjectFit::Fill)
                }
                CameraFit::Cover => img(frame).size_full().object_fit(ObjectFit::Cover),
            };

            div()
                .size_full()
                .relative()
                .flex()
                .items_center()
                .justify_center()
                // The image is opaque; only the letterbox area uses this alpha.
                .bg(config.theme_color.to_gpui(config.background_opacity))
                .child(image)
                .when(self.settings_open, |view| {
                    view.child(self.settings_overlay(cx))
                })
                .child(self.toolbar.clone())
        } else {
            div()
                .size_full()
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .bg(gpui::black())
                .text_color(gpui::white())
                .child(self.status.clone())
                .when(self.settings_open, |view| {
                    view.child(self.settings_overlay(cx))
                })
                .child(self.toolbar.clone())
        }
    }
}

fn contain_size(
    viewport: Size<gpui::Pixels>,
    image: Size<gpui::DevicePixels>,
) -> Size<gpui::Pixels> {
    let image_ratio = image.width.0 as f32 / image.height.0 as f32;
    let viewport_ratio = viewport.width / viewport.height;

    if viewport_ratio > image_ratio {
        size(viewport.height * image_ratio, viewport.height)
    } else {
        size(viewport.width, viewport.width / image_ratio)
    }
}

