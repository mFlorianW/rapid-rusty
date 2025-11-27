use module_core::{test_helper::register_response_event, *};
use std::sync::Arc;

#[tokio::test]
#[test_log::test]
pub async fn events_delivered() {
    let event_bus = EventBus::new();
    let mut receiver = event_bus.subscribe();
    let event = Event {
        kind: EventKind::QuitEvent,
    };
    event_bus.publish(&event);
    let received_event =
        tokio::time::timeout(std::time::Duration::from_millis(100), receiver.recv())
            .await
            .expect("Failed to receive event in required time")
            .unwrap();
    assert_eq!(received_event.event_type(), event.event_type());
}

#[tokio::test]
#[test_log::test]
pub async fn test_wait_for_event() {
    let event_bus = EventBus::new();
    let mut ctx = event_bus.context();
    if register_response_event(
        EventKindType::LoadStoredSessionIdsRequestEvent,
        Event {
            kind: EventKind::LoadStoredSessionIdsResponseEvent(Response::new(
                0,
                0xFA,
                Arc::new(vec!["session1".to_string()]),
            )),
        },
        event_bus.context(),
    )
    .is_err()
    {
        panic!("Failed to register response event");
    }
    if ctx
        .publish_event(EventKind::LoadStoredSessionIdsRequestEvent(
            Request::empty_request(0, 0xFA),
        ))
        .is_err()
    {
        panic!("Failed to publish request event");
    }
    let event = ctx
        .wait_for_event(0, 0xFA, &EventKindType::LoadStoredSessionIdsResponseEvent)
        .await
        .unwrap();
    let response = payload_ref!(event.kind, EventKind::LoadStoredSessionIdsResponseEvent).unwrap();
    assert_eq!(response.id, 0);
    assert_eq!(response.receiver_addr, 0xFA);
    assert_eq!(*response.data, vec!["session1".to_string()]);
}
