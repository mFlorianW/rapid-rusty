// SPDX-FileCopyrightText: 2026 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use crate::RestCtx;
use crate::rocket::futures::StreamExt;
use crate::rocket::futures::TryStreamExt;
use common::serde::duration;
use module_core::EventKind;
use rocket::State;
use rocket_ws::Message;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
struct LaptimeEvent<'a> {
    event: &'a str,
    data: LaptimeData<'a>,
}

#[derive(Serialize)]
struct LaptimeData<'a> {
    #[serde(with = "duration")]
    time: &'a std::time::Duration,
}

#[derive(Serialize)]
struct EmptyEvent<'a> {
    event: &'a str,
    data: serde_json::Value,
}

/// Serializes a laptime event payload to a JSON string.
///
/// Constructs a `LaptimeEvent` with the provided event name and lap time and
/// returns its JSON representation.
///
/// Arguments:
/// - laptime: Lap time duration to include in the payload.
/// - event: Event identifier/name.
///
/// Returns the JSON string for `LaptimeEvent`.
fn serialize_laptime_event(laptime: &std::time::Duration, event: &str) -> String {
    let event = LaptimeEvent {
        event,
        data: LaptimeData { time: laptime },
    };
    match serde_json::to_string(&event) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize laptime event: {}", e);
            "{}".to_string()
        }
    }
}

/// Serialize an empty event into a JSON string.
///
/// Creates an `EmptyEvent` with the provided `event` name and an empty `data` object,
/// then serializes it into a compact JSON string.
///
/// Arguments:
/// - `event`: Name or type of the event.
///
/// Returns:
/// - JSON string representing the empty event.
fn serialize_empty_event(event: &str) -> String {
    let event = EmptyEvent {
        event,
        data: serde_json::Value::Object(serde_json::Map::new()),
    };
    match serde_json::to_string(&event) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize laptime event: {}", e);
            "{}".to_string()
        }
    }
}

/// WebSocket handler that streams live session updates to clients.
///
/// Route: GET /v1/live_session
/// Subscribes to the server event bus and forwards JSON messages.
/// Sends "current_laptime, lap_started," events as Message::Text and terminates on QuitEvent,
/// client close, or errors.
///
/// Params:
/// - ws: Upgraded WebSocket connection.
/// - ctx: Shared RestCtx state for accessing the event receiver.
///
/// Returns a rocket_ws::Stream that yields WebSocket messages.
#[get("/v1/live_session")]
pub(crate) fn ws_live_session_handler(
    ws: rocket_ws::WebSocket,
    ctx: &State<Arc<Mutex<RestCtx>>>,
) -> rocket_ws::Stream! ['static] {
    let ctx = ctx.inner().clone();
    rocket_ws::Stream! { ws =>
        let ctx = ctx.clone();
        let mut stream_ws = ws.into_stream();

        let mut event_receiver = {
            let guard = ctx.lock().await;
            guard.ctx.receiver.resubscribe()
        };

        loop {
            tokio::select!{
                event = event_receiver.recv() => {
                    match event {
                        Ok(event) => {
                            match event.kind {
                                EventKind::QuitEvent => {
                                    info!("Shutting down WebSocket live session handler due to QuitEvent");
                                    break;
                                }
                                EventKind::CurrentLaptimeEvent(laptime) => {
                                    yield Message::Text(serialize_laptime_event(&laptime, "current_laptime"));
                                }
                                EventKind::LapStartedEvent => {
                                    yield Message::Text(serialize_empty_event("lap_started"));
                                }
                                EventKind::LapFinishedEvent (laptimer)=> {
                                    yield Message::Text(serialize_laptime_event(&laptimer, "lap_finished"));
                                }
                                EventKind::SectorFinshedEvent(sector) => {
                                    yield Message::Text(serialize_laptime_event(&sector, "sector_finished"));
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            error!("Error receiving event in WebSocket live session handler: {}", e);
                            break;
                        }
                    }
                }

                Some(msg) = stream_ws.next() => {
                    match msg {
                        Ok(Message::Close(_)) => {
                            info!("WebSocket client disconnected from live session");
                            break;
                        }
                        Ok(_) => {
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    }
}
