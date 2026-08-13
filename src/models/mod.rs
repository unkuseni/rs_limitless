//! Request and response model types for the Limitless Exchange API.
//!
//! All REST API responses are fully typed with concrete structs matching
//! the API's JSON shape. WebSocket event types and order types are also
//! fully typed for compile-time safety.
//!
//! # Convention
//!
//! - Field names use `snake_case` in Rust and `camelCase` on the wire via
//!   `#[serde(rename = "...")]` where needed.
//! - Numeric fields that arrive as JSON strings use `serde_helpers` to
//!   deserialize transparently.

pub mod order;

use crate::ws::FlexFloat;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Generic wrapper ──

/// Wraps data with a server-side timestamp for freshness tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timed<T> {
    pub time: u64,
    pub data: T,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Markets
// ═══════════════════════════════════════════════════════════════════════════

/// Response from `GET /markets/active`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveMarketsResponse {
    pub data: Vec<MarketSummary>,
    #[serde(rename = "totalMarketsCount")]
    pub total_markets_count: i32,
}

/// Summary view of a market in the active-markets list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSummary {
    pub id: i32,
    pub slug: String,
    pub title: String,
    #[serde(rename = "proxyTitle", default)]
    pub proxy_title: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "collateralToken")]
    pub collateral_token: CollateralTokenInfo,
    #[serde(rename = "expirationDate")]
    pub expiration_date: String,
    #[serde(rename = "expirationTimestamp")]
    pub expiration_timestamp: i64,
    #[serde(default)]
    pub expired: Option<bool>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub categories: Vec<String>,
    pub status: String,
    pub creator: MarketCreatorInfo,
    pub tags: Vec<String>,
    #[serde(rename = "tradeType")]
    pub trade_type: String,
    #[serde(rename = "marketType")]
    pub market_type: String,
    #[serde(rename = "priorityIndex")]
    pub priority_index: i32,
    pub metadata: MarketMetadataInfo,
    #[serde(default)]
    pub volume: Option<String>,
    #[serde(rename = "volumeFormatted", default)]
    pub volume_formatted: Option<String>,
    #[serde(rename = "automationType", default)]
    pub automation_type: Option<String>,
    #[serde(rename = "imageUrl", default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub trends: Option<Value>,
    #[serde(rename = "openInterest", default)]
    pub open_interest: Option<String>,
    #[serde(rename = "openInterestFormatted", default)]
    pub open_interest_formatted: Option<String>,
    #[serde(default)]
    pub liquidity: Option<String>,
    #[serde(rename = "liquidityFormatted", default)]
    pub liquidity_formatted: Option<String>,
    #[serde(rename = "positionIds", default)]
    pub position_ids: Vec<String>,
    #[serde(rename = "conditionId", default)]
    pub condition_id: Option<String>,
    #[serde(rename = "negRiskRequestId", default)]
    pub neg_risk_request_id: Option<String>,
    #[serde(default)]
    pub tokens: Option<MarketTokensInfo>,
    #[serde(default)]
    pub prices: Vec<f64>,
    #[serde(rename = "tradePrices", default)]
    pub trade_prices: Option<TradePricesInfo>,
    #[serde(rename = "isRewardable", default)]
    pub is_rewardable: Option<bool>,
    #[serde(default)]
    pub settings: Option<Value>,
    #[serde(default)]
    pub venue: Option<VenueInfo>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(rename = "priceOracleMetadata", default)]
    pub price_oracle_data: Option<Value>,
    #[serde(rename = "orderInGroup", default)]
    pub order_in_group: Option<i32>,
    #[serde(rename = "winningOutcomeIndex", default)]
    pub winning_outcome_idx: Option<i32>,
    #[serde(rename = "outcomeTokens", default)]
    pub outcome_tokens: Vec<String>,
    #[serde(rename = "ogImageURI", default)]
    pub og_image_uri: Option<String>,
    #[serde(rename = "negRiskMarketId", default)]
    pub neg_risk_market_id: Option<String>,
    #[serde(default)]
    pub markets: Vec<MarketSummary>,
    #[serde(rename = "dailyReward", default)]
    pub daily_reward: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(rename = "type", default)]
    pub market_type_legacy: Option<String>,
    #[serde(default)]
    pub outcomes: Vec<OutcomeInfo>,
    #[serde(rename = "resolutionDate", default)]
    pub resolution_date: Option<String>,
}

