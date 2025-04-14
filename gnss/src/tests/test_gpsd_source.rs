use crate::gpsd_source::GpsdPositionInformationSource;
use crate::GnssInformation;
use crate::GnssInformationSource;
use crate::GnssPositionSource;
use crate::GnssStatus;
use crate::Position;
use ::chrono::DateTime;
use std::sync::Arc;
use std::{io::Error, str::FromStr, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
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

async fn test_setup(addr: &str) -> (Arc<Mutex<GpsdPositionInformationSource>>, GpsdServer) {
    let mut server = GpsdServer::new(addr).await;
    let source = GpsdPositionInformationSource::new(addr)
        .await
        .expect("Failed to initialze GPSD source.");
    timeout(
        Duration::from_millis(TIMEOUT_MS.into()),
        server.accept_client(),
    )
    .await
    .unwrap_or_else(|_| panic!("Enable no client connected within timeout of 100ms"));
    (source, server)
}

#[tokio::test]
async fn enable_gpsd_notifications() {
    let (_, mut server) = test_setup("127.0.0.1:35500").await;
    let enable_cmd: &str = r#"?WATCH={"enable":true,"json":true}"#;
    let mut buf: Vec<u8> = vec![0; enable_cmd.len()];
    let _ = timeout(
        Duration::from_millis(TIMEOUT_MS.into()),
        server.receive(&mut buf),
    )
    .await
    .unwrap_or_else(|_| panic!("Enable command not received in {:?} ms", TIMEOUT_MS));
    let received_cmd =
        std::str::from_utf8(&buf).expect("Received enable command is not a valid string");
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
async fn notify_position_consumer() {
    let expected_pos = Position::new(
        1.0,
        1.0,
        22.0,
        &DateTime::<chrono::Utc>::from_str("2005-06-08T10:34:48.283Z").unwrap(),
    );
    let (source, mut server) = test_setup("127.0.0.1:35501").await;
    let (sender, mut receiver) = mpsc::channel::<Arc<Position>>(1);
    source.lock().await.register_pos_consumer(sender);
    server
        .send(TPV_MSG.as_bytes())
        .await
        .expect("Failed to send TPV msg");
    let pos = timeout(Duration::from_millis(TIMEOUT_MS.into()), receiver.recv())
        .await
        .expect("Failed to receive position in required time")
        .unwrap();
    assert_eq!(expected_pos, *pos);
}

#[tokio::test]
async fn notify_information_consumer_on_fix_change() {
    let information = GnssInformation::new(&GnssStatus::Fix3d, 0);
    let (source, mut server) = test_setup("127.0.0.1:35502").await;
    let (sender, mut receiver) = mpsc::channel::<Arc<GnssInformation>>(1);
    source.lock().await.register_info_consumer(sender);
    server
        .send(TPV_MSG.as_bytes())
        .await
        .expect("Failed to send TPV msg");
    let info = timeout(Duration::from_millis(TIMEOUT_MS.into()), receiver.recv())
        .await
        .expect("Failed to receive information in required time")
        .unwrap();
    assert_eq!(information, *info);
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
async fn notify_information_consumer_on_sky_change() {
    let information = GnssInformation::new(&GnssStatus::Unknown, 5);
    let (source, mut server) = test_setup("127.0.0.1:35503").await;
    let (sender, mut receiver) = mpsc::channel::<Arc<GnssInformation>>(1);
    source.lock().await.register_info_consumer(sender);
    server
        .send(SKY_MSG.as_bytes())
        .await
        .expect("Failed to send SKY msg");
    let info = timeout(Duration::from_millis(TIMEOUT_MS.into()), receiver.recv())
        .await
        .expect("Failed to receive information in required time")
        .unwrap();
    assert_eq!(information, *info);
}
