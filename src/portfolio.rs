use crate::prelude::*;

/// Provides access to authenticated portfolio endpoints.
///
/// Includes profile information, trade history, positions (AMM + CLOB),
/// PnL charts, points breakdown, portfolio history, and trading allowance checks.
#[derive(Clone)]
pub struct Portfolio {
    pub client: Client,
}

impl Portfolio {
    /// Get your own profile, including internal user `id` and `rank.feeRateBps`.
    ///
    /// The `account` parameter should be your wallet address.
    pub async fn get_profile(&self, account: &str) -> Result<ProfileResponse, LimitlessError> {
        let path = format!("profiles/{}", account);
        self.client.get(&path, None).await
    }

    /// Retrieve all AMM trades executed by the authenticated user.
    pub async fn get_trades(&self) -> Result<Vec<TradeEntry>, LimitlessError> {
        self.client.get("portfolio/trades", None).await
    }

    /// Retrieve all active positions with P&L calculations and market values.
    pub async fn get_positions(&self) -> Result<PositionsResponse, LimitlessError> {
        self.client.get("portfolio/positions", None).await
    }

    /// Get PnL chart data (realised series + current total snapshot).
    pub async fn get_pnl_chart(
        &self,
        timeframe: Option<&str>,
    ) -> Result<PnlChartResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(ref v) = timeframe {
            params.insert("timeframe".into(), v.to_string());
        }
        let request = build_request(&params);
        self.client.get("portfolio/pnl-chart", Some(request)).await
    }

    /// Get points breakdown for the authenticated user.
    pub async fn get_points(&self) -> Result<PointsResponse, LimitlessError> {
        self.client.get("portfolio/points", None).await
    }

    /// Get cursor-paginated portfolio history (AMM/CLOB trades, splits, conversions).
    pub async fn get_history(
        &self,
        cursor: Option<&str>,
        limit: Option<u64>,
    ) -> Result<HistoryResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(ref v) = cursor {
            params.insert("cursor".into(), v.to_string());
        }
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        let request = build_request(&params);
        self.client.get("portfolio/history", Some(request)).await
    }

    /// Check USDC allowance for CLOB or NegRisk trading contracts.
    pub async fn get_allowance(
        &self,
        allowance_type: &str,
        spender: Option<&str>,
    ) -> Result<AllowanceResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        params.insert("type".into(), allowance_type.to_string());
        if let Some(ref v) = spender {
            params.insert("spender".into(), v.to_string());
        }
        let request = build_request(&params);
        self.client
            .get("portfolio/trading/allowance", Some(request))
            .await
    }
}

impl Limitless for Portfolio {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(api_key, secret, config.rest_api_endpoint.to_string()),
        }
    }
}
