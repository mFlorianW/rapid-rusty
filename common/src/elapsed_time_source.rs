use std::time::{Duration, Instant};

/// A trait for measuring elapsed time relative to a specific starting point.
///
/// Implementations of this trait provide a way to mark a starting moment
/// and to retrieve the duration that has passed since then. This is useful
/// for timing operations, performance measurements, and other scenarios
/// where monotonic elapsed time is needed.
///
/// The underlying time source should be monotonic to ensure that the measured
/// duration is not affected by system clock adjustments.
pub trait ElapsedTimeSource {
    /// Marks the current moment as the starting point for elapsed time measurement.
    ///
    /// Calling this method resets the starting point. Any subsequent calls to
    /// [`elapsed_time`](Self::elapsed_time) will return the duration since this moment.
    fn start(&mut self);

    /// Returns the duration that has passed since the last call to [`start`](Self::start).
    ///
    /// The returned [`Duration`] should be based on a monotonic clock, ensuring
    /// that the elapsed time is unaffected by system clock changes.
    fn elapsed_time(&self) -> Duration;
}

/// A [`TimeSource`] implementation that uses a monotonic clock to measure elapsed time.
///
/// This source is unaffected by system clock adjustments, making it suitable
/// for measuring durations or time intervals reliably.
///
/// The reference time point is the moment when the struct is created.
pub struct MonotonicTimeSource {
    start: Option<Instant>,
}

impl MonotonicTimeSource {
    pub fn new() -> Self {
        MonotonicTimeSource { start: None }
    }
}

impl Default for MonotonicTimeSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ElapsedTimeSource for MonotonicTimeSource {
    /// Creates a new monotonic time source with the current instant as the start time.
    fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    /// Returns the [`Duration`] elapsed since this time source was started.
    fn elapsed_time(&self) -> Duration {
        self.start.map_or(Duration::ZERO, |time| time.elapsed())
    }
}
