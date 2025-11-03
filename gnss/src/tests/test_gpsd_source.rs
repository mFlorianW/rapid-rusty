use crate::gpsd_source::GpsdModule;
use chrono::DateTime;
use common::position::{GnssInformation, GnssPosition, GnssStatus};
use core::panic;
use module_core::{
    EventBus, EventKind, EventKindDiscriminants, Module, ModuleCtx, payload_ref,
    test_helper::{stop_module, wait_for_event},
};
use std::{io::Error, str::FromStr, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};

struct GpsdServer {
    socket: TcpListener,
    client: Option<TcpStream>,
}

impl GpsdServer {
    pub async fn new(addr: &str) -> GpsdServer {
        let listener = TcpListener::bind(addr).await;
        GpsdServer {
            socket: listener.expect("Failed to bind gpsd test server on localhost on port 35500"),
            client: None,
        }
    }

    pub async fn accept_client(&mut self) {
        match self.socket.accept().await {
            Ok((client, _)) => self.client = Some(client),
            Err(e) => panic!("Client connection failed. Error: {:?}", e),
        }
    }

    pub async fn send(&mut self, buf: &[u8]) -> Result<(), Error> {
        match self.client {
            Some(ref mut client) => client.write_all(buf).await,
            None => panic!("GPSD server no client is connected"),
        }
    }

    pub async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        match self.client {
            Some(ref mut client) => client.read(buf).await,
            None => panic!("GPSD server no client is connected"),
        }
    }
}

const TIMEOUT_MS: u8 = 100;

async fn test_setup(
    addr: &str,
    ctx: ModuleCtx,
) -> (tokio::task::JoinHandle<Result<(), ()>>, GpsdServer) {
    let mut server = GpsdServer::new(addr).await;
    let owned_addr = addr.to_owned();
    let gpsd_module_handle = tokio::spawn(async move {
        let gpsd_source = GpsdModule::new(ctx, &owned_addr).await;
        match gpsd_source {
            Ok(mut gpsd_source) => gpsd_source.run().await,
            Err(_) => Err(()),
        }
    });
    timeout(
        std::time::Duration::from_millis(TIMEOUT_MS.into()),
        server.accept_client(),
    )
    .await
    .unwrap_or_else(|_| panic!("Enable no client connected within timeout of 100ms"));
    (gpsd_module_handle, server)
}

#[tokio::test]
async fn enable_gpsd_notifications() {
    let event_bus = EventBus::new();
    let (mut gpsd_handle, mut server) = test_setup("127.0.0.1:35500", event_bus.context()).await;
    let enable_cmd: &str = r#"?WATCH={"enable":true,"json":true}"#;
    let mut buf: Vec<u8> = vec![0; enable_cmd.len()];
    let _ = timeout(
        std::time::Duration::from_millis(TIMEOUT_MS.into()),
        server.receive(&mut buf),
    )
    .await
    .unwrap_or_else(|_| panic!("Enable command not received in {:?} ms", TIMEOUT_MS));
    let received_cmd =
        std::str::from_utf8(&buf).expect("Received enable command is not a valid string");
    let _ = stop_module(&event_bus, &mut gpsd_handle).await;
    assert_eq!(received_cmd, enable_cmd);
}

const TPV_MSG: &str = " \
{ \
    \"class\": \"TPV\", \
    \"time\": \"2005-06-08T10:34:48.283Z\", \
    \"lat\": 1.0, \
    \"lon\": 1.0, \
    \"speed\": 22.0, \
    \"mode\": 3 \
}\n\r";

#[tokio::test]
async fn notify_gnss_position() {
    let event_bus = EventBus::new();
    let datetime = DateTime::<chrono::Utc>::from_str("2005-06-08T10:34:48.283Z").unwrap();
    let (mut source, mut server) = test_setup("127.0.0.1:35501", event_bus.context()).await;
    server
        .send(TPV_MSG.as_bytes())
        .await
        .expect("Failed to send TPV msg");

    let event = wait_for_event(
        &mut event_bus.subscribe(),
        Duration::from_millis(TIMEOUT_MS.into()),
        EventKindDiscriminants::GnssPositionEvent,
    )
    .await;
    assert_eq!(
        **payload_ref!(event.kind, EventKind::GnssPositionEvent).unwrap(),
        GnssPosition::new(1.0, 1.0, 22.0, &datetime.time(), &datetime.date_naive())
    );

    stop_module(&event_bus, &mut source).await;
}

#[tokio::test]
async fn notify_gnss_information_on_fix_change() {
    let event_bus = EventBus::default();
    let (mut source, mut server) = test_setup("127.0.0.1:35502", event_bus.context()).await;
    server
        .send(TPV_MSG.as_bytes())
        .await
        .expect("Failed to send TPV msg");

    let event = wait_for_event(
        &mut event_bus.subscribe(),
        Duration::from_millis(TIMEOUT_MS.into()),
        EventKindDiscriminants::GnssInformationEvent,
    )
    .await;
    assert_eq!(
        **payload_ref!(event.kind, EventKind::GnssInformationEvent).unwrap(),
        GnssInformation::new(&GnssStatus::Fix3d, 0)
    );

    stop_module(&event_bus, &mut source).await;
}

const SKY_MSG: &str = " \
{ \
    \"class\":\"SKY\", \
    \"device\":\"/dev/pts/1\", \
    \"time\":\"2005-07-08T11:28:07.114Z\", \
    \"xdop\":1.55,\"hdop\":1.24,\"pdop\":1.99, \
    \"satellites\":[ \
        {\"PRN\":23,\"el\":6,\"az\":84,\"ss\":0,\"used\":false}, \
        {\"PRN\":28,\"el\":7,\"az\":160,\"ss\":0,\"used\":false}, \
        {\"PRN\":8,\"el\":66,\"az\":189,\"ss\":44,\"used\":true}, \
        {\"PRN\":29,\"el\":13,\"az\":273,\"ss\":0,\"used\":false}, \
        {\"PRN\":10,\"el\":51,\"az\":304,\"ss\":29,\"used\":true}, \
        {\"PRN\":4,\"el\":15,\"az\":199,\"ss\":36,\"used\":true}, \
        {\"PRN\":2,\"el\":34,\"az\":241,\"ss\":43,\"used\":true}, \
        {\"PRN\":27,\"el\":71,\"az\":76,\"ss\":43,\"used\":true} \
    ] \
} \
\n\r";

#[tokio::test]
async fn notify_gnss_information_on_sky_change() {
    let event_bus = EventBus::default();
    let (mut source, mut server) = test_setup("127.0.0.1:35503", event_bus.context()).await;
    server
        .send(SKY_MSG.as_bytes())
        .await
        .expect("Failed to send SKY msg");

    let event = wait_for_event(
        &mut event_bus.subscribe(),
        Duration::from_millis(TIMEOUT_MS.into()),
        EventKindDiscriminants::GnssInformationEvent,
    )
    .await;
    assert_eq!(
        **payload_ref!(event.kind, EventKind::GnssInformationEvent).unwrap(),
        GnssInformation::new(&GnssStatus::Unknown, 5)
    );

    stop_module(&event_bus, &mut source).await;
}
