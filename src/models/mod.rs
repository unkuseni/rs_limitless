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
    pub nonce: i32,
    pub signature: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    #[serde(default)]
    pub price: Option<f64>,
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
    pub nonce: i32,
    pub signature: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    #[serde(default)]
    pub price: Option<f64>,
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
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
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
    #[serde(rename = "eventId")]
    pub event_id: u64,
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
    pub price: String,
    #[serde(rename = "remainingSize")]
    pub remaining_size: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementEvent {
    pub source: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "eventId")]
    pub event_id: String,
    #[serde(rename = "orderId")]
    pub order_id: Option<String>,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: u64,
    #[serde(rename = "takerOrderId")]
    pub taker_order_id: Option<String>,
    #[serde(rename = "takerAccount")]
    pub taker_account: Option<String>,
    #[serde(rename = "makerMatches")]
    pub maker_matches: Option<Vec<MakerMatch>>,
    #[serde(rename = "marketSlug")]
    pub market_slug: Option<String>,
    #[serde(rename = "txHash")]
    pub tx_hash: Option<String>,
    pub timestamp: String,
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
