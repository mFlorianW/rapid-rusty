use super::super::*;
use core::panic;
use std::any::type_name;

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

fn create_empty_session(id: &str, folder_name: &str) {
    let file = format!("{}/{id}", get_path(folder_name));
    if let Ok(true) = std::fs::exists(&file) {
        std::fs::remove_file(&file)
            .unwrap_or_else(|err| panic!("Failed to remove file {file}. Reason: {err}"));
    }
    std::fs::File::create(&file)
        .unwrap_or_else(|err| panic!("Failed to create file {file}. Reason: {err}"));
}

#[tokio::test]
pub async fn read_stored_session_ids() {
    let test_folder_name = "read_stored_session_ids";
    let exp_ids = vec![
        "oschersleben_01.01.1970_00:00:00.000.session",
        "oschersleben_01.01.1970_01:00:00.000.session",
    ];
    setup_empty_test_folder(test_folder_name);
    create_empty_session(exp_ids[0], test_folder_name);
    create_empty_session(exp_ids[1], test_folder_name);
    let storage = SessionFsStorage::new(&get_path(test_folder_name));
    let ids = storage
        .ids()
        .await
        .unwrap_or_else(|_| panic!("Failed to retrieve sessions ids"));
    assert_eq!(ids, exp_ids);
}
