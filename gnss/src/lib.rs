//! GNSS Modul for the laptimer
//!
//! Provides the interfaces and implementation to access GNSS on linux based systems.

use tokio::sync::mpsc::Sender;

/// Common interface that every GNSS position source must support
pub trait GnssPositionSource {
    /// Registers a position consumer in the GNSS source
    ///
    /// All new positions upateds are notified through the channel to the consumer.
    fn register_consumer(&mut self, consumer: Sender<std::sync::Arc<Position>>);
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

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    pub fn velocity(&self) -> f64 {
        self.velocity
    }
}

pub mod gpsd;

#[cfg(test)]
mod tests;
