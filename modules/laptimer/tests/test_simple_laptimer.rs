use common::elapsed_time_source::ElapsedTimeSource;
use common::position::GnssPosition;
use common::test_helper::elapsed_test_time_source::{ElapsedTestTimeSource, set_elapsed_time};
use common::test_helper::track::get_track;
use laptimer::*;
use module_core::test_helper::{register_response_event, stop_module, wait_for_event};
use module_core::{Event, EventBus, EventKind, EventKindType, Module, Response, payload_ref};
use std::sync::Arc;
use std::time::Duration;
mod util;
use util::laptimer_positions::*;

fn publish_position(event_bus: &EventBus, pos: &GnssPosition) {
    event_bus.publish(&Event {
        kind: EventKind::GnssPositionEvent(Arc::new(*pos)),
    });
}

fn create_laptimer<T>(
    event_bus: &EventBus,
    elapsed_time_source: T,
) -> tokio::task::JoinHandle<Result<(), ()>>
where
    T: ElapsedTimeSource + Default + Send + 'static,
{
    if register_response_event(
        EventKindType::DetectTrackRequestEvent,
        Event {
            kind: EventKind::DetectTrackResponseEvent(
                Response {
                    id: 10,
                    receiver_addr: 22,
                    data: vec![get_track()],
                }
                .into(),
            ),
        },
        event_bus.context(),
    )
    .is_err()
    {
        panic!("Failed to register DetectTrackResponseEvent");
    }

    let lp = SimpleLaptimer::new_with_source(elapsed_time_source, event_bus.context());
    tokio::spawn(async move {
        let mut laptimer = lp;
        laptimer.run().await
    })
}

#[tokio::test]
#[test_log::test]
pub async fn drive_whole_map_with_sectors() {
    let event_bus = EventBus::default();
    let elapsed_time_source = ElapsedTestTimeSource::default();
    let elapsed_time_source_sender = elapsed_time_source.sender();
    let mut laptimer_handle = create_laptimer(&event_bus, elapsed_time_source);

    {
        // Lapstart
        publish_position(&event_bus, &get_finishline_postion1());
        publish_position(&event_bus, &get_finishline_postion2());
        publish_position(&event_bus, &get_finishline_postion3());
        publish_position(&event_bus, &get_finishline_postion4());
        let event = wait_for_event(
            &mut event_bus.subscribe(),
            Duration::from_millis(100),
            EventKindType::LapStartedEvent,
        )
        .await;
        assert_eq!(
            EventKindType::from(event.kind),
            EventKindType::LapStartedEvent
        );
    }

    {
        // Sector1
        set_elapsed_time(
            &elapsed_time_source_sender,
            &std::time::Duration::from_millis(10120),
        );
        publish_position(&event_bus, &get_sector1_postion1());
        publish_position(&event_bus, &get_sector1_postion2());
        publish_position(&event_bus, &get_sector1_postion3());
        publish_position(&event_bus, &get_sector1_postion4());
        let event = wait_for_event(
            &mut event_bus.subscribe(),
            Duration::from_millis(100),
            EventKindType::SectorFinshedEvent,
        )
        .await;
        assert_eq!(
            **payload_ref!(event.kind, EventKind::SectorFinshedEvent).unwrap(),
            std::time::Duration::new(10, 120000000)
        );
    }

    {
        //Sector2
        set_elapsed_time(
            &elapsed_time_source_sender,
            &std::time::Duration::from_millis(20250),
        );
        publish_position(&event_bus, &get_sector2_postion1());
        publish_position(&event_bus, &get_sector2_postion2());
        publish_position(&event_bus, &get_sector2_postion3());
        publish_position(&event_bus, &get_sector2_postion4());
        let event = wait_for_event(
            &mut event_bus.subscribe(),
            Duration::from_millis(100),
            EventKindType::SectorFinshedEvent,
        )
        .await;
        assert_eq!(
            **payload_ref!(event.kind, EventKind::SectorFinshedEvent).unwrap(),
            std::time::Duration::new(10, 130000000)
        );
    }

    {
        // LapFinished
        set_elapsed_time(
            &elapsed_time_source_sender,
            &std::time::Duration::from_millis(30390),
        );
        publish_position(&event_bus, &get_finishline_postion1());
        publish_position(&event_bus, &get_finishline_postion2());
        publish_position(&event_bus, &get_finishline_postion3());
        publish_position(&event_bus, &get_finishline_postion4());

        let mut receiver = event_bus.subscribe();
        let sector_finished_event = wait_for_event(
            &mut receiver,
            std::time::Duration::from_millis(100),
            EventKindType::SectorFinshedEvent,
        );
        let mut receiver = event_bus.subscribe();
        let lap_finished_event = wait_for_event(
            &mut receiver,
            std::time::Duration::from_millis(100),
            EventKindType::LapFinishedEvent,
        );
        let mut receiver = event_bus.subscribe();
        let lap_started_event = wait_for_event(
            &mut receiver,
            std::time::Duration::from_millis(100),
            EventKindType::LapStartedEvent,
        );
        let (sector_finished_event, lap_finished_event, lap_started_event) =
            tokio::join!(sector_finished_event, lap_finished_event, lap_started_event);
        assert_eq!(
            **payload_ref!(sector_finished_event.kind, EventKind::SectorFinshedEvent).unwrap(),
            std::time::Duration::new(10, 140000000)
        );
        assert_eq!(
            **payload_ref!(lap_finished_event.kind, EventKind::LapFinishedEvent).unwrap(),
            std::time::Duration::new(30, 390000000)
        );
        assert_eq!(
            EventKindType::from(lap_started_event.kind),
            EventKindType::LapStartedEvent
        );
    }

    stop_module(&event_bus, &mut laptimer_handle).await;
}
