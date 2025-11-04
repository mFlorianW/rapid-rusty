use crate::GnssPosition;
use chrono::Utc;
use common::position::{GnssInformation, GnssStatus, Position};
use module_core::{Event, EventKind, Module, ModuleCtx};
use std::{
    io::{Error, ErrorKind},
    sync::Arc,
    time,
};
use utm::{self, lat_lon_to_zone_number, lat_to_zone_letter, to_utm_wgs84, wsg84_utm_to_lat_lon};

/// A GNSS source that reports GNSS positions in a constant frequency
struct ConstantGnssPositionSourceRuntime {
    points: Vec<UtmPoint>,
    next_position: usize,
    current_position: UtmPoint,
    velocity: f64,
    sender: tokio::sync::broadcast::Sender<Event>,
}

#[derive(Debug, Clone, Copy, Default)]
struct UtmPoint {
    x: f64,
    y: f64,
    zone: u8,
    zone_letter: char,
}

impl ConstantGnssPositionSourceRuntime {
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
            let time = f64::from(ConstantGnssPositionSourceRuntime::POSITION_INTERVAL_MS) / 1000.0;
            let distance_traveled = UtmPoint {
                x: normalized_direction.x * self.velocity * time,
                y: normalized_direction.y * self.velocity * time,
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
            self.velocity,
            &Utc::now().time(),
            &Utc::now().date_naive(),
        ));
        let _ = self.sender.send(Event {
            kind: EventKind::GnssPositionEvent(gnss_pos.clone()),
        });
    }

    const POSITION_INTERVAL_MS: u8 = 100;
}

fn convert_track_points(positions: &[Position]) -> Result<Vec<UtmPoint>, Error> {
    let mut points = Vec::<UtmPoint>::new();
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
        points.push(point);
    }
    Ok(points)
}

#[derive(Clone)]
struct ConstantGnssModuleConfig {
    positions: Vec<UtmPoint>,
    velocity: f64,
    information_interval: std::time::Duration,
}

pub struct ConstantGnssModule {
    ctx: ModuleCtx,
    config: Arc<ConstantGnssModuleConfig>,
}

impl ConstantGnssModule {
    pub fn new(
        ctx: ModuleCtx,
        positions: &[Position],
        velocity: f64,
        information_interval: std::time::Duration,
    ) -> Result<Self, Error> {
        if positions.is_empty() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "positions parameter is empty",
            ));
        }
        let utm_points = convert_track_points(positions).unwrap();
        let module = ConstantGnssModule {
            ctx,
            config: Arc::new(ConstantGnssModuleConfig {
                positions: utm_points,
                velocity,
                information_interval,
            }),
        };
        Ok(module)
    }
}

#[async_trait::async_trait]
impl Module for ConstantGnssModule {
    async fn run(&mut self) -> Result<(), ()> {
        let config = self.config.clone();
        let sender = self.ctx.sender.clone();
        let gnss_pos_task_handle = tokio::spawn(async move {
            constant_gnss_position_task(sender, config).await;
        });
        let config = self.config.clone();
        let sender = self.ctx.sender.clone();
        let gnss_info_task_handle =
            tokio::spawn(async move { constant_gnss_infomation_task(sender, config).await });
        let mut run = true;
        while run {
            tokio::select! {
                event = self.ctx.receiver.recv() => {
                match event {
                    Ok(event) => {
                        if let EventKind::QuitEvent = event.kind {
                            gnss_pos_task_handle.abort();
                            gnss_info_task_handle.abort();
                            run = false;
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                    }
                }
            }
        }
        Ok(())
    }
}

async fn constant_gnss_position_task(
    sender: tokio::sync::broadcast::Sender<Event>,
    config: Arc<ConstantGnssModuleConfig>,
) {
    let mut timer = tokio::time::interval(time::Duration::from_millis(
        ConstantGnssPositionSourceRuntime::POSITION_INTERVAL_MS.into(),
    ));
    let mut runtime = ConstantGnssPositionSourceRuntime {
        points: config.positions.clone(),
        next_position: 0,
        current_position: config.positions[0],
        velocity: config.velocity,
        sender,
    };
    loop {
        timer.tick().await;
        runtime.handle_tick().await;
    }
}

async fn constant_gnss_infomation_task(
    sender: tokio::sync::broadcast::Sender<Event>,
    config: Arc<ConstantGnssModuleConfig>,
) {
    let mut timer = tokio::time::interval(config.information_interval);
    let info = Arc::new(GnssInformation::new(&GnssStatus::Fix3d, 8));
    loop {
        timer.tick().await;
        let _ = sender.send(Event {
            kind: EventKind::GnssInformationEvent(info.clone()),
        });
    }
}
