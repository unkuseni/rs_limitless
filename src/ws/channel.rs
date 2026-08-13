//! WebSocket types — channels, config, state, and subscription options.
//!
//! These model the Limitless Exchange WebSocket API's subscription channels
//! and payloads without depending on any Socket.IO protocol.

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ═══════════════════════════════════════════════════════════════════════════
//  Connection state
// ═══════════════════════════════════════════════════════════════════════════

/// Tracks the lifecycle of a WebSocket connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebSocketState {
    /// Not connected and not trying.
    Disconnected,
    /// Currently performing the initial handshake.
    Connecting,
    /// Connected and receiving events.
    Connected,
    /// Temporarily disconnected; attempting to re-establish.
    Reconnecting,
    /// Connection failed and will not be retried.
    Error,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Subscription channels
// ═══════════════════════════════════════════════════════════════════════════

/// Identifies a WebSocket subscription target on the Limitless Exchange.
///
/// Variants prefixed with `Subscribe` / `Unsubscribe` represent client →
/// server subscription requests. The non-prefixed variants are server-emitted
/// event names used for dispatching incoming messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubscriptionChannel {
    // ── Server → Client event names ──
    /// CLOB orderbook snapshots (`orderbook`).
    Orderbook,
    /// Public trade feed (`trades`).
    Trades,
    /// Order status updates (`orders`).
    Orders,
    /// Fill notifications (`fills`).
    Fills,
    /// Market statistics (`markets`).
    Markets,
    /// Aggregated price feed (`prices`).
    Prices,
    /// Portfolio position updates (`positions`).
    Positions,
    /// Blockchain transaction events (`transactions`).
    Transactions,
    /// OME + settlement lifecycle events (`orderEvent`).
    OrderEvents,
    /// Live sports data (`liveSports`).
    LiveSports,
    /// Live esports data (`liveEsports`).
    LiveEsports,
    /// Market creation / resolution events (`marketLifecycle`).
    MarketLifecycle,
    /// Unrealized PnL leaderboard invalidation hint (`unrealizedPnlProjectionChanged`).
    UnrealizedPnlProjectionChanged,

    // ── Client → Server subscription requests ──
    /// Subscribe to AMM prices + CLOB orderbook.
    SubscribeMarketPrices,
    /// Subscribe to portfolio position updates (requires auth).
    SubscribePositions,
    /// Subscribe to blockchain transaction events (requires auth).
    SubscribeTransactions,
    /// Subscribe to OME + settlement lifecycle events (requires auth).
    SubscribeOrderEvents,
    /// Subscribe to live sports data.
    SubscribeLiveSports,
    /// Subscribe to live esports data.
    SubscribeLiveEsports,
    /// Subscribe to market creation / resolution events.
    SubscribeMarketLifecycle,
    /// Unsubscribe from market lifecycle events.
    UnsubscribeMarketLifecycle,
    /// Subscribe to Unrealized PnL leaderboard invalidation hints.
    SubscribeUnrealizedPnl,
    /// Unsubscribe from Unrealized PnL invalidation hints.
    UnsubscribeUnrealizedPnl,
}

