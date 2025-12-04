// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use async_trait::async_trait;
use common::session::Session;
use module_core::{Event, EventKind, EventKindType, Module, ModuleCtx, Request, payload_ref};
use rocket::{
    State,
    response::content,
    serde::{Serialize, json::Json},
};
use std::{
    env,
    net::Ipv4Addr,
    sync::{Arc, RwLock},
};
use tokio::sync::Mutex;
#[macro_use]
extern crate rocket;

/// Represents the REST module, providing RESTful API functionality.
///
/// This struct encapsulates the shared context and methods for managing the REST server.
pub struct Rest {
    ctx: Arc<Mutex<RestCtx>>,
}

/// Internal context for the REST module.
///
/// Holds shared state and configuration required for RESTful operations.
struct RestCtx {
    ctx: ModuleCtx,
    module_addr: u64,
    request_id: u64,
}

impl RestCtx {
    /// Generates and returns a unique request identifier.
    ///
    /// Increments the internal request counter and returns its previous value.
    /// This is used to assign unique IDs to outgoing requests.
    fn request_id(&mut self) -> u64 {
        let id = self.request_id;
        self.request_id += 1;
        id
    }
}

impl Rest {
    /// Creates a new `Rest` instance.
    ///
    /// Initializes the REST API handler with the provided shared REST context.
    ///
    /// # Arguments
    /// * `ctx` - Shared REST context for managing server state and communication.
    ///
    /// # Returns
    /// A new `Rest` instance.
    pub fn new(ctx: ModuleCtx) -> Self {
        Rest {
            ctx: Arc::new(Mutex::new(RestCtx {
                ctx,
                module_addr: 0xff,
                request_id: 0,
            })),
        }
    }
}

