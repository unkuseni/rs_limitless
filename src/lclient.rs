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
//!     let private_key = "0xYourPrivateKey...";
//!     let token_id = "1234567890";
//!     let owner_id = 42; // from GET /profiles/me
//!     api.buy_gtc(private_key, "btc-above-100k", token_id, 0.51, 10.0, owner_id).await?;
//!
//!     let _ = (active, ob, positions);
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
        let mut trader = Trader::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        );
        trader.client.on_behalf_of = self.client.on_behalf_of;
        trader
    }

    /// Access the raw [`Portfolio`] manager.
    pub fn portfolio(&self) -> Portfolio {
        let mut portfolio = Portfolio::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        );
        portfolio.client.on_behalf_of = self.client.on_behalf_of;
        portfolio
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

    /// Access the raw [`PartnerAccounts`] manager for sub-account management.
    pub fn partner(&self) -> PartnerAccounts {
        PartnerAccounts::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        )
    }

    /// Access the raw [`Amm`] manager for server-wallet AMM trading.
    pub fn amm(&self) -> Amm {
        Amm::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        )
    }

    /// Access the raw [`System`] manager for maintenance status.
    pub fn system(&self) -> System {
        System::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        )
    }

    /// Access the raw [`Referral`] manager.
    pub fn referral(&self) -> Referral {
        Referral::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        )
    }

    /// Access the raw [`Leaderboard`] manager.
    pub fn leaderboard(&self) -> Leaderboard {
        Leaderboard::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        )
    }

    /// Access the raw [`ApiTokens`] manager for scoped token management.
    pub fn api_tokens(&self) -> ApiTokens {
        ApiTokens::new_with_config(
            &self.config,
            self.client.api_key.clone(),
            self.client.secret_key.clone(),
        )
    }

    /// Return a clone of this client whose signed requests carry the
    /// `x-on-behalf-of` header for the given sub-account profile ID.
    ///
    /// Partner read flow: `GET /portfolio/positions`, `GET /portfolio/history`,
    /// `GET /markets/:slug/user-orders`, and `POST /orders/status/batch` then
    /// return the sub-account's data. Requires the `delegated_signing` scope.
    ///
    /// ```no_run
    /// use limitless::prelude::*;
    ///
    /// # async fn example(api: LimitlessClient) -> Result<(), LimitlessError> {
    /// let sub = api.for_sub_account(1292711);
    /// let positions = sub.get_positions().await?;   // sub-account's positions
    /// let history = sub.get_history(None, Some(10), None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn for_sub_account(&self, profile_id: u64) -> Self {
        let mut cloned = self.clone();
        cloned.client = cloned.client.with_on_behalf(Some(profile_id));
        cloned
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

    /// Get the timeline for a recurring market series, anchored on a slug.
    pub async fn get_market_timeline(
        &self,
        slug: &str,
        before: Option<u64>,
        after: Option<u64>,
    ) -> Result<MarketTimelineResponse, LimitlessError> {
        self.markets()
            .get_market_timeline(slug, before, after)
            .await
    }

    /// Get the global timeline for a recurring market series by symbol and frequency.
    pub async fn get_global_timeline(
        &self,
        symbol: &str,
        frequency: &str,
        sub_frequency: Option<&str>,
        before: Option<u64>,
        after: Option<u64>,
    ) -> Result<MarketTimelineResponse, LimitlessError> {
        self.markets()
            .get_global_timeline(symbol, frequency, sub_frequency, before, after)
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

    /// Cancel multiple orders by orderIds or clientOrderIds (combined batch).
    pub async fn batch_cancel_combined(
        &self,
        request_body: &str,
    ) -> Result<CancelBatchResponse, LimitlessError> {
        self.trader().batch_cancel_combined(request_body).await
    }

    /// Cancel an order and place its replacement in one request.
    pub async fn cancel_replace(
        &self,
        request_body: &str,
    ) -> Result<CancelReplaceResponse, LimitlessError> {
        self.trader().cancel_replace(request_body).await
    }

    /// Submit up to 4 cancel-and-replace operations in one request.
    pub async fn cancel_replace_batch(
        &self,
        request_body: &str,
    ) -> Result<CancelReplaceBatchResponse, LimitlessError> {
        self.trader().cancel_replace_batch(request_body).await
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

    /// Place a FAK buy order — one call does it all.
    pub async fn buy_fak(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        price: f64,
        size: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.trader()
            .buy_fak(private_key, market_slug, token_id, price, size, owner_id)
            .await
    }

    /// Place a FAK sell order — one call does it all.
    pub async fn sell_fak(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        price: f64,
        size: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.trader()
            .sell_fak(private_key, market_slug, token_id, price, size, owner_id)
            .await
    }

    /// Place a **delegated** (unsigned) order for a server-wallet sub-account.
    ///
    /// The server signs with the sub-account's managed Privy wallet — no
    /// private key is needed. Requires the `trading` + `delegated_signing`
    /// scopes. GTC/FAK orders take `price` + `size`; FOK orders take `amount`
    /// (USDC to spend for BUY, shares to sell for SELL).
    pub async fn place_delegated_order(
        &self,
        wallet_address: &str,
        market_slug: &str,
        token_id: &str,
        side: OrderSide,
        order_type: OrderType,
        price: Option<f64>,
        size: Option<f64>,
        amount: Option<f64>,
        sub_account_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.trader()
            .place_delegated_order(
                wallet_address,
                market_slug,
                token_id,
                side,
                order_type,
                price,
                size,
                amount,
                sub_account_id,
            )
            .await
    }

    /// Cancel all open orders in a market (convenience alias).
    pub async fn cancel_all(&self, slug: &str) -> Result<CancelAllResponse, LimitlessError> {
        self.trader().cancel_all(slug).await
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Portfolio — profile, positions, PnL, history
    // ═══════════════════════════════════════════════════════════════════

    /// Get the authenticated caller's private profile without passing an address.
    pub async fn get_current_profile(&self) -> Result<ProfileResponse, LimitlessError> {
        self.portfolio().get_current_profile().await
    }

    /// Update the authenticated profile (`PUT /profiles`).
    pub async fn update_profile(
        &self,
        request_body: &str,
    ) -> Result<ProfileResponse, LimitlessError> {
        self.portfolio().update_profile(request_body).await
    }

    /// Switch the profile's trading wallet mode (`PUT /profiles`).
    ///
    /// Self-signed API orders require EOA mode — call
    /// `set_trading_wallet_mode(TradingWalletMode::Eoa)` if the account ever
    /// enabled 1-click (smart wallet) trading.
    pub async fn set_trading_wallet_mode(
        &self,
        mode: TradingWalletMode,
    ) -> Result<ProfileResponse, LimitlessError> {
        self.portfolio().set_trading_wallet_mode(mode).await
    }

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

    /// Get cursor-paginated portfolio history, optionally filtered by market.
    pub async fn get_history(
        &self,
        cursor: Option<&str>,
        limit: Option<u64>,
        market: Option<&str>,
    ) -> Result<HistoryResponse, LimitlessError> {
        self.portfolio().get_history(cursor, limit, market).await
    }

    /// Get another user's trading history (public — no authentication).
    pub async fn get_public_history(
        &self,
        account: &str,
        cursor: Option<&str>,
        limit: Option<u64>,
    ) -> Result<HistoryResponse, LimitlessError> {
        self.portfolio()
            .get_public_history(account, cursor, limit)
            .await
    }

    /// Redeem resolved conditional-token positions from a server-wallet sub-account.
    pub async fn redeem(&self, request_body: &str) -> Result<Value, LimitlessError> {
        self.portfolio().redeem(request_body).await
    }

    /// Transfer ERC20 funds from a managed server wallet.
    pub async fn withdraw(&self, request_body: &str) -> Result<Value, LimitlessError> {
        self.portfolio().withdraw(request_body).await
    }

    /// Add a withdrawal destination allowlist entry (Privy identity token).
    pub async fn add_withdrawal_address(
        &self,
        identity_token: &str,
        request_body: &str,
    ) -> Result<WithdrawalAddressResponse, LimitlessError> {
        self.portfolio()
            .add_withdrawal_address(identity_token, request_body)
            .await
    }

    /// Remove a withdrawal destination allowlist entry (Privy identity token).
    pub async fn delete_withdrawal_address(
        &self,
        identity_token: &str,
        address: &str,
    ) -> Result<Value, LimitlessError> {
        self.portfolio()
            .delete_withdrawal_address(identity_token, address)
            .await
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
    //  Partner — sub-account management (HMAC, scoped tokens)
    // ═══════════════════════════════════════════════════════════════════

    /// Create a partner sub-account (server wallet or EOA verification).
    pub async fn create_sub_account(&self, request_body: &str) -> Result<Value, LimitlessError> {
        self.partner().create_sub_account(request_body).await
    }

    /// List partner-owned sub-accounts, or recover one by account address.
    pub async fn list_partner_accounts(
        &self,
        account: Option<&str>,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Result<ListPartnerAccountsResponse, LimitlessError> {
        self.partner().list_accounts(account, limit, page).await
    }

    /// Inspect delegated-trading allowance readiness for a sub-account.
    pub async fn check_partner_allowances(
        &self,
        profile_id: &str,
    ) -> Result<Value, LimitlessError> {
        self.partner().check_allowances(profile_id).await
    }

    /// Retry delegated-trading allowance recovery for a sub-account.
    pub async fn retry_partner_allowances(
        &self,
        profile_id: &str,
    ) -> Result<Value, LimitlessError> {
        self.partner().retry_allowances(profile_id).await
    }

    // ═══════════════════════════════════════════════════════════════════
    //  API Tokens — scoped token management (Privy identity + HMAC)
    // ═══════════════════════════════════════════════════════════════════

    /// Check partner capability configuration (Privy identity token).
    pub async fn get_api_token_capabilities(
        &self,
        identity_token: &str,
    ) -> Result<Value, LimitlessError> {
        self.api_tokens().get_capabilities(identity_token).await
    }

    /// Derive a new scoped API token (Privy identity token).
    ///
    /// The returned secret is shown once — store it securely.
    pub async fn derive_api_token(
        &self,
        identity_token: &str,
        request: &DeriveApiTokenRequest,
    ) -> Result<DeriveApiTokenResponse, LimitlessError> {
        self.api_tokens()
            .derive_token(identity_token, request)
            .await
    }

    /// List all active (non-revoked) API tokens.
    pub async fn list_api_tokens(&self) -> Result<Value, LimitlessError> {
        self.api_tokens().list_tokens().await
    }

    /// Revoke an active API token by token ID.
    pub async fn revoke_api_token(&self, token_id: &str) -> Result<Value, LimitlessError> {
        self.api_tokens().revoke_token(token_id).await
    }

    // ═══════════════════════════════════════════════════════════════════
    //  AMM — server-wallet AMM trading (HMAC: trading + delegated_signing)
    // ═══════════════════════════════════════════════════════════════════

    /// Spend collateral to acquire outcome shares on an AMM market.
    pub async fn amm_buy(
        &self,
        request: &AmmBuyRequest,
    ) -> Result<AmmTradeResponse, LimitlessError> {
        self.amm().buy(request).await
    }

    /// Return an exact amount of collateral by selling outcome shares.
    pub async fn amm_sell(
        &self,
        request: &AmmSellRequest,
    ) -> Result<AmmTradeResponse, LimitlessError> {
        self.amm().sell(request).await
    }

    /// Read the on-chain approval state for an AMM market and side.
    pub async fn amm_check_allowance(
        &self,
        request: &AmmAllowanceRequest,
    ) -> Result<AmmAllowanceResponse, LimitlessError> {
        self.amm().check_allowance(request).await
    }

    /// Submit a fresh approval from the server wallet.
    pub async fn amm_approve_allowance(
        &self,
        request: &AmmAllowanceRequest,
    ) -> Result<AmmAllowanceResponse, LimitlessError> {
        self.amm().approve_allowance(request).await
    }

    // ═══════════════════════════════════════════════════════════════════
    //  System — maintenance status (public)
    // ═══════════════════════════════════════════════════════════════════

    /// Get active and scheduled maintenance information.
    pub async fn get_maintenance_status(
        &self,
        target: Option<&str>,
    ) -> Result<MaintenanceStatus, LimitlessError> {
        self.system().get_maintenance_status(target).await
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Referral — referral program earnings and leaderboards
    // ═══════════════════════════════════════════════════════════════════

    /// Get your own referral standing (volume basis, earned USDC, tier ladder).
    pub async fn get_referral_stats(&self) -> Result<ReferralMeResponse, LimitlessError> {
        self.referral().get_my_stats().await
    }

    /// Get your referred users, paginated, filtered, and sorted.
    pub async fn get_referred_users(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
        status: Option<&str>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<ReferralReferralsResponse, LimitlessError> {
        self.referral()
            .get_referred_users(limit, offset, status, sort_by, sort_order)
            .await
    }

    /// Get the global referral leaderboard (public).
    pub async fn get_referral_leaderboard(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<ReferralLeaderboardResponse, LimitlessError> {
        self.referral()
            .get_leaderboard(limit, offset, sort_by, sort_order)
            .await
    }

    /// Get the friends referral leaderboard (public).
    pub async fn get_friends_leaderboard(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<ReferralLeaderboardResponse, LimitlessError> {
        self.referral()
            .get_friends_leaderboard(limit, offset, sort_by, sort_order)
            .await
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Leaderboard — live Unrealized PnL (public)
    // ═══════════════════════════════════════════════════════════════════

    /// Get the live Unrealized PnL leaderboard for one market.
    pub async fn get_market_unrealized_pnl(
        &self,
        market_id: u64,
        metric: Option<&str>,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Result<UnrealizedPnlMarketResponse, LimitlessError> {
        self.leaderboard()
            .get_market_leaderboard(market_id, metric, limit, page)
            .await
    }

    /// Get the live biggest-open-positions leaderboard.
    pub async fn get_biggest_positions(
        &self,
        limit: Option<u64>,
    ) -> Result<UnrealizedPnlBiggestPositionsResponse, LimitlessError> {
        self.leaderboard().get_biggest_positions(limit).await
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

    /// Subscribe to WebSocket events with dynamic command support **and authentication**.
    ///
    /// Enables private channels: `subscribe_positions`, `subscribe_order_events`.
    pub async fn ws_subscribe_authenticated_with_commands<F>(
        &self,
        cmd_receiver: tokio::sync::mpsc::UnboundedReceiver<String>,
        handler: F,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(Value) -> Result<(), LimitlessError> + 'static + Send,
    {
        self.stream()
            .ws_subscribe_authenticated_with_commands(cmd_receiver, handler)
            .await
    }

    /// Subscribe to typed WebSocket events with authentication.
    ///
    /// Enables private channels: `positions`, `orderEvent`.
    pub async fn ws_subscribe_authenticated_events<F>(
        &self,
        handler: F,
    ) -> Result<(), LimitlessError>
    where
        F: FnMut(WsEventKind) -> Result<(), LimitlessError> + 'static + Send,
    {
        self.stream()
            .ws_subscribe_authenticated_events(handler)
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
    ///
    /// **Deprecated:** Limitless has no testnet, sandbox, or mock mode — all
    /// integrations run against Base mainnet with real USDC. This method is
    /// kept for backward compatibility and now maps to the production
    /// endpoints.
    #[deprecated(
        note = "Limitless has no testnet deployment; all integrations run against Base mainnet. Use the default endpoints instead."
    )]
    pub fn testnet(self, use_testnet: bool) -> Self {
        if use_testnet {
            log::warn!(
                "Limitless has no testnet deployment — testnet() is a no-op. \
                 All integrations run against Base mainnet."
            );
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