impl SubscriptionChannel {
    /// Returns the wire-protocol string for this channel.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Orderbook => "orderbook",
            Self::Trades => "trades",
            Self::Orders => "orders",
            Self::Fills => "fills",
            Self::Markets => "markets",
            Self::Prices => "prices",
            Self::Positions => "positions",
            Self::Transactions => "transactions",
            Self::OrderEvents => "orderEvent",
            Self::LiveSports => "liveSports",
            Self::LiveEsports => "liveEsports",
            Self::MarketLifecycle => "marketLifecycle",
            Self::UnrealizedPnlProjectionChanged => "unrealizedPnlProjectionChanged",
            Self::SubscribeMarketPrices => "subscribe_market_prices",
            Self::SubscribePositions => "subscribe_positions",
            Self::SubscribeTransactions => "subscribe_transactions",
            Self::SubscribeOrderEvents => "subscribe_order_events",
            Self::SubscribeLiveSports => "subscribe_live_sports",
            Self::SubscribeLiveEsports => "subscribe_live_esports",
            Self::SubscribeMarketLifecycle => "subscribe_market_lifecycle",
            Self::UnsubscribeMarketLifecycle => "unsubscribe_market_lifecycle",
            Self::SubscribeUnrealizedPnl => "subscribe_unrealized_pnl",
            Self::UnsubscribeUnrealizedPnl => "unsubscribe_unrealized_pnl",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Subscription options
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters supplied when subscribing to a channel.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionOptions {
    /// A single market slug (for channels that accept one).
    #[serde(
        rename = "marketSlug",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub market_slug: Option<String>,

    /// One or more market slugs (for multi-market subscriptions).
    #[serde(rename = "marketSlugs", skip_serializing_if = "Vec::is_empty", default)]
    pub market_slugs: Vec<String>,

    /// A single on-chain market address.
    #[serde(
        rename = "marketAddress",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub market_address: Option<String>,

    /// One or more on-chain market addresses.
    #[serde(
        rename = "marketAddresses",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub market_addresses: Vec<String>,

    /// Arbitrary server-side filters (channel-dependent).
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub filters: BTreeMap<String, Value>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Unrealized PnL subscription
// ═══════════════════════════════════════════════════════════════════════════

/// Payload for `subscribe_unrealized_pnl` / `unsubscribe_unrealized_pnl`.
///
/// Two scopes are available: one leaderboard per open market (`MARKET`,
/// with a `market_id`), and the global biggest-open-positions list
/// (`BIGGEST_POSITIONS`). At most 50 `MARKET` scopes per connection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnrealizedPnlSubscription {
    /// Message schema version (currently `1`).
    #[serde(rename = "schemaVersion")]
    pub schema_version: i64,
    /// `MARKET` or `BIGGEST_POSITIONS`.
    pub scope: String,
    /// Market id — required for `MARKET` scope.
    #[serde(skip_serializing_if = "Option::is_none", rename = "marketId")]
    pub market_id: Option<i64>,
}

impl UnrealizedPnlSubscription {
    /// A `MARKET`-scope subscription for one market's leaderboard.
    pub fn market(market_id: i64) -> Self {
        Self {
            schema_version: 1,
            scope: "MARKET".to_string(),
            market_id: Some(market_id),
        }
    }

    /// The `BIGGEST_POSITIONS`-scope subscription.
    pub fn biggest_positions() -> Self {
        Self {
            schema_version: 1,
            scope: "BIGGEST_POSITIONS".to_string(),
            market_id: None,
        }
    }