// NOTE: MarketSummary and MarketDetail share the same shape from the API.
// We alias them for semantic clarity.
pub type MarketDetail = MarketSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollateralTokenInfo {
    pub address: String,
    pub decimals: i32,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketCreatorInfo {
    pub name: String,
    #[serde(rename = "imageURI", default)]
    pub image_uri: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketMetadataInfo {
    pub fee: bool,
    #[serde(rename = "isBannered", default)]
    pub is_bannered: Option<bool>,
    #[serde(rename = "isPolyArbitrage", default)]
    pub is_poly_arbitrage: Option<bool>,
    #[serde(rename = "shouldMarketMake", default)]
    pub should_market_make: Option<bool>,
    #[serde(rename = "openPrice", default)]
    pub open_price: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTokensInfo {
    pub yes: String,
    pub no: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePricesInfo {
    pub buy: PriceSideInfo,
    pub sell: PriceSideInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSideInfo {
    pub market: [f64; 2],
    pub limit: [f64; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueInfo {
    pub exchange: String,
    #[serde(default)]
    pub adapter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeInfo {
    pub id: i32,
    pub title: String,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    #[serde(default)]
    pub price: Option<f64>,
}

/// Response from `GET /markets/categories/count`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCountResponse {
    pub data: Value,
}

/// Active market slug entry from `GET /markets/active/slugs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSlug {
    pub slug: String,
    #[serde(default)]
    pub ticker: Option<String>,
    #[serde(default)]
    pub strike_price: Option<String>,
    #[serde(default)]
    pub deadline: Option<String>,
}

/// Response from `GET /markets/{addr}/oracle-candles`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleCandlesResponse {
    pub data: Vec<OracleCandle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleCandle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

/// Response from `GET /markets/{slug}/get-feed-events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedEventsResponse {
    pub events: Vec<FeedEvent>,
    #[serde(default)]
    pub total: Option<i64>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub data: Option<Value>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// Response from `GET /markets/search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub data: Vec<MarketSummary>,
    #[serde(default)]
    pub total: Option<i64>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Trading
// ═══════════════════════════════════════════════════════════════════════════

/// Response from `POST /orders`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub order: CreatedOrderInfo,
    #[serde(rename = "makerMatches", default)]
    pub maker_matches: Vec<MakerMatchInfo>,
    /// Execution and settlement summary (matching result, fees, totals).
    #[serde(default)]
    pub execution: Option<OrderExecutionInfo>,
}

/// Execution and settlement summary for a created order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderExecutionInfo {
    /// Whether the order was matched immediately.
    #[serde(default)]
    pub matched: Option<bool>,
    /// `UNMATCHED`, `MATCHED`, `MINED`, `CONFIRMED`, `RETRYING`, `FAILED`,
    /// `DELAYED` (taker delay), or `CANCELED` (self-trade prevention).
    #[serde(rename = "settlementStatus", default)]
    pub settlement_status: Option<String>,
    /// Reason the order was canceled (e.g. `STP_TAKER_REJECTED`).
    #[serde(default)]
    pub reason: Option<String>,
    /// ISO-8601 time a delayed order is released to the matching engine.
    #[serde(rename = "eligibleAt", default)]
    pub eligible_at: Option<String>,
    /// Trade event ID (present when matched).
    #[serde(rename = "tradeEventId", default)]
    pub trade_event_id: Option<String>,
    /// On-chain transaction hash (present when mined).
    #[serde(rename = "txHash", default)]
    pub tx_hash: Option<String>,
    /// Echo of the client-provided idempotency key.
    #[serde(rename = "clientOrderId", default)]
    pub client_order_id: Option<String>,
    /// Resting order IDs canceled by self-trade prevention.
    #[serde(rename = "stpMakerCancels", default)]
    pub stp_maker_cancels: Option<Vec<String>>,
    /// Fee rate in basis points applied to this order.
    #[serde(rename = "feeRateBps", default)]
    pub fee_rate_bps: Option<f64>,
    /// Effective fee rate in basis points after rebates.
    #[serde(rename = "effectiveFeeBps", default)]
    pub effective_fee_bps: Option<f64>,
    /// Raw execution totals in contract units.
    #[serde(rename = "totalsRaw", default)]
    pub totals_raw: Option<OrderExecutionTotalsRaw>,
}

/// Raw execution totals in contract units (decimal strings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderExecutionTotalsRaw {
    #[serde(rename = "contractsGross", default)]
    pub contracts_gross: Option<String>,
    #[serde(rename = "contractsFee", default)]
    pub contracts_fee: Option<String>,
    #[serde(rename = "contractsNet", default)]
    pub contracts_net: Option<String>,
    #[serde(rename = "usdGross", default)]
    pub usd_gross: Option<String>,
    #[serde(rename = "usdFee", default)]
    pub usd_fee: Option<String>,
    #[serde(rename = "usdNet", default)]
    pub usd_net: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedOrderInfo {
    pub id: String,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
    #[serde(rename = "makerAmount")]
    pub maker_amount: Value,
    #[serde(rename = "takerAmount")]
    pub taker_amount: Value,
    #[serde(default)]
    pub expiration: Option<String>,
    #[serde(rename = "signatureType")]
    pub signature_type: i32,
    pub salt: Value,
    pub maker: String,
    pub signer: String,
    pub taker: String,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    pub side: Value,
    #[serde(rename = "feeRateBps")]
    pub fee_rate_bps: i32,
    #[serde(default)]
    pub nonce: Option<FlexFloat>,
    pub signature: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    /// Order price (decimal string on the wire, 0.01–0.99).
    #[serde(default)]
    pub price: Option<FlexFloat>,
    #[serde(rename = "marketId")]
    pub market_id: i32,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(rename = "filledSize", default)]
    pub filled_size: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakerMatchInfo {
    pub id: String,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
    #[serde(rename = "matchedSize")]
    pub matched_size: Value,
    #[serde(rename = "orderId")]
    pub order_id: String,
}

/// Response from `POST /orders/status/batch`.
pub type OrderStatusBatchResponse = Value;

/// Response from `POST /orders/cancel` and `DELETE /orders/:id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
}

/// Response from `POST /orders/cancel-batch`.
pub type CancelBatchResponse = Value;

// ── Cancel-and-replace ──

/// Outcome of a cancel-and-replace operation (cancellation + replacement
/// have independent results).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelReplaceResponse {
    pub cancel: CancelReplaceCancelResult,
    pub replacement: CancelReplacePlacementResult,
}

/// Result of the cancellation leg.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelReplaceCancelResult {
    /// `SUCCESS`, `FAILURE`, or `UNKNOWN`.
    pub status: String,
    /// Internal order ID of the canceled order (on success).
    #[serde(rename = "orderId", default)]
    pub order_id: Option<String>,
    /// Client-provided order ID of the canceled order (on success).
    #[serde(rename = "clientOrderId", default)]
    pub client_order_id: Option<String>,
    /// Error details when the cancellation failed.
    #[serde(default)]
    pub error: Option<CancelReplaceErrorInfo>,
}

/// Result of the replacement leg.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelReplacePlacementResult {
    /// `SUCCESS`, `FAILURE`, `UNKNOWN`, or `NOT_ATTEMPTED`.
    pub status: String,
    /// Order response data when the replacement succeeded.
    #[serde(default)]
    pub data: Option<CreateOrderResponse>,
    /// Error details when the replacement failed.
    #[serde(default)]
    pub error: Option<CancelReplaceErrorInfo>,
}

/// Machine-readable error for a cancel-replace leg.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelReplaceErrorInfo {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

/// Response from `POST /orders/cancel-replace/batch`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelReplaceBatchResponse {
    /// Outcomes in input order; each item carries its zero-based input index.
    pub results: Vec<CancelReplaceBatchItem>,
}

/// One operation outcome within a cancel-replace batch response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelReplaceBatchItem {
    /// Zero-based input operation index.
    pub index: u64,
    pub cancel: CancelReplaceCancelResult,
    pub replacement: CancelReplacePlacementResult,
}

/// Response from `DELETE /orders/all/:slug`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelAllResponse {
    #[serde(default)]
    pub message: Option<String>,
}

/// Response from `GET /markets/:slug/orderbook`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookResponse {
    pub bids: Vec<OrderbookEntry>,
    pub asks: Vec<OrderbookEntry>,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    #[serde(rename = "adjustedMidpoint")]
    pub adjusted_midpoint: f64,
    #[serde(rename = "maxSpread")]
    pub max_spread: String,
    #[serde(rename = "minSize")]
    pub min_size: String,
    #[serde(rename = "lastTradePrice")]
    pub last_trade_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookEntry {
    pub price: f64,
    pub size: f64,
    pub side: String,
}

/// Historical price data point from `GET /markets/:slug/historical-price`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPriceData {
    pub timestamp: i64,
    pub price: f64,
}

/// Response from `GET /markets/:slug/locked-balance`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedBalanceResponse {
    #[serde(rename = "lockedBalance")]
    pub locked_balance: String,
    #[serde(rename = "lockedBalanceFormatted", default)]
    pub locked_balance_formatted: Option<String>,
}

/// Response from `GET /markets/:slug/user-orders`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOrdersResponse {
    pub data: Vec<UserOrderInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOrderInfo {
    pub id: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "makerAmount")]
    pub maker_amount: Value,
    #[serde(rename = "takerAmount")]
    pub taker_amount: Value,
    #[serde(default)]
    pub expiration: Option<String>,
    #[serde(rename = "signatureType")]
    pub signature_type: i32,
    pub salt: Value,
    pub maker: String,
    pub signer: String,
    pub taker: String,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    pub side: Value,
    #[serde(rename = "feeRateBps")]
    pub fee_rate_bps: i32,
    #[serde(default)]
    pub nonce: Option<FlexFloat>,
    pub signature: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    #[serde(default)]
    pub price: Option<FlexFloat>,
    #[serde(rename = "marketId")]
    pub market_id: i32,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(rename = "filledSize", default)]
    pub filled_size: Option<Value>,
}

