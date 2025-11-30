// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use crate::*;
use chrono::{NaiveDate, NaiveTime};

pub fn get_finishline_postion1() -> common::position::GnssPosition {
    GnssPosition::new(
        52.0270444,
        11.2805431,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

pub fn get_finishline_postion2() -> common::position::GnssPosition {
    GnssPosition::new(
        52.0270730,
        11.2804234,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

pub fn get_finishline_postion3() -> common::position::GnssPosition {
    GnssPosition::new(
        52.0271084,
        11.2802563,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

pub fn get_finishline_postion4() -> common::position::GnssPosition {
    GnssPosition::new(
        52.0271438,
        11.2800835,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

pub fn get_sector1_postion1() -> common::position::GnssPosition {
    GnssPosition::new(
        52.029819,
        11.274203,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

pub fn get_sector1_postion2() -> common::position::GnssPosition {
    GnssPosition::new(
        52.029821,
        11.274193,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

pub fn get_sector1_postion3() -> common::position::GnssPosition {
    GnssPosition::new(
        52.029821,
        11.274169,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

pub fn get_sector1_postion4() -> common::position::GnssPosition {
    GnssPosition::new(
        52.029822,
        11.274149,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

pub fn get_sector2_postion1() -> common::position::GnssPosition {
    GnssPosition::new(
        52.029970,
        11.277183,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

pub fn get_sector2_postion2() -> common::position::GnssPosition {
    GnssPosition::new(
        52.029968,
        11.277193,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

pub fn get_sector2_postion3() -> common::position::GnssPosition {
    GnssPosition::new(
        52.029967,
        11.277212,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}

pub fn get_sector2_postion4() -> common::position::GnssPosition {
    GnssPosition::new(
        52.029966,
        11.277218,
        0.0,
        &NaiveTime::parse_from_str("00:00:00.000", "%H:%M:%S%.3f").unwrap(),
        &NaiveDate::parse_from_str("01.01.1970", "%d.%m.%Y").unwrap(),
    )
}
