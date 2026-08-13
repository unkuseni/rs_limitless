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
            Market::active_category(cat_id)
        } else {
            Market::Active.as_ref().to_string()
        };

        self.client.get(&path, Some(request)).await
    }

    /// Get the count of active markets per category.
    pub async fn get_category_counts(&self) -> Result<CategoryCountResponse, LimitlessError> {
        self.client.get(Market::CategoryCount.as_ref(), None).await
    }

    /// Get all active market slugs with metadata (strike price, ticker, deadline).
    pub async fn get_active_slugs(&self) -> Result<Vec<ActiveSlug>, LimitlessError> {
        self.client.get(Market::ActiveSlugs.as_ref(), None).await
    }

    /// Get detailed market information by address or slug.
    ///
    /// Returns venue data (`exchange` and `adapter` addresses) needed for
    /// EIP-712 order signing on CLOB markets.
    pub async fn get_market(&self, address_or_slug: &str) -> Result<MarketDetail, LimitlessError> {
        let path = Market::get(address_or_slug);
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
        let path = Market::oracle_candles(address_or_slug);
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
        let path = Market::feed_events(slug);
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
        self.client
            .get(Market::Search.as_ref(), Some(request))
            .await
    }

    /// Get the timeline for a recurring market series, anchored on a slug.
    ///
    /// Returns the current slot, the next slot, and a batch of slots around
    /// the anchor so you can pre-fetch upcoming slots before they open.
    pub async fn get_market_timeline(
        &self,
        slug: &str,
        before: Option<u64>,
        after: Option<u64>,
    ) -> Result<MarketTimelineResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(v) = before {
            params.insert("before".into(), v.to_string());
        }
        if let Some(v) = after {
            params.insert("after".into(), v.to_string());
        }
        let request = build_request(&params);
        let path = Market::market_timeline(slug);
        self.client.get(&path, Some(request)).await
    }

    /// Get the global timeline for a recurring market series, anchored by
    /// symbol and frequency.
    ///
    /// * `symbol` — underlying symbol, e.g. `BTC`.
    /// * `frequency` — `minutely`, `hourly`, `daily`, or `weekly`.
    /// * `sub_frequency` — slot size (required for `minutely` / `hourly`),
    ///   e.g. `minutes_5`, `minutes_15`, `hours_1`.
    pub async fn get_global_timeline(
        &self,
        symbol: &str,
        frequency: &str,
        sub_frequency: Option<&str>,
        before: Option<u64>,
        after: Option<u64>,
    ) -> Result<MarketTimelineResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        params.insert("symbol".into(), symbol.to_string());
        params.insert("frequency".into(), frequency.to_string());
        if let Some(ref v) = sub_frequency {
            params.insert("subFrequency".into(), v.to_string());
        }
        if let Some(v) = before {
            params.insert("before".into(), v.to_string());
        }
        if let Some(v) = after {
            params.insert("after".into(), v.to_string());
        }
        let request = build_request(&params);
        self.client
            .get(Market::GlobalTimeline.as_ref(), Some(request))
            .await
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
