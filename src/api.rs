//! API endpoint enums for the Limitless Exchange REST and WebSocket APIs.
//!
//! All REST endpoints are categorized into logical groups matching the
//! API reference: Authentication, Markets, Trading, Portfolio, Navigation,
//! API Tokens, Partner Accounts, and Public Portfolio.

use crate::Config;

/// Represents a REST API endpoint category and its specific operation.
///
/// Each variant maps to a URL path relative to the base API URL.
/// The `as_ref()` implementation returns the full path string.
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
    /// `POST /orders/cancel-batch` — Batch cancel (by orderIds).
    CancelBatch,
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

// ── Portfolio ──

#[derive(Debug, Clone)]
pub enum PortfolioEndpoint {
    /// `GET /profiles/{account}` — Get your profile.
    GetProfile,
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
    /// `GET /portfolio/trading/allowance` — Trading allowance check.
    Allowance,
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

// ── API Tokens ──

#[derive(Debug, Clone)]
pub enum ApiToken {
    /// `GET /api-tokens/capabilities` — Partner capabilities.
    GetCapabilities,
    /// `POST /api-tokens/derive` — Derive a scoped token.
    Derive,
    /// `GET /api-tokens` — List active tokens.
    ListActive,
    /// `DELETE /api-tokens/{id}` — Revoke a token.
    Revoke,
}

// ── Partner Accounts ──

#[derive(Debug, Clone)]
pub enum Partner {
    /// `POST /profiles/partner-accounts` — Create partner sub-account.
    CreateSubAccount,
    /// `GET /profiles/partner-accounts/{id}/allowances` — Check allowances.
    CheckAllowances,
    /// `POST /profiles/partner-accounts/{id}/allowances/retry` — Retry allowances.
    RetryAllowances,
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

// ── WebSocket API ──

/// WebSocket API endpoints for the Limitless Exchange.
#[derive(Debug, Clone)]
pub enum WebsocketAPI {
    /// Public market data stream (`/markets` namespace).
    Markets,
}

impl AsRef<str> for API {
    fn as_ref(&self) -> &str {
        match self {
            // Authentication
            API::Auth(Auth::CreateApiKey) => "auth/api-keys",
            API::Auth(Auth::GetApiKey) => "auth/api-keys",
            API::Auth(Auth::RevokeApiKey) => "auth/api-keys",

            // Markets
            API::Market(Market::Active) => "markets/active",
            API::Market(Market::ActiveCategory) => "markets/active",
            API::Market(Market::CategoryCount) => "markets/categories/count",
            API::Market(Market::ActiveSlugs) => "markets/active/slugs",
            API::Market(Market::GetMarket) => "markets",
            API::Market(Market::OracleCandles) => "markets",
            API::Market(Market::FeedEvents) => "markets",
            API::Market(Market::Search) => "markets/search",

            // Trading
            API::Trade(Trade::CreateOrder) => "orders",
            API::Trade(Trade::OrderStatusBatch) => "orders/status/batch",
            API::Trade(Trade::CancelCombined) => "orders/cancel",
            API::Trade(Trade::CancelBatch) => "orders/cancel-batch",
            API::Trade(Trade::CancelOrder) => "orders",
            API::Trade(Trade::CancelAll) => "orders/all",
            API::Trade(Trade::Orderbook) => "markets",
            API::Trade(Trade::HistoricalPrice) => "markets",
            API::Trade(Trade::LockedBalance) => "markets",
            API::Trade(Trade::UserOrders) => "markets",
            API::Trade(Trade::MarketEvents) => "markets",

            // Portfolio
            API::PortfolioEndpoint(PortfolioEndpoint::GetProfile) => "profiles",
            API::PortfolioEndpoint(PortfolioEndpoint::Trades) => "portfolio/trades",
            API::PortfolioEndpoint(PortfolioEndpoint::Positions) => "portfolio/positions",
            API::PortfolioEndpoint(PortfolioEndpoint::PnlChart) => "portfolio/pnl-chart",
            API::PortfolioEndpoint(PortfolioEndpoint::Points) => "portfolio/points",
            API::PortfolioEndpoint(PortfolioEndpoint::History) => "portfolio/history",
            API::PortfolioEndpoint(PortfolioEndpoint::Allowance) => "portfolio/trading/allowance",

            // Navigation
            API::Nav(Nav::GetNavigation) => "navigation",
            API::Nav(Nav::GetPageByPath) => "market-pages/by-path",
            API::Nav(Nav::ListPageMarkets) => "market-pages",
            API::Nav(Nav::ListPropertyKeys) => "property-keys",
            API::Nav(Nav::GetPropertyKey) => "property-keys",
            API::Nav(Nav::ListPropertyOptions) => "property-keys",

            // API Tokens
            API::ApiToken(ApiToken::GetCapabilities) => "api-tokens/capabilities",
            API::ApiToken(ApiToken::Derive) => "api-tokens/derive",
            API::ApiToken(ApiToken::ListActive) => "api-tokens",
            API::ApiToken(ApiToken::Revoke) => "api-tokens", // id appended

            // Partner Accounts
            API::Partner(Partner::CreateSubAccount) => "profiles/partner-accounts",
            API::Partner(Partner::CheckAllowances) => "profiles/partner-accounts", // appended
            API::Partner(Partner::RetryAllowances) => "profiles/partner-accounts", // appended

            // Public Portfolio
            API::PublicPortfolio(PublicPortfolio::TradedVolume) => "public/portfolio", // appended
            API::PublicPortfolio(PublicPortfolio::Positions) => "public/portfolio",    // appended
            API::PublicPortfolio(PublicPortfolio::PnlChart) => "public/portfolio",     // appended
        }
    }
}

impl AsRef<str> for WebsocketAPI {
    fn as_ref(&self) -> &str {
        match self {
            WebsocketAPI::Markets => "/markets",
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
