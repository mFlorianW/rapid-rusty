use std::sync::Arc;

use module_core::{
    Event, EventBus, EventKind, EventKindDiscriminants, Module, ModuleCtx, Response,
    test_helper::{ResponseHandler, stop_module},
};
use rest::Rest;
use tokio::task::JoinHandle;

fn create_module(ctx: ModuleCtx) -> JoinHandle<Result<(), ()>> {
    tokio::spawn(async move {
        let mut rest = Rest::new(ctx);
        rest.run().await
    })
}

#[tokio::test]
#[test_log::test]
async fn get_session_request_ids() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());
    let expected_body = r#"{"total":2,"ids":["session_1","session_2"]}"#;
    let _handler = ResponseHandler::new(
        eb.context(),
        EventKindDiscriminants::LoadStoredSessionIdsRequestEvent,
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
    );
    let body = reqwest::get("http://localhost:27015/v1/sessions")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(body, expected_body);
    stop_module(&eb, &mut rest).await;
}
