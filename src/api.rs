//! API endpoint enums for the Limitless Exchange REST and WebSocket APIs.
//!
//! Every URL path in this crate is defined here, organized into logical
//! groups matching the API reference: Authentication, Markets, Trading,
//! Portfolio, Navigation, API Tokens, Partner Accounts, and Public
//! Portfolio.
//!
//! Manager modules build their request paths from these enums:
//!
//! - static paths via [`AsRef<str>`] (e.g. `Market::Search.as_ref()`), and
//! - dynamic paths via the associated builder functions
//!   (e.g. `Trade::orderbook("btc-100k")`).

use crate::Config;

/// Represents a REST API endpoint category and its specific operation.
///
/// Each variant maps to a URL path relative to the base API URL.
/// The [`AsRef<str>`](AsRef) implementation returns the full path string.
#[derive(Debug, Clone)]
pub enum API {
    // ── Authentication ──
    Auth(Auth),
    // ── Markets ──
    Market(Market),
    // ── Trading ──
    Trade(Trade),
    // ── Portfolio ──
    PortfolioEndpoint(PortfolioEndpoint),
    // ── Market Navigation ──
    Nav(Nav),
    // ── API Tokens ──
    ApiToken(ApiToken),
    // ── Partner Accounts ──
    Partner(Partner),
    // ── Public Portfolio ──
    PublicPortfolio(PublicPortfolio),
    // ── AMM Trading ──
    Amm(AmmEndpoint),
    // ── System ──
    System(SystemEndpoint),
    // ── Referral ──
    Referral(ReferralEndpoint),
    // ── Leaderboard ──
    Leaderboard(LeaderboardEndpoint),
}

// ── Authentication ──

#[derive(Debug, Clone)]
pub enum Auth {
    /// `POST /auth/api-keys` — Create a new API key (UI-authenticated only).
    CreateApiKey,
    /// `GET /auth/api-keys` — Get the active API key metadata.
    GetApiKey,
    /// `DELETE /auth/api-keys` — Revoke the active API key.
    RevokeApiKey,
}

impl AsRef<str> for Auth {
    fn as_ref(&self) -> &str {
        match self {
            Auth::CreateApiKey => "auth/api-keys",
            Auth::GetApiKey => "auth/api-keys",
            Auth::RevokeApiKey => "auth/api-keys",
        }
    }
}

// ── Markets ──

#[derive(Debug, Clone)]
pub enum Market {
    /// `GET /markets/active` — Browse active markets.
    Active,
    /// `GET /markets/active/{categoryId}` — Browse active markets by category.
    ActiveCategory,
    /// `GET /markets/categories/count` — Category counts.
    CategoryCount,
    /// `GET /markets/active/slugs` — Active market slugs.
    ActiveSlugs,
    /// `GET /markets/{addressOrSlug}` — Get market details.
    GetMarket,
    /// `GET /markets/{addressOrSlug}/oracle-candles` — Oracle candlestick data.
    OracleCandles,
    /// `GET /markets/{slug}/get-feed-events` — Feed events for a market.
    FeedEvents,
    /// `GET /markets/search` — Semantic search for markets.
    Search,
    /// `GET /markets/timeline` — Global upcoming recurring-market schedule.
    GlobalTimeline,
    /// `GET /markets/{slug}/timeline` — Schedule for a recurring market.
    MarketTimeline,
}

impl AsRef<str> for Market {
    fn as_ref(&self) -> &str {
        match self {
            Market::Active => "markets/active",
            Market::ActiveCategory => "markets/active",
            Market::CategoryCount => "markets/categories/count",
            Market::ActiveSlugs => "markets/active/slugs",
            Market::GetMarket => "markets",
            Market::OracleCandles => "markets",
            Market::FeedEvents => "markets",
            Market::Search => "markets/search",
            Market::GlobalTimeline => "markets/timeline",
            Market::MarketTimeline => "markets",
        }
    }
}

impl Market {
    /// `GET /markets/active/{categoryId}` — active markets in a category.
    pub fn active_category(category_id: u64) -> String {
        format!("markets/active/{category_id}")
    }

    /// `GET /markets/{addressOrSlug}` — market details by address or slug.
    pub fn get(address_or_slug: &str) -> String {
        format!("markets/{address_or_slug}")
    }

