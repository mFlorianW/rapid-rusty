//! Common Modul for the laptimer
//!
//! Provides the common data types that are used across every modul.

use chrono::{Duration, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

/// Represents a geographical coordinate with latitude and longitude.
///
/// The `Position` struct is commonly used to store a point on Earth
/// in decimal degrees. Latitude values range from -90.0 to 90.0, and
/// longitude values range from -180.0 to 180.0.
///
/// This struct derives common traits for debugging, cloning, comparison,
/// and (de)serialization with Serde.
///
/// # Fields
///
/// - `latitude` – The latitude in decimal degrees (positive for north, negative for south).
/// - `longitude` – The longitude in decimal degrees (positive for east, negative for west).
///
/// # Example
///
/// ```rust
/// use common::Position;
///
/// let pos = Position {
///     latitude: 52.5200,
///     longitude: 13.4050,
/// };
///
/// println!("{:?}", pos);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
}

impl Position {
    /// Creates a new [`Position`] with the given latitude and longitude.
    ///
    /// # Arguments
    ///
    /// * `latitude` - A reference to a floating-point number representing the latitude in decimal degrees.
    /// * `longitude` - A reference to a floating-point number representing the longitude in decimal degrees.
    ///
    /// # Returns
    ///
    /// A new `Position` instance with the specified coordinates.
    ///
    /// # Example
    ///
    /// ```rust
    /// use common::Position;
    ///
    /// let lat = 52.5200;
    /// let lon = 13.4050;
    /// let pos = Position::new(&lat, &lon);
    /// ```
    pub fn new(latitude: &f64, longitude: &f64) -> Self {
        Position {
            latitude: *latitude,
            longitude: *longitude,
        }
    }
}

/// Represents a GNSS (Global Navigation Satellite System) position reading.
///
/// This structure stores the latitude, longitude, velocity, and timestamp
/// of a GNSS fix using UTC time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GnssPosition {
    latitude: f64,
    longitude: f64,
    velocity: f64,
    time: chrono::DateTime<chrono::Utc>,
}

impl GnssPosition {
    /// Creates a new [`GnssPosition`] with the specified latitude, longitude, velocity, and time.
    ///
    /// # Arguments
    ///
    /// * `latitude` – Latitude in decimal degrees. Positive for northern hemisphere.
    /// * `longitude` – Longitude in decimal degrees. Positive for eastern hemisphere.
    /// * `velocity` – Speed in meters per second (or another consistent unit).
    /// * `time` – Timestamp of the GNSS fix in UTC.
    ///
    /// # Returns
    ///
    /// A new `GnssPosition` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use common::GnssPosition;
    /// use chrono::DateTime;
    ///
    /// let time = chrono::Utc::now();
    /// let pos = GnssPosition::new(52.0, 13.0, 15.5, &time);
    /// ```
    pub fn new(
        latitude: f64,
        longitude: f64,
        velocity: f64,
        time: &chrono::DateTime<chrono::Utc>,
    ) -> GnssPosition {
        GnssPosition {
            latitude,
            longitude,
            velocity,
            time: *time,
        }
    }

    /// Returns the latitude in decimal degrees.
    ///
    /// # Returns
    ///
    /// `f64` – The latitude of the position.
    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    /// Returns the longitude in decimal degrees.
    ///
    /// # Returns
    ///
    /// `f64` – The longitude of the position.
    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    /// Returns the velocity at this GNSS position.
    ///
    /// # Returns
    ///
    /// `f64` – The velocity in meters per second.
    pub fn velocity(&self) -> f64 {
        self.velocity
    }
}

/// Represents a race track with optional finish line and defined sectors.
///
/// A track consists of a name, a starting line position, an optional
/// finish line (useful for open-ended tracks or circuits), and a list
/// of sector positions (e.g., split markers).
///
/// # Fields
///
/// - `name` – The name of the track (e.g., "Oschersleben").
/// - `start_line` – The GPS position marking the start of the track.
/// - `finish_line` – An optional GPS position for the finish line.
/// - `sectors` – A list of GPS positions marking split points or checkpoints.
///
/// # Example
///
/// ```rust
/// use common::{Track, Position};
///
/// let track = Track {
///     name: "Example Track".into(),
///     start_line: Position { latitude: 52.0, longitude: 13.0 },
///     finish_line: None,
///     sectors: vec![
///         Position { latitude: 52.01, longitude: 13.01 },
///         Position { latitude: 52.02, longitude: 13.02 },
///     ],
/// };
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub start_line: Position,
    pub finish_line: Option<Position>,
    pub sectors: Vec<Position>,
}

/// Represents a single completed lap, including timing and telemetry data.
///
/// A `Lap` contains:
/// - Sector durations: split times that make up the lap.
/// - Log points: raw data points captured throughout the lap (GPS, time, velocity).
///
/// This struct is used to represent and analyze individual laps on a race track.
///
/// # Fields
///
/// - `sectors` – A list of `chrono::Duration` values representing split times.
/// - `log_points` – A list of telemetry data points (`GnssPosition`) collected during the lap.
///
/// # Example
///
/// ```rust
/// use common::{Lap, GnssPosition};
/// use chrono::Duration;
///
/// let lap = Lap {
///     sectors: vec![
///         Duration::seconds(25),
///         Duration::seconds(24),
///     ],
///     log_points: vec![/* LogPoint instances */],
/// };
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Lap {
    pub sectors: Vec<Duration>,
    pub log_points: Vec<GnssPosition>,
}

impl Lap {
    /// Calculates the total lap time by summing all sector durations.
    ///
    /// This method consumes the `Lap` instance (`self`) and iterates over its `sectors`
    /// to compute the total lap time as a single `chrono::Duration`.
    ///
    /// # Returns
    ///
    /// A [`chrono::Duration`] representing the sum of all sector durations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chrono::Duration;
    /// use common::{Lap, GnssPosition};
    ///
    /// let lap = Lap {
    ///     sectors: vec![Duration::seconds(30), Duration::seconds(32)],
    ///     log_points: vec![]
    /// };
    ///
    /// let total = lap.laptime();
    /// assert_eq!(total, Duration::seconds(62));
    /// ```
    ///
    /// # Panics
    ///
    /// This version assumes that the use of `.unwrap()` on `Duration::new` is valid.
    /// If you're using `std::time::Duration::new`, it doesn't return `Result`, so
    /// you likely meant `chrono::Duration::zero()` instead.
    pub fn laptime(self) -> Duration {
        let mut laptime = Duration::zero();
        for sector in self.sectors {
            laptime += sector;
        }
        laptime
    }
}

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
/// use common::{Track, Session, Position};
/// use chrono::{NaiveTime, NaiveDate};
///
/// let session = Session {
///     id: 1,
///     date: NaiveDate::from_ymd_opt(2024, 7, 15).unwrap(),
///     time: NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
///     track: Track {
///         name: "Sample Track".into(),
///         start_line: Position { latitude: 52.0, longitude: 13.0 },
///         finish_line: None,
///         sectors: vec![
///             Position { latitude: 52.01, longitude: 13.01 },
///             Position { latitude: 52.02, longitude: 13.02 },
///         ],
///     },
///     laps: vec![], // Add laps here
/// };
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: u64,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub track: Track,
    pub laps: Vec<Lap>,
}

#[cfg(test)]
mod tests;
