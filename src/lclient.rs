//! Unified client — the one-stop entry point for the Limitless Exchange API.
//!
//! `LimitlessClient` exposes every API method directly, so you never need
//! to reach through intermediary managers. It also implements the
//! [`Limitless`] trait for consistent construction.
//!
//! # Quick Start
//!
//! ```no_run
//! use limitless::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), LimitlessError> {
//!     // Reads LIMITLESS_API_KEY + LIMITLESS_API_SECRET from the environment
//!     let api = LimitlessClient::builder().build()?;
//!
//!     // Public: browse markets (no auth)
//!     let active = api.browse_active(None, None, Some(10), None, None, None).await?;
//!
//!     // Public: get orderbook
//!     let ob = api.get_orderbook("btc-above-100k").await?;
//!
//!     // Authenticated: positions
//!     let positions = api.get_positions().await?;
//!
//!     // Authenticated: place a limit buy — one call does it all
//!     api.buy_gtc(private_key, "btc-above-100k", token_id, 0.51, 10.0, owner_id).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::prelude::*;
use crate::retry::RetryConfig;

/// The primary entry point for all Limitless Exchange API operations.
///
/// Every REST endpoint and convenience method is available directly on this
/// struct. The internal manager types ([`Markets`], [`Trader`], [`Portfolio`],
/// [`Navigation`], [`Stream`]) are still available through accessor methods
/// when you need them, but for the common case you never have to think about
/// them.
///
/// # Authentication
///
/// Credentials are read automatically from environment variables
/// `LIMITLESS_API_KEY` and `LIMITLESS_API_SECRET` when not explicitly set
/// on the builder.
#[derive(Clone)]
pub struct LimitlessClient {
    /// Underlying HTTP/WS client (shared by all managers).
    client: Client,
    /// Configuration (endpoints, recv window).
    config: Config,
    /// Retry policy.
    retry_config: RetryConfig,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Construction — Limitless trait + builder
// ═══════════════════════════════════════════════════════════════════════════

impl Limitless for LimitlessClient {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(
                api_key.clone(),
                secret.clone(),
                config.rest_api_endpoint.to_string(),
            ),
            config: config.clone(),
            retry_config: RetryConfig::default(),
        }
    }
}

impl LimitlessClient {
    /// Create a new [`LimitlessClientBuilder`].
    pub fn builder() -> LimitlessClientBuilder {
        LimitlessClientBuilder::default()
    }

    /// Access the underlying HTTP client for custom / advanced requests.
    pub fn raw_client(&self) -> &Client {
        &self.client
    }

    /// Update credentials at runtime.
    pub fn set_credentials(&mut self, api_key: Option<String>, secret_key: Option<String>) {
        self.client = Client::new(
            api_key.clone(),
            secret_key.clone(),
            self.config.rest_api_endpoint.to_string(),
        );
    }

