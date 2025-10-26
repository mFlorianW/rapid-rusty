use crate::{Event, EventBus, EventKind};
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

/// Waits for an event from a broadcast receiver that satisfies a given predicate, with a timeout.
///
/// This function repeatedly checks the receiver for a matching event, polling in 1/10th
/// intervals of the total duration. It returns `true` as soon as an event satisfying the
/// predicate is received, or `false` if the timeout expires without a matching event.
///
/// # Parameters
/// - `rx`: A mutable reference to a `tokio::sync::broadcast::Receiver<Event>` to receive events from.
/// - `duration`: The total maximum duration to wait for a matching event.
/// - `predicate`: A closure that takes a reference to an `Event` and returns `true` if it matches.
///
/// # Returns
/// - `true` if an event satisfying the predicate was received within the timeout.
/// - `false` if no matching event was received before the timeout expired.
pub async fn wait_for_event<F>(
    rx: &mut tokio::sync::broadcast::Receiver<Event>,
    duration: std::time::Duration,
    mut predicate: F,
) -> bool
where
    F: FnMut(&Event) -> bool,
{
    let steps = duration.as_millis() / 10;
    let step_duration = duration / 10;
    for _ in 0..steps {
        if let Ok(Ok(event)) = timeout(step_duration, rx.recv()).await
            && predicate(&event)
        {
            return true;
        }
    }
    false
}
