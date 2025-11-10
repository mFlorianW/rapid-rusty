use active_session::ActiveSession;
use gnss::gpsd_source::GpsdModule;
use laptimer::SimpleLaptimer;
use module_core::{EventBus, Module};
use storage::FilesSystemStorage;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use track_detection::TrackDetection;

#[tokio::main]
async fn main() -> Result<(), ()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let eb = EventBus::default();

    let mut storage = FilesSystemStorage::new("/tmp/".to_owned(), eb.context());
    let mut gpsd = match GpsdModule::new(eb.context(), "127.0.0.1:2947").await {
        Ok(gpsd) => gpsd,
        Err(e) => {
            error!("Failed to connect to gpsd!. Error: {}", e);
            return Err(());
        }
    };
    let mut laptimer = SimpleLaptimer::new(eb.context());
    let mut track_detection = TrackDetection::new(eb.context());
    let mut active_session = ActiveSession::new(eb.context());

    info!("Starting modules...");
    tokio::join!(
        storage.run(),
        gpsd.run(),
        track_detection.run(),
        laptimer.run(),
        active_session.run()
    )
    .0
}
