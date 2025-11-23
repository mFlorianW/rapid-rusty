use active_session::ActiveSession;
use common::{lap::Lap, test_helper::track::get_track};
use module_core::{
    Event, EventBus, EventKind, EventKindDiscriminants, Module, Response, payload_ref,
    test_helper::{register_response_event, stop_module, wait_for_event},
};
use std::time::Duration;
use tracing::debug;

fn create_module(eb: &EventBus) -> tokio::task::JoinHandle<Result<(), ()>> {
    let session = ActiveSession::new(eb.context());
    if register_response_event(
        EventKindDiscriminants::DetectTrackRequestEvent,
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

    tokio::spawn(async move {
        let mut session = session;
        session.run().await
    })
}

#[tokio::test]
async fn store_session_when_lap_finished() {
    let eb = EventBus::default();
    let mut active_session = create_module(&eb);

    // Before emitting the lap start wait for the track detected event.
    let _track_event = wait_for_event(
        &mut eb.subscribe(),
        Duration::from_millis(100),
        EventKindDiscriminants::DetectTrackResponseEvent,
    )
    .await;

    eb.publish(&Event {
        kind: EventKind::LapStartedEvent,
    });
    eb.publish(&Event {
        kind: EventKind::SectorFinshedEvent(std::time::Duration::from_secs_f32(10.250).into()),
    });
    eb.publish(&Event {
        kind: EventKind::SectorFinshedEvent(std::time::Duration::from_secs_f32(10.250).into()),
    });
    eb.publish(&Event {
        kind: EventKind::SectorFinshedEvent(std::time::Duration::from_secs_f32(10.250).into()),
    });
    eb.publish(&Event {
        kind: EventKind::LapFinishedEvent(std::time::Duration::from_secs_f32(30.750).into()),
    });

    debug!("Waiting for SaveSessionRequestEvent...");
    let store_event = wait_for_event(
        &mut eb.subscribe(),
        Duration::from_millis(100),
        EventKindDiscriminants::SaveSessionRequestEvent,
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
