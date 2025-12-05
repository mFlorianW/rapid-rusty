// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use common::{
    session::{Session, SessionInfo},
    track::Track,
};
use std::{
    io::ErrorKind,
    sync::{
        Arc, RwLock,
        atomic::{self, AtomicUsize},
    },
};
use strum_macros::EnumDiscriminants;
use tokio::time::timeout;
use tracing::info;

/// Represents a high-level event in the system.
///
/// Each `Event` wraps an [`EventKind`], which defines the actual type
/// and data carried by the event.
///
/// This structure is designed to be passed through an [`EventBus`]
/// between asynchronous modules.
#[derive(Clone, Debug)]
pub struct Event {
    /// The inner event type and associated data.
    pub kind: EventKind,
}

impl Event {
    /// Returns the high-level type of this event.
    ///
    /// This converts the event's specific `kind` into an `EventKindType`,
    /// which is useful for grouping or filtering events by category.
    pub fn event_type(&self) -> EventKindType {
        EventKindType::from(&self.kind)
    }

    /// Returns the correlation ID carried by the event, if any.
    ///
    /// - For request events, this is the `id` from the request payload.
    /// - For response events, this is the `id` from the response payload.
    /// - For events without a correlation ID, returns `None`.
    pub fn id(&self) -> Option<u64> {
        match &self.kind {
            EventKind::LoadStoredSessionIdsRequestEvent(req)
            | EventKind::LoadStoredTrackIdsRequest(req)
            | EventKind::LoadAllStoredTracksRequestEvent(req)
            | EventKind::DetectTrackRequestEvent(req) => Some(req.id),
            EventKind::SaveSessionRequestEvent(req) => Some(req.id),
            EventKind::LoadSessionRequestEvent(req) => Some(req.id),
            EventKind::DeleteSessionRequestEvent(req) => Some(req.id),
            EventKind::LoadStoredSessionIdsResponseEvent(res) => Some(res.id),
            EventKind::SaveSessionResponseEvent(res) => Some(res.id),
            EventKind::LoadSessionResponseEvent(res) => Some(res.id),
            EventKind::DeleteSessionResponseEvent(res) => Some(res.id),
            EventKind::LoadStoredTrackIdsResponseEvent(res) => Some(res.id),
            EventKind::LoadAllStoredTracksResponseEvent(res) => Some(res.id),
            EventKind::DetectTrackResponseEvent(res) => Some(res.id),
            _ => None,
        }
    }

    /// Returns the logical address associated with the event, if available.
    ///
    /// - For request events, returns the `sender_addr`.
    /// - For response events, returns the `receiver_addr`.
    /// - For events without an address, returns `None`.
    pub fn addr(&self) -> Option<u64> {
        match &self.kind {
            EventKind::LoadStoredSessionIdsRequestEvent(req) => Some(req.sender_addr),
            EventKind::SaveSessionRequestEvent(req) => Some(req.sender_addr),
            EventKind::LoadSessionRequestEvent(req) => Some(req.sender_addr),
            EventKind::DeleteSessionRequestEvent(req) => Some(req.sender_addr),
            EventKind::LoadStoredTrackIdsRequest(req)
            | EventKind::LoadAllStoredTracksRequestEvent(req)
            | EventKind::DetectTrackRequestEvent(req) => Some(req.sender_addr),
            EventKind::LoadStoredSessionIdsResponseEvent(res) => Some(res.receiver_addr),
            EventKind::SaveSessionResponseEvent(res) => Some(res.receiver_addr),
            EventKind::LoadSessionResponseEvent(res) => Some(res.receiver_addr),
            EventKind::DeleteSessionResponseEvent(res) => Some(res.receiver_addr),
            EventKind::LoadStoredTrackIdsResponseEvent(res) => Some(res.receiver_addr),
            EventKind::LoadAllStoredTracksResponseEvent(res) => Some(res.receiver_addr),
            EventKind::DetectTrackResponseEvent(res) => Some(res.receiver_addr),
            _ => None,
        }
    }
}

