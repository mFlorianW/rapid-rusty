use crate::serde::{date, time};
use chrono::{NaiveDate, NaiveTime};
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
/// use common::position::Position;
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
    /// use common::position::Position;
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

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}

/// Represents a GNSS (Global Navigation Satellite System) position reading.
///
/// This structure stores the latitude, longitude, velocity, and timestamp
/// of a GNSS fix using UTC time.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GnssPosition {
    latitude: f64,
    longitude: f64,
    velocity: f64,
    #[serde(with = "time")]
    time: NaiveTime,
    #[serde(with = "date")]
    date: NaiveDate,
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
    /// use common::position::GnssPosition;
    /// use chrono::{DateTime, NaiveDate, NaiveTime};
    ///
    /// let time = chrono::Utc::now();
    /// let pos = GnssPosition::new(52.0, 13.0, 15.5, &time.time(), &time.date_naive() );
    /// ```
    pub fn new(
        latitude: f64,
        longitude: f64,
        velocity: f64,
        time: &NaiveTime,
        date: &NaiveDate,
    ) -> GnssPosition {
        GnssPosition {
            latitude,
            longitude,
            velocity,
            time: *time,
            date: *date,
        }
    }

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
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
