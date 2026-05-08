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
            Self::SubscribeMarketPrices => "subscribe_market_prices",
            Self::SubscribePositions => "subscribe_positions",
            Self::SubscribeTransactions => "subscribe_transactions",
            Self::SubscribeOrderEvents => "subscribe_order_events",
            Self::SubscribeLiveSports => "subscribe_live_sports",
            Self::SubscribeLiveEsports => "subscribe_live_esports",
            Self::SubscribeMarketLifecycle => "subscribe_market_lifecycle",
            Self::UnsubscribeMarketLifecycle => "unsubscribe_market_lifecycle",
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

/// A per-market AMM price entry within a `NewPriceData` payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AmmPriceEntry {
    #[serde(rename = "marketId")]
    pub market_id: i32,
    #[serde(rename = "marketAddress")]
    pub market_address: String,
    #[serde(rename = "yesPrice")]
    pub yes_price: f64,
    #[serde(rename = "noPrice")]
    pub no_price: f64,
}

/// Server-emitted AMM price update (the `newPriceData` event).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPriceData {
    #[serde(rename = "marketAddress")]
    pub market_address: String,
    #[serde(rename = "updatedPrices")]
    pub updated_prices: Vec<AmmPriceEntry>,
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
        "subscribe_market_prices" => Some(SubscriptionChannel::SubscribeMarketPrices),
        "subscribe_positions" => Some(SubscriptionChannel::SubscribePositions),
        "subscribe_transactions" => Some(SubscriptionChannel::SubscribeTransactions),
        "subscribe_order_events" => Some(SubscriptionChannel::SubscribeOrderEvents),
        "subscribe_live_sports" => Some(SubscriptionChannel::SubscribeLiveSports),
        "subscribe_live_esports" => Some(SubscriptionChannel::SubscribeLiveEsports),
        "subscribe_market_lifecycle" => Some(SubscriptionChannel::SubscribeMarketLifecycle),
        "unsubscribe_market_lifecycle" => Some(SubscriptionChannel::UnsubscribeMarketLifecycle),
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
        ];
        for &event in &server_events {
            assert!(
                channel_from_key(event).is_some(),
                "missing channel variant for server event '{event}'"
            );
        }
    }
}