/// Response from `GET /markets/:slug/events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEventsResponse {
    pub events: Vec<Value>,
    #[serde(default)]
    pub total: Option<i64>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Portfolio
// ═══════════════════════════════════════════════════════════════════════════

/// Response from `GET /profiles/:account`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResponse {
    pub id: i32,
    pub account: String,
    #[serde(default)]
    pub rank: Option<RankInfo>,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "pfpUrl", default)]
    pub pfp_url: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(rename = "socialUrl", default)]
    pub social_url: Option<String>,
    #[serde(rename = "tradeWalletOption", default)]
    pub trade_wallet_option: Option<String>,
    #[serde(rename = "embeddedAccount", default)]
    pub embedded_account: Option<String>,
    #[serde(default)]
    pub points: Option<f64>,
    #[serde(rename = "accumulativePoints", default)]
    pub accumulative_points: Option<f64>,
    #[serde(rename = "enrolledInPointsProgram", default)]
    pub enrolled_in_points_program: Option<bool>,
    #[serde(rename = "leaderboardPosition", default)]
    pub leaderboard_position: Option<i32>,
    #[serde(rename = "referralData", default)]
    pub referral_data: Vec<ReferralDataInfo>,
    #[serde(rename = "referredUsersCount", default)]
    pub referred_users_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankInfo {
    pub id: i32,
    pub name: String,
    #[serde(rename = "feeRateBps")]
    pub fee_rate_bps: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralDataInfo {
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub id: i32,
    #[serde(rename = "referredProfileId")]
    pub referred_profile_id: i32,
    #[serde(rename = "pfpUrl", default)]
    pub pfp_url: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

/// AMM trade entry from `GET /portfolio/trades`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEntry {
    #[serde(rename = "transactionHash", default)]
    pub transaction_hash: Option<String>,
    #[serde(rename = "blockTimestamp")]
    pub block_timestamp: i64,
    #[serde(rename = "collateralAmount", default)]
    pub collateral_amount: Option<String>,
    #[serde(default)]
    pub market: Option<TradeMarketInfo>,
    #[serde(rename = "outcomeIndex", default)]
    pub outcome_index: Option<i32>,
    #[serde(rename = "outcomeTokenAmount", default)]
    pub outcome_token_amount: Option<String>,
    #[serde(default)]
    pub strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeMarketInfo {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub closed: bool,
}

/// Response from `GET /portfolio/positions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionsResponse {
    #[serde(default)]
    pub amm: Vec<AmmPositionEntry>,
    #[serde(default)]
    pub clob: Vec<ClobPositionEntry>,
    #[serde(default)]
    pub group: Vec<Value>,
    #[serde(default)]
    pub points: Option<String>,
    #[serde(rename = "accumulativePoints", default)]
    pub accumulative_points: Option<String>,
    #[serde(default)]
    pub rewards: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmPositionEntry {
    pub market: PositionMarketInfo,
    pub account: String,
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    #[serde(rename = "collateralAmount")]
    pub collateral_amount: String,
    #[serde(rename = "outcomeTokenAmount")]
    pub outcome_token_amount: String,
    #[serde(rename = "averageFillPrice")]
    pub average_fill_price: String,
    #[serde(rename = "totalBuysCost")]
    pub total_buys_cost: String,
    #[serde(rename = "totalSellsCost")]
    pub total_sells_cost: String,
    #[serde(rename = "realizedPnl")]
    pub realized_pnl: String,
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: String,
    #[serde(rename = "latestTrade", default)]
    pub latest_trade: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobPositionEntry {
    pub market: PositionMarketInfo,
    #[serde(rename = "makerAddress")]
    pub maker_address: String,
    pub positions: ClobPositionSides,
    #[serde(rename = "tokensBalance")]
    pub tokens_balance: PositionTokenBalance,
    #[serde(rename = "latestTrade")]
    pub latest_trade: PositionLatestTrade,
    #[serde(default)]
    pub orders: Option<Value>,
    #[serde(default)]
    pub rewards: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionMarketInfo {
    pub id: Value,
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub status: Option<String>,
    pub closed: bool,
    pub deadline: String,
    #[serde(rename = "conditionId", default)]
    pub condition_id: Option<String>,
    #[serde(rename = "winningOutcomeIndex", default)]
    pub winning_outcome_index: Option<i32>,
    #[serde(default)]
    pub group: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobPositionSides {
    pub yes: PositionSideInfo,
    pub no: PositionSideInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSideInfo {
    pub cost: String,
    #[serde(rename = "fillPrice")]
    pub fill_price: String,
    #[serde(rename = "marketValue")]
    pub market_value: String,
    #[serde(rename = "realisedPnl")]
    pub realised_pnl: String,
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionTokenBalance {
    pub yes: String,
    pub no: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLatestTrade {
    #[serde(rename = "latestYesPrice", default)]
    pub latest_yes_price: Option<f64>,
    #[serde(rename = "latestNoPrice", default)]
    pub latest_no_price: Option<f64>,
    #[serde(rename = "outcomeTokenPrice", default)]
    pub outcome_token_price: Option<f64>,
}

/// Response from `GET /portfolio/pnl-chart`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnlChartResponse {
    #[serde(default)]
    pub data: Vec<PnlChartPoint>,
    #[serde(rename = "totalValue", default)]
    pub total_value: Option<f64>,
    #[serde(rename = "totalUnrealizedPnl", default)]
    pub total_unrealized_pnl: Option<f64>,
    #[serde(rename = "totalRealizedPnl", default)]
    pub total_realized_pnl: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnlChartPoint {
    pub timestamp: i64,
    pub value: f64,
}

/// Response from `GET /portfolio/points`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointsResponse {
    #[serde(default)]
    pub points: Option<f64>,
    #[serde(rename = "accumulativePoints", default)]
    pub accumulative_points: Option<f64>,
    #[serde(default)]
    pub breakdown: Vec<Value>,
}

/// Response from `GET /portfolio/history`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub data: Vec<HistoryEntry>,
    #[serde(rename = "nextCursor", default)]
    pub next_cursor: Option<String>,
    #[serde(rename = "totalCount", default)]
    pub total_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    #[serde(rename = "blockTimestamp")]
    pub block_timestamp: i64,
    #[serde(rename = "collateralAmount", default)]
    pub collateral_amount: Option<String>,
    #[serde(default)]
    pub market: Option<HistoryMarketInfo>,
    #[serde(rename = "outcomeIndex", default)]
    pub outcome_index: Option<i32>,
    #[serde(rename = "outcomeTokenAmount", default)]
    pub outcome_token_amount: Option<String>,
    #[serde(rename = "outcomeTokenAmounts", default)]
    pub outcome_token_amounts: Option<Vec<String>>,
    #[serde(rename = "outcomeTokenPrice", default)]
    pub outcome_token_price: Option<Value>,
    #[serde(default)]
    pub strategy: Option<String>,
    #[serde(rename = "transactionHash", default)]
    pub transaction_hash: Option<String>,
    /// Operation kind for CLOB rows (`buy`, `sell`, …).
    #[serde(default)]
    pub operation: Option<String>,
    /// Trade event ID reconciling a CLOB row back to the trade that produced it.
    #[serde(rename = "tradeEventId", default)]
    pub trade_event_id: Option<String>,
    /// Internal order ID for CLOB-sourced rows.
    #[serde(rename = "orderId", default)]
    pub order_id: Option<String>,
    /// Maker match ID for CLOB-sourced rows.
    #[serde(rename = "makerMatchId", default)]
    pub maker_match_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMarketInfo {
    pub closed: bool,
    #[serde(default)]
    pub collateral: Option<Value>,
    #[serde(default)]
    pub group: Option<Value>,
    #[serde(rename = "conditionId", default)]
    pub condition_id: Option<String>,
    #[serde(default)]
    pub funding: Option<String>,
    pub id: String,
    pub slug: String,
    pub title: String,
    #[serde(rename = "expirationDate", default)]
    pub expiration_date: Option<String>,
}

/// Response from `GET /portfolio/trading/allowance`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowanceResponse {
    #[serde(default)]
    pub allowance: Option<String>,
    #[serde(rename = "allowanceFormatted", default)]
    pub allowance_formatted: Option<String>,
    #[serde(rename = "approvedSpender", default)]
    pub approved_spender: Option<String>,
    #[serde(rename = "approvedAmount", default)]
    pub approved_amount: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Navigation
// ═══════════════════════════════════════════════════════════════════════════

/// Navigation tree node from `GET /navigation`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationNode {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub path: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub children: Vec<NavigationNode>,
}

/// Market page from `GET /market-pages/by-path`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPage {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(rename = "fullPath")]
    pub full_path: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "baseFilter")]
    pub base_filter: Value,
    #[serde(rename = "filterGroups")]
    pub filter_groups: Vec<FilterGroupInfo>,
    pub metadata: Value,
    pub breadcrumb: Vec<BreadcrumbItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterGroupInfo {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(rename = "allowMultiple", default)]
    pub allow_multiple: Option<bool>,
    #[serde(default)]
    pub presentation: Option<String>,
    #[serde(default)]
    pub options: Vec<FilterGroupOption>,
    #[serde(default)]
    pub source: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterGroupOption {
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    pub name: String,
    pub slug: String,
    pub path: String,
}

/// Response from `GET /market-pages/:id/markets`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMarketsResponse {
    pub data: Vec<MarketSummary>,
    #[serde(default)]
    pub pagination: Option<OffsetPagination>,
    #[serde(default)]
    pub cursor: Option<CursorPagination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffsetPagination {
    pub page: i32,
    pub limit: i32,
    pub total: i32,
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPagination {
    #[serde(rename = "nextCursor", default)]
    pub next_cursor: Option<String>,
}

/// Property key from `GET /property-keys` and `GET /property-keys/:id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyKey {
    pub id: String,
    pub name: String,
    pub slug: String,
    #[serde(rename = "type")]
    pub property_type: String,
    pub metadata: Value,
    #[serde(rename = "isSystem")]
    pub is_system: bool,
    #[serde(default)]
    pub options: Vec<PropertyOption>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// Property option from `GET /property-keys/:id/options`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyOption {
    pub id: String,
    #[serde(rename = "propertyKeyId")]
    pub property_key_id: String,
    pub value: String,
    pub label: String,
    #[serde(rename = "sortOrder")]
    pub sort_order: i32,
    #[serde(rename = "parentOptionId", default)]
    pub parent_option_id: Option<String>,
    pub metadata: Value,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

// ═══════════════════════════════════════════════════════════════════════════
//  WebSocket events — OME, settlement, and position types
// ═══════════════════════════════════════════════════════════════════════════
//
// NOTE: General-purpose WS event types (OrderbookUpdate, TradeEvent,
// MarketCreatedEvent, NewPriceData, etc.) now live in `ws::channel`.
// The types below are the detailed OME/settlement/position events that
// are unique to this implementation.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmeEvent {
    pub source: String,
    #[serde(rename = "type")]
    pub event_type: String,
    /// Monotonic OME event id (number for lifecycle frames, `terminal:<id>`
    /// string for FAK/FOK `EXECUTION` frames).
    #[serde(rename = "eventId")]
    pub event_id: Value,
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: u64,
    #[serde(rename = "marketId")]
    pub market_id: String,
    pub token: String,
    pub side: String,
    /// Limit price — a JSON number on OME lifecycle frames.
    pub price: FlexFloat,
    /// Size remaining on the book — a JSON number on OME lifecycle frames.
    #[serde(rename = "remainingSize")]
    pub remaining_size: FlexFloat,
    /// Terminal outcome on `EXECUTION` frames (`FILLED`,
    /// `PARTIALLY_FILLED`, or `KILLED`).
    #[serde(default)]
    pub status: Option<String>,
    /// Engine cancellation reason (e.g. `STP_MAKER_CANCELLED`).
    #[serde(default)]
    pub reason: Option<String>,
    /// Deprecated legacy timestamp — kept for backward compatibility.
    pub timestamp: String,
    /// Lifecycle fact time (source transition time on OME frames).
    #[serde(rename = "occurredAt", default)]
    pub occurred_at: Option<String>,
    /// Persisted match time from the trade record (settlement frames).
    #[serde(rename = "matchedAt", default)]
    pub matched_at: Option<String>,
    /// Per-client gateway queue time.
    #[serde(rename = "publishedAt", default)]
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementEvent {
    pub source: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// Trade id shared by the provisional `MATCHED` and the terminal
    /// `MINED` / `FAILED` for this fill.
    #[serde(rename = "tradeEventId", default)]
    pub trade_event_id: Option<String>,
    #[serde(rename = "orderId", default)]
    pub order_id: Option<String>,
    #[serde(rename = "clientOrderId", default)]
    pub client_order_id: Option<String>,
    /// Internal user id of the recipient (absent on provisional `MATCHED` frames).
    #[serde(rename = "userId", default)]
    pub user_id: Option<u64>,
    #[serde(rename = "takerOrderId", default)]
    pub taker_order_id: Option<String>,
    #[serde(rename = "takerAccount", default)]
    pub taker_account: Option<String>,
    #[serde(rename = "makerMatches", default)]
    pub maker_matches: Option<Vec<MakerMatch>>,
    #[serde(rename = "marketSlug", default)]
    pub market_slug: Option<String>,
    /// CTF token id of the recipient's own side.
    #[serde(rename = "tokenId", default)]
    pub token_id: Option<String>,
    /// Recipient order side (`BUY` / `SELL`).
    #[serde(default)]
    pub side: Option<String>,
    /// Fill price as a decimal string.
    #[serde(default)]
    pub price: Option<String>,
    #[serde(rename = "amountContracts", default)]
    pub amount_contracts: Option<String>,
    #[serde(rename = "amountCollateral", default)]
    pub amount_collateral: Option<String>,
    #[serde(rename = "configuredFeeRateBps", default)]
    pub configured_fee_rate_bps: Option<i64>,
    #[serde(rename = "effectiveFeeBps", default)]
    pub effective_fee_bps: Option<i64>,
    /// Fee estimate in contracts (BUY fills).
    #[serde(rename = "feeAmountContracts", default)]
    pub fee_amount_contracts: Option<String>,
    /// Fee estimate in collateral (SELL fills).
    #[serde(rename = "feeAmountCollateral", default)]
    pub fee_amount_collateral: Option<String>,
    /// `true` on provisional `MATCHED` frames — fee fields are estimates.
    #[serde(rename = "isEstimate", default)]
    pub is_estimate: Option<bool>,
    #[serde(rename = "txHash", default)]
    pub tx_hash: Option<String>,
    /// Deprecated legacy timestamp — always the match time on settlement frames.
    pub timestamp: String,
    /// Lifecycle fact time (match time on `MATCHED`, terminal decision time
    /// on `MINED` / `FAILED`).
    #[serde(rename = "occurredAt", default)]
    pub occurred_at: Option<String>,
    /// Persisted match time from the trade record.
    #[serde(rename = "matchedAt", default)]
    pub matched_at: Option<String>,
    /// Per-client gateway queue time.
    #[serde(rename = "publishedAt", default)]
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakerMatch {
    pub account: String,
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "matchedSize")]
    pub matched_size: String,
    pub price: String,
}

// ═══════════════════════════════════════════════════════════════════════════
//  AMM trading (server wallets)
// ═══════════════════════════════════════════════════════════════════════════

/// Request body for `POST /amm/buy`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmBuyRequest {
    /// Market slug or checksummed FPMM address.
    pub market: String,
    /// `0` = YES, `1` = NO.
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: u8,
    /// Collateral base units to spend (positive integer string).
    #[serde(rename = "collateralAmount")]
    pub collateral_amount: String,
    /// Slippage tolerance in basis points. Default 100, max 1000.
    #[serde(skip_serializing_if = "Option::is_none", rename = "slippageBps")]
    pub slippage_bps: Option<u32>,
    /// Partner-provided idempotency key (max 128 chars).
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// Server-wallet sub-account profile ID (partner flow).
    #[serde(skip_serializing_if = "Option::is_none", rename = "onBehalfOf")]
    pub on_behalf_of: Option<u64>,
}

/// Request body for `POST /amm/sell`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmSellRequest {
    /// Market slug or checksummed FPMM address.
    pub market: String,
    /// `0` = YES, `1` = NO.
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: u8,
    /// Collateral base units to receive (positive integer string).
    #[serde(rename = "collateralReturnAmount")]
    pub collateral_return_amount: String,
    /// Slippage tolerance in basis points. Default 100, max 1000.
    #[serde(skip_serializing_if = "Option::is_none", rename = "slippageBps")]
    pub slippage_bps: Option<u32>,
    /// Partner-provided idempotency key (max 128 chars).
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// Server-wallet sub-account profile ID (partner flow).
    #[serde(skip_serializing_if = "Option::is_none", rename = "onBehalfOf")]
    pub on_behalf_of: Option<u64>,
}

/// Submission result for `POST /amm/buy` / `POST /amm/sell`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmTradeResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub market: Option<String>,
    #[serde(rename = "outcomeIndex", default)]
    pub outcome_index: Option<u8>,
    #[serde(rename = "collateralAmount", default)]
    pub collateral_amount: Option<String>,
    #[serde(rename = "collateralReturnAmount", default)]
    pub collateral_return_amount: Option<String>,
    #[serde(rename = "expectedShares", default)]
    pub expected_shares: Option<String>,
    #[serde(rename = "minShares", default)]
    pub min_shares: Option<String>,
    #[serde(rename = "maxShares", default)]
    pub max_shares: Option<String>,
    #[serde(rename = "transactionId", default)]
    pub transaction_id: Option<String>,
    #[serde(rename = "userOperationHash", default)]
    pub user_operation_hash: Option<String>,
    #[serde(rename = "txHash", default)]
    pub tx_hash: Option<String>,
}

/// Request body for `POST /amm/allowances/check` and
/// `POST /amm/allowances/approve`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmAllowanceRequest {
    /// Market slug or checksummed FPMM address.
    pub market: String,
    /// `BUY` (ERC20 collateral approval) or `SELL` (ERC1155 operator).
    pub side: String,
    /// Server-wallet sub-account profile ID (partner flow).
    #[serde(skip_serializing_if = "Option::is_none", rename = "onBehalfOf")]
    pub on_behalf_of: Option<u64>,
}

