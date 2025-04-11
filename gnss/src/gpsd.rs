use super::{GnssPositionSource, Position};
use futures::StreamExt;
use gpsd_proto::{self, Tpv};
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
    consumer: Vec<Sender<Arc<Position>>>,
    /// Handle to the task that constantly reads from the GPSD
    task: Option<JoinHandle<()>>,
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
            consumer: Vec::new(),
            task: None,
        }));
        let task_gpsd = Arc::clone(&gpsd);
        let task = tokio::spawn(async move {
            gpsd_reader(socket, task_gpsd).await;
        });
        gpsd.lock().await.task = Some(task);
        Ok(gpsd)
    }

    async fn notify_consumer(&self, pos: &Arc<Position>) {
        for consumer in self.consumer.iter() {
            consumer.send(pos.clone()).await.unwrap();
        }
    }

    async fn process_msg(&self, tpv: &Tpv) {
        let Some(lat) = tpv.lat else { return };
        let Some(lon) = tpv.lon else { return };
        let Some(speed) = tpv.speed else { return };
        let Some(ref time) = tpv.time else { return };
        let Ok(time) = chrono::DateTime::<chrono::Utc>::from_str(time) else {
            return;
        };
        let position = Arc::new(Position::new(lat, lon, speed.into(), &time));
        self.notify_consumer(&position).await;
    }
}

impl GnssPositionSource for GpsdPositionSource {
    fn register_consumer(&mut self, consumer: Sender<Arc<Position>>) {
        self.consumer.push(consumer);
    }
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
                    gpsd.lock().await.process_msg(&tpv).await
                }
            }
            Err(e) => {
                println!("Invalid JSON received {:?}", e);
            }
        }
    }
}
