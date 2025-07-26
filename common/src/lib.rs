//! Common Modul for the laptimer
//!
//! Provides the common data types that are used across every modul.

pub mod lap;
pub mod position;
pub mod session;
pub mod track;

use chrono::{NaiveDate, NaiveTime};
use serde::de::Error;

/// Extracts a `NaiveDate` from a JSON object by parsing the `"date"` field.
///
/// This function retrieves the `"date"` field from the given `serde_json::Value`
/// and attempts to parse it using the format `"%d.%m.%Y"`.
///
/// # Arguments
///
/// * `values` – A reference to a `serde_json::Value` expected to contain a `"date"` string.
///
/// # Returns
///
/// * `Ok(NaiveDate)` if the field exists and can be parsed.
/// * `Err(serde_json::Error)` if the field is missing or the format is incorrect.
///
/// # Example JSON:
///
/// ```json
/// { "date": "15.07.2025" }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - the `"date"` field is missing,
/// - the field is not a string,
/// - or the string cannot be parsed as a `NaiveDate`.
fn extrac_date(values: &serde_json::Value) -> serde_json::Result<NaiveDate> {
    match values.get("date") {
        Some(value) => {
            let raw_date = value
                .as_str()
                .ok_or_else(|| Error::custom("Date is not a valid string"))?;
            NaiveDate::parse_from_str(raw_date, "%d.%m.%Y").map_err(Error::custom)
        }
        None => Err(Error::missing_field("Missing required element date")),
    }
}

/// Extracts a `NaiveTime` from a JSON object by parsing the `"time"` field.
///
/// This function retrieves the `"time"` field from the given `serde_json::Value`
/// and parses it using the format `"%H:%M:%S%.3f"` (e.g., `"13:00:00.000"`).
///
/// # Arguments
///
/// * `values` – A reference to a `serde_json::Value` expected to contain a `"time"` string.
///
/// # Returns
///
/// * `Ok(NaiveTime)` if the field exists and can be parsed.
/// * `Err(serde_json::Error)` if the field is missing or malformed.
///
/// # Example JSON:
///
/// ```json
/// { "time": "13:00:00.000" }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - the `"time"` field is missing,
/// - the field is not a string,
/// - or the string cannot be parsed as a `NaiveTime`.
fn extrac_time(values: &serde_json::Value) -> serde_json::Result<NaiveTime> {
    match values.get("time") {
        Some(time) => {
            let raw_time = time
                .as_str()
                .ok_or_else(|| Error::custom("Time is not a valid string"))?;
            NaiveTime::parse_from_str(raw_time, "%H:%M:%S%.3f").map_err(Error::custom)
        }
        None => Err(Error::missing_field("Missing required element time")),
    }
}

/// Extracts a `f64` value from a JSON object by key.
///
/// This utility function attempts to retrieve a key from a `serde_json::Value`
/// and extract it as a `f64` floating-point number.
///
/// # Arguments
///
/// * `values` – The JSON object to extract from.
/// * `key` – The key name whose value should be parsed as a `f64`.
///
/// # Returns
///
/// * `Ok(f64)` if the key exists and holds a valid floating-point number.
/// * `Err(serde_json::Error)` otherwise.
///
/// # Example JSON:
///
/// ```json
/// { "velocity": 100.0 }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - the key is missing,
/// - the value is not a number.
fn extrac_double_value(values: &serde_json::Value, key: &str) -> serde_json::Result<f64> {
    match values.get(key) {
        Some(key) => key
            .as_f64()
            .ok_or_else(|| Error::custom(format!("The element {key} is not a double value."))),
        None => Err(Error::custom(format!(
            "Missing required element {key} is missing."
        ))),
    }
}

#[cfg(test)]
mod tests;
