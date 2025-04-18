use std::sync::Arc;

use crate::constant_source::ConstantGnssPositionSource;
use crate::{GnssPosition, GnssPositionSource, Position};
use chrono::DateTime;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

const TIMEOUT_MS: u16 = 100;
const VELOCITY: f32 = 2.77778;

#[tokio::test]
async fn report_creation_error_with_empty_positions() {
    let constant_source = ConstantGnssPositionSource::new(&[], VELOCITY).await;
    assert!(constant_source.is_err());
}

#[tokio::test]
async fn interpolate_between_two_points() {
    let positions = vec![
        Position::new(&52.026649, &11.282535),
        Position::new(&52.026751, &11.282047),
        Position::new(&52.026807, &11.281746),
    ];
    let expected_pos = GnssPosition::new(
        52.026648994186836,
        11.282535438555783,
        VELOCITY.into(),
        &DateTime::default(),
    );
    let constant_source = ConstantGnssPositionSource::new(&positions, VELOCITY)
        .await
        .expect("Failed to create constant source");
    let (sender, mut receiver) = mpsc::channel::<Arc<GnssPosition>>(1);
    constant_source.lock().await.register_pos_consumer(sender);

    let position = timeout(Duration::from_millis(TIMEOUT_MS.into()), receiver.recv())
        .await
        .expect("No position received in timeout")
        .unwrap();
    assert_eq!(position.latitude(), expected_pos.latitude());
    assert_eq!(position.longitude(), expected_pos.longitude());
    assert_eq!(position.velocity(), expected_pos.velocity());

    let expected_pos = GnssPosition::new(
        52.026649795432455,
        11.282531605189291,
        VELOCITY.into(),
        &DateTime::default(),
    );
    let position = timeout(Duration::from_millis(TIMEOUT_MS.into()), receiver.recv())
        .await
        .expect("No position received in timeout")
        .unwrap();
    assert_eq!(position.latitude(), expected_pos.latitude());
    assert_eq!(position.longitude(), expected_pos.longitude());
    assert_eq!(position.velocity(), expected_pos.velocity());
}
