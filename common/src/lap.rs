use crate::position::GnssPosition;
use chrono::{Duration, NaiveTime, Timelike};
use serde::de::Error;

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
/// use common::{lap::Lap, position::GnssPosition};
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
#[derive(Debug, PartialEq)]
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
    /// use common::{lap::Lap, position::GnssPosition};
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

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        let values: serde_json::Value = serde_json::from_str(json)?;
        let sectors = Lap::extrac_sectors(&values)?;
        let log_points = Lap::extrac_log_points(&values)?;

        Ok(Lap {
            sectors,
            log_points,
        })
    }

    fn extrac_sectors(values: &serde_json::Value) -> serde_json::Result<Vec<Duration>> {
        match values.get("sectors") {
            Some(sectors) => match sectors.as_array() {
                Some(sectors) => {
                    let mut durations = vec![];
                    for sector in sectors {
                        let Some(time) = sector.as_str() else {
                            continue;
                        };
                        let Ok(time) = NaiveTime::parse_from_str(time, "%H:%M:%S%.3f") else {
                            continue;
                        };
                        let duration =
                            Duration::new(time.num_seconds_from_midnight() as i64, 0).unwrap();
                        durations.push(duration);
                    }
                    Ok(durations)
                }
                None => Err(Error::custom("The log point is not array object.")),
            },
            None => Err(Error::missing_field(
                "The required field sectors is missing",
            )),
        }
    }

    fn extrac_log_points(values: &serde_json::Value) -> serde_json::Result<Vec<GnssPosition>> {
        match values.get("log_points") {
            Some(log_points) => match log_points.as_array() {
                Some(log_points) => {
                    let mut points: Vec<GnssPosition> = vec![];
                    for point in log_points {
                        let position = GnssPosition::from_json(&point.to_string())?;
                        points.push(position);
                    }
                    Ok(points)
                }
                None => Err(Error::custom("The log point is not array object.")),
            },
            None => Err(Error::missing_field(
                "The required field log_points is missing",
            )),
        }
    }
}
