use std::str::FromStr;

use crate::{
    lap::Lap,
    position::{GnssPosition, Position},
    session::Session,
    track::Track,
};
use chrono::{Duration, NaiveDate, NaiveTime};

fn get_session_as_json<'a>() -> &'a str {
    r#"
    {
    "id": 10,
    "date": "01.01.1970",
    "time": "13:00:00.000",
    "track": {
        "name": "Oschersleben",
        "startline": {
            "latitude": 52.025833,
            "longitude": 11.279166
        },
        "finishline": {
            "latitude": 52.025833,
            "longitude": 11.279166
        },
        "sectors": [
            {
                "latitude": 52.025833,
                "longitude": 11.279166
            },
            {
                "latitude": 52.025833,
                "longitude": 11.279166
            }
        ]
    },
    "laps": [
        {
            "sectors": [
                "00:00:25.144",
                "00:00:25.144",
                "00:00:25.144",
                "00:00:25.144"
            ],
            "log_points": [
                {
                    "velocity": 100.0,
                    "longitude": 11.0,
                    "latitude": 52.0,
                    "time": "00:00:00.000",
                    "date": "01.01.1970"
                },
                {
                    "velocity": 100.0,
                    "longitude": 11.0,
                    "latitude": 52.0,
                    "time": "00:00:00.000",
                    "date": "01.01.1970"
                }
            ]
        }
    ]
    }
    "#
}

fn get_session() -> Session {
    let time = Duration::new(25, 0).unwrap();
    let log_point = GnssPosition::new(
        52.0,
        11.0,
        100.0,
        &NaiveTime::from_str("00:00:00.000").unwrap(),
        &NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
    );
    Session {
        id: 0,
        date: NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
        time: NaiveTime::parse_from_str("13:00:00.000", "%H:%M:%S%.3f").unwrap(),
        track: Track {
            name: "Oschersleben".to_string(),
            startline: Position {
                latitude: 52.025833,
                longitude: 11.279166,
            },
            finishline: Some(Position {
                latitude: 52.025833,
                longitude: 11.279166,
            }),
            sectors: vec![
                Position {
                    latitude: 52.025833,
                    longitude: 11.279166,
                },
                Position {
                    latitude: 52.025833,
                    longitude: 11.279166,
                },
            ],
        },
        laps: vec![Lap {
            sectors: vec![time, time, time, time],
            log_points: vec![log_point, log_point],
        }],
    }
}

#[test]
pub fn deserialize_session_from_json() {
    let session = Session::from_json(get_session_as_json())
        .unwrap_or_else(|e| panic!("Failed to deserialize the raw json. Reason: {e}"));
    assert_eq!(session, get_session());
}
