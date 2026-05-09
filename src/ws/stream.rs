use crate::prelude::*;
use crate::ws::client::WsClient;
use crate::ws::PING_INTERVAL;

use futures::{SinkExt, StreamExt};
use log::{debug, error, trace, warn};
use std::time::Instant;

use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message as WsMessage;

// ── Socket.IO / Engine.IO protocol constants ─────────────────────────────

/// The Socket.IO namespace used by the Limitless Exchange.
pub const SOCKET_NAMESPACE: &str = "/markets";

/// Engine.IO open packet prefix (server → client handshake).
const EIO_OPEN: u8 = b'0';

/// Engine.IO close packet.
const EIO_CLOSE: u8 = b'1';

/// Engine.IO ping packet (server → client).
const EIO_PING: u8 = b'2';

/// Engine.IO pong packet (client → server).
const EIO_PONG: u8 = b'3';

/// Engine.IO message packet (carries Socket.IO payload).
const EIO_MESSAGE: u8 = b'4';

/// Socket.IO connect packet type.
/// Socket.IO disconnect packet type.
/// Socket.IO event packet type.
const SIO_EVENT: u8 = b'2';

/// Socket.IO error packet type.
// ── Helpers ──────────────────────────────────────────────────────────────

/// Build a Socket.IO event frame for emission to the server.
///
/// Format: `42{namespace},["{event}",{data}]`
///
/// Returns a complete frame ready to send as a WebSocket text message.
pub fn frame_socketio_event(event: &str, data: &Value) -> String {
    let payload = serde_json::to_string(&serde_json::json!([event, data]))
        .unwrap_or_else(|_| format!(r#"["{}",null]"#, event));
    format!("42{namespace},{payload}", namespace = SOCKET_NAMESPACE)
}

/// Build a Socket.IO namespace connect frame.
///
/// Format: `40{namespace},`
pub fn frame_socketio_connect() -> String {
    format!("40{namespace},", namespace = SOCKET_NAMESPACE)
}

/// Parse an Engine.IO text message and extract the Socket.IO event name
/// and payload if this is an event (`42{namespace},[...]`).
///
/// Returns `Some((event_name, payload_value))` for events, `None` for
/// non-event messages (open, close, ping/pong, connect ack, etc.).
pub fn parse_socketio_message(text: &str) -> Option<(String, Value)> {
    let bytes = text.as_bytes();

    // Must start with '4' (Engine.IO message)
    if bytes.is_empty() || bytes[0] != EIO_MESSAGE {
        return None;
    }

    let after_eio = &text[1..];

    // Must start with a digit (Socket.IO packet type)
    let first_sio = after_eio.as_bytes().first()?;
    if *first_sio != SIO_EVENT {
        // '40' = connect, '41' = disconnect, '43' = ack, '44' = error
        // None of these carry user events
        return None;
    }

    // Skip the Socket.IO type digit: now we have "2{namespace},[...]"
    let after_sio_type = &after_eio[1..];

    // Strip namespace prefix: "{namespace},"
    let event_payload = if let Some(rest) = after_sio_type.strip_prefix(SOCKET_NAMESPACE) {
        // Strip the comma after namespace
        rest.strip_prefix(',')?
    } else {
        // No namespace prefix — payload starts right after the type digit
        // (e.g., "2[...]")
        after_sio_type
    };

    // Parse the JSON array: ["eventName", {...}]
    let values: Vec<Value> = serde_json::from_str(event_payload).ok()?;
    if values.is_empty() {
        return None;
    }
    let event_name = values[0].as_str()?.to_string();
    let payload = values.get(1).cloned().unwrap_or(Value::Null);

    Some((event_name, payload))
}

/// Check if a text message is an Engine.IO ping (just the character '2').
fn is_eio_ping(text: &str) -> bool {
    text.as_bytes() == [EIO_PING]
}

/// Check if a text message is an Engine.IO close (just the character '1').
fn is_eio_close(text: &str) -> bool {
    text.as_bytes() == [EIO_CLOSE]
}

/// Check if a text message is the Engine.IO open packet (starts with '0').
fn is_eio_open(text: &str) -> bool {
    text.as_bytes().first() == Some(&EIO_OPEN)
}

/// Check if a text message is a Socket.IO namespace connect ack ('40{namespace},').
fn is_namespace_connect_ack(text: &str) -> bool {
    text.starts_with(&format!("40{namespace},", namespace = SOCKET_NAMESPACE))
}

/// Check if a text message is a Socket.IO namespace disconnect.
fn is_namespace_disconnect(text: &str) -> bool {
    text.starts_with(&format!("41{namespace}", namespace = SOCKET_NAMESPACE))
}

// ── Stream ───────────────────────────────────────────────────────────────

/// Manages WebSocket streaming connections to the Limitless Exchange.
///
/// Connects to `wss://ws.limitless.exchange/socket.io/?EIO=4&transport=websocket`
/// and handles the Socket.IO protocol (Engine.IO v4 + Socket.IO) over the
/// raw WebSocket transport.
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
    /// Tests connectivity by performing the full Socket.IO handshake.
    ///
    /// Connects, reads the Engine.IO open packet, sends namespace connect,
    /// and verifies the server acknowledges it.
    pub async fn ws_ping(&self) -> Result<(), LimitlessError> {
        let stream = self.client.wss_connect(None, false, None).await?;
        let mut ws_client = WsClient::new(stream);

        // ── Phase 1: Engine.IO open ──────────────────────────────
        let open_text = Self::read_text_message(ws_client.stream()).await?;
        if !is_eio_open(&open_text) {
            return Err(LimitlessError::Base(format!(
                "Expected Engine.IO open packet (0{{...}}), got: {}",
                &open_text[..open_text.len().min(80)]
            )));
        }
        trace!("Engine.IO open received: {}", open_text);

        // ── Phase 2: Socket.IO namespace connect ─────────────────
        let connect_frame = frame_socketio_connect();
        ws_client
            .stream()
            .send(WsMessage::Text(connect_frame.into()))
            .await
            .map_err(|e| {
                LimitlessError::Base(format!("Failed to send namespace connect: {}", e))
            })?;

        let ack_text = Self::read_text_message(ws_client.stream()).await?;
        if !is_namespace_connect_ack(&ack_text) {
            return Err(LimitlessError::Base(format!(
                "Expected namespace connect ack (40/markets,), got: {}",
                &ack_text[..ack_text.len().min(80)]
            )));
        }
        trace!("Namespace connected: {}", ack_text);

        // ── Send proper close ────────────────────────────────────
        let _ = ws_client
            .stream()
            .send(
                WsMessage::Text(format!("41{namespace},", namespace = SOCKET_NAMESPACE).into())
                    .into(),
            )
            .await;
        let _ = ws_client.disconnect().await;

        Ok(())
    }

    /// Subscribe to a public data stream with an event handler callback.
    ///
    /// The `handler` receives a `Value` that is an array `[event_name, payload]`
    /// for Socket.IO events, or the raw parsed JSON for other messages.
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
        let stream = self.client.wss_connect(None, false, None).await?;
        let mut ws_client = WsClient::new(stream);
        Self::event_loop(&mut ws_client, handler, None).await?;
        Ok(())
    }

    /// Subscribe to a stream with dynamic command support.
    ///
    /// Allows emitting subscription commands (subscribe/unsubscribe) after
    /// the connection is established. Commands should be complete Socket.IO
    /// frames (e.g., `42/markets,["subscribe_market_prices",{...}]`).
    ///
    /// Use [`frame_socketio_event`] to build properly framed commands.
    pub async fn ws_subscribe_with_commands<F>(
        &self,
        cmd_receiver: mpsc::UnboundedReceiver<String>,
        handler: F,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        let stream = self.client.wss_connect(None, false, None).await?;
        let mut ws_client = WsClient::new(stream);
        Self::event_loop(&mut ws_client, handler, Some(cmd_receiver)).await?;
        Ok(())
    }

    /// Subscribe to a stream with dynamic command support **and authentication**.
    ///
    /// Like [`ws_subscribe_with_commands`](Self::ws_subscribe_with_commands) but
    /// sends the `X-API-Key` header on the WebSocket upgrade request, enabling
    /// private channels:
    ///
    /// - `subscribe_positions` — real-time position balance updates
    /// - `subscribe_order_events` — OME state changes + settlement results
    ///
    /// # Requirements
    ///
    /// The [`Stream`] must have been constructed with an API key (via
    /// [`Limitless::new`] or [`LimitlessClient::builder().set_credentials()`]).
    /// Without a key the connection is still established but private
    /// subscriptions will fail with an `exception` event.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use limitless::prelude::*;
    /// use tokio::sync::mpsc;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let ws: Stream = Limitless::new(
    ///         Some("lmts_sk_...".into()),
    ///         Some("base64_secret".into()),
    ///     );
    ///     let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    ///
    ///     // Send subscription commands after connecting
    ///     tokio::spawn(async move {
    ///         let sub = frame_socketio_event("subscribe_order_events", &serde_json::json!({}));
    ///         let _ = cmd_tx.send(sub);
    ///     });
    ///
    ///     ws.ws_subscribe_authenticated_with_commands(cmd_rx, |event| {
    ///         println!("Private event: {event}");
    ///         Ok(())
    ///     }).await.unwrap();
    /// }
    /// ```
    pub async fn ws_subscribe_authenticated_with_commands<F>(
        &self,
        cmd_receiver: mpsc::UnboundedReceiver<String>,
        handler: F,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        let stream = self.client.wss_connect(None, true, None).await?;
        let mut ws_client = WsClient::new(stream);
        Self::event_loop(&mut ws_client, handler, Some(cmd_receiver)).await?;
        Ok(())
    }

    /// Subscribe to market updates for a specific slug.
    ///
    /// Handles the full lifecycle: connect, handshake, subscribe, and
    /// event dispatch. The handler receives `[event_name, payload]` arrays.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use limitless::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let ws: Stream = Limitless::new(None, None);
    ///     ws.ws_subscribe_market("btc-above-100k", |event_name, payload| {
    ///         println!("{event_name}: {payload}");
    ///         Ok(())
    ///     }).await.unwrap();
    /// }
    /// ```
    pub async fn ws_subscribe_market<F>(
        &self,
        market_slug: &str,
        mut handler: F,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(&str, &Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        let stream = self.client.wss_connect(None, false, None).await?;
        let mut ws_client = WsClient::new(stream);

        // ── Handshake ─────────────────────────────────────────────
        Self::perform_handshake(&mut ws_client).await?;

        // ── Subscribe ─────────────────────────────────────────────
        let sub = frame_socketio_event(
            "subscribe_market_prices",
            &serde_json::json!({"marketSlugs": [market_slug]}),
        );
        ws_client
            .stream()
            .send(WsMessage::Text(sub.into()))
            .await
            .map_err(|e| LimitlessError::Base(format!("Failed to send subscription: {}", e)))?;
        debug!("Subscribed to market prices for: {}", market_slug);

        // ── Event loop with typed dispatch ────────────────────────
        Self::typed_event_loop(&mut ws_client, &mut handler, None).await?;
        Ok(())
    }

    /// Subscribe to the WebSocket event stream and receive typed [`WsEventKind`] events.
    ///
    /// Connects, performs the Socket.IO handshake, then enters an event
    /// loop that parses every incoming server event through
    /// [`deserialize_event`] before passing the resulting [`WsEventKind`]
    /// to `handler`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use limitless::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let ws: Stream = Limitless::new(None, None);
    ///     ws.ws_subscribe_events(|event| {
    ///         match event {
    ///             WsEventKind::NewPriceData(p) => println!("AMM prices for {}", p.market_address),
    ///             WsEventKind::TradeEvent(t) => println!("Trade: {} @ {}", t.size, t.price),
    ///             WsEventKind::Unknown(payload) => println!("Unknown: {payload:?}"),
    ///             other => println!("Event: {other:?}"),
    ///         }
    ///         Ok(())
    ///     }).await.unwrap();
    /// }
    /// ```
    pub async fn ws_subscribe_events<F>(&self, mut handler: F) -> Result<(), LimitlessError>
    where
        F: FnMut(WsEventKind) -> Result<(), LimitlessError> + 'static + Send,
    {
        let stream = self.client.wss_connect(None, false, None).await?;
        let mut ws_client = WsClient::new(stream);

        // ── Handshake ─────────────────────────────────────────────
        Self::perform_handshake(&mut ws_client).await?;

        // ── Typed dispatch wrapper ────────────────────────────────
        let mut adapter = move |event_name: &str, payload: &Value| -> Result<(), LimitlessError> {
            match deserialize_event(event_name, payload) {
                Some(kind) => handler(kind),
                None => {
                    debug!("Failed to deserialize event '{}', skipping", event_name);
                    Ok(())
                }
            }
        };

        // ── Event loop with typed dispatch ────────────────────────
        Self::typed_event_loop(&mut ws_client, &mut adapter, None).await?;
        Ok(())
    }

    /// Subscribe to typed WebSocket events **with authentication**.
    ///
    /// Like [`ws_subscribe_events`](Self::ws_subscribe_events) but sends the
    /// `X-API-Key` header on the WebSocket upgrade request, enabling private
    /// channels such as `positions` and `orderEvent`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use limitless::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let ws: Stream = Limitless::new(
    ///         Some("lmts_sk_...".into()),
    ///         Some("base64_secret".into()),
    ///     );
    ///     ws.ws_subscribe_authenticated_events(|event| {
    ///         match event {
    ///             WsEventKind::Positions(p) => println!("Position update: {p:?}"),
    ///             WsEventKind::OrderEvent(o) => println!("Order event: {o:?}"),
    ///             other => println!("Other: {other:?}"),
    ///         }
    ///         Ok(())
    ///     }).await.unwrap();
    /// }
    /// ```
    pub async fn ws_subscribe_authenticated_events<F>(
        &self,
        mut handler: F,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(WsEventKind) -> Result<(), LimitlessError> + 'static + Send,
    {
        let stream = self.client.wss_connect(None, true, None).await?;
        let mut ws_client = WsClient::new(stream);

        Self::perform_handshake(&mut ws_client).await?;

        let mut adapter = move |event_name: &str, payload: &Value| -> Result<(), LimitlessError> {
            match deserialize_event(event_name, payload) {
                Some(kind) => handler(kind),
                None => {
                    debug!("Failed to deserialize event '{}', skipping", event_name);
                    Ok(())
                }
            }
        };

        Self::typed_event_loop(&mut ws_client, &mut adapter, None).await?;
        Ok(())
    }

    /// Perform the Socket.IO handshake: read Engine.IO open, send namespace
    /// connect, wait for ack.
    async fn perform_handshake(ws_client: &mut WsClient) -> Result<(), LimitlessError> {
        // Read Engine.IO open packet
        let open_text = Self::read_text_message(ws_client.stream()).await?;
        if !is_eio_open(&open_text) {
            return Err(LimitlessError::Base(format!(
                "Expected Engine.IO open packet (0{{...}}), got: {}",
                &open_text[..open_text.len().min(120)]
            )));
        }
        debug!("Engine.IO open: {}", open_text);

        // Send namespace connect
        let connect_frame = frame_socketio_connect();
        ws_client
            .stream()
            .send(WsMessage::Text(connect_frame.into()))
            .await
            .map_err(|e| {
                LimitlessError::Base(format!("Failed to send namespace connect: {}", e))
            })?;

        // Read namespace connect ack
        let ack_text = Self::read_text_message(ws_client.stream()).await?;
        if !is_namespace_connect_ack(&ack_text) {
            return Err(LimitlessError::Base(format!(
                "Expected namespace connect ack (40/markets,), got: {}",
                &ack_text[..ack_text.len().min(120)]
            )));
        }
        debug!("Namespace connected: {}", ack_text);

        Ok(())
    }

    /// Read the next text (or binary → text) message from the stream.
    async fn read_text_message(
        stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> Result<String, LimitlessError> {
        match stream.next().await {
            Some(Ok(WsMessage::Text(text))) => Ok(text.to_string()),
            Some(Ok(WsMessage::Binary(data))) => String::from_utf8(data.to_vec())
                .map_err(|e| LimitlessError::Base(format!("Invalid UTF-8 in binary frame: {}", e))),
            Some(Ok(other)) => Err(LimitlessError::Base(format!(
                "Expected text frame, got: {:?}",
                other
            ))),
            Some(Err(e)) => Err(LimitlessError::Tungstenite(e)),
            None => Err(LimitlessError::Base(
                "WebSocket connection closed during handshake".to_string(),
            )),
        }
    }

    /// Core event loop: reads WebSocket messages, dispatches to handler,
    /// sends periodic pings, and processes outgoing subscription commands.
    ///
    /// Performs the Socket.IO handshake before entering the main loop.
    pub(crate) async fn event_loop<F>(
        ws_client: &mut WsClient,
        mut handler: F,
        mut cmd_receiver: Option<mpsc::UnboundedReceiver<String>>,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        // ── Handshake phase ────────────────────────────────────────────
        Self::perform_handshake(ws_client).await?;

        // ── Main event loop ────────────────────────────────────────────
        let mut last_ping = Instant::now();

        loop {
            tokio::select! {
                // ── Incoming WebSocket message ─────────────────────────
                msg = ws_client.stream().next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            Self::handle_incoming_text(&text, &mut handler, ws_client).await?;
                        }
                        Some(Ok(WsMessage::Binary(data))) => {
                            if let Ok(text) = String::from_utf8(data.to_vec()) {
                                Self::handle_incoming_text(&text, &mut handler, ws_client).await?;
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
                            trace!("WebSocket stream ended");
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                // ── Outgoing command ───────────────────────────────────
                cmd = async {
                    match cmd_receiver.as_mut() {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    if let Some(cmd) = cmd {
                        debug!("WS send: {}", &cmd[..cmd.len().min(200)]);
                        if let Err(e) = ws_client
                            .stream()
                            .send(WsMessage::Text(cmd.into()))
                            .await
                        {
                            error!("Failed to send command: {}", e);
                        }
                    }
                }

                // ── Periodic Engine.IO ping ────────────────────────────
                _ = tokio::time::sleep(PING_INTERVAL) => {
                    let now = Instant::now();
                    if now.duration_since(last_ping) >= PING_INTERVAL {
                        // Send Engine.IO ping (the string "2")
                        let _ = ws_client
                            .stream()
                            .send(WsMessage::Text(String::from("2").into()))
                            .await;
                        last_ping = now;
                    }
                }
            }
        }
    }

    /// Typed event loop: like `event_loop` but calls a `FnMut(&str, &Value)`
    /// handler instead of `FnMut(Value)`.
    pub(crate) async fn typed_event_loop<F>(
        ws_client: &mut WsClient,
        handler: &mut F,
        mut cmd_receiver: Option<mpsc::UnboundedReceiver<String>>,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(&str, &Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        let mut last_ping = Instant::now();

        loop {
            tokio::select! {
                // ── Incoming WebSocket message ─────────────────────────
                msg = ws_client.stream().next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            if let Some((event_name, payload)) = parse_socketio_message(&text) {
                                if let Err(e) = handler(&event_name, &payload) {
                                    error!("WS handler error on '{event_name}': {}", e);
                                }
                            } else if is_eio_ping(&text) {
                                let _ = ws_client.stream()
                                    .send(WsMessage::Text(String::from("3").into()))
                                    .await;
                            } else if is_eio_close(&text) || is_namespace_disconnect(&text) {
                                trace!("Socket.IO close/disconnect received");
                                return Ok(());
                            }
                            // Ignore other messages (open, connect ack, pong, etc.)
                        }
                        Some(Ok(WsMessage::Binary(data))) => {
                            if let Ok(text) = String::from_utf8(data.to_vec()) {
                                if let Some((event_name, payload)) = parse_socketio_message(&text) {
                                    if let Err(e) = handler(&event_name, &payload) {
                                        error!("WS handler error on '{event_name}': {}", e);
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
                            trace!("WebSocket stream ended");
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                // ── Outgoing command ───────────────────────────────────
                cmd = async {
                    match cmd_receiver.as_mut() {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    if let Some(cmd) = cmd {
                        debug!("WS send: {}", &cmd[..cmd.len().min(200)]);
                        if let Err(e) = ws_client
                            .stream()
                            .send(WsMessage::Text(cmd.into()))
                            .await
                        {
                            error!("Failed to send command: {}", e);
                        }
                    }
                }

                // ── Periodic Engine.IO ping ────────────────────────────
                _ = tokio::time::sleep(PING_INTERVAL) => {
                    let now = Instant::now();
                    if now.duration_since(last_ping) >= PING_INTERVAL {
                        let _ = ws_client
                            .stream()
                            .send(WsMessage::Text(String::from("2").into()))
                            .await;
                        last_ping = now;
                    }
                }
            }
        }
    }

    /// Handle an incoming text message from the WebSocket.
    ///
    /// Dispatches Engine.IO control frames and Socket.IO events.
    async fn handle_incoming_text<F>(
        text: &str,
        handler: &mut F,
        ws_client: &mut WsClient,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        // Engine.IO ping → reply pong
        if is_eio_ping(text) {
            let _ = ws_client
                .stream()
                .send(WsMessage::Text(String::from("3").into()))
                .await;
            return Ok(());
        }

        // Engine.IO close or Socket.IO namespace disconnect
        if is_eio_close(text) || is_namespace_disconnect(text) {
            trace!("Socket.IO close/disconnect");
            return Err(LimitlessError::Base(
                "Server closed the Socket.IO connection".to_string(),
            ));
        }

        // Engine.IO pong — ignore
        if text.as_bytes() == [EIO_PONG] {
            return Ok(());
        }

        // Try to parse as a Socket.IO event (42/markets,[...])
        if let Some((event_name, payload)) = parse_socketio_message(text) {
            // Pass as [event_name, payload] array for backward compat
            let event_array = serde_json::json!([event_name, payload]);
            if let Err(e) = handler(event_array) {
                error!("WS handler error on '{event_name}': {}", e);
            }
            return Ok(());
        }

        // Socket.IO connect ack, plain open, etc. — ignore
        if is_namespace_connect_ack(text) || is_eio_open(text) {
            return Ok(());
        }

        // Unknown message — log and try to parse as raw JSON
        warn!("Unhandled WS message: {}", &text[..text.len().min(200)]);
        if let Ok(value) = serde_json::from_str::<Value>(text) {
            let _ = handler(value);
        }

        Ok(())
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
