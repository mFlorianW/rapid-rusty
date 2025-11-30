// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use algorithm::calculate_distance;
use common::elapsed_time_source::{ElapsedTimeSource, MonotonicTimeSource};
use common::position::{GnssPosition, Position};
use core::f64;
use module_core::{Event, EventKind, Module, ModuleCtx, Request};
use std::collections::VecDeque;
use std::time::Duration;
use tracing::{error, info};

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
///   Defaults to [`MonotonicTimeSource`].
#[derive(Debug)]
pub struct SimpleLaptimer<T: ElapsedTimeSource = MonotonicTimeSource> {
    track: Option<common::track::Track>,
    last_positions: VecDeque<Position>,
    state: LaptimerState,
    elapsed_time_source: T,
    sector: usize,
    sector_start: std::time::Duration,
    module_ctx: ModuleCtx,
}

impl SimpleLaptimer<MonotonicTimeSource> {
    /// Creates a new lap timer using the default [`MonotonicTimeSource`].
    pub fn new(ctx: ModuleCtx) -> Self {
        SimpleLaptimer::new_with_source(MonotonicTimeSource::default(), ctx)
    }
}

impl<T: ElapsedTimeSource + Default> SimpleLaptimer<T> {
    /// Creates a new lap timer with a custom time source.
    pub fn new_with_source(elapsed_time_source: T, ctx: ModuleCtx) -> Self {
        SimpleLaptimer {
            last_positions: VecDeque::with_capacity(4),
            track: None,
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
        if self.track.is_some() {
            self.calculate_laptimer_state();
        }
    }

    /// Core finite state machine (FSM) logic.
    ///
    /// Depending on the current state and the detected crossing,
    /// this method transitions between states and notifies consumers.
    fn calculate_laptimer_state(&mut self) {
        let track = match self.track {
            Some(ref t) => t.clone(),
            None => {
                error!("calculate laptimer called without track");
                return;
            }
        };

        if self.state == LaptimerState::WaitingForFirstStart
            && self.is_point_passed(&track.startline)
        {
            self.elapsed_time_source.start();
            self.state = LaptimerState::IteratingTrackPoints;
            self.sector_start = Duration::default();
            self.notify_consumer(Event {
                kind: EventKind::LapStartedEvent,
            });
        } else if self.state == LaptimerState::IteratingTrackPoints
            && self.is_point_passed(&track.sectors[self.sector])
        {
            self.sector += 1;
            if self.sector >= track.sectors.len() {
                self.state = LaptimerState::WaitingForFinish;
            }
            self.handle_sector_finsihed();
        } else if self.state == LaptimerState::WaitingForFinish {
            let finish_point = track
                .finishline
                .map_or(track.startline, |finishline| finishline);
            if self.is_point_passed(&finish_point) {
                self.handle_sector_finsihed();
                self.notify_consumer(Event {
                    kind: EventKind::LapFinishedEvent(
                        self.elapsed_time_source.elapsed_time().into(),
                    ),
                });
                if !track.sectors.is_empty() {
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
        let _ = self.module_ctx.sender.send(Event {
            kind: EventKind::DetectTrackRequestEvent(
                Request {
                    id: 10,
                    sender_addr: 22,
                    data: (),
                }
                .into(),
            ),
        });

        let mut run = true;
        while run {
            tokio::select! {
                event = self.module_ctx.receiver.recv() => {
                    match event {
                        Ok(event) => {
                            match event.kind  {
                               EventKind::QuitEvent => {
                                   run = false
                               },
                               EventKind::GnssPositionEvent(pos) => {
                                   self.update_position(&pos);
                               },
                               EventKind::DetectTrackResponseEvent(track) => {
                                   if !track.data.is_empty() && track.id == 10  && track.receiver_addr == 22 {
                                       self.track = Some(track.data[0].clone());
                                       self.calculate_laptimer_state();
                                       info!("Track configured for Track {}", self.track.as_ref().unwrap().name);
                                   }
                               }
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
