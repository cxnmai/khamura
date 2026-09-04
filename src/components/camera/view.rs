use super::*;

impl Render for Camera {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(gallery) = &self.gallery {
            return gallery.clone().into_any_element();
        }
        if self.focus_pending {
            self.focus_pending = false;
            if self.settings_open {
                self.previous_focus = window.focused(cx);
                self.settings.read(cx).focus_handle(cx).focus(window);
            } else if let Some(focus) = self.previous_focus.take() {
                focus.focus(window);
            } else {
                self.root_focus.focus(window);
            }
        }
        let settings = cx.global::<SessionSettings>();
        let capture = cx.global::<CaptureSettings>();
        // An explicit photo ratio describes the saved crop: never crop it a second time.
        let fit = if capture.mode == CameraMode::Photo && capture.aspect != PhotoAspect::Native {
            CameraFit::Contain
        } else {
            settings.fit
        };
        let background = settings.theme_color.to_gpui(settings.background_opacity);
        let show_grid = cx.global::<CaptureSettings>().grid;
        div()
            .size_full()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .track_focus(&self.root_focus)
            .on_key_down(cx.listener(|camera, event: &gpui::KeyDownEvent, _, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => {
                        if camera.error.take().is_none() {
                            if camera.settings_open {
                                camera.set_settings_open(false, cx);
                            } else if matches!(camera.activity, Activity::Countdown(_)) {
                                camera.stop_or_cancel(cx);
                            }
                        }
                        cx.notify();
                        cx.stop_propagation();
                    }
                    "space"
                        if !event.is_held && !camera.settings_open && camera.error.is_none() =>
                    {
                        camera.capture(cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }))
            .bg(background)
            .when_some(self.frame.clone(), |view, frame| {
                let image_size = match fit {
                    CameraFit::Contain => contain_size(window.viewport_size(), frame.size(0)),
                    CameraFit::Cover => window.viewport_size(),
                };
                view.child(
                    div()
                        .relative()
                        .w(image_size.width)
                        .h(image_size.height)
                        .child(
                            img(frame)
                                .size_full()
                                .object_fit(if fit == CameraFit::Contain {
                                    ObjectFit::Fill
                                } else {
                                    ObjectFit::Cover
                                }),
                        )
                        .when(show_grid, |image| image.child(overlays::grid())),
                )
            })
            .when(self.frame.is_none(), |view| {
                view.child(div().text_color(gpui::white()).child(self.status.clone()))
            })
            .when(self.settings_open, |view| {
                view.child(self.settings_overlay(cx))
            })
            .child(self.toolbar.clone())
            .child(self.overlays(cx))
            .into_any_element()
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
