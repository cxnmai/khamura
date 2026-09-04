use super::*;

impl Camera {
    pub(super) fn start_recording(&mut self, cx: &mut Context<Self>) {
        let Some(source) = self.source_frame.clone() else {
            return;
        };
        let prefs = cx.global::<CaptureSettings>();
        let microphone = prefs.microphone_on.then(|| {
            prefs
                .microphone_device
                .clone()
                .unwrap_or_else(|| "default".into())
        });
        let output = cx.global::<Config>().photo_directory.clone();
        let mirror = cx.global::<SessionSettings>().mirror;
        let fps = self.fps;
        self.set_activity(Activity::StartingVideo, cx);
        let startup = cx.background_executor().spawn(async move {
            crate::media::Recorder::start(&output, source, mirror, microphone, fps)
        });
        self.media_task = Some(cx.spawn(async move |this, cx| {
            let result = startup.await;
            let completion = this
                .update(cx, |camera, cx| match result {
                    Ok((recorder, completion)) => {
                        if let Some(source) = &camera.source_frame {
                            recorder.update_frame(source.clone());
                        }
                        camera.recorder = Some(recorder);
                        camera.set_activity(Activity::Recording(Instant::now()), cx);
                        if camera.close_requested || !camera.capture_ready {
                            camera.stop_or_cancel(cx);
                        }
                        Some(completion)
                    }
                    Err(error) => {
                        camera.finish_media(Err(error), cx);
                        None
                    }
                })
                .ok()
                .flatten();
            if let Some(completion) = completion {
                let result = completion
                    .recv()
                    .await
                    .unwrap_or_else(|_| Err("Recording worker disconnected".into()));
                let _ = this.update(cx, |camera, cx| camera.finish_media(result, cx));
            }
        }));
    }
}