    /// Serialize this subscription to the JSON payload used in the event frame.
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  WebSocket config
// ═══════════════════════════════════════════════════════════════════════════

/// Configuration for the Limitless WebSocket connection.
#[derive(Clone, Debug)]
pub struct WebSocketConfig {
    /// The WebSocket endpoint URL.
    pub url: String,
    /// Optional API key / token ID for authenticated streams.
    pub api_key: Option<String>,
    /// Whether to automatically reconnect on disconnection.
    pub auto_reconnect: bool,
    /// Delay (in milliseconds) before each reconnection attempt.
    pub reconnect_delay_ms: u64,
    /// Maximum number of reconnection attempts (0 = unlimited).
    pub max_reconnect_attempts: u32,
    /// Connection and read timeout (in milliseconds).
    pub timeout_ms: u64,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            url: "wss://ws.limitless.exchange/markets".to_string(),
            api_key: std::env::var("LIMITLESS_API_KEY").ok(),
            auto_reconnect: true,
            reconnect_delay_ms: 1_000,
            max_reconnect_attempts: 0,
            timeout_ms: 10_000,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  FlexFloat — handles string-encoded floats in WS payloads
// ═══════════════════════════════════════════════════════════════════════════

/// A flexible `f64` that deserializes from both JSON numbers and strings.
///
/// The Limitless WebSocket occasionally encodes numeric fields as strings
/// (e.g., `"0.55"` instead of `0.55`). This wrapper handles both formats
/// transparently.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexFloat(pub f64);

impl FlexFloat {
    /// Extract the inner `f64`.
    pub fn float64(self) -> f64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for FlexFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Value::deserialize(deserializer)? {
            Value::Number(n) => n
                .as_f64()
                .map(Self)
                .ok_or_else(|| serde::de::Error::custom("expected f64-compatible number")),
            Value::String(s) => s.parse::<f64>().map(Self).map_err(|err| {
                serde::de::Error::custom(format!("cannot parse float '{s}': {err}"))
            }),
            other => Err(serde::de::Error::custom(format!(
                "cannot deserialize FlexFloat from {other}"
            ))),
        }
    }
}

impl Serialize for FlexFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f64(self.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  WebSocket event payloads
// ═══════════════════════════════════════════════════════════════════════════

/// Generic WebSocket event — used as a fallback when the event type is
/// not recognized by the typed dispatch.
pub type WsEvent = Value;

// ── Typed event dispatch ────────────────────────────────────────────────

/// Typed WebSocket event — wraps all known server-emitted events.
///
/// Variants with named structs carry fully deserialized payloads. Variants
/// that hold raw [`Value`] cover events whose schemas are not yet modelled
/// (or are intentionally left as flexible key-value maps).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WsEventKind {
    /// AMM price update (`newPriceData`).
    NewPriceData(NewPriceData),
    /// CLOB orderbook snapshot (`orderbookUpdate`).
    OrderbookUpdate(OrderbookUpdate),
    /// Oracle price data (`oraclePriceData`).
    OraclePriceData(OraclePriceData),
    /// Public trade event (`trades`).
    TradeEvent(TradeEvent),
    /// Order status update (`orders`).
    OrderUpdate(OrderUpdate),
    /// Fill notification (`fills`).
    FillEvent(FillEvent),
    /// Market statistics update (`markets`).
    MarketUpdateEvent(MarketUpdateEvent),
    /// On-chain transaction event (`transactions`).
    TransactionEvent(TransactionEvent),
    /// Market creation event (`marketCreated`).
    MarketCreatedEvent(MarketCreatedEvent),
    /// Market resolution event (`marketResolved`).
    MarketResolvedEvent(MarketResolvedEvent),
    /// Unrealized PnL leaderboard invalidation hint
    /// (`unrealizedPnlProjectionChanged`).
    UnrealizedPnlProjectionChanged(UnrealizedPnlProjectionHint),
    /// Portfolio position update (`positions`, requires auth) — typed by
    /// market type (`AMM` / `CLOB`).
    Positions(PositionUpdate),
    /// OME state / settlement result (`orderEvent`, requires auth) — typed by
    /// `source` (`OME` / `SETTLEMENT`).
    OrderEvent(OrderEventData),
    /// Live sports data — raw payload (`liveSports`).
    LiveSports(Value),
    /// Live esports data — raw payload (`liveEsports`).
    LiveEsports(Value),
    /// System notification (`system`).
    System(SystemEvent),
    /// Authentication confirmation — raw payload (`authenticated`).
    Authenticated(Value),
    /// Error notification — raw payload (`exception`).
    Exception(Value),
    /// Server error — raw payload (`error`). Emitted for e.g. "All requested markets are resolved".
    Error(Value),
    /// Unknown / unrecognized event with its raw payload.
    Unknown(Value),
}

/// A typed `orderEvent` payload, discriminated by the `source` field.
///
/// * `OME` — off-chain matching-engine updates: lifecycle state changes
///   (`PLACEMENT` / `UPDATE` / `CANCELLATION`) and the terminal result of an
///   immediate-or-cancel order (`EXECUTION`).
/// * `SETTLEMENT` — settlement lifecycle: provisional `MATCHED`, then
///   terminal `MINED` / `FAILED`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OrderEventData {
    /// Off-chain matching engine update.
    Ome(Box<OmeEvent>),
    /// On-chain settlement lifecycle event.
    Settlement(Box<SettlementEvent>),
    /// Unrecognized `source` — raw payload preserved.
    Unknown(Value),
}

/// Map a server-emitted event name and its JSON payload to a typed [`WsEventKind`].
///
/// Returns `None` only when the payload fails to deserialize for a known
/// event type. Unknown event names produce [`WsEventKind::Unknown`].
pub fn deserialize_event(event: &str, payload: &Value) -> Option<WsEventKind> {
    match event {
        "newPriceData" => serde_json::from_value::<NewPriceData>(payload.clone())
            .ok()
            .map(WsEventKind::NewPriceData),
        "orderbookUpdate" => serde_json::from_value::<OrderbookUpdate>(payload.clone())
            .ok()
            .map(WsEventKind::OrderbookUpdate),
        "oraclePriceData" => serde_json::from_value::<OraclePriceData>(payload.clone())
            .ok()
            .map(WsEventKind::OraclePriceData),
        "trades" => serde_json::from_value::<TradeEvent>(payload.clone())
            .ok()
            .map(WsEventKind::TradeEvent),
        "orders" => serde_json::from_value::<OrderUpdate>(payload.clone())
            .ok()
            .map(WsEventKind::OrderUpdate),
        "fills" => serde_json::from_value::<FillEvent>(payload.clone())
            .ok()
            .map(WsEventKind::FillEvent),
        "markets" => serde_json::from_value::<MarketUpdateEvent>(payload.clone())
            .ok()
            .map(WsEventKind::MarketUpdateEvent),
        "transactions" => serde_json::from_value::<TransactionEvent>(payload.clone())
            .ok()
            .map(WsEventKind::TransactionEvent),
        "marketCreated" => serde_json::from_value::<MarketCreatedEvent>(payload.clone())
            .ok()
            .map(WsEventKind::MarketCreatedEvent),
        "marketResolved" => serde_json::from_value::<MarketResolvedEvent>(payload.clone())
            .ok()
            .map(WsEventKind::MarketResolvedEvent),
        "unrealizedPnlProjectionChanged" => {
            serde_json::from_value::<UnrealizedPnlProjectionHint>(payload.clone())
                .ok()
                .map(WsEventKind::UnrealizedPnlProjectionChanged)
        }
        "positions" => serde_json::from_value::<PositionUpdate>(payload.clone())
            .ok()
            .map(WsEventKind::Positions)
            .or_else(|| Some(WsEventKind::Unknown(payload.clone()))),
        "orderEvent" => {
            let parsed = match payload.get("source").and_then(Value::as_str) {
                Some("OME") => serde_json::from_value::<OmeEvent>(payload.clone())
                    .ok()
                    .map(|e| OrderEventData::Ome(Box::new(e))),
                Some("SETTLEMENT") => serde_json::from_value::<SettlementEvent>(payload.clone())
                    .ok()
                    .map(|e| OrderEventData::Settlement(Box::new(e))),
                _ => None,
            };
            Some(WsEventKind::OrderEvent(
                parsed.unwrap_or_else(|| OrderEventData::Unknown(payload.clone())),
            ))
        }
        "liveSports" => Some(WsEventKind::LiveSports(payload.clone())),
        "liveEsports" => Some(WsEventKind::LiveEsports(payload.clone())),
        "system" => serde_json::from_value::<SystemEvent>(payload.clone())
            .ok()
            .map(WsEventKind::System),
        "authenticated" => Some(WsEventKind::Authenticated(payload.clone())),
        "exception" => Some(WsEventKind::Exception(payload.clone())),
        "error" => Some(WsEventKind::Error(payload.clone())),
        _other => Some(WsEventKind::Unknown(payload.clone())),
    }
}

// ── Orderbook ────────────────────────────────────────────────────────────

/// A single level in the orderbook (bid or ask).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderbookLevel {
    pub price: f64,
    pub size: f64,
}