    /// `GET /markets/{addressOrSlug}/oracle-candles` — oracle candles.
    pub fn oracle_candles(address_or_slug: &str) -> String {
        format!("markets/{address_or_slug}/oracle-candles")
    }

    /// `GET /markets/{slug}/get-feed-events` — feed events for a market.
    pub fn feed_events(slug: &str) -> String {
        format!("markets/{slug}/get-feed-events")
    }

    /// `GET /markets/{slug}/timeline` — recurring-market schedule by slug.
    pub fn market_timeline(slug: &str) -> String {
        format!("markets/{slug}/timeline")
    }
}

// ── Trading ──

#[derive(Debug, Clone)]
pub enum Trade {
    /// `POST /orders` — Create a new order.
    CreateOrder,
    /// `POST /orders/status/batch` — Batch order status lookup.
    OrderStatusBatch,
    /// `POST /orders/cancel` — Cancel order (combined: by orderId or clientOrderId).
    CancelCombined,
    /// `POST /orders/batch-cancel` — Batch cancel (by orderIds or clientOrderIds).
    CancelBatchCombined,
    /// `POST /orders/cancel-batch` — Batch cancel (by orderIds, legacy).
    CancelBatch,
    /// `POST /orders/cancel-replace` — Cancel an order and place a replacement.
    CancelReplace,
    /// `POST /orders/cancel-replace/batch` — Batch cancel-and-replace.
    CancelReplaceBatch,
    /// `DELETE /orders/{orderId}` — Cancel a single order by ID.
    CancelOrder,
    /// `DELETE /orders/all/{slug}` — Cancel all orders in a market.
    CancelAll,
    /// `GET /markets/{slug}/orderbook` — Get orderbook.
    Orderbook,
    /// `GET /markets/{slug}/historical-price` — Historical prices.
    HistoricalPrice,
    /// `GET /markets/{slug}/locked-balance` — Locked balance.
    LockedBalance,
    /// `GET /markets/{slug}/user-orders` — User's orders in a market.
    UserOrders,
    /// `GET /markets/{slug}/events` — Market events.
    MarketEvents,
}

impl AsRef<str> for Trade {
    fn as_ref(&self) -> &str {
        match self {
            Trade::CreateOrder => "orders",
            Trade::OrderStatusBatch => "orders/status/batch",
            Trade::CancelCombined => "orders/cancel",
            Trade::CancelBatchCombined => "orders/batch-cancel",
            Trade::CancelBatch => "orders/cancel-batch",
            Trade::CancelReplace => "orders/cancel-replace",
            Trade::CancelReplaceBatch => "orders/cancel-replace/batch",
            Trade::CancelOrder => "orders",
            Trade::CancelAll => "orders/all",
            Trade::Orderbook => "markets",
            Trade::HistoricalPrice => "markets",
            Trade::LockedBalance => "markets",
            Trade::UserOrders => "markets",
            Trade::MarketEvents => "markets",
        }
    }
}

impl Trade {
    /// `DELETE /orders/{orderId}` — cancel a single order by ID.
    pub fn cancel_order(order_id: &str) -> String {
        format!("orders/{order_id}")
    }

    /// `DELETE /orders/all/{slug}` — cancel all orders in a market.
    pub fn cancel_all(slug: &str) -> String {
        format!("orders/all/{slug}")
    }

    /// `GET /markets/{slug}/orderbook` — current orderbook.
    pub fn orderbook(slug: &str) -> String {
        format!("markets/{slug}/orderbook")
    }

    /// `GET /markets/{slug}/historical-price` — historical prices.
    pub fn historical_price(slug: &str) -> String {
        format!("markets/{slug}/historical-price")
    }

    /// `GET /markets/{slug}/locked-balance` — locked balance.
    pub fn locked_balance(slug: &str) -> String {
        format!("markets/{slug}/locked-balance")
    }

    /// `GET /markets/{slug}/user-orders` — user's orders in a market.
    pub fn user_orders(slug: &str) -> String {
        format!("markets/{slug}/user-orders")
    }

    /// `GET /markets/{slug}/events` — recent market events.
    pub fn market_events(slug: &str) -> String {
        format!("markets/{slug}/events")
    }
}

// ── Portfolio ──

