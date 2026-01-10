// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use active_session::ActiveSession;
use common::{lap::Lap, position::GnssPosition, test_helper::track::get_track};
use module_core::{
    Event, EventBus, EventKind, EventKindType, Module, Response, payload_ref,
    test_helper::{register_response_event, stop_module, wait_for_event},
};
use std::time::Duration;
use tracing::debug;

fn create_module(eb: &EventBus) -> tokio::task::JoinHandle<Result<(), ()>> {
    if register_response_event(
        EventKindType::DetectTrackRequestEvent,
        Event {
            kind: EventKind::DetectTrackResponseEvent(
                Response {
                    id: 10,
                    receiver_addr: 100,
                    data: vec![get_track()],
                }
                .into(),
            ),
        },
        eb.context(),
    )
    .is_err()
    {
        panic!("Failed to register DetectTrackResponseEvent");
    }

    let session = ActiveSession::new(eb.context());
    tokio::spawn(async move {
        let mut session = session;
        session.run().await
    })
}

#[tokio::test]
#[test_log::test]
async fn test_store_session_when_lap_finished() {
    let eb = EventBus::default();
    let mut active_session = create_module(&eb);

    // Before emitting the lap start wait for the track detected event.
    let _track_event = wait_for_event(
        &mut eb.subscribe(),
        Duration::from_millis(100),
        EventKindType::DetectTrackResponseEvent,
    )
    .await;

    eb.publish(&Event {
        kind: EventKind::LapStartedEvent,
    });
    eb.publish(&Event {
        kind: EventKind::SectorFinishedEvent(std::time::Duration::from_secs_f32(10.250).into()),
    });
    eb.publish(&Event {
        kind: EventKind::SectorFinishedEvent(std::time::Duration::from_secs_f32(10.250).into()),
    });
    eb.publish(&Event {
        kind: EventKind::SectorFinishedEvent(std::time::Duration::from_secs_f32(10.250).into()),
    });
    eb.publish(&Event {
        kind: EventKind::LapFinishedEvent(std::time::Duration::from_secs_f32(30.750).into()),
    });

    debug!("Waiting for SaveSessionRequestEvent...");
    let store_event = wait_for_event(
        &mut eb.subscribe(),
        Duration::from_millis(100),
        EventKindType::SaveSessionRequestEvent,
    )
    .await;

    //scope is needed to clear the rwlock at the end.
    {
        let session = match payload_ref!(store_event.kind, EventKind::SaveSessionRequestEvent) {
            Some(request) => request
                .data
                .read()
                .unwrap_or_else(|session| session.into_inner()),
            None => {
                panic!("Received session doesn't have a payload");
            }
        };
        assert_eq!(session.laps.len(), 1);
        let lap = Lap {
            sectors: vec![Duration::from_secs_f32(10.250); 3],
            log_points: vec![],
        };
        assert_eq!(session.laps[0], lap);
        assert_eq!(session.track, get_track());
    }

    stop_module(&eb, &mut active_session).await;
}

#[tokio::test]
#[test_log::test]
async fn test_store_log_points() {
    let eb = EventBus::default();
    let mut active_session = create_module(&eb);

    // Before emitting the lap start wait for the track detected event.
    let _track_event = wait_for_event(
        &mut eb.subscribe(),
        Duration::from_millis(100),
        EventKindType::DetectTrackResponseEvent,
    )
    .await;

    eb.publish(&Event {
        kind: EventKind::LapStartedEvent,
    });
    let gnss_position = GnssPosition::new(
        52.0,
        11.0,
        100.0,
        &chrono::NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap(),
        &chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
    );
    eb.publish(&Event {
        kind: EventKind::GnssPositionEvent(gnss_position.into()),
    });
    eb.publish(&Event {
        kind: EventKind::GnssPositionEvent(gnss_position.into()),
    });
    eb.publish(&Event {
        kind: EventKind::LapFinishedEvent(std::time::Duration::from_secs_f32(30.750).into()),
    });

    debug!("Waiting for SaveSessionRequestEvent...");
    let store_event = wait_for_event(
        &mut eb.subscribe(),
        Duration::from_millis(100),
        EventKindType::SaveSessionRequestEvent,
    )
    .await;

    //scope is needed to clear the rwlock at the end.
    {
        let session = match payload_ref!(store_event.kind, EventKind::SaveSessionRequestEvent) {
            Some(request) => request
                .data
                .read()
                .unwrap_or_else(|session| session.into_inner()),
            None => {
                panic!("Received session doesn't have a payload");
            }
        };
        assert_eq!(session.laps[0].log_points.len(), 2);
        let lap = Lap {
            sectors: vec![],
            log_points: vec![gnss_position, gnss_position],
        };
        assert_eq!(session.laps[0], lap);
        assert_eq!(session.track, get_track());
    }

    stop_module(&eb, &mut active_session).await;
}
