/// Represents a high-level event in the system.
///
/// Each `Event` wraps an [`EventKind`], which defines the actual type
/// and data carried by the event.
///
/// This structure is designed to be passed through an [`EventBus`]
/// between asynchronous modules.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// The inner event type and associated data.
    pub kind: EventKind,
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
#[derive(Debug, Clone, PartialEq)]
pub struct Request<T> {
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

/// Compares two [`Response`] values containing [`Arc`]-wrapped [`RwLock`] data.
///
/// This implementation considers two responses equal if:
/// - Their [`id`] fields are identical.
/// - The contents of their inner values (protected by the [`RwLock`]) are equal,
///   as determined by the [`PartialEq`] implementation of the inner type `T`.
///
/// Lock poisoning is handled gracefully:
/// if either [`RwLock`] is poisoned (due to a panic while it was previously held
/// for writing), the implementation recovers the underlying data using
/// [`PoisonError::into_inner()`] and continues the comparison instead of panicking.
///
/// # Type Parameters
/// - `T`: The type of the data stored within the [`RwLock`].
///   Must implement [`PartialEq`] to support equality comparison.
///
/// # Thread Safety
/// Only shared (read) access is acquired from both locks during comparison,
/// ensuring that the operation does not block other readers and does not modify data.
impl<T> PartialEq for Response<std::sync::Arc<std::sync::RwLock<T>>>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.id != other.id {
            return false;
        }
        let self_data = self.data.read().unwrap_or_else(|e| e.into_inner());
        let other_data = other.data.read().unwrap_or_else(|e| e.into_inner());
        *self_data == *other_data
    }
}

/// A thread-safe, reference-counted pointer to a [`GnssPosition`].
///
/// This type alias wraps a [`GnssPosition`] inside an [`Arc`], allowing
/// multiple parts of the program (or multiple modules) to share ownership
/// of the same GNSS position data without copying it.
pub type GnssPositionPtr = std::sync::Arc<common::position::GnssPosition>;

/// A thread-safe shared reference-counted pointer to a [`GnssInformation`].
///
/// This type alias wraps a [`GnssInformation`] instance in an [`Arc`],
/// multiple parts of the program (or multiple modules) to share ownership
/// of the same GNSS information data without copying it.
pub type GnssInformationPtr = std::sync::Arc<common::position::GnssInformation>;

/// Enumerates the different kinds of events that can be emitted
/// and transmitted via the [`EventBus`].
#[derive(Clone, Debug, PartialEq)]
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
    LapFinishedEvent(std::time::Duration),

    /// Indicates that a sector has been completed.
    /// This event carries a [`std:time::Duration`] structure
    SectorFinshedEvent(std::time::Duration),

    /// Represents the current laptime (may be used for reporting purposes).
    /// This event carries a [`std:time::Duration`] structure.
    CurrentLaptimeEvent(std::time::Duration),
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
