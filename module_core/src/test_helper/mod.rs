use crate::{Event, EventBus, EventKind, EventKindDiscriminants};
use core::panic;
use tokio::time::timeout;

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
        .expect("ConstantGnssSourceModule doesn't handle quit event in timeout")
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
    exp_event: EventKindDiscriminants,
) -> Event {
    let steps = duration.as_millis() / 10;
    let step_duration = duration / 10;
    for _ in 0..steps {
        if let Ok(Ok(event)) = timeout(step_duration, rx.recv()).await
            && EventKindDiscriminants::from(&event.kind) == exp_event
        {
            return event;
        }
    }
    panic!("Failed to receive event of type {:?}", exp_event);
}

pub struct GenericResponseHandler {
    resp: Event,
}

impl GenericResponseHandler {
    pub fn new(event: Event) -> Self {
        GenericResponseHandler { resp: event }
    }
}
