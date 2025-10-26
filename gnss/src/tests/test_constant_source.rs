use crate::constant_source::ConstantGnssModule;
use chrono::{DateTime, Utc};
use common::position::{GnssInformation, GnssPosition, Position};
use module_core::{
    test_helper::{stop_module, wait_for_event},
    Event, EventBus, EventKind, Module, ModuleCtx,
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
    let expected_pos = GnssPosition::new(
        52.026648994186836,
        11.282535438555783,
        VELOCITY,
        &DateTime::<Utc>::default().time(),
        &DateTime::<Utc>::default().date_naive(),
    );
    let event_bus = EventBus::default();
    let mut receiver = event_bus.subscribe();
    let mut module_handle = start_module(event_bus.context());
    assert!(
        wait_for_event(
            &mut receiver,
            std::time::Duration::from_millis(TIMEOUT_MS.into()),
            |e: &Event| -> bool {
                if let EventKind::GnssPositionEvent(ref position) = e.kind
                    && gnss_pos_validator(position, &expected_pos)
                {
                    return true;
                }
                false
            }
        )
        .await
    );
    stop_module(&event_bus, &mut module_handle).await;
}

#[tokio::test]
async fn notify_gnss_information() {
    let expected_info = GnssInformation::new(&common::position::GnssStatus::Fix3d, 8);
    let event_bus = EventBus::default();
    let mut module_handle = start_module(event_bus.context());
    assert!(
        wait_for_event(
            &mut event_bus.subscribe(),
            std::time::Duration::from_millis(TIMEOUT_MS.into()),
            |e: &Event| -> bool {
                if let EventKind::GnssInformationEvent(ref info) = e.kind
                    && **info == expected_info
                {
                    return true;
                }
                false
            }
        )
        .await
    );
    stop_module(&event_bus, &mut module_handle).await;
}
