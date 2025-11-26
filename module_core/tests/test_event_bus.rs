use module_core::*;

#[tokio::test]
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
