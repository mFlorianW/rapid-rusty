use common::elapsed_time_source::{ElapsedTimeSource, MonotonicTimeSource};
use common::position::{GnssPosition, Position};
use common::track::Track;
use core::f64;
use module_core::{Event, EventKind, Module, ModuleCtx};
use std::collections::VecDeque;
use std::time::Duration;

/// Returns a list of references to tracks whose start line is within a specified detection radius of a given position.
///
/// Iterates through the provided collection of tracks, calculates the distance from each track’s start line to the specified position,
/// and collects references to tracks that are within the given detection radius.
///
/// # Parameters
/// - `tracks`: A reference to a vector of `Track` instances to check.
/// - `pos`: The `Position` from which distances are measured.
/// - `detection_radius`: The maximum distance (in meters) between `pos` and a track’s start line to consider it detected.
///
/// # Returns
/// A vector containing references to tracks whose start line is within the specified detection radius.
pub fn is_on_track<'a>(
    tracks: &'a Vec<Track>,
    pos: &Position,
    detection_radius: u16,
) -> Vec<&'a Track> {
    let mut detected_tracks = Vec::<&Track>::new();
    for track in tracks {
        let distance = calculate_distance(&track.startline, pos);
        if distance <= detection_radius as f64 {
            detected_tracks.push(track);
        }
    }
    detected_tracks
}

/// Calculates the approximate distance in meters between two geographic positions.
///
/// This function uses a simplified equirectangular approximation to determine  
/// the distance between two latitude/longitude points. It assumes that the  
/// Earth's surface is locally flat and therefore does not account for  
/// large-scale curvature or ellipsoidal effects, making it suitable only  
/// for relatively short distances.
///
/// # Parameters
/// - `pos1`: Reference to the first geographic position.
/// - `pos2`: Reference to the second geographic position.
///
/// # Returns
/// The calculated distance between `pos1` and `pos2` in meters as a `f64`.
///
/// # Notes
/// - The function expects latitude and longitude values in **degrees**.
/// - Accuracy decreases over long distances or near the poles.
/// - This method is more efficient than more precise formulas (e.g., Haversine)  
///   but trades some accuracy for performance.
fn calculate_distance(pos1: &Position, pos2: &Position) -> f64 {
    let lat = (pos1.latitude + pos2.latitude) / 2.0 * 0.01745;
    let dx = 111300.0 * lat.cos() * (pos1.longitude - pos2.longitude);
    let dy = 111300.0 * (pos1.latitude - pos2.latitude);
    (dx * dx + dy * dy).sqrt()
}

/// Represents status updates emitted by the lap timer.
///
/// A `LaptimerStatus` is sent to registered consumers whenever an important
/// event occurs in the lap timing process (e.g., start, sector finish, lap finish).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LaptimerStatus {
    /// Indicates that a new lap has started.
    LapStarted,

    /// Indicates that a lap has finished.
    /// Contains the total lap time.
    LapFinished(Duration),

    /// Indicates that a sector has been completed.
    /// Contains the sector time.
    SectorFinshed(Duration),

    /// Represents a generic laptime (may be used for reporting purposes).
    Laptime(Duration),
}

/// Internal finite state machine (FSM) state of the lap timer.
///
/// The lap timer transitions through these states while processing GNSS positions.
#[derive(Clone, Copy, Debug, PartialEq)]
enum LaptimerState {
    /// Waiting for the car to cross the start line for the first time.
    WaitingForFirstStart,

    /// Actively iterating over sector points to measure sector times.
    IteratingTrackPoints,

    /// Waiting for the car to cross the finish line after the last sector.
    WaitingForFinish,
}

/// A simple lap timer that tracks lap and sector times based on GNSS position updates.
///
/// # Type Parameters
/// - `T`: The time source implementation (e.g., [`MonotonicTimeSource`]) used to measure elapsed time.
///         Defaults to [`MonotonicTimeSource`].
#[derive(Debug)]
pub struct SimpleLaptimer<T: ElapsedTimeSource = MonotonicTimeSource> {
    track: common::track::Track,
    last_positions: VecDeque<Position>,
    state: LaptimerState,
    elapsed_time_source: T,
    sector: usize,
    sector_start: std::time::Duration,
    module_ctx: ModuleCtx,
}

impl SimpleLaptimer<MonotonicTimeSource> {
    /// Creates a new lap timer using the default [`MonotonicTimeSource`].
    pub fn new(track: Track, ctx: ModuleCtx) -> Self {
        SimpleLaptimer::new_with_source(track, MonotonicTimeSource::default(), ctx)
    }
}

impl<T: ElapsedTimeSource + Default> SimpleLaptimer<T> {
    /// Creates a new lap timer with a custom time source.
    pub fn new_with_source(track: Track, elapsed_time_source: T, ctx: ModuleCtx) -> Self {
        SimpleLaptimer {
            last_positions: VecDeque::with_capacity(4),
            track,
            state: LaptimerState::WaitingForFirstStart,
            elapsed_time_source,
            sector: 0,
            sector_start: std::time::Duration::default(),
            module_ctx: ctx,
        }
    }

