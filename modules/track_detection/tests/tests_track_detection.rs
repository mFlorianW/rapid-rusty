// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use chrono::NaiveDate;
use chrono::NaiveTime;
use common::{position::GnssPosition, test_helper::track::get_track};
use module_core::ModuleCtx;
use module_core::test_helper::register_response_event;
use module_core::{
    Event, EventBus, EventKind, EventKindType, GnssPositionPtr, Module, Request, Response,
    payload_ref,
    test_helper::{stop_module, wait_for_event},
};
use std::time::Duration;
use tokio::task::JoinHandle;
use track_detection::TrackDetection;

fn create_module(ctx: ModuleCtx) -> JoinHandle<Result<(), ()>> {
    tokio::spawn(async move {
        let mut td = TrackDetection::new(ctx);
        td.run().await
    })
}

#[tokio::test]
pub async fn handle_track_detection_request() {
    let event_bus = EventBus::default();
    let mut td = create_module(event_bus.context());

    let _ = register_response_event(
        EventKindType::LoadAllStoredTracksRequestEvent,
        Event {
            kind: EventKind::LoadAllStoredTracksResponseEvent(
                Response {
                    id: 0,
                    receiver_addr: 20,
                    data: vec![get_track()],
                }
                .into(),
            ),
        },
        event_bus.context(),
    );

    event_bus.publish(&Event {
        kind: EventKind::DetectTrackRequestEvent(
            Request {
                id: 0,
                sender_addr: 11,
                data: (),
            }
            .into(),
        ),
    });
    event_bus.publish(&Event {
        kind: EventKind::GnssPositionEvent(GnssPositionPtr::new(GnssPosition::new(
            52.0258333,
            11.279166666,
            20.0,
            &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
            &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
        ))),
    });

    let event = wait_for_event(
        &mut event_bus.subscribe(),
        Duration::from_millis(100),
        EventKindType::DetectTrackResponseEvent,
    )
    .await;

    let event_payload = payload_ref!(event.kind, EventKind::DetectTrackResponseEvent);
    assert!(event_payload.is_some());
    let event_payload = event_payload.unwrap();
    assert_eq!(event_payload.id, 0);
    assert_eq!(event_payload.receiver_addr, 11);
    assert_eq!(event_payload.data, vec![get_track()]);

    stop_module(&event_bus, &mut td).await
}