#[derive(Debug, Clone)]
pub enum PortfolioEndpoint {
    /// `GET /profiles/me` — Get the authenticated caller's private profile.
    GetCurrentProfile,
    /// `GET /profiles/{account}` — Get your profile.
    GetProfile,
    /// `PUT /profiles` — Update the authenticated profile.
    UpdateProfile,
    /// `GET /portfolio/trades` — Get trades.
    Trades,
    /// `GET /portfolio/positions` — Get positions.
    Positions,
    /// `GET /portfolio/pnl-chart` — PnL chart.
    PnlChart,
    /// `GET /portfolio/points` — Points breakdown.
    Points,
    /// `GET /portfolio/history` — Portfolio history (cursor-paginated).
    History,
    /// `GET /portfolio/{account}/history` — Public history for any address.
    PublicHistory,
    /// `GET /portfolio/trading/allowance` — Trading allowance check.
    Allowance,
    /// `POST /portfolio/redeem` — Redeem resolved server-wallet positions.
    Redeem,
    /// `POST /portfolio/withdraw` — Withdraw funds from a server wallet.
    Withdraw,
    /// `POST /portfolio/withdrawal-addresses` — Add a withdrawal address.
    AddWithdrawalAddress,
    /// `DELETE /portfolio/withdrawal-addresses/{address}` — Remove a withdrawal address.
    DeleteWithdrawalAddress,
}

impl AsRef<str> for PortfolioEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            PortfolioEndpoint::GetCurrentProfile => "profiles/me",
            PortfolioEndpoint::GetProfile => "profiles",
            PortfolioEndpoint::UpdateProfile => "profiles",
            PortfolioEndpoint::Trades => "portfolio/trades",
            PortfolioEndpoint::Positions => "portfolio/positions",
            PortfolioEndpoint::PnlChart => "portfolio/pnl-chart",
            PortfolioEndpoint::Points => "portfolio/points",
            PortfolioEndpoint::History => "portfolio/history",
            PortfolioEndpoint::PublicHistory => "portfolio",
            PortfolioEndpoint::Allowance => "portfolio/trading/allowance",
            PortfolioEndpoint::Redeem => "portfolio/redeem",
            PortfolioEndpoint::Withdraw => "portfolio/withdraw",
            PortfolioEndpoint::AddWithdrawalAddress => "portfolio/withdrawal-addresses",
            PortfolioEndpoint::DeleteWithdrawalAddress => "portfolio/withdrawal-addresses",
        }
    }
}

impl PortfolioEndpoint {
    /// `GET /profiles/{account}` — profile by wallet address.
    pub fn get_profile(account: &str) -> String {
        format!("profiles/{account}")
    }

    /// `GET /portfolio/{account}/history` — public history for any address.
    pub fn public_history(account: &str) -> String {
        format!("portfolio/{account}/history")
    }

    /// `DELETE /portfolio/withdrawal-addresses/{address}` — remove an
    /// allowlisted withdrawal destination.
    pub fn withdrawal_address(address: &str) -> String {
        format!("portfolio/withdrawal-addresses/{address}")
    }
}

// ── Market Navigation ──

#[derive(Debug, Clone)]
pub enum Nav {
    /// `GET /navigation` — Hierarchical navigation tree.
    GetNavigation,
    /// `GET /market-pages/by-path` — Resolve a path to a market page.
    GetPageByPath,
    /// `GET /market-pages/{id}/markets` — List markets for a page.
    ListPageMarkets,
    /// `GET /property-keys` — List all property keys.
    ListPropertyKeys,
    /// `GET /property-keys/{id}` — Get a specific property key.
    GetPropertyKey,
    /// `GET /property-keys/{id}/options` — List options for a property key.
    ListPropertyOptions,
}

impl AsRef<str> for Nav {
    fn as_ref(&self) -> &str {
        match self {
            Nav::GetNavigation => "navigation",
            Nav::GetPageByPath => "market-pages/by-path",
            Nav::ListPageMarkets => "market-pages",
            Nav::ListPropertyKeys => "property-keys",
            Nav::GetPropertyKey => "property-keys",
            Nav::ListPropertyOptions => "property-keys",
        }
    }
}

impl Nav {
    /// `GET /market-pages/{id}/markets` — markets for a page.
    pub fn page_markets(page_id: &str) -> String {
        format!("market-pages/{page_id}/markets")
    }

