pub mod channel;
pub mod client;
pub mod stream;
pub use channel::*;
pub use stream::*;

use crate::prelude::*;
use tokio::sync::mpsc;

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
