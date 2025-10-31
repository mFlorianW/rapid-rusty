use crate::constant_source::ConstantGnssModule;
use chrono::{DateTime, Utc};
use common::position::{GnssInformation, GnssPosition, Position};
use module_core::{
    Event, EventBus, EventKind, GnssPositionPtr, Module, ModuleCtx, payload_ref,
    test_helper::{stop_module, wait_for_event},
};

const TIMEOUT_MS: u16 = 100;
const VELOCITY: f64 = 2.77778;

fn gnss_pos_validator(lhs: &GnssPosition, rhs: &GnssPosition) -> bool {
    if lhs.velocity() == rhs.velocity()
        && lhs.longitude() == rhs.longitude()
        && lhs.latitude() == rhs.latitude()
    {
        return true;
    }
    false
}

fn start_module(ctx: ModuleCtx) -> tokio::task::JoinHandle<Result<(), ()>> {
    let positions = vec![
        Position::new(&52.026649, &11.282535),
        Position::new(&52.026751, &11.282047),
        Position::new(&52.026807, &11.281746),
    ];
    tokio::spawn(async move {
        let mut constant_source = ConstantGnssModule::new(
            ctx,
            &positions,
            VELOCITY,
            std::time::Duration::from_millis(20),
        )
        .unwrap();
        constant_source.run().await
    })
}

#[test]
fn report_creation_error_with_empty_positions() {
    let event_bus = EventBus::default();
    let constant_source = ConstantGnssModule::new(
        event_bus.context(),
        &[],
        VELOCITY,
        std::time::Duration::from_millis(0),
    );
    assert!(constant_source.is_err());
}

#[tokio::test]
async fn interpolate_between_two_points() {
    let event_bus = EventBus::default();
    let mut module_handle = start_module(event_bus.context());

    let exp_pos_event = Event {
        kind: EventKind::GnssPositionEvent(
            GnssPosition::new(
                52.026648994186836,
                11.282535438555783,
                VELOCITY,
                &DateTime::<Utc>::default().time(),
                &DateTime::<Utc>::default().date_naive(),
            )
            .into(),
        ),
    };
    let pos_event = wait_for_event(
        &mut event_bus.subscribe(),
        std::time::Duration::from_millis(TIMEOUT_MS.into()),
        exp_pos_event.kind_discriminant(),
    )
    .await;

    assert!(gnss_pos_validator(
        payload_ref!(pos_event.kind, EventKind::GnssPositionEvent).unwrap(),
        payload_ref!(exp_pos_event.kind, EventKind::GnssPositionEvent).unwrap()
    ));

    stop_module(&event_bus, &mut module_handle).await;
}

#[tokio::test]
async fn notify_gnss_information() {
    let event_bus = EventBus::default();
    let mut module_handle = start_module(event_bus.context());

    let exp_info_event = Event {
        kind: EventKind::GnssInformationEvent(
            GnssInformation::new(&common::position::GnssStatus::Fix3d, 8).into(),
        ),
    };
    let info_event = wait_for_event(
        &mut event_bus.subscribe(),
        std::time::Duration::from_millis(TIMEOUT_MS.into()),
        exp_info_event.kind_discriminant(),
    )
    .await;
    assert_eq!(
        payload_ref!(info_event.kind, EventKind::GnssInformationEvent).unwrap(),
        payload_ref!(exp_info_event.kind, EventKind::GnssInformationEvent).unwrap()
    );

    stop_module(&event_bus, &mut module_handle).await;
}
