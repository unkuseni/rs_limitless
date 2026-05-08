use crate::prelude::*;

/// Manages a WebSocket connection lifecycle.
///
/// Wraps a `WebSocketStream` and provides connect/disconnect with
/// proper WebSocket Close frame semantics.
pub struct WsClient {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WsClient {
    /// Create a new `WsClient` from an already-established `WebSocketStream`.
    pub fn new(stream: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self { stream }
    }

    /// Get a mutable reference to the underlying stream.
    pub fn stream(&mut self) -> &mut WebSocketStream<MaybeTlsStream<TcpStream>> {
        &mut self.stream
    }

    /// Send a WebSocket Close frame and consume the connection.
    pub async fn disconnect(&mut self) -> Result<(), LimitlessError> {
        self.stream
            .close(None)
            .await
            .map_err(|e| LimitlessError::Base(format!("Error closing WebSocket: {}", e)))?;
        Ok(())
    }
}