/// On-chain approval state for an AMM market and side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmAllowanceResponse {
    /// `confirmed` when at or above the ready threshold, otherwise `missing`
    /// (or `submitted` immediately after an approval submission).
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub confirmed: Option<bool>,
    #[serde(default)]
    pub market: Option<String>,
    #[serde(rename = "marketAddress", default)]
    pub market_address: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(rename = "walletAddress", default)]
    pub wallet_address: Option<String>,
    #[serde(rename = "tokenAddress", default)]
    pub token_address: Option<String>,
    #[serde(rename = "spenderOrOperator", default)]
    pub spender_or_operator: Option<String>,
    #[serde(rename = "currentAllowance", default)]
    pub current_allowance: Option<String>,
    #[serde(rename = "isApprovedForAll", default)]
    pub is_approved_for_all: Option<bool>,
    #[serde(rename = "transactionId", default)]
    pub transaction_id: Option<String>,
    #[serde(rename = "userOperationHash", default)]
    pub user_operation_hash: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Market timeline
// ═══════════════════════════════════════════════════════════════════════════

/// A single slot in a recurring market series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSlot {
    #[serde(rename = "slotId", default)]
    pub slot_id: Option<i64>,
    /// Slot offset from the anchor (`batch` entries only).
    #[serde(default)]
    pub index: Option<i64>,
    #[serde(default)]
    pub slug: Option<String>,
    /// `PRE_OPEN`, `OPEN`, `SETTLING`, `SETTLED`, or `FAILED`.
    #[serde(default)]
    pub state: Option<String>,
    /// Whether orders are currently accepted for this slot.
    #[serde(default)]
    pub tradable: Option<bool>,
    #[serde(rename = "startAt", default)]
    pub start_at: Option<String>,
    #[serde(rename = "endAt", default)]
    pub end_at: Option<String>,
    #[serde(rename = "countdownSec", default)]
    pub countdown_sec: Option<i64>,
}

