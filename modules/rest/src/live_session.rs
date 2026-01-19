// SPDX-FileCopyrightText: 2026 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use crate::RestCtx;
use crate::rocket::futures::StreamExt;
use crate::rocket::futures::TryStreamExt;
use common::serde::duration;
use common::session::Session;
use module_core::EventKind;
use module_core::EventKindType;
use module_core::Request;
use module_core::payload_ref;
use rand::{Rng, distr::Alphanumeric, rng};
use rocket::State;
use rocket_ws::Message;
use serde::Serialize;
use std::sync::Arc;
use std::sync::RwLock;
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
struct CurrentSessionEvent<'a> {
    event: &'a str,
    data: CurrentSessionData<'a>,
}

#[derive(Serialize)]
struct CurrentSessionData<'a> {
    session: &'a Session,
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

/// Serializes the current session event into a JSON string.
/// Constructs a `CurrentSessionEvent` with the provided session and
///
/// returns its JSON representation.
fn serialize_current_session_event(session: &Arc<RwLock<Session>>) -> String {
    let session = session.read().unwrap_or_else(|s| s.into_inner());
    let event = CurrentSessionEvent {
        event: "current_session",
        data: CurrentSessionData { session: &session },
    };
    match serde_json::to_string(&event) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize current session event: {}", e);
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
        let session_id = generate_connection_id();

        let mut event_receiver = {
            let guard = ctx.lock().await;
            guard.ctx.receiver.resubscribe()
        };

        ctx.lock().await.register_connection(&session_id);
        info!("WebSocket \"/v1/live_session\" connection established with session_id: {}", session_id);

        match request_current_session(&ctx).await {
            Ok(session_ptr) => {
                yield Message::Text(serialize_current_session_event(&session_ptr));
                ctx.lock().await.set_connection_synced(&session_id, true);
            }
            Err(e) => {
                error!("Error requesting current session for WebSocket live session initial state: {:?}", e);
            }
        }

        loop {
            tokio::select!{
                event = event_receiver.recv() => {
                    match event {
                        Ok(event) => {
                            match event.kind {
                                EventKind::QuitEvent => {
                                    ctx.lock().await.unregister_connection(&session_id);
                                    info!("Shutting down WebSocket live session handler due to QuitEvent");
                                    break;
                                }
                                EventKind::CurrentLaptimeEvent(laptime) => {
                                    if ctx.lock().await.is_connection_synced(&session_id) {
                                        yield Message::Text(serialize_laptime_event(&laptime, "current_laptime"));
                                    }
                                }
                                EventKind::LapStartedEvent => {
                                    if ctx.lock().await.is_connection_synced(&session_id) {
                                        yield Message::Text(serialize_empty_event("lap_started"));
                                    }else{
                                        match request_current_session(&ctx).await {
                                            Ok(session_ptr) => {
                                                yield Message::Text(serialize_current_session_event(&session_ptr));
                                                ctx.lock().await.set_connection_synced(&session_id, true);
                                            }
                                            Err(e) => {
                                                error!("Error requesting current session for WebSocket live session sync: {:?}", e);
                                                break;
                                            }
                                        }
                                    }
                                }
                                EventKind::LapFinishedEvent (laptimer)=> {
                                    if ctx.lock().await.is_connection_synced(&session_id) {
                                        yield Message::Text(serialize_laptime_event(&laptimer, "lap_finished"));
                                    }
                                }
                                EventKind::SectorFinishedEvent(sector) => {
                                    if ctx.lock().await.is_connection_synced(&session_id) {
                                        yield Message::Text(serialize_laptime_event(&sector, "sector_finished"));
                                    }
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            ctx.lock().await.unregister_connection(&session_id);
                            error!("Error receiving event in WebSocket live session handler: {}", e);
                            break;
                        }
                    }
                }

                Some(msg) = stream_ws.next() => {
                    match msg {
                        Ok(Message::Close(_)) => {
                            ctx.lock().await.unregister_connection(&session_id);
                            info!("WebSocket client disconnected from live session");
                            break;
                        }
                        Ok(_) => {
                        }
                        Err(e) => {
                            ctx.lock().await.unregister_connection(&session_id);
                            error!("WebSocket error: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Generates a random connection ID.
///
/// This function creates a random alphanumeric string of length 16,
/// which can be used as a unique identifier for connections.
///
/// # Returns
/// A randomly generated connection ID as a `String`.
pub fn generate_connection_id() -> String {
    let id: String = rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    id
}

/// Requests the current session from the event bus.
///
/// Sends a CurrentSessionRequestEvent and waits for the corresponding response.
///
/// Returns the current session wrapped in an Arc<RwLock<Session>> on success.
/// Returns an std::io::ErrorKind on failure.
///
/// Arguments:
/// - ctx: Shared RestCtx for publishing and receiving events.
///
/// Return:
/// - Ok(Arc<RwLock<Session>>) on success.
/// - Err(std::io::ErrorKind) on failure.
async fn request_current_session(
    ctx: &Arc<Mutex<RestCtx>>,
) -> Result<Arc<RwLock<Session>>, std::io::ErrorKind> {
    let mut ctx = ctx.lock().await;
    let req_id = ctx.request_id();
    let _ = ctx.ctx.publish_event(EventKind::CurrentSessionRequestEvent(
        Request {
            id: req_id,
            sender_addr: ctx.module_addr,
            data: (),
        }
        .into(),
    ));
    info!(
        "Published CurrentSessionRequestEvent with req_id: {}, addr {:?}",
        req_id, ctx.module_addr
    );
    let addr = ctx.module_addr;
    match ctx
        .ctx
        .wait_for_event(req_id, addr, &EventKindType::CurrentSessionResponseEvent)
        .await
    {
        Ok(event) => {
            let session = match payload_ref!(event.kind, EventKind::CurrentSessionResponseEvent) {
                Some(response) => response.data.clone(),
                None => {
                    error!("Received session doesn't have a payload");
                    return Err(std::io::ErrorKind::InvalidData);
                }
            };
            match session {
                Some(session) => Ok(session),
                None => {
                    error!("Received session data is None");
                    Err(std::io::ErrorKind::InvalidData)
                }
            }
        }
        Err(e) => {
            error!("Error waiting for CurrentSessionResponseEvent: {:?}", e);
            Err(std::io::ErrorKind::TimedOut)
        }
    }
}
