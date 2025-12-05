// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use common::test_helper::session::get_session;
use core::panic;
use module_core::{
    EmptyRequestPtr, Event, EventBus, EventKind, EventKindType, Request, SaveSessionRequestPtr,
    payload_ref,
    test_helper::{stop_module, wait_for_event},
};
use std::{
    fs::create_dir,
    sync::{Arc, RwLock},
};
use std::{os::unix::fs::MetadataExt, time::Duration};

mod helper;
use helper::{create_storage_module, get_path, setup_empty_test_folder};

fn create_empty_session(id: &str, folder_name: &str) {
    let file = format!("{}/session/{id}.session", get_path(folder_name));
    let _ = create_dir(format!("{}/session", get_path(folder_name)));
    if let Ok(true) = std::fs::exists(&file) {
        std::fs::remove_file(&file)
            .unwrap_or_else(|err| panic!("Failed to remove file {file}. Reason: {err}"));
    }
    std::fs::File::create(&file)
        .unwrap_or_else(|err| panic!("Failed to create file {file}. Reason: {err}"));
}

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

fn get_session_ids(folder_name: &str) -> Vec<String> {
    let path = format!("{}/session", get_path(folder_name));
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

async fn get_session_size_in_bytes(folder_name: &str, id: &str) -> u64 {
    let folder = get_path(folder_name);
    let session_path = format!("{folder}/session/{id}.session");
    let session_file = tokio::fs::File::open(&session_path)
        .await
        .unwrap_or_else(|e| panic!("Failed to get file size of {session_path}. Reason: {e}"));
    session_file
        .metadata()
        .await
        .unwrap_or_else(|e| panic!("Failed to get file size. Reason {e}"))
        .size()
}

#[tokio::test]
#[test_log::test]
pub async fn read_stored_session_ids() {
    let event_bus = EventBus::default();
    let test_folder_name = "read_stored_session_ids";
    let exp_ids = init_none_empty_test(test_folder_name);
    let mut handle = create_storage_module(test_folder_name, &event_bus);
    event_bus.publish(&Event {
        kind: EventKind::LoadStoredSessionIdsRequestEvent(EmptyRequestPtr::new(Request {
            id: 10,
            sender_addr: 20,
            data: (),
        })),
    });

    let ids_event = wait_for_event(
        &mut event_bus.subscribe(),
        std::time::Duration::from_millis(100),
        module_core::EventKindType::LoadStoredSessionIdsResponseEvent,
    )
    .await;
    let payload =
        &**payload_ref!(ids_event.kind, EventKind::LoadStoredSessionIdsResponseEvent).unwrap();
    assert_eq!(*payload.data[0].id, exp_ids[0]);
    assert_eq!(*payload.data[1].id, exp_ids[1]);
    assert_eq!(payload.id, 10);
    assert_eq!(payload.receiver_addr, 20);

    stop_module(&event_bus, &mut handle).await;
}

#[tokio::test]
#[test_log::test]
pub async fn save_load_not_existing_session() {
    let event_bus = EventBus::default();
    let test_folder_name = "save_load_session_not_existing";
    setup_empty_test_folder("save_load_session_not_existing");
    let mut storage = create_storage_module(test_folder_name, &event_bus);
    let exp_id = "oschersleben_01_01_1970_13_00_00_000".to_owned();

    event_bus.publish(&Event {
        kind: EventKind::SaveSessionRequestEvent(SaveSessionRequestPtr::new(Request {
            id: 11,
            sender_addr: 20,
            data: Arc::new(RwLock::new(get_session())),
        })),
    });
    let save_resp = wait_for_event(
        &mut event_bus.subscribe(),
        std::time::Duration::from_millis(100),
        module_core::EventKindType::SaveSessionResponseEvent,
    )
    .await;
    let save_resp_payload =
        payload_ref!(save_resp.kind, EventKind::SaveSessionResponseEvent).unwrap();
    assert_eq!(save_resp_payload.id, 11);
    assert_eq!(save_resp_payload.receiver_addr, 20);
    assert!(save_resp_payload.data.is_ok());
    assert_eq!(save_resp_payload.data.clone().unwrap(), exp_id);

    event_bus.publish(&Event {
        kind: EventKind::LoadSessionRequestEvent(
            Request {
                id: 12,
                sender_addr: 20,
                data: exp_id.clone(),
            }
            .into(),
        ),
    });
    let load_resp = wait_for_event(
        &mut event_bus.subscribe(),
        Duration::from_millis(100),
        EventKindType::LoadSessionResponseEvent,
    )
    .await;

    let response = &**payload_ref!(load_resp.kind, EventKind::LoadSessionResponseEvent).unwrap();
    assert!(&response.data.is_ok());
    assert_eq!(
        *response.data.as_ref().unwrap().read().unwrap(),
        get_session()
    );
    assert_eq!(response.id, 12);
    assert_eq!(response.receiver_addr, 20);

    stop_module(&event_bus, &mut storage).await;
}

#[tokio::test]
pub async fn delete_existing_session() {
    let event_bus = EventBus::default();
    let test_folder_name = "delete_existing_session";
    let session_ids = init_none_empty_test(test_folder_name);
    let mut storage = create_storage_module(test_folder_name, &event_bus);

    event_bus.publish(&Event {
        kind: EventKind::DeleteSessionRequestEvent(
            Request {
                id: 13,
                sender_addr: 20,
                data: session_ids[0].clone(),
            }
            .into(),
        ),
    });
    let delete_resp = wait_for_event(
        &mut event_bus.subscribe(),
        Duration::from_millis(100),
        EventKindType::DeleteSessionResponseEvent,
    )
    .await;
    let delete_payload =
        payload_ref!(delete_resp.kind, EventKind::DeleteSessionResponseEvent).unwrap();
    assert_eq!(delete_payload.data, Ok(()));
    assert_eq!(delete_payload.id, 13);
    assert_eq!(delete_payload.receiver_addr, 20);
    let ids = get_session_ids(test_folder_name);
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], session_ids[1]);

    stop_module(&event_bus, &mut storage).await;
}

#[tokio::test]
pub async fn update_existing_session() {
    let event_bus = EventBus::default();
    let test_folder_name = "update_existing_session";
    let session_ids = init_none_empty_test(test_folder_name);
    let mut session_size = get_session_size_in_bytes(test_folder_name, &session_ids[1]).await;
    let mut storage = create_storage_module(test_folder_name, &event_bus);

    assert_eq!(0, session_size);

    event_bus.publish(&Event {
        kind: EventKind::SaveSessionRequestEvent(
            Request {
                id: 10,
                sender_addr: 20,
                data: Arc::new(RwLock::new(get_session())),
            }
            .into(),
        ),
    });

    // loop until max 100ms to wait until the file is written.
    for _ in 0..10 {
        session_size = tokio::time::timeout(
            Duration::from_millis(10),
            get_session_size_in_bytes(test_folder_name, &session_ids[1]),
        )
        .await
        .unwrap();
        if session_size > 0 {
            break;
        }
    }
    assert_ne!(0, session_size);
    stop_module(&event_bus, &mut storage).await;
}
