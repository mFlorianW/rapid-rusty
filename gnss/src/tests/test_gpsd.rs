use crate::gpsd::GpsdPositionSource;
use crate::GnssPositionSource;
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

async fn test_setup(addr: &str) -> (Arc<Mutex<GpsdPositionSource>>, GpsdServer) {
    let mut server = GpsdServer::new(addr).await;
    let source = GpsdPositionSource::new(addr)
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
async fn notify_consumer() {
    let expected_pos = Position::new(
        1.0,
        1.0,
        22.0,
        &DateTime::<chrono::Utc>::from_str("2005-06-08T10:34:48.283Z").unwrap(),
    );
    let (source, mut server) = test_setup("127.0.0.1:35501").await;
    let (sender, mut receiver) = mpsc::channel::<Arc<Position>>(1);
    source.lock().await.register_consumer(sender);
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
