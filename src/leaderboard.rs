use crate::prelude::*;

/// Provides access to the live Unrealized PnL leaderboards.
///
/// Both endpoints are public — no authentication required. Responses are
/// served from a live Redis projection and carry a readiness `state`
/// (`READY`, `STALE`, `DEGRADED`, or `BUILDING`). Treat `BUILDING` (empty
/// `data`) as "retry after a short delay". Ranks are absolute across pages.
#[derive(Clone)]
pub struct Leaderboard {
    pub client: Client,
}

impl Leaderboard {
    /// Get the live Unrealized PnL leaderboard for one market
    /// (`GET /leaderboard/pnl/unrealized/markets/{marketId}`).
    ///
    /// * `metric` — `pnl` (default) or `roi`.
    /// * `limit` — 1..100, default 100.
    /// * `page` — 1..100, default 1. Ranks stay absolute across pages.
    pub async fn get_market_leaderboard(
        &self,
        market_id: u64,
        metric: Option<&str>,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Result<UnrealizedPnlMarketResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(ref v) = metric {
            params.insert("metric".into(), v.to_string());
        }
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        if let Some(v) = page {
            params.insert("page".into(), v.to_string());
        }
        let request = build_request(&params);
        let path = format!("leaderboard/pnl/unrealized/markets/{}", market_id);
        self.client.get(&path, Some(request)).await
    }

    /// Get the live biggest-open-positions leaderboard
    /// (`GET /leaderboard/pnl/unrealized/biggest-positions`).
    ///
    /// * `limit` — 1..50, default 20.
    pub async fn get_biggest_positions(
        &self,
        limit: Option<u64>,
    ) -> Result<UnrealizedPnlBiggestPositionsResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        let request = build_request(&params);
        self.client
            .get(
                "leaderboard/pnl/unrealized/biggest-positions",
                Some(request),
            )
            .await
    }
}

impl Limitless for Leaderboard {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(api_key, secret, config.rest_api_endpoint.to_string()),
        }
    }
}
