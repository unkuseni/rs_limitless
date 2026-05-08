use crate::prelude::*;

/// Provides access to public market data on the Limitless Exchange.
///
/// No authentication is required for these endpoints. Use this manager
/// to browse active markets, fetch details, search, get oracle data,
/// and retrieve feed events.
#[derive(Clone)]
pub struct Markets {
    pub client: Client,
}

impl Markets {
    /// Browse all active (unresolved) markets.
    ///
    /// Supports optional filtering by category, trade type, and automation type
    /// with pagination via `page` and `limit`.
    pub async fn browse_active(
        &self,
        category_id: Option<u64>,
        page: Option<u64>,
        limit: Option<u64>,
        sort_by: Option<String>,
        trade_type: Option<String>,
        automation_type: Option<String>,
    ) -> Result<ActiveMarketsResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(v) = page {
            params.insert("page".into(), v.to_string());
        }
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        if let Some(ref v) = sort_by {
            params.insert("sortBy".into(), v.clone());
        }
        if let Some(ref v) = trade_type {
            params.insert("tradeType".into(), v.clone());
        }
        if let Some(ref v) = automation_type {
            params.insert("automationType".into(), v.clone());
        }
        let request = build_request(&params);

        let path = if let Some(cat_id) = category_id {
            format!("markets/active/{}", cat_id)
        } else {
            "markets/active".to_string()
        };

        self.client.get(&path, Some(request)).await
    }

    /// Get the count of active markets per category.
    pub async fn get_category_counts(&self) -> Result<CategoryCountResponse, LimitlessError> {
        self.client.get("markets/categories/count", None).await
    }

    /// Get all active market slugs with metadata (strike price, ticker, deadline).
    pub async fn get_active_slugs(&self) -> Result<Vec<ActiveSlug>, LimitlessError> {
        self.client.get("markets/active/slugs", None).await
    }

    /// Get detailed market information by address or slug.
    ///
    /// Returns venue data (`exchange` and `adapter` addresses) needed for
    /// EIP-712 order signing on CLOB markets.
    pub async fn get_market(&self, address_or_slug: &str) -> Result<MarketDetail, LimitlessError> {
        let path = format!("markets/{}", address_or_slug);
        self.client.get(&path, None).await
    }

    /// Get Chainlink oracle candlestick data for markets with Data Streams.
    pub async fn get_oracle_candles(
        &self,
        address_or_slug: &str,
        interval: Option<&str>,
        from: Option<u64>,
        to: Option<u64>,
    ) -> Result<OracleCandlesResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(ref v) = interval {
            params.insert("interval".into(), v.to_string());
        }
        if let Some(v) = from {
            params.insert("from".into(), v.to_string());
        }
        if let Some(v) = to {
            params.insert("to".into(), v.to_string());
        }
        let request = build_request(&params);
        let path = format!("markets/{}/oracle-candles", address_or_slug);
        self.client.get(&path, Some(request)).await
    }

    /// Get feed events (trades, orders, liquidity changes) for a specific market.
    pub async fn get_feed_events(
        &self,
        slug: &str,
        page: Option<u64>,
        limit: Option<u64>,
    ) -> Result<FeedEventsResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(v) = page {
            params.insert("page".into(), v.to_string());
        }
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        let request = build_request(&params);
        let path = format!("markets/{}/get-feed-events", slug);
        self.client.get(&path, Some(request)).await
    }

    /// Semantic search for markets using natural language queries.
    ///
    /// Supports configurable similarity threshold and pagination.
    pub async fn search(
        &self,
        query: &str,
        limit: Option<u64>,
        page: Option<u64>,
        similarity_threshold: Option<f64>,
    ) -> Result<SearchResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        params.insert("query".into(), query.to_string());
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        if let Some(v) = page {
            params.insert("page".into(), v.to_string());
        }
        if let Some(v) = similarity_threshold {
            params.insert("similarityThreshold".into(), v.to_string());
        }
        let request = build_request(&params);
        self.client.get("markets/search", Some(request)).await
    }
}

impl Limitless for Markets {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(api_key, secret, config.rest_api_endpoint.to_string()),
        }
    }
}
