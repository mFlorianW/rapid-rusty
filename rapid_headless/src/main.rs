use active_session::ActiveSession;
use clap::{CommandFactory, Parser};
use dirs::data_local_dir;
use gnss::{constant_source::ConstantGnssModule, gpsd_source::GpsdModule};
use laptimer::SimpleLaptimer;
use module_core::{EventBus, Module};
use rest::Rest;
use std::str::FromStr;
use std::time::Duration;
use storage::FilesSystemStorage;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;
use track_detection::TrackDetection;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    gps_fake: bool,
    #[arg(short = 'f', long)]
    gps_source_file: Option<String>,
    #[arg(short = 'd', long)]
    gpsd: bool,
}

fn read_lap_points_from_file(file_path: &str) -> Result<Vec<common::position::Position>, ()> {
    let mut rdr = csv::Reader::from_path(file_path).unwrap();
    let mut positions = Vec::new();

    for result in rdr.records() {
        let record = result.unwrap();
        let longitude: f64 = f64::from_str(record.get(0).unwrap()).unwrap();
        let latitude: f64 = f64::from_str(record.get(1).unwrap()).unwrap();
        positions.push(common::position::Position {
            longitude,
            latitude,
        });
    }
    debug!("length of positions: {}", positions.len());
    Ok(positions)
}

async fn get_gpsd_module(eb: &EventBus) -> Result<Box<dyn Module>, ()> {
    match GpsdModule::new(eb.context(), "127.0.0.1:2947").await {
        Ok(gpsd) => Ok(Box::new(gpsd)),
        Err(e) => {
            error!("Failed to connect to gpsd!. Error: {}", e);
            Err(())
        }
    }
}

fn create_fake_gps_module(eb: &EventBus, cli: &Cli) -> Result<Box<dyn Module>, ()> {
    if let Some(source_file) = &cli.gps_source_file {
        let positions = read_lap_points_from_file(source_file).unwrap();
        Ok(Box::new(
            ConstantGnssModule::new(eb.context(), &positions, 10.0, Duration::from_secs(5))
                .unwrap(),
        ))
    } else {
        error!("Failed to create ConstantGnssModule. Error: gps_source_file not set");
        Cli::command().print_help().unwrap();
        Err(())
    }
}

fn get_storage_dir() -> Result<std::path::PathBuf, ()> {
    let mut storage_dir = data_local_dir().ok_or_else(|| {
        error!("Could not determine local data directory");
    })?;
    storage_dir.push("rapid");
    Ok(storage_dir)
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let storage_dir = get_storage_dir()?;
    let eb = EventBus::default();
    let mut gpsd: Box<dyn Module> = if cli.gpsd {
        get_gpsd_module(&eb).await?
    } else if cli.gps_fake {
        create_fake_gps_module(&eb, &cli)?
    } else {
        error!("No GPS source specified. Use --gpsd or --gps-fake");
        Cli::command().print_help().unwrap();
        return Err(());
    };
    let mut storage = FilesSystemStorage::new(&storage_dir, eb.context());
    let mut laptimer = SimpleLaptimer::new(eb.context());
    let mut track_detection = TrackDetection::new(eb.context());
    let mut active_session = ActiveSession::new(eb.context());
    let mut rest = Rest::new(eb.context());

    info!("Starting modules...");
    tokio::join!(
        storage.run(),
        gpsd.run(),
        track_detection.run(),
        laptimer.run(),
        active_session.run(),
        rest.run()
    )
    .0
}