/// Represents a generic request message.
///
/// # Fields
/// - `id`: A unique identifier for the request.  Used to correlate
///   requests with responses.
/// - `sender_address`: An identifier for the sender (e.g., node ID, IP address
///   as `u32`, or similar). Allows the receiver to know where the request came from.
/// - `data`: The payload or content of the request. The type `T` is generic
///   so that `Request` can carry any kind of data.//
///
/// # Type Parameters
/// - `T`: The type of the request payload.
#[derive(Debug, Clone)]
pub struct Request<T = ()> {
    pub id: u64,
    pub sender_addr: u64,
    pub data: T,
}

impl<T> Request<T> {
    /// Constructs a new `Request` with the given metadata and payload.
    ///
    /// - `id`: Correlation identifier used to match responses.
    /// - `sender_addr`: Logical address of the sender.
    /// - `data`: Payload carried by the request.
    ///
    /// Returns a `Request<T>` wrapping `data`.
    pub fn new(id: u64, sender_addr: u64, data: T) -> Arc<Self> {
        Arc::new(Request {
            id,
            sender_addr,
            data,
        })
    }
}

impl Request {
    /// Creates a request with an empty payload (`()`).
    ///
    /// Use for control or signal messages that only need a correlation `id` and the sender's address.
    /// - `id`: Correlation identifier for the request.
    /// - `sender_addr`: Logical address of the sender.
    ///
    /// Returns a `Request<()>` carrying no data.
    pub fn empty_request(id: u64, sender_addr: u64) -> Arc<Self> {
        Arc::new(Request {
            id,
            sender_addr,
            data: (),
        })
    }
}

/// Represents a generic response message.
///
/// # Fields
/// - `id`: A unique identifier for the request.  Used to correlate
///   requests with responses.
/// - `sender_address`: An identifier for the sender (e.g., node ID, IP address
///   as `u32`, or similar). Allows the receiver to know where the request came from.
/// - `data`: The payload or content of the request. The type `T` is generic
///   so that `Request` can carry any kind of data.//
///
/// # Type Parameters
/// - `T`: The type of the request payload.
#[derive(Debug, Clone)]
pub struct Response<T = ()> {
    pub id: u64,
    pub receiver_addr: u64,
    pub data: T,
}

impl<T> Response<T> {
    /// Constructs a new `Response` with the given metadata and payload.
    ///
    /// - `id`: Correlation identifier used to match responses.
    /// - `receiver_addr`: Logical address of the receiver.
    /// - `data`: Payload carried by the response.
    ///
    /// Returns a `Response<T>` wrapping `data`.
    pub fn new(id: u64, receiver_addr: u64, data: T) -> Arc<Self> {
        Arc::new(Response {
            id,
            receiver_addr,
            data,
        })
    }
}

/// A thread-safe, reference-counted pointer to a [`GnssPosition`].
///
/// This type alias wraps a [`GnssPosition`] inside an [`Arc`], allowing
/// multiple parts of the program (or multiple modules) to share ownership
/// of the same GNSS position data without copying it.
pub type GnssPositionPtr = Arc<common::position::GnssPosition>;

/// A thread-safe shared reference-counted pointer to a [`GnssInformation`].
///
/// This type alias wraps a [`GnssInformation`] instance in an [`Arc`],
/// multiple parts of the program (or multiple modules) to share ownership
/// of the same GNSS information data without copying it.
pub type GnssInformationPtr = Arc<common::position::GnssInformation>;

/// A thread-safe, shared pointer to an std::time::duration.
pub type DurationPtr = Arc<std::time::Duration>;

/// A thread-safe, shared pointer to an empty request.
pub type EmptyRequestPtr = Arc<Request<()>>;

/// A thread-safe, shared response containing stored session identifiers.
pub type StoredSessionIdsResponsePtr = Arc<Response<Arc<Vec<SessionInfo>>>>;

/// A thread-safe, shared pointer to a save session request.
pub type SaveSessionRequestPtr = Arc<Request<Arc<RwLock<Session>>>>;

/// A thread-safe, shared pointer to a save session response.
pub type SaveSessionResponsePtr = Arc<Response<Result<String, ErrorKind>>>;

