use std::str::FromStr;

use crate::{lap::Lap, position::GnssPosition};
use chrono::{Duration, NaiveDate, NaiveTime};

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

fn get_lap_as_json<'a>() -> &'a str {
    r#"
    {
        "sectors": [
            "00:00:25.000",
            "00:00:25.000",
            "00:00:25.000"
        ],
        "log_points": [
            {
                "latitude": 52.0,
                "longitude": 11.0,
                "velocity": 100.0,
                "time": "00:00:00.000",
                "date": "01.01.1970"
            },
            {
                "latitude": 52.0,
                "longitude": 11.0,
                "velocity": 100.0,
                "time": "00:00:00.000",
                "date": "01.01.1970"
            }
        ]
    }
    "#
}

fn get_lap() -> Lap {
    let time = Duration::new(25, 0).unwrap();
    let log_point = GnssPosition::new(
        52.0,
        11.0,
        100.0,
        &NaiveTime::from_str("00:00:00.000").unwrap(),
        &NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
    );
    Lap {
        sectors: vec![time, time, time],
        log_points: vec![log_point, log_point],
    }
}

#[test]
fn derserialize_lap_from_json() {
    let lap = Lap::from_json(get_lap_as_json())
        .unwrap_or_else(|e| panic!("Failed to deserialize the raw json. Reason: {e}"));
    assert_eq!(lap, get_lap());
}
