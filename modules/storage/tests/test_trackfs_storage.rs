use common::track::Track;
use module_core::{
    EmptyRequestPtr, Event, EventBus, EventKind, EventKindType, Request, payload_ref,
    test_helper::{stop_module, wait_for_event},
};
use std::{
    fs::{File, create_dir_all},
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
mod helper;
use helper::{create_storage_module, get_path, setup_empty_test_folder};

fn init_none_empty_test(test_folder_name: &str) -> Vec<String> {
    let ids = vec!["Oschersleben".to_owned(), "Most".to_owned()];
    let osl = include_str!("../../../assets/tracks/Oschersleben.json");
    let most = include_str!("../../../assets/tracks/Most.json");

    setup_empty_test_folder(test_folder_name);
    let mut track_folder = PathBuf::from_str(&get_path(test_folder_name)).unwrap();
    track_folder.push("track");
    create_dir_all(&track_folder).unwrap_or_else(|e| {
        panic!(
            "Failed to create track folder {}. Error: {e}",
            track_folder.to_string_lossy()
        )
    });
    create_track(&track_folder, &format!("{}.track", &ids[0]), Some(osl));
    create_track(&track_folder, &format!("{}.track", &ids[1]), Some(most));

    ids
}

fn create_track(test_folder_name: &Path, id: &str, content: Option<&str>) {
    let mut file_name = test_folder_name.to_path_buf();
    file_name.push(id);
    let mut file = File::create_new(&file_name).unwrap_or_else(|e| {
        panic!(
            "Failed to create track {}. Error: {e}",
            file_name.to_string_lossy()
        )
    });
    if let Some(content) = content {
        file.write_all(content.as_bytes()).unwrap_or_else(|e| {
            panic!(
                "Failed to write track content for {}. Error: {e}",
                file_name.to_string_lossy()
            )
        });
    };
}

#[tokio::test]
pub async fn load_stored_track_ids() {
    let eb = EventBus::default();
    let test_folder_name = "load_stored_track_ids";
    let mut exp_ids = init_none_empty_test(test_folder_name);
    exp_ids.sort();
    let mut storage = create_storage_module(test_folder_name, &eb);

    eb.publish(&Event {
        kind: EventKind::LoadStoredTrackIdsRequest(EmptyRequestPtr::new(Request {
            id: 10,
            sender_addr: 22,
            data: (),
        })),
    });
    let load_stored_event = wait_for_event(
        &mut eb.subscribe(),
        Duration::from_millis(100),
        EventKindType::LoadStoredTrackIdsResponseEvent,
    )
    .await;

    let payload = payload_ref!(
        load_stored_event.kind,
        EventKind::LoadStoredTrackIdsResponseEvent
    )
    .unwrap();

    assert_eq!(payload.id, 10);
    assert_eq!(payload.receiver_addr, 22);
    let mut ids = payload.data.clone();
    ids.sort();
    assert_eq!(exp_ids, ids);

    stop_module(&eb, &mut storage).await;
}

#[tokio::test]
pub async fn read_stored_session_ids() {
    let eb = EventBus::default();
    let test_folder_name = "load_stored_all_track";
    init_none_empty_test(test_folder_name);
    let tracks = vec![
        Track::from_json(include_str!("../../../assets/tracks/Most.json")).unwrap(),
        Track::from_json(include_str!("../../../assets/tracks/Oschersleben.json")).unwrap(),
    ];
    let mut storage = create_storage_module(test_folder_name, &eb);

    eb.publish(&Event {
        kind: EventKind::LoadAllStoredTracksRequestEvent(
            Request {
                id: 10,
                sender_addr: 22,
                data: (),
            }
            .into(),
        ),
    });
    let event = wait_for_event(
        &mut eb.subscribe(),
        Duration::from_millis(100),
        EventKindType::LoadAllStoredTracksResponseEvent,
    )
    .await;

    let payload = payload_ref!(event.kind, EventKind::LoadAllStoredTracksResponseEvent).unwrap();
    assert_eq!(payload.id, 10);
    assert_eq!(payload.receiver_addr, 22);
    assert_eq!(payload.data, tracks);

    stop_module(&eb, &mut storage).await;
}
