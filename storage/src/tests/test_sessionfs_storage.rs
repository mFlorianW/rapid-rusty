use super::super::*;
use common::test_helper::session::{get_session, get_session_as_json};
use core::panic;

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

fn init_none_empty_test(test_folder_name: &str) -> Vec<String> {
    let ids = vec![
        "oschersleben_01.01.1970_00:00:00.000.session".to_owned(),
        "oschersleben_01.01.1970_01:00:00.000.session".to_owned(),
    ];
    setup_empty_test_folder(test_folder_name);
    create_empty_session(&ids[0], test_folder_name);
    create_empty_session(&ids[1], test_folder_name);
    ids
}

#[tokio::test]
pub async fn read_stored_session_ids() {
    let test_folder_name = "read_stored_session_ids";
    let exp_ids = init_none_empty_test(test_folder_name);
    let storage = SessionFsStorage::new(&get_path(test_folder_name));
    let ids = storage
        .ids()
        .await
        .unwrap_or_else(|_| panic!("Failed to retrieve sessions ids"));
    assert_eq!(ids, exp_ids);
}

#[tokio::test]
pub async fn save_load_not_existing_session() {
    let test_folder_name = "save_load_session_not_existing";
    setup_empty_test_folder("save_load_session_not_existing");
    let storage = SessionFsStorage::new(&get_path(test_folder_name));
    let session = get_session();

    let id = storage
        .save(&session)
        .await
        .unwrap_or_else(|e| panic!("Failed to store session. Reason: {e}"));
    assert_eq!(id, "oschersleben_01.01.1970_13:00:00.000");

    let loaded_session = storage
        .load(&id)
        .await
        .unwrap_or_else(|e| panic!("Failed to load session. Reason:{e}"));
    assert_eq!(session, loaded_session);
}
