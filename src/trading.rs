use crate::prelude::*;
use crate::signing::Eip712Signer;

/// Manages trading operations: order creation, cancellation, status lookup,
/// orderbook access, historical prices, and user-specific order/market data.
///
/// Most endpoints require authentication via scoped HMAC token or legacy API key.
///
/// # Convenience Methods
///
/// For the most common trading workflows, use the high-level methods:
///
/// - [`buy_gtc`](Trader::buy_gtc) / [`sell_gtc`](Trader::sell_gtc) — Place limit orders
/// - [`buy_fok`](Trader::buy_fok) / [`sell_fok`](Trader::sell_fok) — Place market orders
/// - [`cancel_all`](Trader::cancel_all) — Cancel all orders in a market
#[derive(Clone)]
pub struct Trader {
    pub client: Client,
}

impl Trader {
    /// Create a new order on a prediction market.
    ///
    /// Supports GTC (Good Till Cancelled) and FOK (Fill or Kill) order types.
    /// CLOB orders require EIP-712 signatures; AMM orders use a different flow.
    ///
    /// # Arguments
    ///
    /// * `order_request` — Serialized JSON body matching the Create Order schema.
    pub async fn create_order(
        &self,
        order_request: &str,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.client
            .post_signed("orders", Some(order_request.to_string()))
            .await
    }

    /// Fetch statuses for multiple orders in batch.
    ///
    /// Look up by `orderId` or `clientOrderId` (provide exactly one per item).
    /// Accepts up to 50 items per request.
    pub async fn order_status_batch(
        &self,
        request_body: &str,
    ) -> Result<OrderStatusBatchResponse, LimitlessError> {
        self.client
            .post_signed("orders/status/batch", Some(request_body.to_string()))
            .await
    }

    /// Cancel a single order by `orderId` or `clientOrderId` (combined endpoint).
    pub async fn cancel_combined(
        &self,
        request_body: &str,
    ) -> Result<CancelOrderResponse, LimitlessError> {
        self.client
            .post_signed("orders/cancel", Some(request_body.to_string()))
            .await
    }

    /// Cancel multiple orders by internal `orderId`s (batch).
    pub async fn cancel_batch(
        &self,
        request_body: &str,
    ) -> Result<CancelBatchResponse, LimitlessError> {
        self.client
            .post_signed("orders/cancel-batch", Some(request_body.to_string()))
            .await
    }

    /// Cancel a single order by internal `orderId` (legacy endpoint).
    pub async fn cancel_order_by_id(
        &self,
        order_id: &str,
    ) -> Result<CancelOrderResponse, LimitlessError> {
        let path = format!("orders/{}", order_id);
        self.client.delete_signed(&path).await
    }

    /// Cancel all orders for the authenticated user in a specific market.
    pub async fn cancel_all_in_market(
        &self,
        slug: &str,
    ) -> Result<CancelAllResponse, LimitlessError> {
        let path = format!("orders/all/{}", slug);
        self.client.delete_signed(&path).await
    }

    /// Get the current orderbook for a market.
    pub async fn get_orderbook(&self, slug: &str) -> Result<OrderbookResponse, LimitlessError> {
        let path = format!("markets/{}/orderbook", slug);
        self.client.get(&path, None).await
    }

