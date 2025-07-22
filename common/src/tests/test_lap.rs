use crate::Lap;
use chrono::Duration;

#[test]
fn calculate_laptime_with_one_sector() {
    let exp_time = Duration::new(90, 0).unwrap();
    let lap = Lap {
        sectors: vec![exp_time],
        log_points: vec![],
    };

    let laptime = lap.laptime();

    assert_eq!(laptime, exp_time);
}

#[test]
fn calculate_laptime_with_multiple_sectors() {
    let sec_time = Duration::new(90, 0).unwrap();
    let lap_time = Duration::new(270, 0).unwrap();
    let lap = Lap {
        sectors: vec![sec_time, sec_time, sec_time],
        log_points: vec![],
    };

    let laptime = lap.laptime();

    assert_eq!(laptime, lap_time);
}
