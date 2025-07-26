use crate::constant_source::{ConstantGnssInformationSource, ConstantGnssPositionSource};
use crate::{GnssInformation, GnssInformationSource, GnssPositionSource, GnssStatus};
use chrono::{DateTime, Utc};
use common::{position::GnssPosition, position::Position};
use std::sync::Arc;
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
        &DateTime::<Utc>::default().time(),
        &DateTime::<Utc>::default().date_naive(),
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
        &DateTime::<Utc>::default().time(),
        &DateTime::<Utc>::default().date_naive(),
    );
    let position = timeout(Duration::from_millis(TIMEOUT_MS.into()), receiver.recv())
        .await
        .expect("No position received in timeout")
        .unwrap();
    assert_eq!(position.latitude(), expected_pos.latitude());
    assert_eq!(position.longitude(), expected_pos.longitude());
    assert_eq!(position.velocity(), expected_pos.velocity());
}

#[tokio::test]
async fn notify_gnss_informations_on_registeration() {
    let info_source = ConstantGnssInformationSource::new(GnssStatus::Fix3d, 8);
    let (sender, mut receiver) = mpsc::channel::<Arc<GnssInformation>>(1);
    info_source.lock().await.register_info_consumer(sender);
    let gnss_info = timeout(Duration::from_millis(TIMEOUT_MS.into()), receiver.recv())
        .await
        .expect("No info received in timeout")
        .unwrap();
    static SATELLITES: usize = 8;
    assert_eq!(gnss_info.status, GnssStatus::Fix3d);
    assert_eq!(gnss_info.satellites, SATELLITES);
}
