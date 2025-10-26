use crate::GnssPosition;
use common::position::{self, GnssInformation, GnssStatus};
use futures::StreamExt;
use gpsd_proto::{self, Mode, Satellite, Sky, Tpv};
use module_core::Event;
use module_core::{EventKind, Module, ModuleCtx};
use std::{
    io::{self, Error, ErrorKind},
    net::SocketAddr,
    str::FromStr,
    sync::Arc,
};
use tokio::sync::Notify;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_util::codec::{Framed, LinesCodec};

/// GPSD daemon based GNSS source
struct GpsdPositionInformationRuntime {
    /// The status of GNSS system
    mode: GnssStatus,
    /// The amount of satellites used for the GNSS position
    sats: usize,
    /// The start signal for the GPSD task to start execution
    notify: Arc<Notify>,
    /// The sender of the event_bus to emit the events
    sender: tokio::sync::broadcast::Sender<Event>,
}

impl GpsdPositionInformationRuntime {
    /// Creates a new instance of the GPSD runtime.
    pub fn new(sender: tokio::sync::broadcast::Sender<Event>) -> Self {
        GpsdPositionInformationRuntime {
            mode: GnssStatus::Unknown,
            sats: 0,
            notify: Arc::new(Notify::new()),
            sender,
        }
    }

    async fn process_tpv_msg(&mut self, tpv: &Tpv) {
        let Some(lat) = tpv.lat else { return };
        let Some(lon) = tpv.lon else { return };
        let Some(speed) = tpv.speed else { return };
        let Some(ref time) = tpv.time else { return };
        let Ok(datetime) = chrono::DateTime::<chrono::Utc>::from_str(time) else {
            return;
        };
        let position = Arc::new(GnssPosition::new(
            lat,
            lon,
            speed.into(),
            &datetime.time(),
            &datetime.date_naive(),
        ));
        let _ = self.sender.send(Event {
            kind: EventKind::GnssPositionEvent(position.clone()),
        });
        self.mode = convert_mode(&tpv.mode);
        let info = Arc::new(GnssInformation::new(&self.mode, self.sats));
        let _ = self.sender.send(Event {
            kind: EventKind::GnssInformationEvent(info.clone()),
        });
    }

    async fn process_sky_msg(&mut self, sky: &Sky) {
        let Some(ref sat) = sky.satellites else {
            return;
        };
        self.sats = used_satellites(sat);
        let info = Arc::new(GnssInformation::new(&self.mode, self.sats));
        let _ = self.sender.send(Event {
            kind: EventKind::GnssInformationEvent(info.clone()),
        });
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

async fn gpsd_reader(mut stream: TcpStream, mut runtime: GpsdPositionInformationRuntime) {
    runtime.notify.notified().await;
    stream
        .write_all(gpsd_proto::ENABLE_WATCH_CMD.as_bytes())
        .await
        .unwrap();
    let mut framed = Framed::new(stream, LinesCodec::new());
    while let Some(result) = framed.next().await {
        match result {
            Ok(ref line) => {
                if let Ok(tpv) = serde_json::from_str::<Tpv>(line) {
                    runtime.process_tpv_msg(&tpv).await;
                }
                if let Ok(sky) = serde_json::from_str::<Sky>(line) {
                    runtime.process_sky_msg(&sky).await;
                }
            }
            Err(e) => {
                println!("GPSD receive error {e:?}");
            }
        }
    }
}

pub struct GpsdModule {
    ctx: ModuleCtx,
    gpsd_handle: tokio::task::JoinHandle<()>,
    task_notify: Arc<Notify>,
}

impl GpsdModule {
    pub async fn new(ctx: ModuleCtx, address: &str) -> Result<Self, Error> {
        let address: SocketAddr = match address.parse() {
            Ok(addr) => addr,
            Err(e) => return Err(io::Error::new(ErrorKind::InvalidInput, e)),
        };
        let socket = TcpStream::connect(address).await?;
        let rt = GpsdPositionInformationRuntime::new(ctx.sender.clone());
        let notify = rt.notify.clone();
        let gpsd_reader_task_handle = tokio::spawn(async move { gpsd_reader(socket, rt).await });
        Ok(GpsdModule {
            ctx,
            gpsd_handle: gpsd_reader_task_handle,
            task_notify: notify,
        })
    }
}

#[async_trait::async_trait]
impl Module for GpsdModule {
    async fn run(&mut self) -> Result<(), ()> {
        self.task_notify.notify_one();
        let mut run = true;
        while run {
            tokio::select! {
                event = self.ctx.receiver.recv() => {
                    match event {
                        Ok(event) => {
                            if let EventKind::QuitEvent = event.kind {
                                self.gpsd_handle.abort();
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
