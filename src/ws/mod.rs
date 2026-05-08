pub mod channel;
pub mod client;
pub mod stream;
pub use channel::*;
pub use stream::*;

use tokio::time::Duration;

use crate::prelude::*;
use tokio::sync::mpsc;

/// Interval at which the WebSocket event loop sends a ping to keep the
/// connection alive (the Limitless WS uses Socket.IO-level pings internally,
/// but we maintain our own keep-alive on the raw WebSocket).
pub(crate) const PING_INTERVAL: Duration = Duration::from_secs(25);

/// Helper: send an item through an unbounded channel, mapping the error
/// to `LimitlessError`.
#[allow(dead_code)]
pub(crate) fn send_or_err<T>(
    sender: &mpsc::UnboundedSender<T>,
    item: T,
) -> Result<(), LimitlessError> {
    sender
        .send(item)
        .map_err(|e| LimitlessError::ChannelSendError {
            underlying: e.to_string(),
        })
}