#[async_trait]
impl Module for Rest {
    /// Runs the REST server in a separate asynchronous task.
    ///
    /// This function launches the REST server using Rocket in a background task, allowing it to
    /// handle incoming HTTP requests concurrently with other application logic. It also waits for
    /// a quit event to gracefully shut down the server when requested.
    ///
    /// # Arguments
    /// * `ctx` - Shared context required for server operation.
    ///
    /// # Returns
    /// An asynchronous task handle for the running REST server.
    async fn run(&mut self) -> Result<(), ()> {
        let ctx = self.ctx.clone();
        let rocket = match launch_rest_server(ctx.clone()).await {
            Ok(rocket) => rocket,
            Err(e) => {
                error!("Failed to launch REST server: {}", e);
                return Err(());
            }
        };
        let shutdown = rocket.shutdown();
        let server_handle = tokio::spawn(async move {
            if let Err(e) = rocket.launch().await {
                error!("Rocket server failed: {}", e);
            } else {
                info!("Rocket server terminated gracefully.");
            }
        });

        let lock_guard = self.ctx.lock().await;
        let mut receiver = lock_guard.ctx.receiver.resubscribe();
        drop(lock_guard);

        loop {
            let event = receiver.recv().await;
            match event {
                Ok(event) => {
                    if let EventKind::QuitEvent = event.kind {
                        info!("Shutting down REST module and server.");
                        shutdown.notify();
                        tokio::join!(server_handle)
                            .0
                            .map_err(|e| error!("Error while shutting down server: {}", e))?;
                        break;
                    }
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }
        Ok(())
    }
}

/// Requests session IDs from the session storage and waits for the response.
///
/// This asynchronous function sends a request to load stored session IDs using the provided context,
/// then waits for the corresponding response. It returns the session IDs as a vector of strings.
///
/// # Arguments
/// * `ctx` - Shared context containing the event sender and receiver.
///
/// # Returns
/// * `Vec<String>` - The received session IDs.
async fn request_session_ids(ctx: &Arc<Mutex<RestCtx>>) -> Arc<Vec<String>> {
    let mut ctx_lock = ctx.lock().await;
    let req_id = ctx_lock.request_id();
    let addr = ctx_lock.module_addr;
    let _ = ctx_lock.ctx.sender.send(Event {
        kind: EventKind::LoadStoredSessionIdsRequestEvent(
            Request {
                sender_addr: ctx_lock.module_addr,
                id: req_id,
                data: (),
            }
            .into(),
        ),
    });
    if ctx_lock
        .ctx
        .publish_event(EventKind::LoadStoredSessionIdsRequestEvent(Request::new(
            ctx_lock.module_addr,
            req_id,
            (),
        )))
        .is_err()
    {
        error!("Failed to publish LoadStoredSessionIdsRequestEvent");
        Arc::new(Vec::<String>::new());
    }
    debug!("Sent LoadStoredSessionIdsRequestEvent with id {}", req_id);
    match ctx_lock
        .ctx
        .wait_for_event(
            req_id,
            addr,
            &EventKindType::LoadStoredSessionIdsResponseEvent,
        )
        .await
    {
        Ok(event) => match payload_ref!(event.kind, EventKind::LoadStoredSessionIdsResponseEvent) {
            Some(resp) => resp.data.clone(),
            None => {
                error!("Received invalid LoadStoredSessionIdsResponseEvent payload");
                Arc::new(Vec::<String>::new())
            }
        },
        Err(e) => {
            error!(
                "Error while waiting for LoadStoredSessionIdsResponseEvent: {:?}",
                e
            );
            Arc::new(Vec::<String>::new())
        }
    }
}

/// Response structure for listing session IDs.
///
/// Contains a vector of session ID strings returned by the REST API.
#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct SessionIdsResponse {
    total: usize,
    ids: Vec<String>,
}

/// Retrieves all stored session IDs.
///
/// # Arguments
/// * `ctx` - Shared context containing the event sender and receiver.
///
/// # Returns
/// * `SessionIdsResponse` - A JSON object containing the total number of sessions and a list of session IDs.
#[get("/v1/sessions")]
async fn get_session_ids(ctx: &State<Arc<Mutex<RestCtx>>>) -> Json<SessionIdsResponse> {
    let ids = request_session_ids(ctx).await;
    let resp = SessionIdsResponse {
        total: ids.len(),
        ids: (*ids).clone(),
    };
    Json(resp)
}

/// Sends a request to load a session by its ID and waits for the response.
///
/// This asynchronous function sends a `LoadSessionRequestEvent` to the event bus using the provided context,
/// then waits for the corresponding response. It returns the loaded session wrapped in an `Arc<RwLock<Session>>`
/// on success, or an `std::io::ErrorKind` on failure.
///
/// # Arguments
/// * `id` - The session ID to load.
/// * `ctx` - Shared context containing the event sender and receiver.
///
/// # Returns
/// * `Result<Arc<RwLock<Session>>, std::io::ErrorKind>` - The loaded session or an error.
async fn request_session(
    id: &str,
    ctx: &Arc<Mutex<RestCtx>>,
) -> Result<Arc<RwLock<Session>>, std::io::ErrorKind> {
    let mut ctx_lock = ctx.lock().await;
    let req_id = ctx_lock.request_id();
    let addr = ctx_lock.module_addr;
    let _ = ctx_lock.ctx.sender.send(Event {
        kind: EventKind::LoadSessionRequestEvent(
            Request {
                sender_addr: ctx_lock.module_addr,
                id: req_id,
                data: id.to_string(),
            }
            .into(),
        ),
    });
    debug!("Sent LoadSessionRequestEvent with id {}", req_id);
    match ctx_lock
        .ctx
        .wait_for_event(req_id, addr, &EventKindType::LoadSessionResponseEvent)
        .await
    {
        Ok(event) => match payload_ref!(event.kind, EventKind::LoadSessionResponseEvent) {
            Some(resp) => resp.data.clone(),
            None => {
                error!("Received invalid LoadSessionResponseEvent payload");
                Err(std::io::ErrorKind::InvalidData)
            }
        },
        Err(e) => {
            error!("Error while waiting for LoadSessionResponseEvent: {:?}", e);
            Err(std::io::ErrorKind::TimedOut)
        }
    }
}

/// Retrieves a session by its ID from the event bus.
///
/// This asynchronous REST API handler sends a request to load a session with the specified ID,
/// then waits for the response. It returns the loaded session wrapped in an `Arc<RwLock<Session>>`
/// on success, or an error if the session could not be retrieved.
///
/// # Arguments
/// * `id` - The session ID to retrieve.
/// * `ctx` - Shared context containing the event sender and receiver.
///
/// # Returns
/// * `Result<Arc<RwLock<Session>>, std::io::ErrorKind>` - The loaded session or an error.
#[get("/v1/sessions/<id>")]
async fn get_session(
    id: &str,
    ctx: &State<Arc<Mutex<RestCtx>>>,
) -> Option<content::RawJson<String>> {
    let session = request_session(id, ctx).await;
    match &session {
        Ok(session_lock) => {
            let session_guard = match session_lock.read() {
                Ok(guard) => guard,
                Err(e) => {
                    error!("Failed to acquire read lock on session {}: {}", id, e);
                    return None;
                }
            };
            Session::to_json(&session_guard).map_or_else(
                |e| {
                    error!("Failed to serialize session to JSON: {}", e);
                    None
                },
                |json| Some(content::RawJson(json)),
            )
        }
        Err(e) => {
            error!("Failed to load session {}: {:?}", id, e);
            None
        }
    }
}

/// Delete a session identified by `id`.
///
/// Route: DELETE /v1/sessions/<id>
///
/// Sends a DeleteSessionRequestEvent to the backend and waits
/// for a matching DeleteSessionResponseEvent. On success returns Ok(()),
/// otherwise returns InternalServerError.
///
/// Parameters:
/// - id: Path parameter identifying the session to delete.
/// - ctx: Shared RestCtx wrapped in Rocket State + Arc<Mutex<_>>.
///
/// Errors:
/// - Returns InternalServerError if waiting for the response fails or
///   the received event payload is invalid.
#[delete("/v1/sessions/<id>")]
async fn delete_session(
    id: &str,
    ctx: &State<Arc<Mutex<RestCtx>>>,
) -> Result<(), rocket::http::Status> {
    let mut ctx_lock = ctx.lock().await;
    let req_id = ctx_lock.request_id();
    let addr = ctx_lock.module_addr;
    let _ = ctx_lock.ctx.sender.send(Event {
        kind: EventKind::DeleteSessionRequestEvent(
            Request {
                sender_addr: ctx_lock.module_addr,
                id: req_id,
                data: id.to_string(),
            }
            .into(),
        ),
    });
    debug!("Sent DeleteSessionRequestEvent with id {}", req_id);
    match ctx_lock
        .ctx
        .wait_for_event(req_id, addr, &EventKindType::DeleteSessionResponseEvent)
        .await
    {
        Ok(event) => match payload_ref!(event.kind, EventKind::DeleteSessionResponseEvent) {
            Some(_) => {
                debug!("Session {} deleted successfully", id);
                Ok(())
            }
            None => {
                error!("Received invalid DeleteSessionResponseEvent payload");
                Err(rocket::http::Status::InternalServerError)
            }
        },
        Err(e) => {
            error!(
                "Error while waiting for DeleteSessionResponseEvent: {:?}",
                e
            );
            Err(rocket::http::Status::InternalServerError)
        }
    }
}

/// The default port used for the REST server.
static DEFAULT_PORT: u16 = 27015;

/// Launches and configures the REST server.
///
/// This function sets up the Rocket server with address and port from environment variables,
/// or uses defaults if not provided. It configures logging and color settings, and mounts
/// the session endpoint.
///
/// # Returns
/// A configured instance of `rocket::Rocket<rocket::Build>`.
async fn launch_rest_server(
    ctx: Arc<Mutex<RestCtx>>,
) -> Result<rocket::Rocket<rocket::Ignite>, rocket::Error> {
    // TODO: Change this when introducing the whole configuration concept.
    // Then this should be started after the configuration is loaded from the configuration module.
    let address = env::var("ROCKET_ADDRESS").unwrap_or(Ipv4Addr::LOCALHOST.to_string());
    let port = match env::var("ROCKET_PORT") {
        Ok(port_str) => port_str.parse::<u16>().unwrap_or(DEFAULT_PORT),
        Err(_) => DEFAULT_PORT,
    };
    let figment = rocket::Config::figment()
        .merge(("address", address))
        .merge(("port", port))
        .merge(("log_level", "critical"))
        .merge(("cli_colors", false));

    rocket::custom(figment)
        .mount(
            "/",
            rocket::routes![get_session_ids, get_session, delete_session],
        )
        .manage(ctx)
        .ignite()
        .await
}
