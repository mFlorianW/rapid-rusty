use common::{session::Session, track::Track};
use std::{io::ErrorKind, sync::Arc, sync::RwLock};

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
    pub fn kind_discriminant(&self) -> std::mem::Discriminant<EventKind> {
        std::mem::discriminant(&self.kind)
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
pub type StoredSessionIdsResponsePtr = Arc<Response<Arc<Vec<String>>>>;

/// A thread-safe, shared pointer to a save session request.
pub type SaveSessionRequestPtr = Arc<Request<RwLock<Session>>>;

/// A thread-safe, shared pointer to a save session response.
pub type SaveSessionResponsePtr = Arc<Response<Result<String, ErrorKind>>>;

/// A thread-safe, shared pointer to a load session request.
pub type LoadSessionRequestPtr = Arc<Request<String>>;

/// A thread-safe, shared pointer to a load session response.
pub type LoadSessionResponsePtr = Arc<Response<Result<RwLock<Session>, ErrorKind>>>;

/// A thread-safe, shared pointer to a delete session request.
pub type DeleteSessionRequestPtr = Arc<Request<String>>;

/// A thread-safe, shared pointer to a delete session response.
pub type DeleteSessionResponsePtr = Arc<Response<Result<(), ErrorKind>>>;

/// A thread-safe, shared pointer to a load stored track ids request.
pub type LoadStoredTrackIdsResponsePtr = Arc<Response<Vec<String>>>;

/// A thread-safe shared pointer to a load all stored tracks request.
pub type LoadStoredTracksReponsePtr = Arc<Response<Vec<Track>>>;

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
#[derive(Clone, Debug)]
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
}

/// A simple asynchronous event bus for publishing and subscribing to [`Event`]s.
///
/// The event bus uses a [`tokio::sync::broadcast::channel`] under the hood,
/// allowing multiple receivers to listen for the same stream of events.
///
/// Each published event is cloned and distributed to all active subscribers.
/// If no subscribers exist at the time of publication, the event is discarded silently.
pub struct EventBus {
    /// The broadcast sender used internally to distribute events.
    sender: tokio::sync::broadcast::Sender<Event>,
}

impl EventBus {
    /// Creates a new [`EventBus`] with a fixed buffer capacity of 100 messages.
    ///
    /// When the buffer is full, the oldest messages are dropped automatically
    /// as new ones are published.
    pub fn new() -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(100);
        EventBus { sender }
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
    /// The broadcast sender used to publish events.
    pub sender: tokio::sync::broadcast::Sender<Event>,

    /// The broadcast receiver used to listen for events.
    pub receiver: tokio::sync::broadcast::Receiver<Event>,
}

impl ModuleCtx {
    /// Constructs a new [`ModuleCtx`] from the given [`EventBus`].
    ///
    /// Clones the internal broadcast sender and creates a new receiver.
    /// ```
    pub fn new(event_bus: &EventBus) -> Self {
        ModuleCtx {
            sender: event_bus.sender.clone(),
            receiver: event_bus.subscribe(),
        }
    }
}

pub mod test_helper;

#[cfg(test)]
mod tests;