/// Response from `GET /markets/{slug}/timeline` and `GET /markets/timeline`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTimelineResponse {
    /// The current open slot (or nearest slot when none is open).
    #[serde(default)]
    pub current: Option<TimelineSlot>,
    /// The next slot after the current one.
    #[serde(default)]
    pub next: Option<TimelineSlot>,
    /// The full batch of slots around the anchor.
    #[serde(default)]
    pub batch: Vec<TimelineSlot>,
    /// Recurring-series anchor metadata (slug-anchored variant only).
    #[serde(default)]
    pub anchor: Option<Value>,
    /// Recurring-series job metadata (slug-anchored variant only).
    #[serde(default)]
    pub job: Option<Value>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Maintenance status
// ═══════════════════════════════════════════════════════════════════════════

/// Response from `GET /maintenance/status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceStatus {
    /// Maintenance effects currently in force.
    #[serde(default)]
    pub active: Vec<MaintenanceWindow>,
    /// Future maintenance notices.
    #[serde(default)]
    pub scheduled: Vec<MaintenanceWindow>,
}

/// A maintenance window or notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    #[serde(rename = "startsAt", default)]
    pub starts_at: Option<String>,
    #[serde(rename = "endsAt", default)]
    pub ends_at: Option<String>,
    #[serde(rename = "publicMessage", default)]
    pub public_message: Option<String>,
    #[serde(default)]
    pub effects: Vec<MaintenanceEffect>,
}

