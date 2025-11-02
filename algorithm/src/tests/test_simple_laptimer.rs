use crate::*;
use common::test_helper::elapsed_test_time_source::{ElapsedTestTimeSource, set_elapsed_time};
use module_core::test_helper::{stop_module, wait_for_event};
use module_core::{EventBus, Module, payload_ref};
use std::sync::Arc;
use tests::laptimer_positions::*;

fn publish_position(event_bus: &EventBus, pos: &GnssPosition) {
    event_bus.publish(&Event {
        kind: EventKind::GnssPositionEvent(Arc::new(*pos)),
    });
}

#[tokio::test]
pub async fn drive_whole_map_with_sectors() {
    let event_bus = EventBus::default();
    let elapsed_time_source = ElapsedTestTimeSource::default();
    let elapsed_time_source_sender = elapsed_time_source.sender();
    let laptimer_module_ctx = event_bus.context();
    let mut laptimer_handle = tokio::spawn(async {
        let mut laptimer = SimpleLaptimer::new_with_source(
            common::test_helper::track::get_track(),
            elapsed_time_source,
            laptimer_module_ctx,
        );
        laptimer.run().await
    });

    {
        // Lapstart
        publish_position(&event_bus, &get_finishline_postion1());
        publish_position(&event_bus, &get_finishline_postion2());
        publish_position(&event_bus, &get_finishline_postion3());
        publish_position(&event_bus, &get_finishline_postion4());
        let exp_event = Event {
            kind: EventKind::LapStartedEvent,
        };
        let event = wait_for_event(
            &mut event_bus.subscribe(),
            Duration::from_millis(100),
            exp_event.kind_discriminant(),
        )
        .await;
        assert_eq!(event.kind_discriminant(), exp_event.kind_discriminant());
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
        let exp_event = Event {
            kind: EventKind::SectorFinshedEvent(std::time::Duration::new(10, 120000000).into()),
        };
        let event = wait_for_event(
            &mut event_bus.subscribe(),
            Duration::from_millis(100),
            exp_event.kind_discriminant(),
        )
        .await;
        assert_eq!(
            payload_ref!(event.kind, EventKind::SectorFinshedEvent).unwrap(),
            payload_ref!(exp_event.kind, EventKind::SectorFinshedEvent).unwrap()
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
        let exp_event = Event {
            kind: EventKind::SectorFinshedEvent(std::time::Duration::new(10, 130000000).into()),
        };
        let event = wait_for_event(
            &mut event_bus.subscribe(),
            Duration::from_millis(100),
            exp_event.kind_discriminant(),
        )
        .await;
        assert_eq!(
            payload_ref!(event.kind, EventKind::SectorFinshedEvent).unwrap(),
            payload_ref!(exp_event.kind, EventKind::SectorFinshedEvent).unwrap()
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
        let exp_sec_finished_event = Event {
            kind: EventKind::SectorFinshedEvent(std::time::Duration::new(10, 140000000).into()),
        };
        let sector_finished_event = wait_for_event(
            &mut receiver,
            std::time::Duration::from_millis(100),
            exp_sec_finished_event.kind_discriminant(),
        );
        let mut receiver = event_bus.subscribe();
        let exp_lap_finshed_event = Event {
            kind: EventKind::LapFinishedEvent(std::time::Duration::new(30, 390000000).into()),
        };
        let lap_finished_event = wait_for_event(
            &mut receiver,
            std::time::Duration::from_millis(100),
            exp_lap_finshed_event.kind_discriminant(),
        );
        let mut receiver = event_bus.subscribe();
        let exp_lap_started_event = Event {
            kind: EventKind::LapStartedEvent,
        };
        let lap_started_event = wait_for_event(
            &mut receiver,
            std::time::Duration::from_millis(100),
            exp_lap_started_event.kind_discriminant(),
        );
        let (sector_finished_event, lap_finished_event, lap_started_event) =
            tokio::join!(sector_finished_event, lap_finished_event, lap_started_event);
        assert_eq!(
            payload_ref!(sector_finished_event.kind, EventKind::SectorFinshedEvent).unwrap(),
            payload_ref!(exp_sec_finished_event.kind, EventKind::SectorFinshedEvent).unwrap()
        );
        assert_eq!(
            payload_ref!(lap_finished_event.kind, EventKind::LapFinishedEvent).unwrap(),
            payload_ref!(exp_lap_finshed_event.kind, EventKind::LapFinishedEvent).unwrap()
        );
        assert_eq!(
            lap_started_event.kind_discriminant(),
            exp_lap_started_event.kind_discriminant()
        );
    }

    stop_module(&event_bus, &mut laptimer_handle).await;
}