/// Full CLOB orderbook snapshot for a market.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderbookData {
    pub bids: Vec<OrderbookLevel>,
    pub asks: Vec<OrderbookLevel>,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    #[serde(rename = "adjustedMidpoint")]
    pub adjusted_midpoint: f64,
    #[serde(rename = "maxSpread")]
    pub max_spread: FlexFloat,
    #[serde(rename = "minSize")]
    pub min_size: FlexFloat,
}

/// Server-emitted orderbook update event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderbookUpdate {
    #[serde(rename = "marketSlug")]
    pub market_slug: String,
    pub orderbook: OrderbookData,
    pub timestamp: Value,
}

// ── Trades ───────────────────────────────────────────────────────────────

/// A single public trade.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeEvent {
    #[serde(rename = "marketSlug")]
    pub market_slug: String,
    pub side: String,
    pub price: f64,
    pub size: f64,
    pub timestamp: f64,
    #[serde(rename = "tradeId")]
    pub trade_id: String,
}

// ── Orders ───────────────────────────────────────────────────────────────

/// An order status update emitted by the server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderUpdate {
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "marketSlug")]
    pub market_slug: String,
    pub side: String,
    #[serde(default)]
    pub price: Option<f64>,
    pub size: f64,
    pub filled: f64,
    pub status: String,
    pub timestamp: f64,
}

// ── Fills ────────────────────────────────────────────────────────────────

/// A fill (matched trade) notification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FillEvent {
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "marketSlug")]
    pub market_slug: String,
    pub side: String,
    pub price: f64,
    pub size: f64,
    pub timestamp: f64,
    #[serde(rename = "fillId")]
    pub fill_id: String,
}