    /// Returns the current lap time.
    ///
    /// If the lap timer has not yet started (`WaitingForFirstStart`),
    /// this returns `Duration::zero()`.
    pub fn lap_time(&self) -> Duration {
        if self.state != LaptimerState::WaitingForFirstStart {
            return self.elapsed_time_source.elapsed_time();
        }
        Duration::default()
    }

    /// Updates the lap timer with a new GNSS position.
    ///
    /// This method:
    /// - Adds the position to the position history.
    /// - Ensures enough positions are stored to detect line crossing.
    /// - Triggers FSM state transitions and event notifications if needed.
    pub fn update_position(&mut self, pos: &GnssPosition) {
        if self.last_positions.len() == self.last_positions.capacity() {
            self.last_positions.pop_back();
        }
        self.last_positions.push_front(pos.to_position());
        if self.last_positions.len() < 4 {
            return;
        }
        self.calculate_laptimer_state();
    }

    /// Core finite state machine (FSM) logic.
    ///
    /// Depending on the current state and the detected crossing,
    /// this method transitions between states and notifies consumers.
    fn calculate_laptimer_state(&mut self) {
        if self.state == LaptimerState::WaitingForFirstStart
            && self.is_point_passed(&self.track.startline)
        {
            self.elapsed_time_source.start();
            self.state = LaptimerState::IteratingTrackPoints;
            self.sector_start = Duration::default();
            self.notify_consumer(Event {
                kind: EventKind::LapStartedEvent,
            });
        } else if self.state == LaptimerState::IteratingTrackPoints
            && self.is_point_passed(&self.track.sectors[self.sector])
        {
            self.sector += 1;
            if self.sector >= self.track.sectors.len() {
                self.state = LaptimerState::WaitingForFinish;
            }
            self.handle_sector_finsihed();
        } else if self.state == LaptimerState::WaitingForFinish {
            let finish_point = self
                .track
                .finishline
                .map_or(self.track.startline, |finishline| finishline);
            if self.is_point_passed(&finish_point) {
                self.handle_sector_finsihed();
                self.notify_consumer(Event {
                    kind: EventKind::LapFinishedEvent(
                        self.elapsed_time_source.elapsed_time().into(),
                    ),
                });
                if !self.track.sectors.is_empty() {
                    // Start a new lap immediately
                    self.sector = 0;
                    self.sector_start = Duration::default();
                    self.elapsed_time_source.start();
                    self.state = LaptimerState::IteratingTrackPoints;
                    self.notify_consumer(Event {
                        kind: EventKind::LapStartedEvent,
                    });
                }
            }
        }
    }

    /// Handles sector completion:
    /// - Computes the sector time relative to the previous sector start.
    /// - Notifies consumers with [`LaptimerStatus::SectorFinshed`].
    /// - Updates the sector start timestamp.
    fn handle_sector_finsihed(&mut self) {
        let duration = self.elapsed_time_source.elapsed_time() - self.sector_start;
        self.notify_consumer(Event {
            kind: EventKind::SectorFinshedEvent(duration.into()),
        });
        self.sector_start = self.elapsed_time_source.elapsed_time();
    }

    /// Detects whether a position marker (start line, sector, or finish line) has been crossed.
    ///
    /// Uses the last 4 recorded positions to determine:
    /// - Whether the vehicle is within the detection range.
    /// - Whether the crossing direction indicates a valid pass.
    ///
    /// Returns `true` if the point has been passed, `false` otherwise.
    fn is_point_passed(&self, pos: &Position) -> bool {
        let detection_range = 25_u8;
        let mut distances = Vec::<f64>::with_capacity(4);
        let is_in_range = self.last_positions.iter().all(|pos1| {
            let distance = calculate_distance(pos1, pos);
            distances.push(distance);
            distance < detection_range.into()
        });

        if !is_in_range {
            return false;
        }

        let first_distance = distances[0] > distances[1];
        let last_distance = distances[2] < distances[3];
        if first_distance && last_distance && distances[1] != distances[2] {
            return true;
        }
        false
    }

    /// Notifies all registered consumers of a new lap timer status update.
    fn notify_consumer(&self, event: Event) {
        let _ = self.module_ctx.sender.send(event);
    }
}

#[async_trait::async_trait]
impl<T: ElapsedTimeSource + Default + Send> Module for SimpleLaptimer<T> {
    async fn run(&mut self) -> Result<(), ()> {
        let mut run = true;
        while run {
            tokio::select! {
                event = self.module_ctx.receiver.recv() => {
                    match event {
                        Ok(event) => {
                            match event.kind  {
                               EventKind::QuitEvent => run = false,
                               EventKind::GnssPositionEvent(pos) => self.update_position(&pos),
                                _ => (),
                            }
                        },
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