    /// `GET /property-keys/{id}` — a specific property key.
    pub fn property_key(key_id: &str) -> String {
        format!("property-keys/{key_id}")
    }

    /// `GET /property-keys/{id}/options` — options for a property key.
    pub fn property_options(key_id: &str) -> String {
        format!("property-keys/{key_id}/options")
    }
}

// ── API Tokens ──

#[derive(Debug, Clone)]
pub enum ApiToken {
    /// `GET /auth/api-tokens/capabilities` — Partner capabilities (Privy).
    GetCapabilities,
    /// `POST /auth/api-tokens/derive` — Derive a scoped token (Privy).
    Derive,
    /// `GET /auth/api-tokens` — List active tokens.
    ListActive,
    /// `DELETE /auth/api-tokens/{id}` — Revoke a token.
    Revoke,
}

impl AsRef<str> for ApiToken {
    fn as_ref(&self) -> &str {
        match self {
            ApiToken::GetCapabilities => "auth/api-tokens/capabilities",
            ApiToken::Derive => "auth/api-tokens/derive",
            ApiToken::ListActive => "auth/api-tokens",
            ApiToken::Revoke => "auth/api-tokens",
        }
    }
}

impl ApiToken {
    /// `DELETE /auth/api-tokens/{tokenId}` — revoke a token.
    pub fn revoke(token_id: &str) -> String {
        format!("auth/api-tokens/{token_id}")
    }
}

// ── Partner Accounts ──

#[derive(Debug, Clone)]
pub enum Partner {
    /// `POST /profiles/partner-accounts` — Create partner sub-account.
    CreateSubAccount,
    /// `GET /profiles/partner-accounts` — List partner sub-accounts.
    ListSubAccounts,
    /// `GET /profiles/partner-accounts/{id}/allowances` — Check allowances.
    CheckAllowances,
    /// `POST /profiles/partner-accounts/{id}/allowances/retry` — Retry allowances.
    RetryAllowances,
}

impl AsRef<str> for Partner {
    fn as_ref(&self) -> &str {
        match self {
            Partner::CreateSubAccount => "profiles/partner-accounts",
            Partner::ListSubAccounts => "profiles/partner-accounts",
            Partner::CheckAllowances => "profiles/partner-accounts",
            Partner::RetryAllowances => "profiles/partner-accounts",
        }
    }
}

impl Partner {
    /// `GET /profiles/partner-accounts/{id}/allowances` — check allowance
    /// readiness for a server-wallet sub-account.
    pub fn allowances(profile_id: &str) -> String {
        format!("profiles/partner-accounts/{profile_id}/allowances")
    }

    /// `POST /profiles/partner-accounts/{id}/allowances/retry` — retry
    /// allowance recovery.
    pub fn allowances_retry(profile_id: &str) -> String {
        format!("profiles/partner-accounts/{profile_id}/allowances/retry")
    }
}

// ── AMM Trading ──

#[derive(Debug, Clone)]
pub enum AmmEndpoint {
    /// `POST /amm/buy` — Buy outcome shares from a server wallet.
    Buy,
    /// `POST /amm/sell` — Sell outcome shares from a server wallet.
    Sell,
    /// `POST /amm/allowances/check` — Read on-chain approval state.
    AllowancesCheck,
    /// `POST /amm/allowances/approve` — Submit a fresh approval.
    AllowancesApprove,
}

impl AsRef<str> for AmmEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            AmmEndpoint::Buy => "amm/buy",
            AmmEndpoint::Sell => "amm/sell",
            AmmEndpoint::AllowancesCheck => "amm/allowances/check",
            AmmEndpoint::AllowancesApprove => "amm/allowances/approve",
        }
    }
}

// ── System ──

#[derive(Debug, Clone)]
pub enum SystemEndpoint {
    /// `GET /maintenance/status` — Active and scheduled maintenance.
    MaintenanceStatus,
}

impl AsRef<str> for SystemEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            SystemEndpoint::MaintenanceStatus => "maintenance/status",
        }
    }
}

// ── Referral ──

