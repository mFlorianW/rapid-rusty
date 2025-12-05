// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use common::{
    session::{Session, SessionInfo},
    test_helper::session::get_session,
};
use module_core::{
    Event, EventBus, EventKind, EventKindType, Module, ModuleCtx, Response,
    test_helper::{register_response_event, stop_module},
};
use rest::Rest;
use serial_test::serial;
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;

fn create_module(ctx: ModuleCtx) -> JoinHandle<Result<(), ()>> {
    tokio::spawn(async move {
        let mut rest = Rest::new(ctx);
        rest.run().await
    })
}

#[tokio::test]
#[test_log::test]
#[serial]
async fn get_session_request_ids() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());
    let expected_body = include_str!("response_request_session_info.json").trim();
    if register_response_event(
        EventKindType::LoadStoredSessionIdsRequestEvent,
        Event {
            kind: EventKind::LoadStoredSessionIdsResponseEvent(
                Response {
                    id: 0,
                    receiver_addr: 0xff,
                    data: Arc::new(vec![
                        SessionInfo {
                            id: "session_1".to_string(),
                            date: chrono::NaiveDateTime::default(),
                            track_name: "".to_string(),
                            laps: 0,
                        },
                        SessionInfo {
                            id: "session_2".to_string(),
                            date: chrono::NaiveDateTime::default(),
                            track_name: "".to_string(),
                            laps: 0,
                        },
                    ]),
                }
                .into(),
            ),
        },
        eb.context(),
    )
    .is_err()
    {
        panic!("Failed to register LoadStoredSessionIdsResponseEvent");
    }

    let body = reqwest::get("http://localhost:27015/v1/sessions")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(body, expected_body);
    stop_module(&eb, &mut rest).await;
}

#[tokio::test]
#[test_log::test]
#[serial]
async fn request_session() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());
    if register_response_event(
        EventKindType::LoadSessionRequestEvent,
        Event {
            kind: EventKind::LoadSessionResponseEvent(
                Response {
                    id: 0,
                    receiver_addr: 0xff,
                    data: Ok(Arc::new(RwLock::new(get_session()))),
                }
                .into(),
            ),
        },
        eb.context(),
    )
    .is_err()
    {
        panic!("Failed to register LoadSessionResponseEvent");
    }

    let body = reqwest::get("http://localhost:27015/v1/sessions/session_1")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let received_session = Session::from_json(&body).unwrap();
    assert_eq!(received_session, get_session());
    stop_module(&eb, &mut rest).await;
}

#[tokio::test]
#[test_log::test]
#[serial]
async fn test_delete_session() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());
    let resp = Response::new(0, 0xff, Ok(()));
    if register_response_event(
        EventKindType::DeleteSessionRequestEvent,
        Event {
            kind: EventKind::DeleteSessionResponseEvent(resp),
        },
        eb.context(),
    )
    .is_err()
    {
        panic!("Failed to register DeleteSessionResponseEvent");
    }

    let client = reqwest::Client::new();
    let response = client
        .delete("http://localhost:27015/v1/sessions/session_1")
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    stop_module(&eb, &mut rest).await;
}
