use domain::events::{MessageEvent, RoomEvent, UserEvent};
use tokio::sync::broadcast;

const DEFAULT_CAPACITY: usize = 1024;
const DEFAULT_MESSAGE_CAPACITY: usize = 2048;

/// A typed broadcast channel. Cheap to clone (internally Arc-wrapped).
#[derive(Clone)]
pub struct Channel<T: Clone> {
    sender: broadcast::Sender<T>,
}

impl<T: Clone> Channel<T> {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all current subscribers.
    /// Returns the number of receivers that got the message.
    pub fn publish(&self, event: T) -> usize {
        self.sender.send(event).unwrap_or(0) // If there are no subscribers, send returns an error; treat it as 0 receivers.
    }

    /// Subscribe to events on this channel.
    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }
}

/// Per-topic event bus.
///
/// Each event category gets its own channel so a slow consumer on one
/// topic cannot back-pressure publishers on another.
///
/// Cheap to clone (internally Arc-wrapped).
#[derive(Clone)]
pub struct EventBus {
    pub user: Channel<UserEvent>,
    pub room: Channel<RoomEvent>,
    pub message: Channel<MessageEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            user: Channel::new(DEFAULT_CAPACITY),
            room: Channel::new(DEFAULT_CAPACITY),
            message: Channel::new(DEFAULT_MESSAGE_CAPACITY),
        }
    }
}