    /// The current retry configuration.
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }

    /// The current REST API base URL.
    pub fn base_url(&self) -> &str {
        &self.config.rest_api_endpoint
    }

    // ── Sub-manager accessors (for advanced / standalone use) ──────────

    /// Access the raw [`Markets`] manager.
    pub fn markets(&self) -> Markets {
        Markets::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        )
    }

    /// Access the raw [`Trader`] manager.
    pub fn trader(&self) -> Trader {
        Trader::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        )
    }

    /// Access the raw [`Portfolio`] manager.
    pub fn portfolio(&self) -> Portfolio {
        Portfolio::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        )
    }

    /// Access the raw [`Navigation`] manager.
    pub fn navigation(&self) -> Navigation {
        Navigation::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        )
    }

    /// Access the raw [`Stream`] manager for WebSocket subscriptions.
    pub fn stream(&self) -> Stream {
        Stream::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        )
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Markets — public market data
    // ═══════════════════════════════════════════════════════════════════

    /// Browse all active (unresolved) markets with optional filters.
    pub async fn browse_active(
        &self,
        category_id: Option<u64>,
        page: Option<u64>,
        limit: Option<u64>,
        sort_by: Option<String>,
        trade_type: Option<String>,
        automation_type: Option<String>,
    ) -> Result<ActiveMarketsResponse, LimitlessError> {
        self.markets()
            .browse_active(
                category_id,
                page,
                limit,
                sort_by,
                trade_type,
                automation_type,
            )
            .await
    }

    /// Get the count of active markets per category.
    pub async fn get_category_counts(&self) -> Result<CategoryCountResponse, LimitlessError> {
        self.markets().get_category_counts().await
    }

    /// Get all active market slugs with metadata.
    pub async fn get_active_slugs(&self) -> Result<Vec<ActiveSlug>, LimitlessError> {
        self.markets().get_active_slugs().await
    }

    /// Get detailed market information by address or slug.
    pub async fn get_market(&self, address_or_slug: &str) -> Result<MarketDetail, LimitlessError> {
        self.markets().get_market(address_or_slug).await
    }

    /// Get Chainlink oracle candlestick data for a market.
    pub async fn get_oracle_candles(
        &self,
        address_or_slug: &str,
        interval: Option<&str>,
        from: Option<u64>,
        to: Option<u64>,
    ) -> Result<OracleCandlesResponse, LimitlessError> {
        self.markets()
            .get_oracle_candles(address_or_slug, interval, from, to)
            .await
    }

    /// Get feed events (trades, orders, liquidity) for a market.
    pub async fn get_feed_events(
        &self,
        slug: &str,
        page: Option<u64>,
        limit: Option<u64>,
    ) -> Result<FeedEventsResponse, LimitlessError> {
        self.markets().get_feed_events(slug, page, limit).await
    }

    /// Semantic search for markets using natural language queries.
    pub async fn search_markets(
        &self,
        query: &str,
        limit: Option<u64>,
        page: Option<u64>,
        similarity_threshold: Option<f64>,
    ) -> Result<SearchResponse, LimitlessError> {
        self.markets()
            .search(query, limit, page, similarity_threshold)
            .await
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Trading — orders, orderbook, cancels
    // ═══════════════════════════════════════════════════════════════════

    /// Create a new order from a raw JSON body.
    pub async fn create_order(
        &self,
        order_request: &str,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.trader().create_order(order_request).await
    }

    /// Fetch statuses for multiple orders in batch.
    pub async fn order_status_batch(
        &self,
        request_body: &str,
    ) -> Result<OrderStatusBatchResponse, LimitlessError> {
        self.trader().order_status_batch(request_body).await
    }

    /// Cancel a single order by orderId or clientOrderId.
    pub async fn cancel_combined(
        &self,
        request_body: &str,
    ) -> Result<CancelOrderResponse, LimitlessError> {
        self.trader().cancel_combined(request_body).await
    }

    /// Cancel multiple orders by internal orderIds.
    pub async fn cancel_batch(
        &self,
        request_body: &str,
    ) -> Result<CancelBatchResponse, LimitlessError> {
        self.trader().cancel_batch(request_body).await
    }

    /// Cancel a single order by internal orderId.
    pub async fn cancel_order_by_id(
        &self,
        order_id: &str,
    ) -> Result<CancelOrderResponse, LimitlessError> {
        self.trader().cancel_order_by_id(order_id).await
    }

    /// Cancel all orders for the authenticated user in a specific market.
    pub async fn cancel_all_in_market(
        &self,
        slug: &str,
    ) -> Result<CancelAllResponse, LimitlessError> {
        self.trader().cancel_all_in_market(slug).await
    }

    /// Get the current orderbook for a market.
    pub async fn get_orderbook(&self, slug: &str) -> Result<OrderbookResponse, LimitlessError> {
        self.trader().get_orderbook(slug).await
    }

    /// Get historical price data for a market.
    pub async fn get_historical_prices(
        &self,
        slug: &str,
        interval: Option<&str>,
    ) -> Result<Vec<HistoricalPriceData>, LimitlessError> {
        self.trader().get_historical_prices(slug, interval).await
    }

    /// Get the amount of funds locked in open orders.
    pub async fn get_locked_balance(
        &self,
        slug: &str,
    ) -> Result<LockedBalanceResponse, LimitlessError> {
        self.trader().get_locked_balance(slug).await
    }

    /// Get all orders placed by the authenticated user in a market.
    pub async fn get_user_orders(
        &self,
        slug: &str,
        statuses: Option<&[&str]>,
        limit: Option<u64>,
    ) -> Result<UserOrdersResponse, LimitlessError> {
        self.trader().get_user_orders(slug, statuses, limit).await
    }

    /// Get recent market events (trades, orders, liquidity changes).
    pub async fn get_market_events(
        &self,
        slug: &str,
        page: Option<u64>,
        limit: Option<u64>,
    ) -> Result<MarketEventsResponse, LimitlessError> {
        self.trader().get_market_events(slug, page, limit).await
    }

    // ── High-level order placement ────────────────────────────────────

    /// Place a GTC buy limit order — one call does it all.
    ///
    /// Automatically fetches the venue contract, builds, signs, and submits.
    pub async fn buy_gtc(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        price: f64,
        size: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.trader()
            .buy_gtc(private_key, market_slug, token_id, price, size, owner_id)
            .await
    }

    /// Place a GTC sell limit order — one call does it all.
    pub async fn sell_gtc(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        price: f64,
        size: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.trader()
            .sell_gtc(private_key, market_slug, token_id, price, size, owner_id)
            .await
    }

    /// Place a FOK buy market order — one call does it all.
    pub async fn buy_fok(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        usdc_amount: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.trader()
            .buy_fok(private_key, market_slug, token_id, usdc_amount, owner_id)
            .await
    }

    /// Place a FOK sell market order — one call does it all.
    pub async fn sell_fok(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        share_amount: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.trader()
            .sell_fok(private_key, market_slug, token_id, share_amount, owner_id)
            .await
    }

    /// Cancel all open orders in a market (convenience alias).
    pub async fn cancel_all(&self, slug: &str) -> Result<CancelAllResponse, LimitlessError> {
        self.trader().cancel_all(slug).await
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Portfolio — profile, positions, PnL, history
    // ═══════════════════════════════════════════════════════════════════

    /// Get your own profile by wallet address.
    pub async fn get_profile(&self, account: &str) -> Result<ProfileResponse, LimitlessError> {
        self.portfolio().get_profile(account).await
    }

    /// Retrieve all AMM trades for the authenticated user.
    pub async fn get_trades(&self) -> Result<Vec<TradeEntry>, LimitlessError> {
        self.portfolio().get_trades().await
    }

    /// Retrieve all active positions with P&L and market values.
    pub async fn get_positions(&self) -> Result<PositionsResponse, LimitlessError> {
        self.portfolio().get_positions().await
    }

    /// Get PnL chart data.
    pub async fn get_pnl_chart(
        &self,
        timeframe: Option<&str>,
    ) -> Result<PnlChartResponse, LimitlessError> {
        self.portfolio().get_pnl_chart(timeframe).await
    }

    /// Get points breakdown for the authenticated user.
    pub async fn get_points(&self) -> Result<PointsResponse, LimitlessError> {
        self.portfolio().get_points().await
    }

    /// Get cursor-paginated portfolio history.
    pub async fn get_history(
        &self,
        cursor: Option<&str>,
        limit: Option<u64>,
    ) -> Result<HistoryResponse, LimitlessError> {
        self.portfolio().get_history(cursor, limit).await
    }

    /// Check USDC allowance for CLOB or NegRisk trading.
    pub async fn get_allowance(
        &self,
        allowance_type: &str,
        spender: Option<&str>,
    ) -> Result<AllowanceResponse, LimitlessError> {
        self.portfolio()
            .get_allowance(allowance_type, spender)
            .await
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Navigation — market discovery
    // ═══════════════════════════════════════════════════════════════════

    /// Get the full hierarchical navigation tree.
    pub async fn get_navigation_tree(&self) -> Result<Vec<NavigationNode>, LimitlessError> {
        self.navigation().get_navigation_tree().await
    }

    /// Resolve a URL path to a market page configuration.
    pub async fn get_page_by_path(&self, path: &str) -> Result<MarketPage, LimitlessError> {
        self.navigation().get_page_by_path(path).await
    }

    /// List markets belonging to a specific market page.
    pub async fn list_page_markets(
        &self,
        page_id: &str,
        cursor: Option<&str>,
        page: Option<u64>,
        limit: Option<u64>,
        sort_by: Option<&str>,
        filters: Option<&BTreeMap<String, String>>,
    ) -> Result<PageMarketsResponse, LimitlessError> {
        self.navigation()
            .list_page_markets(page_id, cursor, page, limit, sort_by, filters)
            .await
    }

    /// List all property keys with their options.
    pub async fn list_property_keys(&self) -> Result<Vec<PropertyKey>, LimitlessError> {
        self.navigation().list_property_keys().await
    }

    /// Get a specific property key by ID.
    pub async fn get_property_key(&self, key_id: &str) -> Result<PropertyKey, LimitlessError> {
        self.navigation().get_property_key(key_id).await
    }

    /// List options for a specific property key.
    pub async fn list_property_options(
        &self,
        key_id: &str,
        parent_id: Option<&str>,
    ) -> Result<Vec<PropertyOption>, LimitlessError> {
        self.navigation()
            .list_property_options(key_id, parent_id)
            .await
    }

    // ═══════════════════════════════════════════════════════════════════
    //  WebSocket
    // ═══════════════════════════════════════════════════════════════════

    /// Test WebSocket connectivity with a ping/pong.
    pub async fn ws_ping(&self) -> Result<(), LimitlessError> {
        self.stream().ws_ping().await
    }

    /// Subscribe to WebSocket events with a handler callback.
    pub async fn ws_subscribe<F>(&self, handler: F) -> Result<(), LimitlessError>
    where
        F: FnMut(Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        self.stream().ws_subscribe(handler).await
    }

    /// Subscribe to WebSocket events with dynamic command support.
    pub async fn ws_subscribe_with_commands<F>(
        &self,
        cmd_receiver: tokio::sync::mpsc::UnboundedReceiver<String>,
        handler: F,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        self.stream()
            .ws_subscribe_with_commands(cmd_receiver, handler)
            .await
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Builder
// ═══════════════════════════════════════════════════════════════════════════

/// Builder for [`LimitlessClient`].
///
/// # Example
///
/// ```no_run
/// use limitless::prelude::*;
///
/// let api = LimitlessClient::builder()
///     .api_key("lmts_sk_...")
///     .secret("base64_secret")
///     .build()
///     .unwrap();
/// ```
#[derive(Default)]
pub struct LimitlessClientBuilder {
    api_key: Option<String>,
    secret_key: Option<String>,
    rest_endpoint: Option<String>,
    ws_endpoint: Option<String>,
    recv_window: Option<u64>,
    retry_config: Option<RetryConfig>,
}

impl LimitlessClientBuilder {
    /// Set the API key (token ID for scoped HMAC tokens, or legacy API key).
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the API secret (base64-encoded HMAC secret for scoped tokens).
    pub fn secret(mut self, secret: impl Into<String>) -> Self {
        self.secret_key = Some(secret.into());
        self
    }

    /// Use a custom REST API endpoint.
    pub fn rest_endpoint(mut self, url: impl Into<String>) -> Self {
        self.rest_endpoint = Some(url.into());
        self
    }

    /// Use a custom WebSocket endpoint.
    pub fn ws_endpoint(mut self, url: impl Into<String>) -> Self {
        self.ws_endpoint = Some(url.into());
        self
    }

    /// Set the receive window for HMAC request validation (milliseconds).
    pub fn recv_window(mut self, ms: u64) -> Self {
        self.recv_window = Some(ms);
        self
    }

    /// Use Base Sepolia testnet endpoints.
    pub fn testnet(mut self, use_testnet: bool) -> Self {
        if use_testnet {
            self.rest_endpoint = Some("https://api.testnet.limitless.exchange".into());
            self.ws_endpoint = Some("wss://ws.testnet.limitless.exchange/markets".into());
        }
        self
    }

    /// Configure retry behavior.
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }

    /// Disable automatic retries.
    pub fn no_retry(mut self) -> Self {
        self.retry_config = Some(RetryConfig::none());
        self
    }

    /// Build the [`LimitlessClient`].
    ///
    /// Falls back to `LIMITLESS_API_KEY` and `LIMITLESS_API_SECRET` env vars.
    pub fn build(self) -> Result<LimitlessClient, LimitlessError> {
        let api_key = self
            .api_key
            .or_else(|| std::env::var("LIMITLESS_API_KEY").ok())
            .filter(|k| !k.is_empty());

        let secret_key = self
            .secret_key
            .or_else(|| std::env::var("LIMITLESS_API_SECRET").ok())
            .filter(|s| !s.is_empty());

        let rest_endpoint = self
            .rest_endpoint
            .unwrap_or_else(|| Config::DEFAULT_REST_API_ENDPOINT.into());

        let ws_endpoint = self
            .ws_endpoint
            .unwrap_or_else(|| Config::DEFAULT_WS_ENDPOINT.into());

        let recv_window = self.recv_window.unwrap_or(5000);

        let config = Config::new(rest_endpoint, ws_endpoint, recv_window);
        let retry_config = self.retry_config.unwrap_or_default();

        if api_key.is_none() && secret_key.is_none() {
            log::warn!(
                "No API credentials provided — authenticated endpoints will fail. \
                 Set LIMITLESS_API_KEY + LIMITLESS_API_SECRET environment variables \
                 or pass credentials to the builder."
            );
        }

        Ok(LimitlessClient {
            client: Client::new(
                api_key.clone(),
                secret_key.clone(),
                config.rest_api_endpoint.to_string(),
            ),
            config,
            retry_config,
        })
    }
}

// ── Convenience: default client from env vars ──

impl Default for LimitlessClient {
    fn default() -> Self {
        Self::builder()
            .build()
            .expect("Failed to create default LimitlessClient")
    }
}
