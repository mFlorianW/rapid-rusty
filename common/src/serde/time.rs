// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use chrono::NaiveTime;
use serde::{self, Deserialize, Deserializer, Serializer};

const FORMAT: &str = "%H:%M:%S%.3f"; // Custom format with milliseconds

pub fn serialize<S>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = time.format(FORMAT).to_string();
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
}
