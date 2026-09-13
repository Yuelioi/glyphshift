use crate::diagnostics::trace;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_millis(250);
const SLEEP_STEP: Duration = Duration::from_millis(25);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ObserverLoopError {
    StartFailed,
}

pub(crate) struct ObserverLoop {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl ObserverLoop {
    pub(crate) fn start(callback: fn()) -> Result<Self, ObserverLoopError> {
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("glyphshift-il2cpp-observer".into())
            .spawn(move || run(callback, &worker_stop))
            .map_err(|_| ObserverLoopError::StartFailed)?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }

    pub(crate) fn stop(mut self) {
        self.request_stop();
        if let Some(worker) = self.worker.take() {
            if worker.thread().id() != thread::current().id() {
                let _ = worker.join();
            }
        }
    }

    pub(crate) fn request_stop(&self) {
        self.stop.store(true, Ordering::Release);
    }
}

fn run(callback: fn(), stop: &AtomicBool) {
    trace("observer.loop.enter");
    while !stop.load(Ordering::Acquire) {
        if std::panic::catch_unwind(callback).is_err() {
            trace("observer.callback.panicked");
            return;
        }
        for _ in 0..(POLL_INTERVAL.as_millis() / SLEEP_STEP.as_millis()) {
            if stop.load(Ordering::Acquire) {
                return;
            }
            thread::sleep(SLEEP_STEP);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polling_interval_is_bounded_and_interruptible() {
        assert_eq!(POLL_INTERVAL, Duration::from_millis(250));
        assert_eq!(SLEEP_STEP, Duration::from_millis(25));
        assert_eq!(POLL_INTERVAL.as_millis() % SLEEP_STEP.as_millis(), 0);
    }
}