/// A public effect clients should apply during maintenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceEffect {
    /// The public API surface affected (`trading`).
    #[serde(default)]
    pub target: Option<String>,
    /// `post_only`, `cancel_only`, or `disabled`.
    #[serde(default)]
    pub mode: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Partner accounts
// ═══════════════════════════════════════════════════════════════════════════

/// Response from `GET /profiles/partner-accounts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPartnerAccountsResponse {
    #[serde(default)]
    pub data: Vec<PartnerAccountListItem>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(rename = "hasMore", default)]
    pub has_more: Option<bool>,
}

/// A partner-owned sub-account entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnerAccountListItem {
    #[serde(rename = "profileId", default)]
    pub profile_id: Option<i64>,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Referral
// ═══════════════════════════════════════════════════════════════════════════

/// Response from `GET /referral/usdc/me`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralMeResponse {
    /// Pinned minimum tier name (acts as a floor when present).
    #[serde(rename = "customTier", default)]
    pub custom_tier: Option<String>,
    /// The active tier ladder, ascending.
    #[serde(default)]
    pub tiers: Vec<ReferralTierEntry>,
    /// Own CLOB trading volume (raw USDC, 6 decimals, string).
    #[serde(rename = "totalBasisRaw", default)]
    pub total_basis_raw: Option<String>,
    /// Total USDC earned from the referral program (raw, string).
    #[serde(rename = "totalEarnedRaw", default)]
    pub total_earned_raw: Option<String>,
}

/// One rung of the referral tier ladder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralTierEntry {
    /// Trading volume required to reach this tier (raw USDC, string).
    #[serde(rename = "minBasisRaw", default)]
    pub min_basis_raw: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    /// Referral rate as a decimal fraction (0.18 = 18%).
    #[serde(default)]
    pub rate: Option<String>,
}

/// Response from `GET /referral/usdc/referrals`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralReferralsResponse {
    #[serde(default)]
    pub counts: Option<ReferralCounts>,
    #[serde(default)]
    pub entries: Vec<ReferralReferralEntry>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub total: Option<i64>,
}

/// Referral counts across all referred users regardless of the active filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralCounts {
    #[serde(default)]
    pub all: Option<i64>,
    #[serde(default)]
    pub awaiting: Option<i64>,
    #[serde(default)]
    pub earning: Option<i64>,
}

/// A referred user entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralReferralEntry {
    #[serde(default)]
    pub account: Option<String>,
    #[serde(rename = "avatarAccount", default)]
    pub avatar_account: Option<String>,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "earnedRaw", default)]
    pub earned_raw: Option<String>,
    #[serde(rename = "feesGeneratedRaw", default)]
    pub fees_generated_raw: Option<String>,
    #[serde(rename = "pfpUrl", default)]
    pub pfp_url: Option<String>,
    #[serde(rename = "referredProfileId", default)]
    pub referred_profile_id: Option<i64>,
    #[serde(rename = "volumeRaw", default)]
    pub volume_raw: Option<String>,
}

/// Response from `GET /referral/usdc/leaderboard` and
/// `GET /referral/usdc/leaderboard-friends`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralLeaderboardResponse {
    #[serde(default)]
    pub entries: Vec<ReferralLeaderboardEntry>,
    #[serde(default)]
    pub limit: Option<i64>,
    /// Your own rank and entry (global board only, authenticated requests).
    #[serde(default)]
    pub me: Option<ReferralLeaderboardMe>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub total: Option<i64>,
}

/// A referral leaderboard row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralLeaderboardEntry {
    #[serde(default)]
    pub account: Option<String>,
    #[serde(rename = "avatarAccount", default)]
    pub avatar_account: Option<String>,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "earnedRaw", default)]
    pub earned_raw: Option<String>,
    #[serde(rename = "feesGeneratedRaw", default)]
    pub fees_generated_raw: Option<String>,
    #[serde(rename = "pfpUrl", default)]
    pub pfp_url: Option<String>,
    #[serde(default)]
    pub rank: Option<i64>,
    #[serde(rename = "referredCount", default)]
    pub referred_count: Option<i64>,
    #[serde(rename = "referrerProfileId", default)]
    pub referrer_profile_id: Option<i64>,
}

/// The authenticated caller's pinned rank and entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralLeaderboardMe {
    #[serde(default)]
    pub entry: Option<ReferralLeaderboardEntry>,
    #[serde(default)]
    pub rank: Option<i64>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Unrealized PnL leaderboards
// ═══════════════════════════════════════════════════════════════════════════

/// Readiness of the underlying leaderboard projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum UnrealizedPnlSnapshotState {
    Building,
    Degraded,
    Ready,
    Stale,
}

/// Numeric amount rendered as a raw integer string plus a formatted decimal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnlAmountValue {
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub formatted: Option<String>,
}

/// Money amount in the market's collateral with an optional USD value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnlMoneyValue {
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub formatted: Option<String>,
    #[serde(default)]
    pub usd: Option<String>,
}

/// Mark price used to value an open position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnlMarkValue {
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub formatted: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(rename = "asOf", default)]
    pub as_of: Option<String>,
}

/// Collateral token descriptor on leaderboard responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnlCollateralToken {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub decimals: Option<i32>,
    #[serde(rename = "priceOracleId", default)]
    pub price_oracle_id: Option<String>,
}

/// Market identity pinned to a biggest-positions row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnlMarketIdentity {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

/// Response from `GET /leaderboard/pnl/unrealized/markets/{marketId}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnlMarketResponse {
    #[serde(rename = "schemaVersion", default)]
    pub schema_version: Option<i32>,
    pub state: Option<UnrealizedPnlSnapshotState>,
    #[serde(rename = "projectionVersion", default)]
    pub projection_version: Option<String>,
    #[serde(rename = "scopeVersion", default)]
    pub scope_version: Option<String>,
    #[serde(rename = "presentationVersion", default)]
    pub presentation_version: Option<String>,
    #[serde(rename = "asOf", default)]
    pub as_of: Option<String>,
    #[serde(rename = "markAsOf", default)]
    pub mark_as_of: Option<String>,
    #[serde(rename = "staleReason", default)]
    pub stale_reason: Option<String>,
    #[serde(rename = "collateralToken", default)]
    pub collateral_token: Option<UnrealizedPnlCollateralToken>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(rename = "marketId", default)]
    pub market_id: Option<i64>,
    #[serde(default)]
    pub metric: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(rename = "totalRows", default)]
    pub total_rows: Option<i64>,
    #[serde(rename = "totalPages", default)]
    pub total_pages: Option<i64>,
    #[serde(default)]
    pub data: Vec<UnrealizedPnlMarketEntry>,
}

