use crate::{Event, EventBus, EventKind, EventKindType, ModuleCtx};
use core::panic;
use std::{
    collections::HashMap,
    io::ErrorKind,
    sync::{LazyLock, RwLock},
};
use tokio::time::timeout;
use tracing::{debug, error};

/// Sends a quit signal to a running module and waits for it to stop gracefully.
///
/// This function publishes a [`QuitEvent`](EventKind::QuitEvent) through the given [`EventBus`],
/// signaling the target module to terminate. It then waits asynchronously for the module’s task
/// (represented by the provided [`tokio::task::JoinHandle`]) to complete within a fixed timeout.
///
/// If the module fails to terminate within the timeout duration, the function will panic with
/// an error message, indicating that the module did not handle the quit event properly.
///
/// # Arguments
/// * `event_bus` – A reference to the [`EventBus`] used to send the quit event.
/// * `handle` – A mutable reference to the [`JoinHandle`] of the running module’s asynchronous task.
///
/// # Panics
/// This function panics if:
/// - The module does not stop within the specified timeout duration.
/// - The task returns an error (`Err(())`).
///
/// # Behavior
/// 1. Sends a `QuitEvent` through the event bus.
/// 2. Waits up to `TIMEOUT_MS` milliseconds for the module’s task to finish.
/// 3. If the task completes successfully, the function returns normally.
/// 4. If the timeout expires or the task fails, the function panics.
///
/// # Async
/// This function is asynchronous and must be awaited.
pub async fn stop_module(
    event_bus: &EventBus,
    handle: &mut tokio::task::JoinHandle<Result<(), ()>>,
) {
    event_bus.publish(&Event {
        kind: EventKind::QuitEvent,
    });
    let _ = timeout(std::time::Duration::from_millis(100), handle)
        .await
        .expect("Module doesn't handle quit event in timeout")
        .unwrap();
}

/// Waits asynchronously for a specific type of [`Event`] to be received on a
/// [`tokio::sync::broadcast::Receiver`] within a given duration.
///
/// This function repeatedly polls the provided broadcast receiver for incoming
/// [`Event`] messages, checking if any match the expected [`EventKind`]
/// discriminant. The total waiting time is divided into small polling steps
/// (each one-tenth of the total duration), allowing intermediate timeouts so the
/// function remains responsive.
///
/// If a matching event is received before the timeout expires, it is returned.
/// Otherwise, the function panics after the duration elapses.
///
/// # Arguments
///
/// * `rx` — A mutable reference to a [`tokio::sync::broadcast::Receiver<Event>`]
///   from which events are received.
/// * `duration` — The maximum amount of time to wait for the expected event.
/// * `exp_event` — The expected event type, represented as a
///   [`EventKindDiscriminants`]. Only the variant type is compared;
///   payload data is ignored.
///
/// # Panics
///
/// This function panics if no matching event is received within the specified
/// `duration`.
///
/// # Returns
///
/// Returns the first [`Event`] whose [`EventKind`] discriminant matches
/// `exp_event`.
pub async fn wait_for_event(
    rx: &mut tokio::sync::broadcast::Receiver<Event>,
    duration: std::time::Duration,
    exp_event: EventKindType,
) -> Event {
    let steps = duration.as_millis() / 10;
    let step_duration = duration / 10;
    for _ in 0..steps {
        if let Ok(Ok(event)) = timeout(step_duration, rx.recv()).await
            && EventKindType::from(&event.kind) == exp_event
        {
            return event;
        }
    }
    panic!("Failed to receive event of type {:?}", exp_event);
}