/// A thread-safe, shared pointer to a load session request.
pub type LoadSessionRequestPtr = Arc<Request<String>>;

/// A thread-safe, shared pointer to a load session response.
pub type LoadSessionResponsePtr = Arc<Response<Result<Arc<RwLock<Session>>, ErrorKind>>>;

/// A thread-safe, shared pointer to a delete session request.
pub type DeleteSessionRequestPtr = Arc<Request<String>>;

/// A thread-safe, shared pointer to a delete session response.
pub type DeleteSessionResponsePtr = Arc<Response<Result<(), ErrorKind>>>;

/// A thread-safe, shared pointer to a load stored track ids request.
pub type LoadStoredTrackIdsResponsePtr = Arc<Response<Vec<String>>>;

/// A thread-safe shared pointer to a load all stored tracks request.
pub type LoadStoredTracksReponsePtr = Arc<Response<Vec<Track>>>;

/// A thread-safe shared pointer to a track detection request.
pub type TrackDetectionResponsePtr = Arc<Response<Vec<Track>>>;

/// Generic helper macro to extract enum payloads
#[macro_export]
macro_rules! payload_ref {
    ($enum_val:expr, $pattern:path) => {
        if let $pattern(ref payload) = $enum_val {
            Some(payload)
        } else {
            None
        }
    };
}

/// Enumerates the different kinds of events that can be emitted
/// and transmitted via the [`EventBus`].
#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
#[strum_discriminants(name(EventKindType))]
pub enum EventKind {
    /// Indicates that a module shall terminate.
    QuitEvent,

    /// A GNSS (Global Navigation Satellite System) position update.
    ///
    /// This event carries a [`common::position::GnssPosition`] structure
    /// with the current geolocation and related information.
    GnssPositionEvent(GnssPositionPtr),

    /// A GNSS (Global Navigation Satellite System) position update.
    ///
    /// This event carries a [`common::position::GnssInformation`] structure
    /// with the current information of the navigation system.
    GnssInformationEvent(GnssInformationPtr),

    /// Indicates that a new lap has started.
    LapStartedEvent,

    /// Indicates that a lap has finished.
    /// This event carries a [`std:time::Duration`] structure
    LapFinishedEvent(DurationPtr),

    /// Indicates that a sector has been completed.
    /// This event carries a [`std:time::Duration`] structure
    SectorFinshedEvent(DurationPtr),

    /// Represents the current laptime (may be used for reporting purposes).
    /// This event carries a [`std:time::Duration`] structure.
    CurrentLaptimeEvent(DurationPtr),

    /// Requests the list of all stored session identifiers.
    /// This event variant carries a [`EmptyRequestPtr`] with no payload (`()`),
    /// signaling that the sender is asking for all session IDs currently stored or available in persistent storage.
    LoadStoredSessionIdsRequestEvent(EmptyRequestPtr),

    /// Returns the list of stored session identifiers in response to a [`RequestStoredSessionIdsEvent`].
    LoadStoredSessionIdsResponseEvent(StoredSessionIdsResponsePtr),

    /// Request to store a session in the persistent storage.
    /// This event variant carries a [`SaveSessionRequestPtr`] with payload (`Arc<RwLock<Session>`).
    SaveSessionRequestEvent(SaveSessionRequestPtr),

    /// Response to store a session request in the persistent storage.
    /// This event variant carries a [`SaveSessionResponsePtr`] with payload (`Result<String, std::io::ErrorKind>`).
    /// The string is the ID under which the session was stored.
    SaveSessionResponseEvent(SaveSessionResponsePtr),

    /// Request to store a session in the persistent storage.
    /// This event variant carries a [`LoadSessionRequestPtr`] with payload (`String`).
    /// The string is the ID of the session that shall be loaded.
    LoadSessionRequestEvent(LoadSessionRequestPtr),

    /// Response to store a session request in the persistent storage.
    /// This event variant carries a [`SaveSessionResponsePtr`] with payload (`Result<RwLock<Session>, std::io::ErrorKind>`).
    LoadSessionResponseEvent(LoadSessionResponsePtr),

