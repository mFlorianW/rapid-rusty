use crate::{
    GnssInformation, GnssInformationSource, GnssPosition, GnssPositionSource, GnssStatus, Position,
};
use chrono::Utc;
use std::{
    io::{Error, ErrorKind},
    sync::{Arc, Weak},
    time,
};
use tokio::{
    sync::{mpsc::Sender, Mutex},
    task,
};
use utm::{self, lat_lon_to_zone_number, lat_to_zone_letter, to_utm_wgs84, wsg84_utm_to_lat_lon};

/// A GNSS source that reports GNSS positions in a constant frequency
pub struct ConstantGnssPositionSource {
    pos_consumer: Vec<Sender<Arc<GnssPosition>>>,
    points: Vec<UtmPoint>,
    next_position: usize,
    current_position: UtmPoint,
    velocity: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UtmPoint {
    x: f64,
    y: f64,
    zone: u8,
    zone_letter: char,
}

impl GnssPositionSource for ConstantGnssPositionSource {
    /// Register a consumer for the provided positions
    ///
    /// # Arguments
    ///
    /// - `consumer` The consumer that is notified when a new position is available
    ///
    fn register_pos_consumer(&mut self, consumer: Sender<Arc<GnssPosition>>) {
        self.pos_consumer.push(consumer);
    }
}

impl ConstantGnssPositionSource {
    /// Creates a new ConstantGnssPositionSource
    ///
    /// # Arguments
    ///
    /// * `positions` - Initial positions that is used to calculate the positions of the source
    /// * `velocity` - The velocity that is reported for each positions and it's always the same for every position.
    ///
    /// # Returns
    ///
    /// * `Ok(Arc<Mutex<<ConstantGnssPositionSource>>)` - The new created ConstantGnssPositionSource
    /// * `Err(io::Error)` - If failes to create the ConstantGnssPositionSource
    ///
    pub async fn new(
        positions: &[Position],
        velocity: f32,
    ) -> Result<Arc<Mutex<ConstantGnssPositionSource>>, Error> {
        if positions.is_empty() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Positions parameter is empty",
            ));
        }
        let source = Arc::new(Mutex::new(ConstantGnssPositionSource {
            pos_consumer: Vec::new(),
            points: Vec::new(),
            next_position: 0,
            current_position: UtmPoint::default(),
            velocity,
        }));
        source.lock().await.convert_track_points(positions)?;
        let source_weak = Arc::downgrade(&source);
        tokio::spawn(async move {
            constant_gnss_source_task(source_weak).await;
        });
        Ok(source)
    }

    fn convert_track_points(&mut self, positions: &[Position]) -> Result<(), Error> {
        for pos in positions.iter() {
            let zone = lat_lon_to_zone_number(pos.latitude, pos.longitude);
            let Some(zone_letter) = lat_to_zone_letter(pos.latitude) else {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Position lat: {}, long{} can't converted to UTM zone",
                        pos.latitude, pos.longitude
                    ),
                ));
            };
            let (northing, easting, _) = to_utm_wgs84(pos.latitude, pos.longitude, zone);
            let point = UtmPoint {
                x: northing,
                y: easting,
                zone,
                zone_letter,
            };
            self.points.push(point);
        }
        self.current_position = self.points[0];
        Ok(())
    }

    async fn handle_tick(&mut self) {
        if self.next_position > 0 && self.next_position <= self.points.len() {
            println!("point: {:?}", self.current_position);
            let p0 = &self.points[self.next_position];
            let direction = UtmPoint {
                x: p0.x - self.current_position.x,
                y: p0.y - self.current_position.y,
                zone: 0,
                zone_letter: '\0',
            };
            let length = (direction.x * direction.x + direction.y * direction.y).sqrt();
            let normalized_direction = UtmPoint {
                x: direction.x / length,
                y: direction.y / length,
                zone: 0,
                zone_letter: '\0',
            };
            let time = f64::from(ConstantGnssPositionSource::POSITION_INTERVAL_MS) / 1000.0;
            let distance_traveled = UtmPoint {
                x: normalized_direction.x * f64::from(self.velocity) * time,
                y: normalized_direction.y * f64::from(self.velocity) * time,
                zone: 0,
                zone_letter: '\0',
            };
            self.current_position.x += distance_traveled.x;
            self.current_position.y += distance_traveled.y;
            let new_length = (self.current_position.x * self.current_position.x
                + self.current_position.y * self.current_position.y)
                .sqrt();
            if new_length > length {
                self.next_position += 2;
                if self.next_position >= self.points.len() {
                    self.next_position = 0;
                }
            }
        } else if self.next_position == 0 {
            self.next_position += 1;
        } else {
            self.next_position = 0;
        }

        println!("point: {:?}", self.current_position);

        let Ok((lat, long)) = wsg84_utm_to_lat_lon(
            self.current_position.y,
            self.current_position.x,
            self.current_position.zone,
            self.current_position.zone_letter,
        ) else {
            return;
        };

        let gnss_pos = Arc::new(GnssPosition::new(
            lat,
            long,
            self.velocity.into(),
            &Utc::now(),
        ));

        for consumer in self.pos_consumer.iter() {
            consumer.send(gnss_pos.clone()).await.unwrap();
        }
    }

    const POSITION_INTERVAL_MS: u8 = 100;
}

async fn constant_gnss_source_task(
    gnss_source: std::sync::Weak<Mutex<ConstantGnssPositionSource>>,
) {
    let mut timer = tokio::time::interval(time::Duration::from_millis(
        ConstantGnssPositionSource::POSITION_INTERVAL_MS.into(),
    ));
    while let Some(source) = Weak::upgrade(&gnss_source) {
        timer.tick().await;
        source.lock().await.handle_tick().await;
    }
}

/// A GNSS information source that provides GNSS information only once on subscriptions for updates
pub struct ConstantGnssInformationSource {
    status: GnssStatus,
    satellites: usize,
}

impl ConstantGnssInformationSource {
    /// Creates a new ConstantGnssPositionSource
    ///
    /// # Arguments
    ///
    /// * `status` - The status that shall be constantly reported by the source
    /// * `satellites` - The amount of the satellites that shall be constantly by the source
    ///
    /// # Returns
    ///
    /// * `ConstantGnssInformationSource` - The created ConstantGnssInformationSource
    ///
    pub fn new(status: GnssStatus, satellites: usize) -> Arc<Mutex<ConstantGnssInformationSource>> {
        Arc::new(Mutex::new(ConstantGnssInformationSource {
            status,
            satellites,
        }))
    }
}

impl GnssInformationSource for ConstantGnssInformationSource {
    /// Register a consumer for the provided GNSS informations
    /// Important each registered consumer is only notified only once on registations because this
    /// a constant information source.
    ///
    /// # Arguments
    ///
    /// - `consumer` The consumer that is notified once on registration
    ///
    fn register_info_consumer(&mut self, consumer: Sender<std::sync::Arc<GnssInformation>>) {
        let status = self.status;
        let satellites = self.satellites;
        task::spawn(async move {
            let info = Arc::new(GnssInformation::new(&status, satellites));
            consumer.send(info).await.unwrap();
        });
    }
}