static RESPONSE_HANDLERS_CACHE: LazyLock<RwLock<HashMap<(usize, EventKindType), ResponseHandler>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Registers a new automatic response handler for a given request event type.
///
/// When an incoming event whose discriminant matches `request_type` is received on `ctx`,
/// the predefined `response_event` is sent back through the same context.
///
/// Arguments:
/// * `request_type` - Discriminant of the request event to listen for.
/// * `response_event` - Event to send when a matching request is observed.
/// * `ctx` - Module context providing sender/receiver channels.
///
/// Returns:
/// * `Ok(())` if the handler was successfully registered.
/// * `Err` with `ErrorKind::AlreadyExists` if a handler for `request_type` is already present.
///
/// Side effects:
/// * Spawns a background task (owned by `ResponseHandler`) and stores it in a global cache.
pub fn register_response_event(
    request_type: EventKindType,
    response_event: Event,
    ctx: ModuleCtx,
) -> Result<(), std::io::Error> {
    let bus_id = ctx.bus_id();
    let handler = ResponseHandler::new(ctx, request_type, response_event);
    let mut cache = RESPONSE_HANDLERS_CACHE.write().unwrap();
    if cache.insert((bus_id, request_type), handler).is_some() {
        error!(
            "Response handler for request type {:?} already exists in cache after insertion",
            (bus_id, request_type)
        );
        return Err(std::io::Error::new(
            ErrorKind::AlreadyExists,
            format!(
                "Response handler for request type {:?} already exists",
                request_type
            ),
        ));
    }
    debug!(
        "Registered response handler for request type {:?}",
        request_type
    );
    Ok(())
}

/// Unregisters (removes) a previously registered automatic response handler.
///
/// Arguments:
/// * `request_type` - Discriminant of the request event whose handler should be removed.
///
/// Behavior:
/// * If a handler exists, it is removed from the global cache and its background task is aborted.
/// * If no handler exists for `request_type`, the function is a no-op.
///
/// Side effects:
/// * Mutates the global `RESPONSE_HANDLERS_CACHE`.
/// * Aborts the spawned task associated with the handler (if present).
pub fn unregister_response_event(bus_id: usize, request_type: &EventKindType) {
    let mut cache = RESPONSE_HANDLERS_CACHE.write().unwrap();
    if let Some(handler) = cache.remove(&(bus_id, *request_type)) {
        debug!(
            "Unregistered response handler for request type {:?}",
            request_type
        );
        handler.handle.abort();
    }
}

struct ResponseHandlerRuntime {
    pub resp: Event,
    pub request_type: EventKindType,
    pub ctx: ModuleCtx,
}

/// Manages the automatic handling of asynchronous response events.
///
/// The [`ResponseHandler`] spawns a background task that listens for incoming
/// events on the associated module context and sends a predefined response
/// when an event of a specific type is received.  
/// It provides scoped, self-contained lifecycle management for asynchronous
/// response handling tasks.
///
/// When the handler is dropped, its background task is automatically aborted
/// to prevent resource leaks or dangling tasks.
#[derive(Debug)]
struct ResponseHandler {
    handle: tokio::task::JoinHandle<()>,
}

impl ResponseHandler {
    /// Creates and starts a new [`ResponseHandler`] instance.
    ///
    /// This function initializes a runtime context and spawns an asynchronous
    /// task that monitors the event receiver for matching request types.
    /// When a matching event is detected, the associated response is sent
    /// through the module context.
    pub fn new(ctx: ModuleCtx, request_type: EventKindType, response_event: Event) -> Self {
        let rt = ResponseHandlerRuntime {
            resp: response_event,
            request_type,
            ctx,
        };
        let handle = run(rt);
        ResponseHandler { handle }
    }
}

/// Spawns the background task that performs event monitoring and response dispatch.
///
/// This function runs an asynchronous loop that continuously waits for incoming
/// events. When an event matches the expected request type, it triggers the
/// transmission of the associated response.
fn run(mut rt: ResponseHandlerRuntime) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                event = rt.ctx.receiver.recv() =>
                match event {
                    Ok(event) => {
                        debug!("ResponseHandler received event {:?}", event);
                        if EventKindType::from(event.kind) == rt.request_type {
                            debug!("ResponseHandler sending response for request type {:?}", rt.request_type);
                            let _ = rt.ctx.sender.send(rt.resp.clone());
                        }
                    }
                    Err(e) => print!("Failed to receive request. Error: {}",  e)
                }
            }
        }
    })
}

impl Drop for ResponseHandler {
    /// Aborts the background task when the handler is dropped.
    ///
    /// This ensures that no asynchronous task remains active after
    /// the handler goes out of scope
    fn drop(&mut self) {
        self.handle.abort();
        debug!("ResponseHandler dropped and background task aborted.");
    }
}
