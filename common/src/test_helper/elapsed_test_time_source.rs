use crate::elapsed_time_source::ElapsedTimeSource;
use std::sync::mpsc;

/// A test implementation of [`ElapsedTimeSource`] that allows
/// deterministic control of elapsed time in unit tests.
///
/// Internally, it uses an [`mpsc::channel`] to receive externally
/// provided durations, and a [`RefCell`] to hold the latest received
/// duration value.
pub struct ElapsedTestTimeSource {
    sender: std::sync::mpsc::Sender<std::time::Duration>,
    receiver: std::sync::mpsc::Receiver<std::time::Duration>,
    duration: std::cell::RefCell<std::time::Duration>,
}

impl Default for ElapsedTestTimeSource {
    /// Creates a new instance with an internal channel for sending
    /// durations and an initial duration of zero.
    fn default() -> Self {
        let (tx, rx) = mpsc::channel::<std::time::Duration>();
        Self {
            sender: tx,
            receiver: rx,
            // Not synchronized, but sufficient for test usage.
            duration: std::cell::RefCell::new(std::time::Duration::ZERO),
        }
    }
}

impl ElapsedTestTimeSource {
    /// Returns a clone of the internal sender used to provide
    /// durations from outside the test time source.
    pub fn sender(&self) -> std::sync::mpsc::Sender<std::time::Duration> {
        self.sender.clone()
    }

    /// Attempts to receive a new duration value from the channel.
    ///
    /// If a new duration is available, it replaces the current value.
    /// Returns the latest duration held by this time source.
    fn receive(&self) -> std::time::Duration {
        if let Ok(duration) = self.receiver.try_recv() {
            *self.duration.borrow_mut() = duration;
        }
        *self.duration.borrow()
    }
}

impl ElapsedTimeSource for ElapsedTestTimeSource {
    /// Does nothing in this implementation, as test time is externally controlled.
    fn start(&mut self) {}

    /// Returns the current elapsed time value, receiving the latest
    /// duration from the channel if available.
    fn elapsed_time(&self) -> std::time::Duration {
        self.receive()
    }
}

/// Sends a given duration through the provided sender to update
/// an [`ElapsedTestTimeSource`] instance.
///
/// Panics if sending fails, for example if the receiver has been dropped.
pub fn set_elapsed_time(
    sender: &std::sync::mpsc::Sender<std::time::Duration>,
    duration: &std::time::Duration,
) {
    sender
        .send(*duration)
        .unwrap_or_else(|_| panic!("Failed to send duration to the test elapsed time source"));
}
