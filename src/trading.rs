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
/// - [`buy_fak`](Trader::buy_fak) / [`sell_fak`](Trader::sell_fak) — Fill-and-kill limit orders
/// - [`buy_fok`](Trader::buy_fok) / [`sell_fok`](Trader::sell_fok) — Place market orders
/// - [`place_delegated_order`](Trader::place_delegated_order) — Unsigned order for a
///   server-wallet sub-account (server signs via managed Privy wallet)
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

    /// Cancel multiple orders by `orderId` or `clientOrderId` (combined batch
    /// endpoint). Provide exactly one identifier array (max 50 items).
    pub async fn batch_cancel_combined(
        &self,
        request_body: &str,
    ) -> Result<CancelBatchResponse, LimitlessError> {
        self.client
            .post_signed("orders/batch-cancel", Some(request_body.to_string()))
            .await
    }

    /// Cancel an order and submit its replacement in one request.
    ///
    /// The two actions are non-atomic; each reports its own outcome
    /// (`SUCCESS`, `FAILURE`, `UNKNOWN`, or `NOT_ATTEMPTED` for the
    /// replacement). Full success returns `200`; otherwise `409`.
    pub async fn cancel_replace(
        &self,
        request_body: &str,
    ) -> Result<CancelReplaceResponse, LimitlessError> {
        self.client
            .post_signed("orders/cancel-replace", Some(request_body.to_string()))
            .await
    }

    /// Submit up to 4 cancel-and-replace operations in one request.
    ///
    /// After request-wide preflight, operations execute sequentially with
    /// independent results, preserving their input indexes. All-success
    /// returns `200`; partial success returns `207`.
    pub async fn cancel_replace_batch(
        &self,
        request_body: &str,
    ) -> Result<CancelReplaceBatchResponse, LimitlessError> {
        self.client
            .post_signed(
                "orders/cancel-replace/batch",
                Some(request_body.to_string()),
            )
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

    /// Place a FAK (fill-and-kill) buy order — matches immediately available
    /// liquidity up to `size` at `price` and cancels any unmatched remainder.
    ///
    /// FAK uses the same price/size formulas as GTC; it never rests on the book.
    pub async fn buy_fak(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        price: f64,
        size: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.place_fak_order(
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

    /// Place a FAK (fill-and-kill) sell order.
    pub async fn sell_fak(
        &self,
        private_key: &str,
        market_slug: &str,
        token_id: &str,
        price: f64,
        size: f64,
        owner_id: u64,
    ) -> Result<CreateOrderResponse, LimitlessError> {
        self.place_fak_order(
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

    /// Place a **delegated** order on behalf of a server-wallet sub-account.
    ///
    /// Requires an API token with the `trading` + `delegated_signing` scopes.
    /// The order is sent **unsigned** — the server signs it with the Privy
    /// server wallet linked to the sub-account, so no private key is needed
    /// on the bot side.
    ///
    /// # Arguments
    /// * `wallet_address` — the sub-account's wallet address (`account` from
    ///   `POST /profiles/partner-accounts`). The server may override
    ///   `maker`/`signer` with the managed wallet.
    /// * `market_slug` — market identifier.
    /// * `token_id` — outcome token ID (decimal string).
    /// * `side` — BUY or SELL.
    /// * `order_type` — `Gtc`, `Fak`, or `Fok`.
    /// * `price` / `size` — limit price and size for GTC/FAK orders.
    /// * `amount` — for FOK: USDC to spend (BUY) or shares to sell (SELL).
    /// * `sub_account_id` — the sub-account's `profileId`.
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
        let order_data = build_unsigned_order(
            wallet_address,
            token_id,
            side,
            order_type,
            price,
            size,
            amount,
        )
        .map_err(LimitlessError::ValidationError)?;

        let request = CreateOrderRequest {
            order: order_data,
            owner_id: sub_account_id,
            order_type,
            market_slug: market_slug.to_string(),
            client_order_id: None,
            on_behalf_of: Some(sub_account_id),
            post_only: None,
            timestamp: None,
            recv_window: None,
            stp_policy: None,
        };

        let body = serde_json::to_string(&request).map_err(LimitlessError::Json)?;

        self.create_order(&body).await
    }

    /// Return a trader whose signed requests carry the `x-on-behalf-of` header
    /// for the given sub-account profile ID.
    ///
    /// Supported read endpoints: user orders, order status batch. Requires
    /// the `delegated_signing` scope on the API token.
    pub fn for_sub_account(&self, profile_id: u64) -> Self {
        Self {
            client: self.client.with_on_behalf(Some(profile_id)),
        }
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
            .map_err(LimitlessError::ValidationError)?;

        let order_data = signer
            .build_gtc_order(
                &signer.wallet_address(),
                token_id,
                side,
                price,
                size,
                0, // fee_rate_bps — use default
            )
            .map_err(LimitlessError::ValidationError)?;

        let request = CreateOrderRequest {
            order: order_data,
            owner_id,
            order_type: OrderType::Gtc,
            market_slug: market_slug.to_string(),
            client_order_id: None,
            on_behalf_of: None,
            post_only: None,
            timestamp: None,
            recv_window: None,
            stp_policy: None,
        };

        let body = serde_json::to_string(&request).map_err(LimitlessError::Json)?;

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
            .map_err(LimitlessError::ValidationError)?;

        let order_data = signer
            .build_fok_order(&signer.wallet_address(), token_id, side, amount, 0)
            .map_err(LimitlessError::ValidationError)?;

        let request = CreateOrderRequest {
            order: order_data,
            owner_id,
            order_type: OrderType::Fok,
            market_slug: market_slug.to_string(),
            client_order_id: None,
            on_behalf_of: None,
            post_only: None,
            timestamp: None,
            recv_window: None,
            stp_policy: None,
        };

        let body = serde_json::to_string(&request).map_err(LimitlessError::Json)?;

        self.create_order(&body).await
    }

    async fn place_fak_order(
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
            .map_err(LimitlessError::ValidationError)?;

        // FAK uses the same price/size amount formulas as GTC.
        let order_data = signer
            .build_gtc_order(
                &signer.wallet_address(),
                token_id,
                side,
                price,
                size,
                0, // fee_rate_bps — use default
            )
            .map_err(LimitlessError::ValidationError)?;

        let request = CreateOrderRequest {
            order: order_data,
            owner_id,
            order_type: OrderType::Fak,
            market_slug: market_slug.to_string(),
            client_order_id: None,
            on_behalf_of: None,
            post_only: None,
            timestamp: None,
            recv_window: None,
            stp_policy: None,
        };

        let body = serde_json::to_string(&request).map_err(LimitlessError::Json)?;

        self.create_order(&body).await
    }
}

// ── Unsigned order building (delegated signing) ──────────────────────────

/// Build an **unsigned** [`OrderData`] for delegated signing.
///
/// With the `delegated_signing` scope the server signs the order using the
/// Privy server wallet linked to the target sub-account, so `signature` and
/// `signature_type` are omitted. GTC/FAK use price + size; FOK uses the raw
/// amount (USDC to spend for BUY, shares to sell for SELL).
pub fn build_unsigned_order(
    wallet_address: &str,
    token_id: &str,
    side: OrderSide,
    order_type: OrderType,
    price: Option<f64>,
    size: Option<f64>,
    amount: Option<f64>,
) -> Result<OrderData, String> {
    use crate::signing::generate_salt;

    let zero_taker = "0x0000000000000000000000000000000000000000".to_string();

    match order_type {
        OrderType::Gtc | OrderType::Fak => {
            let price = price.ok_or("GTC/FAK orders require `price`")?;
            let size = size.ok_or("GTC/FAK orders require `size`")?;
            validate_gtc_order(price, size, None)?;
            let (maker_amount, taker_amount) = gtc_amounts(side, price, size);
            Ok(OrderData {
                salt: generate_salt(),
                maker: wallet_address.to_string(),
                signer: wallet_address.to_string(),
                taker: zero_taker,
                token_id: token_id.to_string(),
                maker_amount,
                taker_amount,
                expiration: "0".to_string(),
                nonce: 0,
                fee_rate_bps: 0,
                side: side.to_u8(),
                signature: None,
                signature_type: None,
            })
        }
        OrderType::Fok => {
            let amount = amount.ok_or("FOK orders require `amount`")?;
            validate_fok_order(amount)?;
            let maker_amount = fok_amount(side, amount);
            Ok(OrderData {
                salt: generate_salt(),
                maker: wallet_address.to_string(),
                signer: wallet_address.to_string(),
                taker: zero_taker,
                token_id: token_id.to_string(),
                maker_amount,
                taker_amount: 1, // FOK: always 1
                expiration: "0".to_string(),
                nonce: 0,
                fee_rate_bps: 0,
                side: side.to_u8(),
                signature: None,
                signature_type: None,
            })
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned_order_omits_signature_fields() {
        let order = build_unsigned_order(
            "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
            "12345",
            OrderSide::Buy,
            OrderType::Gtc,
            Some(0.55),
            Some(10.0),
            None,
        )
        .unwrap();

        let json = serde_json::to_string(&order).unwrap();
        assert!(!json.contains("signature"), "unsigned order: {json}");
        assert_eq!(order.maker_amount, 5_500_000);
        assert_eq!(order.taker_amount, 10_000_000);
        assert_eq!(order.expiration, "0");
        assert_eq!(order.nonce, 0);
    }

    #[test]
    fn unsigned_fok_uses_taker_amount_one() {
        let order = build_unsigned_order(
            "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
            "12345",
            OrderSide::Sell,
            OrderType::Fok,
            None,
            None,
            Some(18.64),
        )
        .unwrap();

        assert_eq!(order.maker_amount, 18_640_000);
        assert_eq!(order.taker_amount, 1);
    }

    #[test]
    fn unsigned_order_requires_right_args() {
        assert!(build_unsigned_order(
            "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
            "12345",
            OrderSide::Buy,
            OrderType::Gtc,
            None,
            Some(10.0),
            None,
        )
        .is_err());
        assert!(build_unsigned_order(
            "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
            "12345",
            OrderSide::Buy,
            OrderType::Fok,
            Some(0.55),
            Some(10.0),
            None,
        )
        .is_err());
    }
}