// ── Market stats ─────────────────────────────────────────────────────────

/// Periodic market-level statistics update.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketUpdateEvent {
    #[serde(rename = "marketSlug")]
    pub market_slug: String,
    #[serde(rename = "lastPrice", default)]
    pub last_price: Option<f64>,
    #[serde(rename = "volume24h", default)]
    pub volume_24h: Option<f64>,
    #[serde(rename = "priceChange24h", default)]
    pub price_change_24h: Option<f64>,
    pub timestamp: f64,
}

// ── AMM prices ───────────────────────────────────────────────────────────

/// The `updatedPrices` object in a `newPriceData` payload: the YES and NO
/// prices of the market (string-encoded decimals on the wire).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdatedPrices {
    pub yes: FlexFloat,
    pub no: FlexFloat,
}

/// Server-emitted AMM price update (the `newPriceData` event).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPriceData {
    #[serde(rename = "marketAddress")]
    pub market_address: String,
    #[serde(rename = "updatedPrices")]
    pub updated_prices: UpdatedPrices,
    #[serde(rename = "blockNumber")]
    pub block_number: i64,
    pub timestamp: Value,
}

// ── Oracle prices ────────────────────────────────────────────────────────

/// Oracle price data for a market.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OraclePriceData {
    #[serde(rename = "marketAddress", default)]
    pub market_address: Option<String>,
    #[serde(rename = "marketSlug")]
    pub market_slug: String,
    pub timestamp: i64,
    pub value: f64,
}

// ── Transactions ─────────────────────────────────────────────────────────

/// On-chain transaction event (deposit, withdrawal, trade settlement).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionEvent {
    #[serde(rename = "userId", default)]
    pub user_id: Option<i32>,
    #[serde(rename = "txHash", default)]
    pub tx_hash: Option<String>,
    pub status: String,
    pub source: String,
    pub timestamp: String,
    #[serde(rename = "marketAddress", default)]
    pub market_address: Option<String>,
    #[serde(rename = "marketSlug", default)]
    pub market_slug: Option<String>,
    #[serde(rename = "tokenId", default)]
    pub token_id: Option<String>,
    #[serde(rename = "conditionId", default)]
    pub condition_id: Option<String>,
    #[serde(rename = "amountContracts", default)]
    pub amount_contracts: Option<String>,
    #[serde(rename = "amountCollateral", default)]
    pub amount_collateral: Option<String>,
    #[serde(default)]
    pub price: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
}

// ── Market lifecycle ─────────────────────────────────────────────────────

/// Emitted when a new market is created and funded.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketCreatedEvent {
    pub slug: String,
    pub title: String,
    #[serde(rename = "type")]
    pub market_type: String,
    #[serde(rename = "groupSlug", default)]
    pub group_slug: Option<String>,
    #[serde(rename = "categoryIds", default)]
    pub category_ids: Vec<i32>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Emitted when a market is resolved with a winning outcome.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketResolvedEvent {
    pub slug: String,
    #[serde(rename = "type")]
    pub market_type: String,
    #[serde(rename = "winningOutcome")]
    pub winning_outcome: String,
    #[serde(rename = "winningIndex")]
    pub winning_index: i32,
    #[serde(rename = "resolutionDate")]
    pub resolution_date: String,
}

/// System notification event (`system`).
///
/// Emitted by the server for subscription confirmations and other
/// informational messages. The `markets` field is present when the
/// message relates to specific market subscriptions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemEvent {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub markets: Option<Vec<String>>,
}

// ── Unrealized PnL projection hint ────────────────────────────────────────

/// Invalidation hint delivered to `subscribe_unrealized_pnl` subscribers.
///
/// Carries no leaderboard rows — refetch the matching REST route
/// (`GET /leaderboard/pnl/unrealized/markets/{marketId}` or
/// `GET /leaderboard/pnl/unrealized/biggest-positions`) when it arrives.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnrealizedPnlProjectionHint {
    #[serde(rename = "schemaVersion", default)]
    pub schema_version: Option<i64>,
    /// `MARKET` or `BIGGEST_POSITIONS`.
    #[serde(default)]
    pub scope: Option<String>,
    /// Market id — present on `MARKET` scope only.
    #[serde(rename = "marketId", default)]
    pub market_id: Option<i64>,
    /// Readiness of the projection you'll read on refetch
    /// (`READY`, `STALE`, `BUILDING`, or `DEGRADED`).
    #[serde(default)]
    pub state: Option<String>,
    #[serde(rename = "projectionVersion", default)]
    pub projection_version: Option<String>,
    #[serde(rename = "scopeVersion", default)]
    pub scope_version: Option<String>,
    #[serde(rename = "presentationVersion", default)]
    pub presentation_version: Option<String>,
    #[serde(rename = "asOf", default)]
    pub as_of: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Normalize `SubscriptionOptions` by copying the singular `market_slug` /
