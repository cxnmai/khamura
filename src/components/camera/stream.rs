use super::*;

impl Camera {
    pub(super) fn receive_frames(
        cx: &mut Context<Self>,
        receiver: Receiver<CaptureMessage>,
    ) -> Task<()> {
        cx.spawn(async move |this, cx| {
            while let Ok(message) = receiver.recv().await {
                if this
                    .update(cx, |camera, cx| {
                        match message {
                            CaptureMessage::Ready(revision, choices, fps)
                                if revision == camera.request.revision =>
                            {
                                camera.fps = fps;
                                cx.update_global::<CaptureSettings, _>(|settings, _| {
                                    settings.qualities = choices
                                });
                            }
                            CaptureMessage::Frame(revision, frame)
                                if revision == camera.request.revision =>
                            {
                                if let Some(source) =
                                    RgbaImage::from_raw(frame.width, frame.height, frame.pixels)
                                {
                                    let source = Arc::new(source);
                                    if let Some(recorder) = &camera.recorder {
                                        recorder.update_frame(source.clone());
                                    }
                                    camera.source_frame = Some(source);
                                    camera.capture_ready = true;
                                    camera.refresh_preview(cx);
                                    camera.status.clear();
                                }
                            }
                            CaptureMessage::Error(revision, error)
                                if revision == camera.request.revision =>
                            {
                                camera.capture_ready = false;
                                camera.status = error.clone();
                                camera.error = Some(error);
                                camera.stop_or_cancel(cx);
                            }
                            _ => {}
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
    }

    pub(super) fn preferences_changed(&mut self, cx: &mut Context<Self>) {
        let prefs = cx.global::<CaptureSettings>();
        let quality = if prefs.mode == CameraMode::Video {
            prefs.quality
        } else {
            None
        };
        let save_error = prefs.save_error.clone();
        let device = prefs.camera_device.clone();
        if device != self.request.device || quality != self.request.quality {
            self.request = CaptureRequest {
                revision: self.request.revision + 1,
                device,
                quality,
            };
            self.capture_ready = false;
            self.source_frame = None;
            if let Some(frame) = self.frame.take() {
                cx.drop_image(frame, None);
            }
            self.status = "Opening camera…".into();
            let _ = self.requests.try_send(self.request.clone());
        }
        if let Some(error) = &save_error {
            self.error = Some(format!("Could not save capture settings: {error}"));
        }
        self.refresh_preview(cx);
        cx.notify();
    }

    pub(super) fn retry_camera(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.activity, Activity::Idle) {
            return;
        }
        self.request.revision += 1;
        self.error = None;
        self.capture_ready = false;
        self.source_frame = None;
        if let Some(frame) = self.frame.take() {
            cx.drop_image(frame, None);
        }
        self.status = "Opening camera…".into();
        let _ = self.requests.try_send(self.request.clone());
        cx.notify();
    }

    pub(super) fn tick_task(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| loop {
            gpui::Timer::after(Duration::from_millis(100)).await;
            if this.update(cx, |camera, cx| {
                if matches!(camera.activity, Activity::Countdown(deadline) if Instant::now() >= deadline) { camera.take_photo(cx); }
                if camera.notice.as_ref().is_some_and(|(_, expiry)| Instant::now() >= *expiry) { camera.notice = None; cx.notify(); }
                if !matches!(camera.activity, Activity::Idle) { cx.notify(); }
            }).is_err() { break; }
        })
    }
}
