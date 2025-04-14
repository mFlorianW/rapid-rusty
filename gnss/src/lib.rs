//! GNSS Modul for the laptimer
//!
//! Provides the interfaces and implementation to access GNSS on linux based systems.

use tokio::sync::mpsc::Sender;

/// Common interface that every GNSS position source must support
pub trait GnssPositionSource {
    /// Registers a position consumer in the GNSS position source
    ///
    /// All new positions upateds are notified through the channel to the consumer.
    /// ´consumer´ - The conusumer that is notified on changes
    fn register_pos_consumer(&mut self, consumer: Sender<std::sync::Arc<Position>>);
}

/// Position values that are notified by a GNNSS source
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    latitude: f64,
    longitude: f64,
    velocity: f64,
    time: chrono::DateTime<chrono::Utc>,
}

impl Position {
    pub fn new(
        latitude: f64,
        longitude: f64,
        velocity: f64,
        time: &chrono::DateTime<chrono::Utc>,
    ) -> Position {
        Position {
            latitude,
            longitude,
            velocity,
            time: *time,
        }
    }

    // Simple getter for the latitude value of a position
    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    // Getter for the longitude value of a position
    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    // Getter for the velocitiy at this position
    pub fn velocity(&self) -> f64 {
        self.velocity
    }
}

//
pub trait GnssInformationSource {
    /// Registers a GNSS information consumer in the GNSS information source
    ///
    /// All new informations upateds are notified through the channel to the consumer.
    ///
    /// ´consumer´ - The consumer that is notfied on changes
    fn register_info_consumer(&mut self, consumer: Sender<std::sync::Arc<GnssInformation>>);
}

#[derive(Clone, Copy, Debug, PartialEq)]
// The GNSS status from a GNSS source
pub enum GnssStatus {
    // The Status of the GNSS is unknow
    Unknown,
    // The GNSS system has no fix all reported positions are maybe wrong
    NoFix,
    // The GNSS system is in the 2d fix mode only latitude and longitude are valid
    Fix2d,
    // The GNSS system is in the 3d Fix mode latitue, longitude and alitude(currently not reported) are valid
    Fix3d,
}

#[derive(Clone, Debug, PartialEq)]
// Information of the GNSS.
// The information contains the status of the receiver and the amount of satellites that are used
// for the position, time and velocitiy informations.
pub struct GnssInformation {
    status: GnssStatus,
    satellites: usize,
}

impl GnssInformation {
    pub fn new(status: &GnssStatus, satellites: usize) -> GnssInformation {
        GnssInformation {
            status: *status,
            satellites,
        }
    }
}

pub mod gpsd_source;

#[cfg(test)]
mod tests;