/// `market_address` into the plural vecs when the plural vecs are empty.
///
/// This mirrors the reference SDK behaviour so that callers can supply
/// either form.
pub fn normalize_subscription_options(opts: SubscriptionOptions) -> SubscriptionOptions {
    let mut opts = opts;
    if opts.market_slugs.is_empty() {
        if let Some(ref slug) = opts.market_slug {
            opts.market_slugs = vec![slug.clone()];
        }
    }
    if opts.market_addresses.is_empty() {
        if let Some(ref addr) = opts.market_address {
            opts.market_addresses = vec![addr.clone()];
        }
    }
    opts
}

/// Build a deterministic key for a `(channel, options)` pair, suitable for
/// tracking active subscriptions.
pub fn subscription_key(channel: SubscriptionChannel, opts: &SubscriptionOptions) -> String {
    let slugs = if opts.market_slugs.is_empty() {
        String::new()
    } else {
        let mut sorted: Vec<&str> = opts.market_slugs.iter().map(String::as_str).collect();
        sorted.sort_unstable();
        sorted.join(",")
    };

    let addresses = if opts.market_addresses.is_empty() {
        String::new()
    } else {
        let mut sorted: Vec<&str> = opts.market_addresses.iter().map(String::as_str).collect();
        sorted.sort_unstable();
        sorted.join(",")
    };

    format!("{}|{}|{}", channel.as_str(), slugs, addresses)
}

/// Attempt to recover a `SubscriptionChannel` from its wire-protocol string.
pub fn channel_from_key(key: &str) -> Option<SubscriptionChannel> {
    // The key format is "channel|slugs|addresses" — extract the channel part.
    let channel_str = key.split('|').next().unwrap_or(key);
    match channel_str {
        "orderbook" => Some(SubscriptionChannel::Orderbook),
        "trades" => Some(SubscriptionChannel::Trades),
        "orders" => Some(SubscriptionChannel::Orders),
        "fills" => Some(SubscriptionChannel::Fills),
        "markets" => Some(SubscriptionChannel::Markets),
        "prices" => Some(SubscriptionChannel::Prices),
        "positions" => Some(SubscriptionChannel::Positions),
        "transactions" => Some(SubscriptionChannel::Transactions),
        "orderEvent" => Some(SubscriptionChannel::OrderEvents),
        "liveSports" => Some(SubscriptionChannel::LiveSports),
        "liveEsports" => Some(SubscriptionChannel::LiveEsports),
        "marketLifecycle" => Some(SubscriptionChannel::MarketLifecycle),
        "unrealizedPnlProjectionChanged" => {
            Some(SubscriptionChannel::UnrealizedPnlProjectionChanged)
        }
        "subscribe_market_prices" => Some(SubscriptionChannel::SubscribeMarketPrices),
        "subscribe_positions" => Some(SubscriptionChannel::SubscribePositions),
        "subscribe_transactions" => Some(SubscriptionChannel::SubscribeTransactions),
        "subscribe_order_events" => Some(SubscriptionChannel::SubscribeOrderEvents),
        "subscribe_live_sports" => Some(SubscriptionChannel::SubscribeLiveSports),
        "subscribe_live_esports" => Some(SubscriptionChannel::SubscribeLiveEsports),
        "subscribe_market_lifecycle" => Some(SubscriptionChannel::SubscribeMarketLifecycle),
        "unsubscribe_market_lifecycle" => Some(SubscriptionChannel::UnsubscribeMarketLifecycle),
        "subscribe_unrealized_pnl" => Some(SubscriptionChannel::SubscribeUnrealizedPnl),
        "unsubscribe_unrealized_pnl" => Some(SubscriptionChannel::UnsubscribeUnrealizedPnl),
        _ => None,
    }
}

