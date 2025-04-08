use super::{GnssPositionSource, Position};
use futures::StreamExt;
use gpsd_proto::{self, Tpv, UnifiedResponse};
use std::{
    io::{self, Error, ErrorKind},
    net::SocketAddr,
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
    consumer: Vec<Sender<Position>>,
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

    async fn notify_consumer(&self, pos: &Position) {
        for consumer in self.consumer.iter() {
            consumer.send(*pos).await.unwrap();
        }
    }

    async fn process_msg(&self, tpv: &Tpv) {
        let position = Position::new(
            tpv.lat.ok_or(0.0).unwrap(),
            tpv.lon.ok_or(0.0).unwrap(),
            tpv.speed.ok_or(0.0).unwrap().into(),
        );
        self.notify_consumer(&position).await;
    }
}

impl GnssPositionSource for GpsdPositionSource {
    fn register_consumer(&mut self, consumer: Sender<Position>) {
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
            Ok(ref line) => match serde_json::from_str(line) {
                Ok(rd) => match rd {
                    UnifiedResponse::Tpv(ref t) => {
                        println!("tpv tpv tpv");
                        gpsd.lock().await.process_msg(t).await;
                        println!("after process msg");
                    }
                    _ => {
                        println!("{:?}", line);
                    }
                },
                Err(ref e) => println!("Error: {:?}", e),
            },
            Err(e) => {
                println!("Invalid JSON received {:?}", e);
            }
        }
    }
}