/// A single ranked row on the market-scoped Unrealized PnL leaderboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnlMarketEntry {
    #[serde(default)]
    pub rank: Option<i64>,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "pfpUrl", default)]
    pub pfp_url: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(rename = "outcomeLabel", default)]
    pub outcome_label: Option<String>,
    #[serde(rename = "heldShares", default)]
    pub held_shares: Option<UnrealizedPnlAmountValue>,
    #[serde(rename = "avgEntry", default)]
    pub avg_entry: Option<UnrealizedPnlAmountValue>,
    #[serde(rename = "costBasis", default)]
    pub cost_basis: Option<UnrealizedPnlMoneyValue>,
    #[serde(default)]
    pub mark: Option<UnrealizedPnlMarkValue>,
    #[serde(rename = "marketValue", default)]
    pub market_value: Option<UnrealizedPnlMoneyValue>,
    #[serde(rename = "unrealizedPnl", default)]
    pub unrealized_pnl: Option<UnrealizedPnlMoneyValue>,
    #[serde(rename = "unrealizedRoi", default)]
    pub unrealized_roi: Option<f64>,
}

/// Response from `GET /leaderboard/pnl/unrealized/biggest-positions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnlBiggestPositionsResponse {
    #[serde(rename = "schemaVersion", default)]
    pub schema_version: Option<i32>,
    pub state: Option<UnrealizedPnlSnapshotState>,
    #[serde(rename = "projectionVersion", default)]
    pub projection_version: Option<String>,
    #[serde(rename = "scopeVersion", default)]
    pub scope_version: Option<String>,
    #[serde(rename = "presentationVersion", default)]
    pub presentation_version: Option<String>,
    #[serde(rename = "asOf", default)]
    pub as_of: Option<String>,
    #[serde(rename = "markAsOf", default)]
    pub mark_as_of: Option<String>,
    #[serde(rename = "staleReason", default)]
    pub stale_reason: Option<String>,
    #[serde(rename = "collateralToken", default)]
    pub collateral_token: Option<UnrealizedPnlCollateralToken>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub data: Vec<UnrealizedPnlBiggestPositionEntry>,
}

/// A single row on the biggest-open-positions leaderboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrealizedPnlBiggestPositionEntry {
    #[serde(default)]
    pub rank: Option<i64>,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "pfpUrl", default)]
    pub pfp_url: Option<String>,
    #[serde(rename = "positionSize", default)]
    pub position_size: Option<UnrealizedPnlMoneyValue>,
    #[serde(rename = "heldShares", default)]
    pub held_shares: Option<UnrealizedPnlAmountValue>,
    #[serde(rename = "toWin", default)]
    pub to_win: Option<UnrealizedPnlMoneyValue>,
    #[serde(default)]
    pub mark: Option<UnrealizedPnlMarkValue>,
    #[serde(rename = "unrealizedPnl", default)]
    pub unrealized_pnl: Option<UnrealizedPnlMoneyValue>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(rename = "outcomeLabel", default)]
    pub outcome_label: Option<String>,
    #[serde(default)]
    pub market: Option<UnrealizedPnlMarketIdentity>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Server-wallet portfolio operations
// ═══════════════════════════════════════════════════════════════════════════

/// Request body for `POST /portfolio/redeem`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemRequest {
    /// CTF condition id (`bytes32` hex string).
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    /// Managed sub-account profile id (partner flow).
    #[serde(skip_serializing_if = "Option::is_none", rename = "onBehalfOf")]
    pub on_behalf_of: Option<u64>,
}

/// Request body for `POST /portfolio/withdraw`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawRequest {
    /// Token amount in smallest unit (USDC: 1_000_000 = 1 USDC).
    pub amount: String,
    /// ERC20 token address (defaults to USDC when omitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Managed sub-account profile id (partner flow).
    #[serde(skip_serializing_if = "Option::is_none", rename = "onBehalfOf")]
    pub on_behalf_of: Option<u64>,
    /// Explicit destination address (must be allowlisted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
}

/// Response from `POST /portfolio/withdrawal-addresses`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalAddressResponse {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "profileId", default)]
    pub profile_id: Option<i64>,
    #[serde(rename = "destinationAddress", default)]
    pub destination_address: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
    #[serde(rename = "deletedAt", default)]
    pub deleted_at: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Profile update / trading wallet mode
// ═══════════════════════════════════════════════════════════════════════════

/// The profile's trading wallet mode.
///
/// Self-signed API orders require `Eoa` mode. `SmartWallet` means the profile
/// trades through a 1-click (Privy embedded / smart) wallet whose key cannot
/// be exported, so EOA-signed orders are rejected in that mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TradingWalletMode {
    /// Raw EOA wallet — required for self-signed API orders.
    Eoa,
    /// 1-click smart wallet (managed embedded key).
    SmartWallet,
}

impl TradingWalletMode {
    /// The wire value (`"eoa"` or `"smartWallet"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            TradingWalletMode::Eoa => "eoa",
            TradingWalletMode::SmartWallet => "smartWallet",
        }
    }
}

/// Request body for `PUT /profiles`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    /// Trading wallet mode (`eoa` or `smartWallet`).
    #[serde(skip_serializing_if = "Option::is_none", rename = "tradeWalletOption")]
    pub trade_wallet_option: Option<TradingWalletMode>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  API tokens
// ═══════════════════════════════════════════════════════════════════════════

/// Request body for `POST /auth/api-tokens/derive`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeriveApiTokenRequest {
    /// Human-readable label for the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Requested scopes: `trading`, `account_creation`, `delegated_signing`,
    /// `withdrawal`. Must be a subset of the partner's allowed scopes.
    pub scopes: Vec<String>,
}

