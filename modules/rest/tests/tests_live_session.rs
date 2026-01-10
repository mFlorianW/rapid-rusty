// SPDX-FileCopyrightText: 2026 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

mod test_utils;

use futures_util::StreamExt;
use module_core::{Event, EventBus, EventKind, test_helper::stop_module};
use serial_test::serial;
use std::time::Duration;
use test_utils::create_module;
use tokio_tungstenite::connect_async;

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

#[tokio::test]
#[test_log::test]
#[serial]
async fn test_current_laptime() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());

    let (ws_stream, _) = connect_async("ws://localhost:27015/v1/live_session")
        .await
        .expect("Failed to connect to WebSocket");
    let (_, mut read) = ws_stream.split();
    eb.publish(&Event {
        kind: EventKind::CurrentLaptimeEvent(Duration::from_millis(1).into()),
    });

    let msg = tokio::time::timeout(Duration::from_millis(100), read.next())
        .await
        .expect("No message received")
        .expect("Error reading message")
        .expect("No message in time");

    match msg {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            let expected = get_current_laptime_msg(Duration::from_millis(1), "current_laptime");
            let msg = serde_json::from_slice::<serde_json::Value>(text.as_bytes()).unwrap();
            assert_eq!(msg, expected, "Laptime message does not match expected");
        }
        _ => panic!("Unexpected message type received. Msg: {:?}", msg),
    }
    stop_module(&eb, &mut rest).await;
}

#[tokio::test]
#[test_log::test]
#[serial]
async fn test_lap_finished_event() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());

    let (ws_stream, _) = connect_async("ws://localhost:27015/v1/live_session")
        .await
        .expect("Failed to connect to WebSocket");
    let (_, mut read) = ws_stream.split();
    eb.publish(&Event {
        kind: EventKind::LapFinishedEvent(Duration::from_millis(1).into()),
    });

    let msg = tokio::time::timeout(Duration::from_millis(100), read.next())
        .await
        .expect("No message received")
        .expect("Error reading message")
        .expect("No message in time");

    match msg {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            let expected = get_current_laptime_msg(Duration::from_millis(1), "lap_finished");
            let msg = serde_json::from_slice::<serde_json::Value>(text.as_bytes()).unwrap();
            assert_eq!(msg, expected, "Laptime message does not match expected");
        }
        _ => panic!("Unexpected message type received. Msg: {:?}", msg),
    }
    stop_module(&eb, &mut rest).await;
}

#[tokio::test]
#[test_log::test]
#[serial]
async fn test_sector_finished() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());

    let (ws_stream, _) = connect_async("ws://localhost:27015/v1/live_session")
        .await
        .expect("Failed to connect to WebSocket");
    let (_, mut read) = ws_stream.split();
    eb.publish(&Event {
        kind: EventKind::SectorFinshedEvent(Duration::from_millis(1).into()),
    });

    let msg = tokio::time::timeout(Duration::from_millis(100), read.next())
        .await
        .expect("No message received")
        .expect("Error reading message")
        .expect("No message in time");

    match msg {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            let expected = get_current_laptime_msg(Duration::from_millis(1), "sector_finished");
            let msg = serde_json::from_slice::<serde_json::Value>(text.as_bytes()).unwrap();
            assert_eq!(msg, expected, "Laptime message does not match expected");
        }
        _ => panic!("Unexpected message type received. Msg: {:?}", msg),
    }
    stop_module(&eb, &mut rest).await;
}

#[tokio::test]
#[test_log::test]
#[serial]
async fn test_lap_started_event() {
    let eb = EventBus::default();
    let mut rest = create_module(eb.context());

    let (ws_stream, _) = connect_async("ws://localhost:27015/v1/live_session")
        .await
        .expect("Failed to connect to WebSocket");
    let (_, mut read) = ws_stream.split();
    eb.publish(&Event {
        kind: EventKind::LapStartedEvent,
    });

    let msg = tokio::time::timeout(Duration::from_millis(100), read.next())
        .await
        .expect("No message received")
        .expect("Error reading message")
        .expect("No message in time");

    match msg {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            let expected = get_lap_started_msg();
            let msg = serde_json::from_slice::<serde_json::Value>(text.as_bytes()).unwrap();
            assert_eq!(msg, expected, "Laptime message does not match expected");
        }
        _ => panic!("Unexpected message type received. Msg: {:?}", msg),
    }
    stop_module(&eb, &mut rest).await;
}
