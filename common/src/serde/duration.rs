use chrono::{Duration, NaiveTime, Timelike};
use serde::{self, Deserialize, Deserializer, Serializer};

const FORMAT: &str = "%H:%M:%S%.3f";

pub fn duration_to_string<S: Serializer>(duration: &Duration) -> Result<String, S::Error> {
    let total_seconds = duration.num_seconds();
    let nanos = (duration.num_nanoseconds().unwrap_or(0) % 1_000_000_000) as u32;

    let time = NaiveTime::from_num_seconds_from_midnight_opt(total_seconds as u32, nanos)
        .ok_or_else(|| serde::ser::Error::custom("Invalid duration for NaiveTime"))?;

    Ok(time.format(FORMAT).to_string())
}

#[allow(dead_code)]
pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let naive = duration_to_string::<S>(duration)?;
    serializer.serialize_str(&naive)
}

/// Deserialize a time string like "00:00:25.144" into a `chrono::Duration`.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let time = NaiveTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;

    let total_seconds =
        (time.hour() as i64 * 3600) + (time.minute() as i64 * 60) + time.second() as i64;
    let nanos = time.nanosecond() as i64;

    Ok(Duration::seconds(total_seconds) + Duration::nanoseconds(nanos))
}
