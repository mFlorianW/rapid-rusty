// SPDX-FileCopyrightText: 2026 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

mod test_utils;

use common::test_helper::session::get_session;
use futures_util::{StreamExt, stream::SplitStream};
use module_core::{
    Event, EventBus, EventKind, EventKindType, Response,
    test_helper::stop_module,
    test_helper::{register_response_event, unregister_response_event},
};
use serial_test::serial;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use test_utils::create_module;
use tokio_tungstenite::{WebSocketStream, connect_async, tungstenite::Message};

fn get_current_laptime_msg(laptime: Duration, event: &str) -> serde_json::Value {
    let event = format!(
        r#"{{ "event": "{}", "data": {{ "time": "{:02}:{:02}:{:02}.{:03}" }} }}"#,
        event,
        laptime.as_secs() / 3600,
        (laptime.as_secs() % 3600) / 60,
        laptime.as_secs() % 60,
        laptime.subsec_millis()
    );
    serde_json::from_str(&event).unwrap()
}

fn get_lap_started_msg() -> serde_json::Value {
    let event = r#"{"event": "lap_started", "data":{}}"#;
    serde_json::from_str(event).unwrap()
}

fn register_current_session_response_event(eb: &EventBus) {
    if register_response_event(
        EventKindType::CurrentSessionRequestEvent,
        Event {
            kind: EventKind::CurrentSessionResponseEvent(
                Response {
                    id: 0,
                    receiver_addr: 0xff,
                    data: Some(Arc::new(RwLock::new(get_session()))),
                }
                .into(),
            ),
        },
        eb.context(),
    )
    .is_err()
    {
        panic!("Failed to register CurrentSessionResponseEvent");
    }
}

async fn read_next_websocket_event<S>(read_stream: &mut SplitStream<WebSocketStream<S>>) -> Message
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    tokio::time::timeout(Duration::from_millis(100), read_stream.next())
        .await
        .expect("No message received")
        .expect("Error reading message")
        .expect("No message in time")
}

fn unregister_current_session_response_event(eb: &EventBus) {
    unregister_response_event(eb.id(), &EventKindType::CurrentSessionRequestEvent)
}

#[tokio::test]
#[test_log::test]
#[serial]
async fn test_current_laptime() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());
    register_current_session_response_event(&eb);

    let (ws_stream, _) = connect_async("ws://localhost:27015/v1/live_session")
        .await
        .expect("Failed to connect to WebSocket");
    let (_, mut read) = ws_stream.split();
    let _ = read_next_websocket_event(&mut read).await; // Consume the current_session event

    eb.publish(&Event {
        kind: EventKind::CurrentLaptimeEvent(Duration::from_millis(1).into()),
    });
    let msg = read_next_websocket_event(&mut read).await;
    match msg {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            let expected = get_current_laptime_msg(Duration::from_millis(1), "current_laptime");
            let msg = serde_json::from_slice::<serde_json::Value>(text.as_bytes()).unwrap();
            assert_eq!(msg, expected, "Laptime message does not match expected");
        }
        _ => panic!("Unexpected message type received. Msg: {:?}", msg),
    }

    unregister_current_session_response_event(&eb);
    stop_module(&eb, &mut rest).await;
}

#[tokio::test]
#[test_log::test]
#[serial]
async fn test_lap_finished_event() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());
    register_current_session_response_event(&eb);

    let (ws_stream, _) = connect_async("ws://localhost:27015/v1/live_session")
        .await
        .expect("Failed to connect to WebSocket");
    let (_, mut read) = ws_stream.split();
    let _ = read_next_websocket_event(&mut read).await; // Consume the current_session event

    eb.publish(&Event {
        kind: EventKind::LapFinishedEvent(Duration::from_millis(1).into()),
    });
    let msg = read_next_websocket_event(&mut read).await;
    match msg {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            let expected = get_current_laptime_msg(Duration::from_millis(1), "lap_finished");
            let msg = serde_json::from_slice::<serde_json::Value>(text.as_bytes()).unwrap();
            assert_eq!(msg, expected, "Laptime message does not match expected");
        }
        _ => panic!("Unexpected message type received. Msg: {:?}", msg),
    }

    unregister_current_session_response_event(&eb);
    stop_module(&eb, &mut rest).await;
}

#[tokio::test]
#[test_log::test]
#[serial]
async fn test_sector_finished() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());
    register_current_session_response_event(&eb);

    let (ws_stream, _) = connect_async("ws://localhost:27015/v1/live_session")
        .await
        .expect("Failed to connect to WebSocket");
    let (_, mut read) = ws_stream.split();
    let _ = read_next_websocket_event(&mut read).await; // Consume the current_session event

    eb.publish(&Event {
        kind: EventKind::SectorFinishedEvent(Duration::from_millis(1).into()),
    });
    let msg = read_next_websocket_event(&mut read).await;
    match msg {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            let expected = get_current_laptime_msg(Duration::from_millis(1), "sector_finished");
            let msg = serde_json::from_slice::<serde_json::Value>(text.as_bytes()).unwrap();
            assert_eq!(msg, expected, "Laptime message does not match expected");
        }
        _ => panic!("Unexpected message type received. Msg: {:?}", msg),
    }

    unregister_current_session_response_event(&eb);
    stop_module(&eb, &mut rest).await;
}

#[tokio::test]
#[test_log::test]
#[serial]
async fn test_lap_started_event() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());
    register_current_session_response_event(&eb);

    let (ws_stream, _) = connect_async("ws://localhost:27015/v1/live_session")
        .await
        .expect("Failed to connect to WebSocket");
    let (_, mut read) = ws_stream.split();
    let _ = read_next_websocket_event(&mut read).await; // Consume the current_session event
    //
    eb.publish(&Event {
        kind: EventKind::LapStartedEvent,
    });
    let msg = read_next_websocket_event(&mut read).await;
    match msg {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            let expected = get_lap_started_msg();
            let msg = serde_json::from_slice::<serde_json::Value>(text.as_bytes()).unwrap();
            assert_eq!(msg, expected, "Laptime message does not match expected");
        }
        _ => panic!("Unexpected message type received. Msg: {:?}", msg),
    }

    unregister_current_session_response_event(&eb);
    stop_module(&eb, &mut rest).await;
}

#[tokio::test]
#[test_log::test]
#[serial]
async fn test_current_session_event_on_connect() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());
    register_current_session_response_event(&eb);

    let (ws_stream, _) = connect_async("ws://localhost:27015/v1/live_session")
        .await
        .expect("Failed to connect to WebSocket");
    let (_, mut read) = ws_stream.split();

    let msg = read_next_websocket_event(&mut read).await;
    match msg {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            let expected = serde_json::json!({
                "event": "current_session",
                "data": {
                    "session": get_session()
                }
            });
            let msg = serde_json::from_slice::<serde_json::Value>(text.as_bytes()).unwrap();
            assert_eq!(msg, expected, "Session message does not match expected");
        }
        _ => panic!("Unexpected message type received. Msg: {:?}", msg),
    }

    unregister_current_session_response_event(&eb);
    stop_module(&eb, &mut rest).await;
}
