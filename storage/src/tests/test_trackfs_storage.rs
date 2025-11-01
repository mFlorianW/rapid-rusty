use super::*;
use crate::tests::create_storage_module;
use module_core::{
    Request, payload_ref,
    test_helper::{stop_module, wait_for_event},
};
use std::{
    fs::{File, create_dir_all},
    io::Write,
    mem::discriminant,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

fn init_none_empty_test(test_folder_name: &str) -> Vec<String> {
    let ids = vec!["Oschersleben".to_owned(), "Most".to_owned()];
    let osl = include_str!("Oschersleben.json");
    let most = include_str!("Most.json");

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
    let exp_ids = init_none_empty_test(test_folder_name);
    let mut storage = create_storage_module(test_folder_name, &eb);

    eb.publish(&Event {
        kind: EventKind::LoadStoredTrackIdsRequest(EmptyRequestPtr::new(Request {
            id: 10,
            sender_addr: 22,
            data: (),
        })),
    });
    let exp_event = EventKind::LoadStoredTrackIdsResponseEvent(
        Response {
            id: 10,
            receiver_addr: 22,
            data: exp_ids,
        }
        .into(),
    );
    let load_stored_event = wait_for_event(
        &mut eb.subscribe(),
        Duration::from_millis(100),
        discriminant(&exp_event),
    )
    .await;

    let exp_payload = payload_ref!(exp_event, EventKind::LoadStoredTrackIdsResponseEvent).unwrap();
    let payload = payload_ref!(
        load_stored_event.kind,
        EventKind::LoadStoredTrackIdsResponseEvent
    )
    .unwrap();

    assert_eq!(exp_payload.id, payload.id);
    assert_eq!(exp_payload.receiver_addr, payload.receiver_addr);
    let mut exp_ids = exp_payload.data.clone();
    exp_ids.sort();
    let mut ids = payload.data.clone();
    ids.sort();
    assert_eq!(exp_ids, ids);

    stop_module(&eb, &mut storage).await;
}
