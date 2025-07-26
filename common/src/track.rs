use crate::position::Position;
use serde::{Deserialize, Serialize};

/// Represents a race track with optional finish line and defined sectors.
///
/// A track consists of a name, a starting line position, an optional
/// finish line (useful for open-ended tracks or circuits), and a list
/// of sector positions (e.g., split markers).
///
/// # Fields
///
/// - `name` – The name of the track (e.g., "Oschersleben").
/// - `startline` – The GPS position marking the start of the track.
/// - `finishline` – An optional GPS position for the finish line.
/// - `sectors` – A list of GPS positions marking split points or checkpoints.
///
/// # Example
///
/// ```rust
/// use common::{track::Track, position::Position};
///
/// let track = Track {
///     name: "Example Track".into(),
///     startline: Position { latitude: 52.0, longitude: 13.0 },
///     finishline: None,
///     sectors: vec![
///         Position { latitude: 52.01, longitude: 13.01 },
///         Position { latitude: 52.02, longitude: 13.02 },
///     ],
/// };
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub startline: Position,
    pub finishline: Option<Position>,
    pub sectors: Vec<Position>,
}

impl Track {
    /// Creates a `Track` instance by deserializing it from a JSON string.
    ///
    /// This method attempts to parse the given JSON string into a [`Track`] struct
    /// using [`serde_json`]. It returns a `Result` containing either the successfully
    /// parsed `Track` or a `serde_json::Error` if the input is invalid.
    ///
    /// # Arguments
    ///
    /// * `json` – A JSON-formatted string representing a `Track`.
    ///
    /// # Returns
    ///
    /// * `Ok(Track)` – If the JSON string was successfully parsed.
    /// * `Err(serde_json::Error)` – If parsing failed due to invalid format or type mismatch.
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}
