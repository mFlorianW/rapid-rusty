use super::*;
use module_core::{EventBus, Module};
use tokio::task::JoinHandle;

fn get_path(folder_name: &str) -> String {
    format!("/tmp/rapid-rusty/{folder_name}")
}

fn setup_empty_test_folder(folder_name: &str) {
    let path = get_path(folder_name);
    if let Ok(true) = std::fs::exists(&path) {
        std::fs::remove_dir_all(&path)
            .unwrap_or_else(|_| panic!("Failed to cleanup test dir {path}"));
    }
    std::fs::create_dir_all(&path)
        .unwrap_or_else(|err| panic!("Failed to create test dir for {path}. Reason: {err}"));
}

fn create_storage_module(folder: &str, event_bus: &EventBus) -> JoinHandle<Result<(), ()>> {
    let ctx = event_bus.context();
    let folder = get_path(folder);
    tokio::spawn(async move {
        let mut storage = FilesSystemStorage::new(folder, ctx);
        storage.run().await
    })
}

pub mod test_sessionfs_storage;