/// Returns `true` when the given channel requires API-key authentication.
pub fn requires_websocket_auth(channel: SubscriptionChannel) -> bool {
    matches!(
        channel,
        SubscriptionChannel::SubscribePositions
            | SubscriptionChannel::SubscribeTransactions
            | SubscriptionChannel::SubscribeOrderEvents
    )
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_key_is_order_independent() {
        let opts_a = SubscriptionOptions {
            market_slugs: vec!["btc-above-100k".into(), "eth-merge".into()],
            ..Default::default()
        };
        let opts_b = SubscriptionOptions {
            market_slugs: vec!["eth-merge".into(), "btc-above-100k".into()],
            ..Default::default()
        };
        assert_eq!(
            subscription_key(SubscriptionChannel::SubscribeMarketPrices, &opts_a),
            subscription_key(SubscriptionChannel::SubscribeMarketPrices, &opts_b),
        );
    }

    #[test]
    fn normalize_copies_singular_into_plural() {
        let opts = SubscriptionOptions {
            market_slug: Some("test-slug".into()),
            market_address: Some("0xdead".into()),
            ..Default::default()
        };
        let normalized = normalize_subscription_options(opts);
        assert_eq!(normalized.market_slugs, vec!["test-slug"]);
        assert_eq!(normalized.market_addresses, vec!["0xdead"]);
    }

    #[test]
    fn normalize_preserves_existing_plurals() {
        let opts = SubscriptionOptions {
            market_slugs: vec!["existing".into()],
            ..Default::default()
        };
        let normalized = normalize_subscription_options(opts);
        assert_eq!(normalized.market_slugs, vec!["existing"]);
    }

    #[test]
    fn channel_from_key_roundtrips() {
        for channel in &[
            SubscriptionChannel::Orderbook,
            SubscriptionChannel::Trades,
            SubscriptionChannel::SubscribeMarketPrices,
            SubscriptionChannel::SubscribePositions,
            SubscriptionChannel::OrderEvents,
            SubscriptionChannel::MarketLifecycle,
        ] {
            let key = subscription_key(*channel, &SubscriptionOptions::default());
            let recovered = channel_from_key(&key);
            assert_eq!(
                recovered,
                Some(*channel),
                "round-trip failed for {channel:?}"
            );
        }
    }

    #[test]
    fn requires_auth_returns_true_for_private_channels() {
        assert!(requires_websocket_auth(
            SubscriptionChannel::SubscribePositions
        ));
        assert!(requires_websocket_auth(
            SubscriptionChannel::SubscribeTransactions
        ));
        assert!(requires_websocket_auth(
            SubscriptionChannel::SubscribeOrderEvents
        ));
    }

    #[test]
    fn requires_auth_returns_false_for_public_channels() {
        assert!(!requires_websocket_auth(
            SubscriptionChannel::SubscribeMarketPrices
        ));
        assert!(!requires_websocket_auth(
            SubscriptionChannel::SubscribeMarketLifecycle
        ));
    }

    #[test]
    fn flexfloat_parses_number_and_string() {
        let from_number: FlexFloat = serde_json::from_str("0.55").unwrap();
        assert!((from_number.float64() - 0.55).abs() < f64::EPSILON);

        let from_string: FlexFloat = serde_json::from_str(r#""0.55""#).unwrap();
        assert!((from_string.float64() - 0.55).abs() < f64::EPSILON);
    }

    #[test]
    fn websocket_channel_inventory_includes_all_server_events() {
        // Ensure every server-emitted event name has a corresponding variant.
        let server_events = [
            "orderbook",
            "trades",
            "orders",
            "fills",
            "markets",
            "prices",
            "positions",
            "transactions",
            "orderEvent",
            "liveSports",
            "liveEsports",
            "marketLifecycle",
            "unrealizedPnlProjectionChanged",
        ];
        for &event in &server_events {
            assert!(
                channel_from_key(event).is_some(),
                "missing channel variant for server event '{event}'"
            );
        }
    }

    #[test]
    fn new_price_data_parses_documented_shape() {
        let json = r#"{
            "marketAddress": "0x1234...",
            "updatedPrices": { "yes": "0.65", "no": "0.35" },
            "blockNumber": 12345678,
            "timestamp": "2024-01-01T00:00:00.000Z"
        }"#;
        let parsed: NewPriceData = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.market_address, "0x1234...");
        assert!((parsed.updated_prices.yes.float64() - 0.65).abs() < f64::EPSILON);
        assert!((parsed.updated_prices.no.float64() - 0.35).abs() < f64::EPSILON);
    }

    #[test]
    fn order_event_dispatches_by_source() {
        let ome = serde_json::json!({
            "source": "OME", "type": "PLACEMENT", "eventId": 1234567,
            "orderId": "550e8400-e29b-41d4-a716-446655440000",
            "userId": 42, "marketId": "17", "token": "878930",
            "side": "BUY", "price": 0.53, "remainingSize": 100,
            "timestamp": "2026-04-20T10:15:30.000Z"
        });
        match deserialize_event("orderEvent", &ome) {
            Some(WsEventKind::OrderEvent(OrderEventData::Ome(e))) => {
                assert_eq!(e.event_type, "PLACEMENT");
            }
            other => panic!("unexpected dispatch: {other:?}"),
        }

        let settlement = serde_json::json!({
            "source": "SETTLEMENT", "type": "MATCHED",
            "eventId": "matched:77985c10:d45b884d",
            "tradeEventId": "77985c10", "orderId": "d45b884d",
            "takerOrderId": "d45b884d", "marketSlug": "will-abc-happen-by-2026",
            "tokenId": "27102822276156300166", "token": "NO", "side": "BUY",
            "price": "0.53", "amountContracts": "25", "amountCollateral": "13.25",
            "configuredFeeRateBps": 30, "effectiveFeeBps": 27,
            "feeAmountContracts": "0.0675", "isEstimate": true,
            "timestamp": "2026-04-20T10:15:40.000Z"
        });
        match deserialize_event("orderEvent", &settlement) {
            Some(WsEventKind::OrderEvent(OrderEventData::Settlement(e))) => {
                assert_eq!(e.event_type, "MATCHED");
                assert_eq!(e.is_estimate, Some(true));
                assert_eq!(e.user_id, None);
                assert_eq!(e.trade_event_id.as_deref(), Some("77985c10"));
            }
            other => panic!("unexpected dispatch: {other:?}"),
        }
    }

    #[test]
    fn positions_dispatches_by_market_type() {
        let amm = serde_json::json!({
            "account": "0xabcd...", "marketAddress": "0x1234...",
            "positions": [{
                "tokenId": "123456", "balance": "1000000",
                "outcomeIndex": 0, "collateralOutOnSell": "950000"
            }],
            "type": "AMM"
        });
        match deserialize_event("positions", &amm) {
            Some(WsEventKind::Positions(PositionUpdate::Amm(p))) => {
                assert_eq!(p.market_address, "0x1234...");
                assert_eq!(p.positions.len(), 1);
            }
            other => panic!("unexpected dispatch: {other:?}"),
        }

        let clob = serde_json::json!({
            "account": "0xabcd...", "marketSlug": "btc-100k-weekly",
            "positions": [{
                "tokenId": "19633204485790", "ctfBalance": "10000000",
                "averageFillPrice": "0.65", "costBasis": "6500000",
                "marketValue": "7000000", "marketId": 7348
            }],
            "tokenIds": ["19633204485790"],
            "timestamp": 1783728000000i64,
            "type": "CLOB"
        });
        match deserialize_event("positions", &clob) {
            Some(WsEventKind::Positions(PositionUpdate::Clob(p))) => {
                assert_eq!(p.market_slug, "btc-100k-weekly");
                assert_eq!(p.timestamp, Some(1783728000000));
            }
            other => panic!("unexpected dispatch: {other:?}"),
        }
    }

    #[test]
    fn unrealized_pnl_subscription_serializes() {
        let market = UnrealizedPnlSubscription::market(7348);
        let value = market.to_value();
        assert_eq!(value["scope"], "MARKET");
        assert_eq!(value["marketId"], 7348);
        assert_eq!(value["schemaVersion"], 1);

        let biggest = UnrealizedPnlSubscription::biggest_positions();
        let value = biggest.to_value();
        assert_eq!(value["scope"], "BIGGEST_POSITIONS");
        assert!(value.get("marketId").is_none());
    }

    #[test]
    fn no_payload_frame_has_single_element_array() {
        let frame = crate::ws::stream::frame_socketio_event_no_payload("subscribe_order_events");
        assert_eq!(frame, "42/markets,[\"subscribe_order_events\"]");
    }
}