    /// Get historical price data for a market.
    pub async fn get_historical_prices(
        &self,
        slug: &str,
        interval: Option<&str>,
    ) -> Result<Vec<HistoricalPriceData>, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(ref v) = interval {
            params.insert("interval".into(), v.to_string());
        }
        let request = build_request(&params);
        let path = format!("markets/{}/historical-price", slug);
        self.client.get(&path, Some(request)).await
    }

    /// Get the amount of funds locked in open orders for the authenticated user.
    pub async fn get_locked_balance(
        &self,
        slug: &str,
    ) -> Result<LockedBalanceResponse, LimitlessError> {
        let path = format!("markets/{}/locked-balance", slug);
        self.client.get(&path, None).await
    }

    /// Get all orders placed by the authenticated user for a specific market.
    pub async fn get_user_orders(
        &self,
        slug: &str,
        statuses: Option<&[&str]>,
        limit: Option<u64>,
    ) -> Result<UserOrdersResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(s) = statuses {
            params.insert("statuses".into(), s.join(","));
        }
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        let request = build_request(&params);
        let path = format!("markets/{}/user-orders", slug);
        self.client.get(&path, Some(request)).await
    }

    /// Get recent market events (trades, orders, liquidity changes).
    pub async fn get_market_events(
        &self,
        slug: &str,
        page: Option<u64>,
        limit: Option<u64>,
    ) -> Result<MarketEventsResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(v) = page {
            params.insert("page".into(), v.to_string());
        }
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        let request = build_request(&params);
        let path = format!("markets/{}/events", slug);
        self.client.get(&path, Some(request)).await
    }

    // ── High-level convenience methods ──────────────────────────────────

    /// Place a GTC buy limit order — the simplest way to buy YES/NO shares.
    ///
    /// Handles: fetch venue contract → validate → build EIP-712 order → sign → submit.
    ///
    /// # Arguments
    /// * `private_key` — 0x-prefixed hex private key for signing
    /// * `market_slug` — Market identifier (e.g., "btc-above-100k-jul-4")
    /// * `token_id` — The outcome token ID as a decimal string (e.g., from `market.outcomes[0].token_id`)
    /// * `price` — Price between 0 and 1 (e.g., 0.55 for $0.55)
    /// * `size` — Number of shares to buy
    /// * `owner_id` — Your profile ID (from `GET /profiles/:address`)
    pub async fn buy_gtc(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        price: f64,
        size: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.place_gtc_order(
            private_key,
            market_slug,
            token_id,
            OrderSide::Buy,
            price,
            size,
            owner_id,
        )
        .await
    }

    /// Place a GTC sell limit order — the simplest way to sell YES/NO shares.
    pub async fn sell_gtc(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        price: f64,
        size: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.place_gtc_order(
            private_key,
            market_slug,
            token_id,
            OrderSide::Sell,
            price,
            size,
            owner_id,
        )
        .await
    }

    /// Place a FOK buy market order — buy shares at market price.
    pub async fn buy_fok(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        usdc_amount: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.place_fok_order(
            private_key,
            market_slug,
            token_id,
            OrderSide::Buy,
            usdc_amount,
            owner_id,
        )
        .await
    }

    /// Place a FOK sell market order — sell shares at market price.
    pub async fn sell_fok(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        share_amount: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.place_fok_order(
            private_key,
            market_slug,
            token_id,
            OrderSide::Sell,
            share_amount,
            owner_id,
        )
        .await
    }

    /// Cancel all open orders in a market (convenience alias).
    pub async fn cancel_all(&self, slug: &str) -> Result<CancelAllResponse, LimitlessError> {
        self.cancel_all_in_market(slug).await
    }

    // ── Internal helpers ───────────────────────────────────────────────

    /// Resolve the verifying contract for a market slug by fetching market details.
    async fn get_verifying_contract(&self, slug: &str) -> Result<String, LimitlessError> {
        let market: MarketDetail = self.client.get(&format!("markets/{}", slug), None).await?;
        let venue = market.venue.ok_or_else(|| {
            LimitlessError::ValidationError(format!(
                "Market '{}' has no venue info — is it a CLOB market?",
                slug
            ))
        })?;
        Ok(venue.exchange)
    }

    async fn place_gtc_order(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        side: OrderSide,
        price: f64,
        size: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        let verifying_contract = self.get_verifying_contract(market_slug).await?;
        let signer = Eip712Signer::new(private_key, &verifying_contract)
            .map_err(|e| LimitlessError::ValidationError(e))?;

        let order_data = signer
            .build_gtc_order(
                &signer.wallet_address(),
                token_id,
                side,
                price,
                size,
                0, // fee_rate_bps — use default
            )
            .map_err(|e| LimitlessError::ValidationError(e))?;

        let request = CreateOrderRequest {
            order: order_data,
            owner_id,
            order_type: OrderType::Gtc,
            market_slug: market_slug.to_string(),
            client_order_id: None,
            on_behalf_of: None,
        };

        let body = serde_json::to_string(&request).map_err(|e| LimitlessError::Json(e))?;

        self.create_order(&body).await
    }

    async fn place_fok_order(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        side: OrderSide,
        amount: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        let verifying_contract = self.get_verifying_contract(market_slug).await?;
        let signer = Eip712Signer::new(private_key, &verifying_contract)
            .map_err(|e| LimitlessError::ValidationError(e))?;

        let order_data = signer
            .build_fok_order(&signer.wallet_address(), token_id, side, amount, 0)
            .map_err(|e| LimitlessError::ValidationError(e))?;

        let request = CreateOrderRequest {
            order: order_data,
            owner_id,
            order_type: OrderType::Fok,
            market_slug: market_slug.to_string(),
            client_order_id: None,
            on_behalf_of: None,
        };

        let body = serde_json::to_string(&request).map_err(|e| LimitlessError::Json(e))?;

        self.create_order(&body).await
    }
}

impl Limitless for Trader {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(api_key, secret, config.rest_api_endpoint.to_string()),
        }
    }
}
