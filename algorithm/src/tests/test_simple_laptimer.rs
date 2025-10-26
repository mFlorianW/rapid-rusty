use crate::*;
use common::test_helper::elapsed_test_time_source::{set_elapsed_time, ElapsedTestTimeSource};
use module_core::test_helper::{stop_module, wait_for_event};
use module_core::{EventBus, Module};
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

    // Lapstart
    publish_position(&event_bus, &get_finishline_postion1());
    publish_position(&event_bus, &get_finishline_postion2());
    publish_position(&event_bus, &get_finishline_postion3());
    publish_position(&event_bus, &get_finishline_postion4());
    assert!(
        wait_for_event(
            &mut event_bus.subscribe(),
            std::time::Duration::from_millis(100),
            |e| {
                if let EventKind::LapStartedEvent = e.kind {
                    return true;
                }
                false
            },
        )
        .await,
    );

    // Sector1
    set_elapsed_time(
        &elapsed_time_source_sender,
        &std::time::Duration::from_millis(10120),
    );
    publish_position(&event_bus, &get_sector1_postion1());
    publish_position(&event_bus, &get_sector1_postion2());
    publish_position(&event_bus, &get_sector1_postion3());
    publish_position(&event_bus, &get_sector1_postion4());
    assert!(
        wait_for_event(
            &mut event_bus.subscribe(),
            std::time::Duration::from_millis(100),
            |e| {
                if let EventKind::SectorFinshedEvent(duration) = e.kind && duration == std::time::Duration::new(10, 120000000) {
                    return true;
                }
                false
            },
        )
        .await,
    );

    //Sector2
    set_elapsed_time(
        &elapsed_time_source_sender,
        &std::time::Duration::from_millis(20250),
    );
    publish_position(&event_bus, &get_sector2_postion1());
    publish_position(&event_bus, &get_sector2_postion2());
    publish_position(&event_bus, &get_sector2_postion3());
    publish_position(&event_bus, &get_sector2_postion4());
    assert!(
        wait_for_event(
            &mut event_bus.subscribe(),
            std::time::Duration::from_millis(100),
            |e| {
                if let EventKind::SectorFinshedEvent(duration) = e.kind && duration == std::time::Duration::new(10, 130000000) {
                    return true;
                }
                false
            },
        )
        .await,
    );

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
        |e| {
            if let EventKind::SectorFinshedEvent(duration) = e.kind && duration == std::time::Duration::new(10, 140000000) {
                return true;
            }
            false
        },
    );
    let mut receiver = event_bus.subscribe();
    let lap_finished_event = wait_for_event(
        &mut receiver,
        std::time::Duration::from_millis(100),
        |e| {
            if let EventKind::LapFinishedEvent(duration) = e.kind && duration == std::time::Duration::new(30, 390000000) {
                return true;
            }
            false
        },
    );
    let mut receiver = event_bus.subscribe();
    let lap_started_event = wait_for_event(
        &mut receiver,
        std::time::Duration::from_millis(100),
        |e| {
            if let EventKind::LapStartedEvent = e.kind  {
                return true;
            }
            false
        },
    );
    let (first, second,third) = tokio::join!(sector_finished_event, lap_finished_event, lap_started_event);
    assert!(first);
    assert!(second);
    assert!(third);

    stop_module(&event_bus, &mut laptimer_handle).await;
}
