use crate::{lap::Lap, serde::date, serde::time, track::Track};
use chrono::{NaiveDate, NaiveTime};
use serde::{de::Error, Deserialize, Serialize};

/// Represents a recorded driving session consisting of one or more laps.
///
/// A `Session` is a top-level structure used to store the result of a
/// track session, including metadata such as the date, time, and track
/// layout, as well as a list of completed laps.
///
/// # Fields
///
/// - `id` – A unique identifier for the session (can be used as a file ID or database key).
/// - `date` – The calendar date when the session took place.
/// - `time` – The time of day when the session started.
/// - `track` – The track configuration (`Track`) used during the session.
/// - `laps` – A list of completed laps (`Lap`) with sector times and telemetry.
///
/// # Example
///
/// ```rust
/// use common::{track::Track, session::Session, position::Position};
/// use chrono::{NaiveTime, NaiveDate};
///
/// let session = Session {
///     id: 1,
///     date: NaiveDate::from_ymd_opt(2024, 7, 15).unwrap(),
///     time: NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
///     track: Track {
///         name: "Sample Track".into(),
///         startline: Position { latitude: 52.0, longitude: 13.0 },
///         finishline: None,
///         sectors: vec![
///             Position { latitude: 52.01, longitude: 13.01 },
///             Position { latitude: 52.02, longitude: 13.02 },
///         ],
///     },
///     laps: vec![], // Add laps here
/// };
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: u64,
    #[serde(with = "date")]
    pub date: NaiveDate,
    #[serde(with = "time")]
    pub time: NaiveTime,
    pub track: Track,
    pub laps: Vec<Lap>,
}

impl Session {
    /// Deserializes a [`Session`] instance from a JSON string.
    ///
    /// This method parses the provided JSON string and attempts to construct a [`Session`]
    /// object using Serde. It returns a `Result` containing either the successfully
    /// deserialized `Session` or an error if the input is not valid JSON or does not
    /// match the expected structure.
    ///
    /// # Arguments
    ///
    /// * `json` – A string slice containing the JSON representation of a session.
    ///
    /// # Returns
    ///
    /// * `Ok(Session)` – If the JSON string is well-formed and matches the `Session` structure.
    /// * `Err(serde_json::Error)` – If the string is not valid JSON or fails to deserialize.
    /// ```    
    pub fn from_json(json: &str) -> serde_json::Result<Session> {
        serde_json::from_str(json)
    }

    pub fn to_json(session: &Session) -> serde_json::Result<String> {
        serde_json::to_string(session)
    }
}