    /// Request to store a session in the persistent storage.
    /// This event variant carries a [`DeleteSessionRequestPtr`] with payload (`String`).
    /// The string is the ID of the session that shall be deleted.
    DeleteSessionRequestEvent(DeleteSessionRequestPtr),

    /// Response to store a session request in the persistent storage.
    /// This event variant carries a [`SaveSessionResponsePtr`] with payload (`Result<(), std::io::ErrorKind>`).
    DeleteSessionResponseEvent(DeleteSessionResponsePtr),

    /// Request to load all stored track ids in the persistent storage.
    /// This event variant carries a [`EmptyRequestPtr`].
    LoadStoredTrackIdsRequest(EmptyRequestPtr),

    /// Reponse to load all stored track ids in the persistent storage.
    /// This event variant carries a [`Vec<String>`].
    /// The vector contains all track ids found in the persistent storage.
    LoadStoredTrackIdsResponseEvent(LoadStoredTrackIdsResponsePtr),

    /// Request to load all stored tracks in the persistent storage.
    /// This event variant carries a [`EmptyRequestPtr`].
    LoadAllStoredTracksRequestEvent(EmptyRequestPtr),

    /// Reponse to load all stored track ids in the persistent storage.
    /// This event variant carries a [`Vec<String>`].
    /// The vector contains all tracks found in the persistent storage.
    LoadAllStoredTracksResponseEvent(LoadStoredTracksReponsePtr),

    /// Event carrying a request to start a track detection operation.
    /// Uses `EmptyRequestPtr` as a signal-only payload (no parameters).
    DetectTrackRequestEvent(EmptyRequestPtr),

    /// Event emitted after track detection finishes.
    /// Contains the `TrackDetectionResponsePtr` with detection results.
    DetectTrackResponseEvent(TrackDetectionResponsePtr),
}

/// A simple asynchronous event bus for publishing and subscribing to [`Event`]s.
///
/// The event bus uses a [`tokio::sync::broadcast::channel`] under the hood,
/// allowing multiple receivers to listen for the same stream of events.
///
/// Each published event is cloned and distributed to all active subscribers.
/// If no subscribers exist at the time of publication, the event is discarded silently.
pub struct EventBus {
    id: usize,
    /// The broadcast sender used internally to distribute events.
    sender: tokio::sync::broadcast::Sender<Event>,
}

/// Global counter used to assign unique, monotonically increasing IDs to bus instances.
/// Starts at 0 and is incremented atomically for thread-safe ID generation.
static BUS_ID: AtomicUsize = AtomicUsize::new(0);

impl EventBus {
    /// Creates a new [`EventBus`] with a fixed buffer capacity of 100 messages.
    ///
    /// When the buffer is full, the oldest messages are dropped automatically
    /// as new ones are published.
    pub fn new() -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(100);
        let id = BUS_ID.fetch_add(1, atomic::Ordering::Relaxed);
        info!("Creating EventBus with id {}", id);
        BUS_ID.store(id, atomic::Ordering::SeqCst);
        EventBus { id, sender }
    }

    /// Subscribes to the event bus and returns a [`tokio::sync::broadcast::Receiver`].
    ///
    /// The returned receiver will receive all future events published after the
    /// subscription is created.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Publishes an [`Event`] to all active subscribers.
    ///
    /// This method clones the event and attempts to send it to each receiver.
    /// If no subscribers exist, the event is discarded silently.
    ///
    /// # Arguments
    ///
    /// * `event` - The event instance to be published.
    pub fn publish(&self, event: &Event) {
        let _ = self.sender.send(event.clone());
    }

    /// Creates a [`ModuleCtx`] bound to this [`EventBus`].
    ///
    /// The returned context can be used by modules implementing [`Module`]
    /// to send and receive events within their execution scope.
    pub fn context(&self) -> ModuleCtx {
        ModuleCtx::new(self)
    }

    /// Returns the numeric identifier for this event bus.
    pub fn id(&self) -> usize {
        self.id
    }
}

