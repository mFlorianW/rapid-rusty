// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use algorithm::is_on_track;
use async_trait::async_trait;
use common::{position::Position, track::Track};
use module_core::{
    EmptyRequestPtr, Event, EventKind, Module, ModuleCtx, Request, Response,
    TrackDetectionResponsePtr,
};
use std::{collections::VecDeque, result::Result};
use tracing::{error, info};

/// The `TrackDetection` module is responsible for detecting which tracks
/// the system is currently located on, based on GNSS position updates and
/// previously loaded track data.
///
/// It manages incoming detection requests and responds asynchronously
/// once position and track information are available.
pub struct TrackDetection {
    ctx: ModuleCtx,
    position: Option<Position>,
    pending_requests: VecDeque<EmptyRequestPtr>,
    tracks: Vec<Track>,
}

impl TrackDetection {
    /// Creates a new `TrackDetection` instance with an empty state and
    /// initialized communication context.
    pub fn new(ctx: ModuleCtx) -> Self {
        TrackDetection {
            ctx,
            position: None,
            pending_requests: VecDeque::new(),
            tracks: vec![],
        }
    }

    /// Processes any pending detection requests if both position and
    /// track data are available.
    ///
    /// For each request, it determines which tracks are within a
    /// configured proximity threshold of the current position and sends
    /// a corresponding detection response event.
    fn handle_pending_requests(&mut self) {
        if self.position.is_none() {
            return;
        }
        if self.pending_requests.is_empty() || self.tracks.is_empty() {
            return;
        }
        let detected_tracks: Vec<Track> =
            is_on_track(&self.tracks, self.position.as_ref().unwrap(), 500)
                .into_iter()
                .cloned()
                .collect();
        while !self.pending_requests.is_empty() {
            let request = self.pending_requests.pop_front().unwrap();
            let response =
                EventKind::DetectTrackResponseEvent(TrackDetectionResponsePtr::new(Response {
                    id: request.id,
                    receiver_addr: request.sender_addr,
                    data: detected_tracks.clone(),
                }));
            let _ = self.ctx.sender.send(Event { kind: response });
            info!(
                "Sent track detection response for request id {}, receiver id {}",
                request.id, request.sender_addr
            );
        }
    }
}

#[async_trait]
impl Module for TrackDetection {
    /// Runs the `TrackDetection` module's main event loop.
    ///
    /// It listens for GNSS position updates, track data loading responses,
    /// and detection requests. Upon receiving relevant events, it updates
    /// its internal state and triggers detection handling accordingly.
    ///
    /// The loop terminates when a `QuitEvent` is received.
    async fn run(&mut self) -> Result<(), ()> {
        let _ = self.ctx.sender.send(Event {
            kind: EventKind::LoadAllStoredTracksRequestEvent(
                Request {
                    id: 0,
                    sender_addr: 20,
                    data: (),
                }
                .into(),
            ),
        });
        let mut run = true;
        while run {
            tokio::select! {
                event = self.ctx.receiver.recv() => {
                    match event {
                        Ok(event) => {
                            match event.kind {
                                EventKind::QuitEvent => run = false,
                                EventKind::GnssPositionEvent(position) => {
                                    self.position = Some(Position { latitude: position.latitude(), longitude: position.longitude() });
                                    self.handle_pending_requests();
                                }
                                EventKind::LoadAllStoredTracksResponseEvent(tracks) => {
                                    self.tracks = tracks.data.clone();
                                    self.handle_pending_requests();
                                }
                                EventKind::DetectTrackRequestEvent(request) => {
                                    info!("Received track detection request. id: {}, sender id: {}", request.id, request.sender_addr);
                                    self.pending_requests.push_back(request);
                                    self.handle_pending_requests();
                                }
                                _ => (),
                            }
                        }
                        Err(e) => error!("Failed to receive event. Error {}", e)
                    }
                }
            }
        }
        Ok(())
    }
}
