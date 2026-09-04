//! Sleep until the next frame or a stop request, without polling every 5ms.
use std::{
    sync::{Condvar, Mutex},
    time::Instant,
};

#[derive(Default)]
pub(super) struct RecordingStop {
    time: Mutex<Option<Instant>>,
    wake: Condvar,
}

impl RecordingStop {
    pub fn request(&self) {
        self.time.lock().unwrap().get_or_insert_with(Instant::now);
        self.wake.notify_all();
    }

    pub fn time(&self) -> Option<Instant> {
        *self.time.lock().unwrap()
    }

    pub fn wait_until(&self, deadline: Instant) {
        let mut stopped = self.time.lock().unwrap();
        while stopped.is_none() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            stopped = self.wake.wait_timeout(stopped, remaining).unwrap().0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Arc, time::Duration};

    #[test]
    fn stop_wakes_frame_wait_and_preserves_first_timestamp() {
        let stop = Arc::new(RecordingStop::default());
        let worker_stop = stop.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            worker_stop.wait_until(Instant::now() + Duration::from_secs(10));
            tx.send(()).unwrap();
        });
        stop.request();
        let first = stop.time();
        stop.request();
        assert_eq!(stop.time(), first);
        rx.recv_timeout(Duration::from_secs(1)).unwrap();
        worker.join().unwrap();
    }
}
