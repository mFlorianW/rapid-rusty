// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use chrono::{NaiveDate, NaiveTime};
use common::position::GnssPosition;

fn get_gnss_position_as_json<'a>() -> &'a str {
    r#"
    {
        "latitude": 52.025833,
        "longitude": 11.279166,
        "velocity": 10,
        "time": "00:00:00.000",
        "date": "01.01.1970"
    }
    "#
}

fn get_gnss_position() -> GnssPosition {
    GnssPosition::new(
        52.025833,
        11.279166,
        10.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

#[test]
pub fn dserialize_gnss_position_from_json() {
    let pos = GnssPosition::from_json(get_gnss_position_as_json())
        .unwrap_or_else(|e| panic!("Failed to deserialize the raw json. Reason: {e}"));
    assert_eq!(pos, get_gnss_position());
}