#[derive(Debug, Clone)]
pub enum ReferralEndpoint {
    /// `GET /referral/usdc/me` — Your referral standing.
    MyStats,
    /// `GET /referral/usdc/referrals` — Your referred users.
    MyReferrals,
    /// `GET /referral/usdc/leaderboard` — Global referral leaderboard.
    Leaderboard,
    /// `GET /referral/usdc/leaderboard-friends` — Friends referral leaderboard.
    FriendsLeaderboard,
}

impl AsRef<str> for ReferralEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            ReferralEndpoint::MyStats => "referral/usdc/me",
            ReferralEndpoint::MyReferrals => "referral/usdc/referrals",
            ReferralEndpoint::Leaderboard => "referral/usdc/leaderboard",
            ReferralEndpoint::FriendsLeaderboard => "referral/usdc/leaderboard-friends",
        }
    }
}

// ── Leaderboard ──

#[derive(Debug, Clone)]
pub enum LeaderboardEndpoint {
    /// `GET /leaderboard/pnl/unrealized/markets/{marketId}` — Market Unrealized PnL.
    UnrealizedPnlMarket,
    /// `GET /leaderboard/pnl/unrealized/biggest-positions` — Biggest open positions.
    BiggestPositions,
}

impl AsRef<str> for LeaderboardEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            LeaderboardEndpoint::UnrealizedPnlMarket => "leaderboard/pnl/unrealized/markets",
            LeaderboardEndpoint::BiggestPositions => "leaderboard/pnl/unrealized/biggest-positions",
        }
    }
}

impl LeaderboardEndpoint {
    /// `GET /leaderboard/pnl/unrealized/markets/{marketId}` — Unrealized PnL
    /// leaderboard for one market.
    pub fn unrealized_pnl_market(market_id: u64) -> String {
        format!("leaderboard/pnl/unrealized/markets/{market_id}")
    }
}

// ── Public Portfolio ──

#[derive(Debug, Clone)]
pub enum PublicPortfolio {
    /// `GET /public/portfolio/{address}/volume` — User traded volume.
    TradedVolume,
    /// `GET /public/portfolio/{address}/positions` — Public positions.
    Positions,
    /// `GET /public/portfolio/{address}/pnl-chart` — Public PnL chart.
    PnlChart,
}

impl AsRef<str> for PublicPortfolio {
    fn as_ref(&self) -> &str {
        match self {
            PublicPortfolio::TradedVolume => "public/portfolio",
            PublicPortfolio::Positions => "public/portfolio",
            PublicPortfolio::PnlChart => "public/portfolio",
        }
    }
}

// ── WebSocket API ──

/// WebSocket API endpoints for the Limitless Exchange.
#[derive(Debug, Clone)]
pub enum WebsocketAPI {
    /// Public market data stream (`/markets` namespace).
    Markets,
}

impl AsRef<str> for WebsocketAPI {
    fn as_ref(&self) -> &str {
        match self {
            WebsocketAPI::Markets => "/markets",
        }
    }
}

// ── `API` — delegation to the per-category enums ────────────────────────

impl AsRef<str> for API {
    fn as_ref(&self) -> &str {
        match self {
            API::Auth(endpoint) => endpoint.as_ref(),
            API::Market(endpoint) => endpoint.as_ref(),
            API::Trade(endpoint) => endpoint.as_ref(),
            API::PortfolioEndpoint(endpoint) => endpoint.as_ref(),
            API::Nav(endpoint) => endpoint.as_ref(),
            API::ApiToken(endpoint) => endpoint.as_ref(),
            API::Partner(endpoint) => endpoint.as_ref(),
            API::PublicPortfolio(endpoint) => endpoint.as_ref(),
            API::Amm(endpoint) => endpoint.as_ref(),
            API::System(endpoint) => endpoint.as_ref(),
            API::Referral(endpoint) => endpoint.as_ref(),
            API::Leaderboard(endpoint) => endpoint.as_ref(),
        }
    }
}

// ── The `Limitless` trait ──

/// Trait implemented by all manager types for consistent construction.
///
/// Each manager provides either public (no auth) or authenticated
/// API methods scoped to a specific domain (markets, trading, portfolio, etc.).
pub trait Limitless {
    /// Create a new manager instance with optional API key and secret.
    ///
    /// Use `None` for both when accessing public endpoints only.
    fn new(api_key: Option<String>, secret: Option<String>) -> Self;

    /// Create a new manager instance with a custom `Config`.
    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self;
}
