use crate::*;
use chrono::Duration;
use common::test_helper::elapsed_test_time_source::{set_elapsed_time, ElapsedTestTimeSource};
use std::sync::mpsc;
use tests::laptimer_positions::*;

#[tokio::test]
pub async fn drive_whole_map_with_sectors() {
    let elapsed_time_source = ElapsedTestTimeSource::default();
    let elapsed_time_source_sender = elapsed_time_source.sender();
    let mut laptimer = SimpleLaptimer::new_with_source(
        common::test_helper::track::get_track(),
        elapsed_time_source,
    );
    let (sender, receiver) = mpsc::channel::<Arc<Mutex<LaptimerStatus>>>();
    laptimer.register_status_consumer(sender);

    laptimer.update_position(&get_finishline_postion1());
    laptimer.update_position(&get_finishline_postion2());
    laptimer.update_position(&get_finishline_postion3());
    laptimer.update_position(&get_finishline_postion4());

    let mut status = receiver
        .recv_timeout(std::time::Duration::from_millis(100))
        .unwrap_or_else(|_| panic!("Failed to receive laptimer status within timeout 100ms"));
    assert_eq!(LaptimerStatus::LapStarted, *status.lock().unwrap());
    assert_eq!(Duration::zero(), laptimer.lap_time());

    set_elapsed_time(
        &elapsed_time_source_sender,
        &std::time::Duration::from_millis(10120),
    );
    laptimer.update_position(&get_sector1_postion1());
    laptimer.update_position(&get_sector1_postion2());
    laptimer.update_position(&get_sector1_postion3());
    laptimer.update_position(&get_sector1_postion4());

    status = receiver
        .recv_timeout(std::time::Duration::from_millis(100))
        .unwrap_or_else(|_| panic!("Failed to receive laptimer status within timeout 100ms"));
    assert_eq!(
        LaptimerStatus::SectorFinshed(Duration::new(10, 120000000).unwrap()),
        *status.lock().unwrap()
    );

    set_elapsed_time(
        &elapsed_time_source_sender,
        &std::time::Duration::from_millis(20250),
    );
    laptimer.update_position(&get_sector2_postion1());
    laptimer.update_position(&get_sector2_postion2());
    laptimer.update_position(&get_sector2_postion3());
    laptimer.update_position(&get_sector2_postion4());

    status = receiver
        .recv_timeout(std::time::Duration::from_millis(100))
        .unwrap_or_else(|_| panic!("Failed to receive laptimer status within timeout 100ms"));
    assert_eq!(
        LaptimerStatus::SectorFinshed(Duration::new(10, 130000000).unwrap()),
        *status.lock().unwrap()
    );

    set_elapsed_time(
        &elapsed_time_source_sender,
        &std::time::Duration::from_millis(30390),
    );
    laptimer.update_position(&get_finishline_postion1());
    laptimer.update_position(&get_finishline_postion2());
    laptimer.update_position(&get_finishline_postion3());
    laptimer.update_position(&get_finishline_postion4());

    status = receiver
        .recv_timeout(std::time::Duration::from_millis(100))
        .unwrap_or_else(|_| panic!("Failed to receive laptimer status within timeout 100ms"));
    assert_eq!(
        LaptimerStatus::SectorFinshed(Duration::new(10, 140000000).unwrap()),
        *status.lock().unwrap()
    );
    status = receiver
        .recv_timeout(std::time::Duration::from_millis(100))
        .unwrap_or_else(|_| panic!("Failed to receive laptimer status within timeout 100ms"));
    assert_eq!(
        LaptimerStatus::LapFinished(Duration::new(30, 390000000).unwrap()),
        *status.lock().unwrap()
    );
    status = receiver
        .recv_timeout(std::time::Duration::from_millis(100))
        .unwrap_or_else(|_| panic!("Failed to receive laptimer status within timeout 100ms"));
    assert_eq!(LaptimerStatus::LapStarted, *status.lock().unwrap());
}
