// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use crate::{position::GnssPosition, serde::duration_list};
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
/// use std::time::Duration;
///
/// let lap = Lap {
///     sectors: vec![
///         Duration::from_secs(25),
///         Duration::from_secs(24),
///     ],
///     log_points: vec![/* LogPoint instances */],
/// };
/// ```
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Lap {
    #[serde(with = "duration_list")]
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
    /// use std::time::Duration;
    /// use common::{lap::Lap, position::GnssPosition};
    ///
    /// let lap = Lap {
    ///     sectors: vec![Duration::from_secs(30), Duration::from_secs(32)],
    ///     log_points: vec![]
    /// };
    ///
    /// let total = lap.laptime();
    /// assert_eq!(total, Duration::from_secs(62));
    /// ```
    ///
    /// # Panics
    ///
    /// This version assumes that the use of `.unwrap()` on `Duration::new` is valid.
    /// If you're using `std::time::Duration::new`, it doesn't return `Result`, so
    /// you likely meant `chrono::Duration::zero()` instead.
    pub fn laptime(self) -> Duration {
        let mut laptime = Duration::default();
        for sector in self.sectors {
            laptime += sector;
        }
        laptime
    }

    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}
