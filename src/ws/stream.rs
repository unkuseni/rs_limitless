use crate::prelude::*;
use crate::ws::client::WsClient;
use crate::ws::PING_INTERVAL;

use futures::{SinkExt, StreamExt};
use log::{error, trace};
use std::time::Instant;

use tokio::sync::mpsc;

use tokio_tungstenite::tungstenite::Message as WsMessage;

/// Manages WebSocket streaming connections to the Limitless Exchange.
///
/// Connects to `wss://ws.limitless.exchange/markets` and provides
/// methods for subscribing to public and private data streams.
///
/// **Protocol note:** The Limitless WS uses Socket.IO protocol over raw
/// WebSocket. This implementation handles the raw WebSocket transport;
/// callers should frame Socket.IO packets (namespace connect, event
/// emit/receive) on top of this stream.
///
/// # Event Reference
///
/// | Client → Server (emit)         | Auth | Description                        |
/// |-------------------------------|------|------------------------------------|
/// | `subscribe_market_prices`     | No   | AMM prices + CLOB orderbook       |
/// | `subscribe_positions`         | Yes  | Portfolio position updates         |
/// | `subscribe_order_events`      | Yes  | OME + settlement lifecycle        |
/// | `subscribe_market_lifecycle`  | No   | Market creation / resolution       |
///
/// | Server → Client (on)  | Auth | Description                          |
/// |-----------------------|------|--------------------------------------|
/// | `newPriceData`        | No   | AMM price update                     |
/// | `orderbookUpdate`     | No   | CLOB orderbook snapshot              |
/// | `positions`           | Yes  | Position balance change              |
/// | `orderEvent`          | Yes  | OME state or settlement result       |
/// | `marketCreated`       | No   | New market funded and visible        |
/// | `marketResolved`      | No   | Market resolved with winning outcome |
/// | `system`              | —    | System notifications                 |
/// | `authenticated`       | Yes  | Auth confirmation                    |
/// | `exception`           | —    | Error notifications                  |
#[derive(Clone)]
pub struct Stream {
    pub client: Client,
}

impl Stream {
    /// Tests connectivity by sending a WebSocket ping.
    pub async fn ws_ping(&self) -> Result<(), LimitlessError> {
        let response = self.client.wss_connect(None, false, None).await?;

        let mut ws_client = WsClient::new(response);
        let _ = ws_client
            .stream()
            .send(WsMessage::Ping(vec![].into()))
            .await;

        let Some(data) = ws_client.stream().next().await else {
            return Err(LimitlessError::Base(
                "Failed to receive pong response".to_string(),
            ));
        };
        match data {
            Ok(WsMessage::Pong(_)) => {
                trace!("Pong received successfully");
            }
            Ok(other) => {
                trace!("Unexpected WS message on ping: {:?}", other);
            }
            Err(e) => {
                return Err(LimitlessError::Tungstenite(e));
            }
        }
        Ok(())
    }

    /// Subscribe to a public data stream with an event handler callback.
    ///
    /// The `handler` receives raw JSON `Value` for each incoming message
    /// that is not a control frame (Ping/Pong/Close).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use limitless::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let stream: Stream = Limitless::new(None, None);
    ///     stream
    ///         .ws_subscribe(|event| {
    ///             println!("Received: {:?}", event);
    ///             Ok(())
    ///         })
    ///         .await
    ///         .unwrap();
    /// }
    /// ```
    pub async fn ws_subscribe<F>(&self, handler: F) -> Result<(), LimitlessError>
    where
        F: FnMut(Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        let response = self.client.wss_connect(None, false, None).await?;
        let mut ws_client = WsClient::new(response);
        Self::event_loop(&mut ws_client, handler, None).await?;
        Ok(())
    }

    /// Subscribe to a stream with dynamic command support.
    ///
    /// Allows emitting subscription commands (subscribe/unsubscribe) after
    /// the connection is established. Send JSON command strings through
    /// the `cmd_sender` channel.
    pub async fn ws_subscribe_with_commands<F>(
        &self,
        cmd_receiver: mpsc::UnboundedReceiver<String>,
        handler: F,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        let response = self.client.wss_connect(None, false, None).await?;
        let mut ws_client = WsClient::new(response);
        Self::event_loop(&mut ws_client, handler, Some(cmd_receiver)).await?;
        Ok(())
    }

    /// Core event loop: reads WebSocket messages, dispatches to handler,
    /// sends periodic pings, and processes outgoing subscription commands.
    pub(crate) async fn event_loop<F>(
        ws_client: &mut WsClient,
        mut handler: F,
        mut cmd_receiver: Option<mpsc::UnboundedReceiver<String>>,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        let mut last_ping = Instant::now();

        loop {
            tokio::select! {
                // ── Incoming WebSocket message ─────────────────────────
                msg = ws_client.stream().next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            if let Ok(event) = serde_json::from_str::<Value>(&text) {
                                if let Err(e) = handler(event) {
                                    error!("WebSocket handler error: {}", e);
                                }
                            }
                        }
                        Some(Ok(WsMessage::Binary(data))) => {
                            // Socket.IO may send binary frames
                            if let Ok(text) = String::from_utf8(data.to_vec()) {
                                if let Ok(event) = serde_json::from_str::<Value>(&text) {
                                    if let Err(e) = handler(event) {
                                        error!("WebSocket handler error: {}", e);
                                    }
                                }
                            }
                        }
                        Some(Ok(WsMessage::Ping(data))) => {
                            let _ = ws_client.stream().send(WsMessage::Pong(data)).await;
                        }
                        Some(Ok(WsMessage::Close(_))) => {
                            trace!("WebSocket closed by server");
                            return Ok(());
                        }
                        Some(Err(e)) => {
                            return Err(LimitlessError::Tungstenite(e));
                        }
                        None => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                // ── Outgoing subscription command ──────────────────────
                cmd = async {
                    match cmd_receiver.as_mut() {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    if let Some(cmd) = cmd {
                        let _ = ws_client
                            .stream()
                            .send(WsMessage::Text(cmd.into()))
                            .await;
                    }
                }

                // ── Periodic ping ──────────────────────────────────────
                _ = tokio::time::sleep(PING_INTERVAL) => {
                    let now = Instant::now();
                    if now.duration_since(last_ping) >= PING_INTERVAL {
                        let _ = ws_client
                            .stream()
                            .send(WsMessage::Ping(vec![].into()))
                            .await;
                        last_ping = now;
                    }
                }
            }
        }
    }
}

impl Limitless for Stream {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(api_key, secret, config.rest_api_endpoint.to_string()),
        }
    }
}