/// Response from `POST /auth/api-tokens/derive`.
///
/// The secret is returned **once** at creation — store it securely.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeriveApiTokenResponse {
    /// Token ID — send as the `lmts-api-key` header.
    #[serde(rename = "tokenId")]
    pub token_id: String,
    /// Base64-encoded HMAC secret — send to the SDK as the secret.
    pub secret: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
    #[serde(rename = "expiresAt", default)]
    pub expires_at: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_info_parses_full_payload() {
        let json = r#"{
            "matched": true,
            "settlementStatus": "MINED",
            "reason": null,
            "eligibleAt": null,
            "tradeEventId": "4aa706dd-6c57-4f3c-945a-99818dfd95f1",
            "txHash": "0xabc123",
            "clientOrderId": "client-order-001",
            "stpMakerCancels": [],
            "feeRateBps": 25,
            "effectiveFeeBps": 26,
            "totalsRaw": {
                "contractsGross": "1000000",
                "contractsFee": "1000",
                "contractsNet": "999000",
                "usdGross": "500000",
                "usdFee": "500",
                "usdNet": "499500"
            }
        }"#;
        let parsed: OrderExecutionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.settlement_status.as_deref(), Some("MINED"));
        assert_eq!(
            parsed.totals_raw.unwrap().contracts_net.as_deref(),
            Some("999000")
        );
    }

    #[test]
    fn create_order_response_parses_execution() {
        let json = r#"{
            "order": {
                "id": "9e31c452-8a2b-42d1-b327-65f18d07dc96",
                "salt": "1778155025318314496",
                "maker": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
                "signer": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
                "taker": "0x0000000000000000000000000000000000000000",
                "tokenId": "19633204485790857949828516737993423758628930235371629943999544859324645414627",
                "makerAmount": "5000000",
                "takerAmount": "10000000",
                "signatureType": 0,
                "feeRateBps": 0,
                "signature": "0x1234",
                "orderType": "GTC",
                "price": "0.5",
                "side": 0,
                "marketId": 7348,
                "nonce": "0"
            },
            "makerMatches": [],
            "execution": {"matched": false, "settlementStatus": "UNMATCHED",
                          "feeRateBps": 0, "effectiveFeeBps": 0,
                          "totalsRaw": {"contractsGross": "0", "contractsFee": "0",
                                        "contractsNet": "0", "usdGross": "0",
                                        "usdFee": "0", "usdNet": "0"}}
        }"#;
        let parsed: CreateOrderResponse = serde_json::from_str(json).unwrap();
        assert!(parsed.execution.is_some());
        assert_eq!(parsed.order.id, "9e31c452-8a2b-42d1-b327-65f18d07dc96");
        assert_eq!(parsed.order.price.unwrap().float64(), 0.5);
    }

    #[test]
    fn cancel_replace_batch_response_parses() {
        let json = r#"{
            "results": [
                {
                    "index": 0,
                    "cancel": {"status": "SUCCESS", "orderId": "uuid-1"},
                    "replacement": {"status": "NOT_ATTEMPTED"}
                },
                {
                    "index": 1,
                    "cancel": {"status": "FAILURE",
                               "error": {"code": "ORDER_NOT_FOUND",
                                         "message": "Order not found"}},
                    "replacement": {"status": "FAILURE",
                                    "error": {"code": "400",
                                              "message": "Insufficient balance"}}
                }
            ]
        }"#;
        let parsed: CancelReplaceBatchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.results.len(), 2);
        assert_eq!(parsed.results[1].cancel.status, "FAILURE");
        assert_eq!(
            parsed.results[1]
                .cancel
                .error
                .as_ref()
                .unwrap()
                .code
                .as_deref(),
            Some("ORDER_NOT_FOUND")
        );
    }

    #[test]
    fn ome_event_accepts_string_and_numeric_event_ids() {
        let numeric = r#"{
            "source": "OME", "type": "PLACEMENT", "eventId": 1234567,
            "orderId": "550e8400-e29b-41d4-a716-446655440000",
            "userId": 42, "marketId": "17", "token": "878930",
            "side": "BUY", "price": 0.53, "remainingSize": 100,
            "timestamp": "2026-04-20T10:15:30.000Z",
            "occurredAt": "2026-04-20T10:15:30.000Z",
            "publishedAt": "2026-04-20T10:15:30.042Z"
        }"#;
        let parsed: OmeEvent = serde_json::from_str(numeric).unwrap();
        assert_eq!(parsed.event_id.as_u64(), Some(1234567));

        let terminal = r#"{
            "source": "OME", "type": "EXECUTION", "status": "FILLED",
            "eventId": "terminal:550e8400-e29b-41d4-a716-446655440000",
            "orderId": "550e8400-e29b-41d4-a716-446655440000",
            "userId": 42, "marketId": "17", "token": "878930",
            "side": "BUY", "price": "0.53", "remainingSize": "0",
            "timestamp": "2026-04-20T10:15:40.000Z"
        }"#;
        let parsed: OmeEvent = serde_json::from_str(terminal).unwrap();
        assert_eq!(parsed.status.as_deref(), Some("FILLED"));
        assert!(parsed.event_id.as_str().is_some());
    }

    #[test]
    fn maintenance_status_parses() {
        let json = r#"{
            "active": [{
                "startsAt": "2026-06-22T15:00:00.000Z",
                "endsAt": "2026-06-22T18:00:00.000Z",
                "publicMessage": "Trading is temporarily limited to cancellations.",
                "effects": [{"target": "trading", "mode": "cancel_only"}]
            }],
            "scheduled": []
        }"#;
        let parsed: MaintenanceStatus = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.active.len(), 1);
        assert_eq!(
            parsed.active[0].effects[0].mode.as_deref(),
            Some("cancel_only")
        );
    }

    #[test]
    fn timeline_response_parses() {
        let json = r#"{
            "current": {"slotId": 4821, "slug": "btc-up-or-down-5-min-1771934700",
                        "state": "OPEN", "tradable": true,
                        "startAt": "2026-05-28T12:05:00.000Z",
                        "endAt": "2026-05-28T12:10:00.000Z", "countdownSec": 142},
            "next": {"slotId": 4822, "slug": "btc-up-or-down-5-min-1771935000",
                     "state": "PRE_OPEN", "tradable": false},
            "batch": []
        }"#;
        let parsed: MarketTimelineResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.current.unwrap().state.as_deref(), Some("OPEN"));
        assert_eq!(parsed.next.as_ref().unwrap().tradable, Some(false));
    }

    #[test]
    fn referral_me_parses() {
        let json = r#"{
            "customTier": "Gold",
            "tiers": [{"minBasisRaw": "25000000000", "name": "Bronze",
                        "rate": "0.10"}],
            "totalBasisRaw": "31500000000",
            "totalEarnedRaw": "184250000"
        }"#;
        let parsed: ReferralMeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.custom_tier.as_deref(), Some("Gold"));
        assert_eq!(parsed.tiers[0].name.as_deref(), Some("Bronze"));
    }

    #[test]
    fn history_entry_parses_clob_identifiers() {
        let json = r#"{
            "blockTimestamp": 1744115608,
            "collateralAmount": "25.5",
            "market": null,
            "outcomeTokenAmount": "50",
            "outcomeTokenAmounts": ["50", "0"],
            "outcomeIndex": 0,
            "outcomeTokenPrice": 0.51,
            "strategy": "Limit Buy",
            "operation": "buy",
            "tradeEventId": "4aa706dd-6c57-4f3c-945a-99818dfd95f1",
            "orderId": "550e8400-e29b-41d4-a716-446655440000",
            "makerMatchId": "cb12",
            "transactionHash": "0xabc"
        }"#;
        let parsed: HistoryEntry = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.operation.as_deref(), Some("buy"));
        assert!(parsed.trade_event_id.is_some());
        assert!(parsed.order_id.is_some());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PositionUpdate {
    #[serde(rename = "AMM")]
    Amm(AmmPositionData),
    #[serde(rename = "CLOB")]
    Clob(ClobPositionData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmPositionData {
    pub account: String,
    #[serde(rename = "marketAddress")]
    pub market_address: String,
    pub positions: Vec<AmmPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmPosition {
    #[serde(rename = "tokenId")]
    pub token_id: String,
    pub balance: String,
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: u8,
    #[serde(rename = "collateralOutOnSell")]
    pub collateral_out_on_sell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobPositionData {
    pub account: String,
    #[serde(rename = "marketSlug")]
    pub market_slug: String,
    pub positions: Vec<ClobPosition>,
    #[serde(rename = "tokenIds")]
    pub token_ids: Vec<String>,
    /// Epoch milliseconds of the on-chain balance change that triggered this
    /// update (absent on the initial snapshot after `subscribe_positions`).
    #[serde(default)]
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobPosition {
    #[serde(rename = "tokenId")]
    pub token_id: String,
    #[serde(rename = "ctfBalance")]
    pub ctf_balance: String,
    #[serde(rename = "averageFillPrice")]
    pub average_fill_price: String,
    #[serde(rename = "costBasis")]
    pub cost_basis: String,
    #[serde(rename = "marketValue")]
    pub market_value: String,
    #[serde(rename = "marketId")]
    pub market_id: u64,
}
