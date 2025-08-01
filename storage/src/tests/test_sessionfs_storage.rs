#[cfg(test)]
use super::super::*;
#[cfg(test)]
use common::test_helper::session::get_session;
#[cfg(test)]
use std::{os::unix::fs::MetadataExt, u64};

#[cfg(test)]
fn get_path(folder_name: &str) -> String {
    format!("/tmp/rapid-rusty/{folder_name}")
}

#[cfg(test)]
fn setup_empty_test_folder(folder_name: &str) {
    let path = get_path(folder_name);
    if let Ok(true) = std::fs::exists(&path) {
        std::fs::remove_dir_all(&path)
            .unwrap_or_else(|_| panic!("Failed to cleanup test dir {path}"));
    }
    std::fs::create_dir_all(&path)
        .unwrap_or_else(|err| panic!("Failed to create test dir for {path}. Reason: {err}"));
}

#[cfg(test)]
fn create_empty_session(id: &str, folder_name: &str) {
    let file = format!("{}/{id}.session", get_path(folder_name));
    if let Ok(true) = std::fs::exists(&file) {
        std::fs::remove_file(&file)
            .unwrap_or_else(|err| panic!("Failed to remove file {file}. Reason: {err}"));
    }
    std::fs::File::create(&file)
        .unwrap_or_else(|err| panic!("Failed to create file {file}. Reason: {err}"));
}

#[cfg(test)]
fn init_none_empty_test(test_folder_name: &str) -> Vec<String> {
    let ids = vec![
        "oschersleben_01_01_1970_00_00_00_000".to_owned(),
        "oschersleben_01_01_1970_13_00_00_000".to_owned(),
    ];
    setup_empty_test_folder(test_folder_name);
    create_empty_session(&ids[0], test_folder_name);
    create_empty_session(&ids[1], test_folder_name);
    ids
}

#[cfg(test)]
fn get_session_ids(folder_name: &str) -> Vec<String> {
    let path = get_path(folder_name);
    let mut ids: Vec<String> = vec![];
    if let Ok(entries) = std::fs::read_dir(&path) {
        entries.for_each(|entry| {
            if let Ok(entry) = entry
                && let Some(extension) = entry.path().extension()
                && extension == "session"
            {
                ids.push(
                    entry
                        .path()
                        .file_stem()
                        .unwrap_or_else(|| {
                            panic!("Failed to convert the file name: {:?}", entry.path())
                        })
                        .to_string_lossy()
                        .to_string(),
                );
            }
        });
    } else {
        panic!("Failed to read session ids in {}", &path);
    }
    ids
}

#[cfg(test)]
fn get_session_size_in_bytes(folder_name: &str, id: &str) -> u64 {
    let folder = get_path(folder_name);
    let session_path = format!("{folder}/{id}.session");
    let session_file = std::fs::File::open(&session_path)
        .unwrap_or_else(|e| panic!("Failed to get file size of {session_path}. Reason: {e}"));
    session_file
        .metadata()
        .unwrap_or_else(|e| panic!("Failed to get file size. Reason {e}"))
        .size()
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
    assert_eq!(id, "oschersleben_01_01_1970_13_00_00_000");

    let loaded_session = storage
        .load(&id)
        .await
        .unwrap_or_else(|e| panic!("Failed to load session. Reason:{e}"));
    assert_eq!(session, loaded_session);
}

#[tokio::test]
pub async fn delete_existing_session() {
    let test_folder_name = "delete_existing_session";
    let session_ids = init_none_empty_test(test_folder_name);

    let storage = SessionFsStorage::new(&get_path(test_folder_name));

    storage
        .delete(&session_ids[0])
        .await
        .unwrap_or_else(|e| panic!("Failed to delete session. Reason {e}"));

    let ids = get_session_ids(test_folder_name);
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], session_ids[1]);
}

#[tokio::test]
pub async fn update_existing_session() {
    let test_folder_name = "update_existing_session";
    let session_ids = init_none_empty_test(test_folder_name);
    let mut session_size = get_session_size_in_bytes(test_folder_name, &session_ids[1]);

    assert_eq!(0, session_size);

    let storage = SessionFsStorage::new(&get_path(test_folder_name));
    storage
        .save(&get_session())
        .await
        .unwrap_or_else(|e| panic!("Failed to update session. Reason: {e}"));

    session_size = get_session_size_in_bytes(test_folder_name, &session_ids[1]);
    assert_ne!(0, session_size);
}