/// Provides a default instance of [`EventBus`].
impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Defines the common interface for an asynchronous module
/// that can be executed and communicate via the [`EventBus`].
#[async_trait::async_trait]
pub trait Module {
    /// Runs the module asynchronously until completion.
    ///
    /// This function typically contains the module's main event loop,
    /// reacting to messages received through the [`ModuleCtx`].
    async fn run(&mut self) -> Result<(), ()>;
}

/// Provides a module-scoped context for interacting with the [`EventBus`].
///
/// Each `ModuleCtx` owns both a sender and a receiver, allowing the module
/// to both publish and listen for events concurrently.
#[derive(Debug)]
pub struct ModuleCtx {
    /// Unique identifier of the event bus that this context belongs to.
    id: usize,

    /// The broadcast sender used to publish events.
    pub sender: tokio::sync::broadcast::Sender<Event>,

    /// The broadcast receiver used to listen for events.
    pub receiver: tokio::sync::broadcast::Receiver<Event>,
}

#[derive(Debug)]
pub enum ModuleCtxError {
    PublishError(String),
    ReceiveError(String),
    ReceiveTimeout,
}

impl ModuleCtx {
    pub fn publish_event(&self, event: EventKind) -> Result<(), ModuleCtxError> {
        self.sender
            .send(Event { kind: event })
            .map(|_| ())
            .map_err(|e| ModuleCtxError::PublishError(format!("Failed to publish event: {}", e)))
    }

    pub async fn wait_for_event(
        &mut self,
        id: u64,
        addr: u64,
        response_type: &EventKindType,
    ) -> Result<Event, ModuleCtxError> {
        wait_for_event(self, id, addr, response_type).await
    }
}

impl Clone for ModuleCtx {
    fn clone(&self) -> Self {
        ModuleCtx {
            id: self.id,
            sender: self.sender.clone(),
            receiver: self.receiver.resubscribe(),
        }
    }
}

impl ModuleCtx {
    /// Constructs a new [`ModuleCtx`] from the given [`EventBus`].
    ///
    /// Clones the internal broadcast sender and creates a new receiver.
    /// ```
    pub(crate) fn new(event_bus: &EventBus) -> Self {
        ModuleCtx {
            id: event_bus.id(),
            sender: event_bus.sender.clone(),
            receiver: event_bus.subscribe(),
        }
    }

    /// Returns a new broadcast receiver subscribed to this event bus.
    ///
    /// This creates an independent subscription using `resubscribe()`. The
    /// returned receiver:
    /// - Only receives events published after this call (no replay).
    /// - Does not affect other receivers or advance any internal cursor.
    /// - May yield `tokio::sync::broadcast::error::RecvError::Lagged(_)`
    ///   if the consumer falls behind.
    pub fn receiver(&mut self) -> tokio::sync::broadcast::Receiver<Event> {
        self.receiver.resubscribe()
    }

    /// Returns the unique identifier of the event bus that this module context belongs to.
    /// The ID is stable for the lifetime of the context and can be used for logging.
    pub fn bus_id(&self) -> usize {
        self.id
    }
}

async fn wait_for_event(
    ctx: &mut ModuleCtx,
    id: u64,
    addr: u64,
    response_type: &EventKindType,
) -> Result<Event, ModuleCtxError> {
    let func = async move {
        loop {
            match ctx.receiver.recv().await {
                Ok(event) => {
                    if EventKindType::from(&event.kind) == *response_type
                        && event.id() == Some(id)
                        && event.addr() == Some(addr)
                    {
                        return Ok(event);
                    }
                }
                Err(e) => match e {
                    tokio::sync::broadcast::error::RecvError::Lagged(skipped) => {
                        info!(
                            "ModuleCtx (bus id {}) lagged behind, skipped {} messages",
                            ctx.id, skipped
                        );
                        continue;
                    }
                    _ => {
                        return Err(ModuleCtxError::ReceiveError(format!(
                            "Failed to receive event: {}",
                            e
                        )));
                    }
                },
            }
        }
    };
    timeout(std::time::Duration::from_secs(20), func)
        .await
        .map_err(|_| ModuleCtxError::ReceiveTimeout)?
}

pub mod test_helper;
