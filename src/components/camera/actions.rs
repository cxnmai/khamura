use super::*;
use std::path::PathBuf;

impl Camera {
    pub(super) fn set_activity(&mut self, activity: Activity, cx: &mut Context<Self>) {
        self.activity = activity;
        cx.update_global::<CaptureSettings, _>(|settings, _| {
            settings.busy = !matches!(activity, Activity::Idle);
            settings.recording = matches!(activity, Activity::Recording(_));
        });
        cx.notify();
    }

    pub(super) fn capture(&mut self, cx: &mut Context<Self>) {
        if matches!(
            self.activity,
            Activity::Recording(_) | Activity::Countdown(_)
        ) {
            self.stop_or_cancel(cx);
            return;
        }
        if !matches!(self.activity, Activity::Idle) {
            return;
        }
        if !self.capture_ready || self.source_frame.is_none() {
            self.error = Some("Camera is not ready. Check the selected camera in settings.".into());
            cx.notify();
            return;
        }
        self.error = None;
        let prefs = cx.global::<CaptureSettings>();
        match prefs.mode {
            CameraMode::Photo if prefs.timer_seconds > 0 => self.set_activity(
                Activity::Countdown(Instant::now() + Duration::from_secs(prefs.timer_seconds)),
                cx,
            ),
            CameraMode::Photo => self.take_photo(cx),
            CameraMode::Video => self.start_recording(cx),
        }
    }

    pub(super) fn take_photo(&mut self, cx: &mut Context<Self>) {
        if !self.capture_ready {
            self.set_activity(Activity::Idle, cx);
            return;
        }
        let Some(source) = self.source_frame.clone() else {
            self.set_activity(Activity::Idle, cx);
            return;
        };
        let output = cx.global::<Config>().photo_directory.clone();
        let mirror = cx.global::<SessionSettings>().mirror;
        let aspect = cx.global::<CaptureSettings>().aspect;
        self.set_activity(Activity::SavingPhoto, cx);
        let task = cx
            .background_executor()
            .spawn(async move { crate::media::save_photo(&output, &source, mirror, aspect) });
        self.media_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |camera, cx| camera.finish_media(result, cx));
        }));
    }

    pub(super) fn stop_or_cancel(&mut self, cx: &mut Context<Self>) {
        match self.activity {
            Activity::Countdown(_) => self.set_activity(Activity::Idle, cx),
            Activity::Recording(_) => {
                if let Some(recorder) = &self.recorder {
                    recorder.stop();
                }
                self.set_activity(Activity::Finalizing, cx);
            }
            _ => {}
        }
    }

    pub(super) fn finish_media(&mut self, result: Result<PathBuf, String>, cx: &mut Context<Self>) {
        self.recorder = None;
        self.set_activity(Activity::Idle, cx);
        match result {
            Ok(path) => {
                self.notice = Some((
                    format!(
                        "Saved {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ),
                    Instant::now() + Duration::from_secs(3),
                ));
                if self.close_requested {
                    let handle = self.window_handle;
                    cx.defer(move |cx| {
                        let _ = handle.update(cx, |_, window, _| window.remove_window());
                    });
                }
            }
            Err(error) => {
                self.error = Some(error);
                self.close_requested = false;
            }
        }
        cx.notify();
    }

    pub(super) fn request_close(&mut self, cx: &mut Context<Self>) -> bool {
        self.stop_or_cancel(cx);
        if matches!(self.activity, Activity::Idle) {
            return true;
        }
        self.close_requested = true;
        false
    }
}
