use common::{session::Session, test_helper::session::get_session};
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
    let expected_body = r#"{"total":2,"ids":["session_1","session_2"]}"#;
    if register_response_event(
        EventKindType::LoadStoredSessionIdsRequestEvent,
        Event {
            kind: EventKind::LoadStoredSessionIdsResponseEvent(
                Response {
                    id: 0,
                    receiver_addr: 0xff,
                    data: Arc::new(vec!["session_1".to_string(), "session_2".to_string()]),
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
