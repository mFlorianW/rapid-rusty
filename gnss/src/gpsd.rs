use crate::{GnssInformation, GnssInformationSource, GnssStatus};
use crate::{GnssPositionSource, Position};
use futures::StreamExt;
use gpsd_proto::{self, Mode, Satellite, Sky, Tpv};
use std::{
    io::{self, Error, ErrorKind},
    net::SocketAddr,
    str::FromStr,
    sync::Arc,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{mpsc::Sender, Mutex},
    task::JoinHandle,
};
use tokio_util::codec::{Framed, LinesCodec};

/// GPSD daemon based GNSS source
pub struct GpsdPositionSource {
    /// List of consumer that are notified on positions updates
    pos_consumer: Vec<Sender<Arc<Position>>>,
    /// Handle to the task that constantly reads from the GPSD
    task: Option<JoinHandle<()>>,
    /// List of consumer that tare notified on GNSS information updates
    info_consumer: Vec<Sender<Arc<GnssInformation>>>,
    sats: usize,
    mode: GnssStatus,
}

impl GpsdPositionSource {
    /// Creates a new instance of the GPSD source.
    ///
    /// Every new instance creates a new connection to the GPSD daemon.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the GPSD daemon to try to connect to.
    ///
    /// # Returns
    ///
    /// * `Ok(Arc<Mutex<<GpsdPositionSource>>)` - If the connection is successful established.
    /// * `Err(io::Error)` - If the GPSD socket connection fails or initialization fails.
    pub async fn new(address: &str) -> Result<Arc<Mutex<GpsdPositionSource>>, Error> {
        let address: SocketAddr = match address.parse() {
            Ok(addr) => addr,
            Err(e) => return Err(io::Error::new(ErrorKind::InvalidInput, e)),
        };
        let socket = TcpStream::connect(address).await.unwrap();
        let gpsd = Arc::new(tokio::sync::Mutex::new(GpsdPositionSource {
            pos_consumer: Vec::new(),
            task: None,
            info_consumer: Vec::new(),
            sats: 0,
            mode: GnssStatus::Unknown,
        }));
        let task_gpsd = Arc::clone(&gpsd);
        let task = tokio::spawn(async move {
            gpsd_reader(socket, task_gpsd).await;
        });
        gpsd.lock().await.task = Some(task);
        Ok(gpsd)
    }

    async fn notify_consumer(&self, pos: &Arc<Position>) {
        for consumer in self.pos_consumer.iter() {
            consumer.send(pos.clone()).await.unwrap();
        }
    }

    async fn notify_info_consumer(&self, info: &Arc<GnssInformation>) {
        for consumer in self.info_consumer.iter() {
            consumer.send(info.clone()).await.unwrap();
        }
    }

    async fn process_tpv_msg(&mut self, tpv: &Tpv) {
        let Some(lat) = tpv.lat else { return };
        let Some(lon) = tpv.lon else { return };
        let Some(speed) = tpv.speed else { return };
        let Some(ref time) = tpv.time else { return };
        let Ok(time) = chrono::DateTime::<chrono::Utc>::from_str(time) else {
            return;
        };
        let position = Arc::new(Position::new(lat, lon, speed.into(), &time));
        self.notify_consumer(&position).await;
        self.mode = convert_mode(&tpv.mode);
        let info = Arc::new(GnssInformation::new(&self.mode, self.sats));
        self.notify_info_consumer(&info).await;
    }

    async fn process_sky_msg(&mut self, sky: &Sky) {
        let Some(ref sat) = sky.satellites else {
            return;
        };
        self.sats = used_satellites(sat);
        let info = Arc::new(GnssInformation::new(&self.mode, self.sats));
        self.notify_info_consumer(&info).await;
    }
}

impl GnssPositionSource for GpsdPositionSource {
    fn register_pos_consumer(&mut self, consumer: Sender<Arc<Position>>) {
        self.pos_consumer.push(consumer);
    }
}

impl GnssInformationSource for GpsdPositionSource {
    fn register_info_consumer(&mut self, consumer: Sender<std::sync::Arc<GnssInformation>>) {
        self.info_consumer.push(consumer);
    }
}

fn convert_mode(mode: &Mode) -> GnssStatus {
    match mode {
        Mode::NoFix => GnssStatus::NoFix,
        Mode::Fix2d => GnssStatus::Fix2d,
        Mode::Fix3d => GnssStatus::Fix3d,
    }
}

fn used_satellites(sattelites: &[Satellite]) -> usize {
    sattelites.iter().filter(|s| s.used).count()
}

async fn gpsd_reader(mut stream: TcpStream, gpsd: Arc<Mutex<GpsdPositionSource>>) {
    stream
        .write_all(gpsd_proto::ENABLE_WATCH_CMD.as_bytes())
        .await
        .unwrap();

    let mut framed = Framed::new(stream, LinesCodec::new());
    while let Some(result) = framed.next().await {
        match result {
            Ok(ref line) => {
                if let Ok(tpv) = serde_json::from_str::<Tpv>(line) {
                    gpsd.lock().await.process_tpv_msg(&tpv).await
                }
                if let Ok(sky) = serde_json::from_str::<Sky>(line) {
                    gpsd.lock().await.process_sky_msg(&sky).await;
                }
            }
            Err(e) => {
                println!("GPSD receive error {:?}", e);
            }
        }
    }
}
