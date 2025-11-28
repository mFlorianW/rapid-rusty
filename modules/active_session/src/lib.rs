use async_trait::async_trait;
use chrono::Utc;
use common::{lap::Lap, position::GnssPosition, session::Session};
use module_core::{
    DurationPtr, EventKind, Module, ModuleCtx, Request, SaveSessionRequestPtr,
    TrackDetectionResponsePtr,
};
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info};

pub struct ActiveSession {
    ctx: ModuleCtx,
    session: Option<Arc<RwLock<Session>>>,
    active_lap: Option<Lap>,
}

impl ActiveSession {
    pub fn new(ctx: ModuleCtx) -> Self {
        ActiveSession {
            ctx,
            session: None,
            active_lap: None,
        }
    }

    fn on_track_detected(&mut self, track_request: TrackDetectionResponsePtr) {
        if track_request.id != 10 || track_request.receiver_addr != 100 {
            return;
        }
        let track = match track_request.data.first() {
            Some(t) => t.clone(),
            None => return, // TODO! send here a new request.
        };

        let utc_date = Utc::now();
        let session = Arc::new(RwLock::new(Session::new(
            utc_date.date_naive(),
            utc_date.time(),
            track,
        )));
        info!(
            "Active Session started on Track {}",
            session.read().unwrap().track.name
        );
        self.session = Some(session);
    }

    fn on_lap_started(&mut self) {
        self.active_lap = Some(Lap::default());
    }

    fn on_sector_finished(&mut self, duration: DurationPtr) {
        if let Some(active_lap) = &mut self.active_lap {
            active_lap.sectors.push(*duration);
            info!(
                "Sector {} finished with duration {:?}",
                active_lap.sectors.len(),
                duration
            );
        }
    }

    fn on_lap_finished(&mut self, duration: DurationPtr) {
        if let Some(session_ptr) = &self.session {
            let mut session = session_ptr
                .write()
                .unwrap_or_else(|session| session.into_inner());
            if let Some(active_lap) = self.active_lap.take() {
                session.laps.push(active_lap);
                info!(
                    "Lap {} finished with duration {:?}",
                    session.laps.len(),
                    duration
                );
            }
            let request = SaveSessionRequestPtr::new(Request {
                id: 30,
                sender_addr: 40,
                data: session_ptr.clone(),
            });
            let _ = self
                .ctx
                .publish_event(EventKind::SaveSessionRequestEvent(request));
        }
    }

    /// Handles a new GNSS position update.
    ///
    /// If a lap is currently active, the position is appended to its log for tracking.
    fn on_gnss_position(&mut self, gnss_pos: GnssPosition) {
        if let Some(active_lap) = &mut self.active_lap {
            active_lap.log_points.push(gnss_pos);
        }
    }
}

#[async_trait]
impl Module for ActiveSession {
    async fn run(&mut self) -> std::result::Result<(), ()> {
        let request = Request::empty_request(10, 100);
        let _ = self
            .ctx
            .publish_event(EventKind::DetectTrackRequestEvent(request));
        let mut run = true;
        let mut receiver = self.ctx.receiver();
        while run {
            tokio::select! {
                event = receiver.recv() => {
                    match event {
                        Ok(event) => {
                            match event.kind {
                                EventKind::QuitEvent => run = false,
                                EventKind::DetectTrackResponseEvent(response) => {
                                    self.on_track_detected(response);
                                },
                                EventKind::LapStartedEvent => {
                                    debug!("Lap Started Event received in ActiveSession module");
                                    self.on_lap_started();
                                },
                                EventKind::SectorFinshedEvent(duration) => {
                                    debug!("Sector Finished Event received in ActiveSession module");
                                    self.on_sector_finished(duration);
                                },
                                EventKind::LapFinishedEvent(duration) => {
                                    debug!("Lap Finished Event received in ActiveSession module");
                                    self.on_lap_finished(duration);
                                }
                                EventKind::GnssPositionEvent(gnss_pos) => {
                                    self.on_gnss_position(*gnss_pos);
                                }
                                _ => (),
                            }
                        },
                        Err(e) => {
                            error!("Failed to receive event in module ActiveSession. Error:{e}");
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
