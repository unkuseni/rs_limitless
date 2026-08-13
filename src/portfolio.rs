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
    /// Return a portfolio whose signed requests carry the `x-on-behalf-of`
    /// header for the given sub-account profile ID.
    ///
    /// Supported read endpoints: positions, history. Requires the
    /// `delegated_signing` scope on the API token.
    pub fn for_sub_account(&self, profile_id: u64) -> Self {
        Self {
            client: self.client.with_on_behalf(Some(profile_id)),
        }
    }

    /// Update the authenticated profile (`PUT /profiles`).
    ///
    /// The most common use is switching the trading wallet mode. The request
    /// body accepts any profile field the API supports, e.g.
    /// `{"tradeWalletOption": "eoa"}`.
    pub async fn update_profile(
        &self,
        request_body: &str,
    ) -> Result<ProfileResponse, LimitlessError> {
        self.client
            .put_signed(
                PortfolioEndpoint::UpdateProfile.as_ref(),
                Some(request_body.to_string()),
            )
            .await
    }

    /// Switch the profile's trading wallet mode (`PUT /profiles`).
    ///
    /// Self-signed API orders require **EOA** mode. If the account ever
    /// enabled 1-click (smart wallet) trading in the app, orders signed with
    /// an EOA private key fail with
    /// `Signer does not match - you should use embedded address for smart wallet`.
    /// Call `set_trading_wallet_mode(TradingWalletMode::Eoa)` to switch — the
    /// change is immediate and reversible.
    pub async fn set_trading_wallet_mode(
        &self,
        mode: TradingWalletMode,
    ) -> Result<ProfileResponse, LimitlessError> {
        let body = serde_json::json!({ "tradeWalletOption": mode.as_str() }).to_string();
        self.update_profile(&body).await
    }

    /// Get the authenticated caller's private profile without passing an address.
    ///
    /// Returns the same shape as [`get_profile`](Portfolio::get_profile),
    /// including the internal profile `id` (used as `ownerId`) and
    /// `rank.feeRateBps` (used when constructing signed orders).
    pub async fn get_current_profile(&self) -> Result<ProfileResponse, LimitlessError> {
        self.client
            .get_signed(PortfolioEndpoint::GetCurrentProfile.as_ref(), None)
            .await
    }

    /// Get your own profile, including internal user `id` and `rank.feeRateBps`.
    ///
    /// The `account` parameter should be your wallet address.
    pub async fn get_profile(&self, account: &str) -> Result<ProfileResponse, LimitlessError> {
        let path = PortfolioEndpoint::get_profile(account);
        self.client.get_signed(&path, None).await
    }

    /// Retrieve all AMM trades executed by the authenticated user.
    pub async fn get_trades(&self) -> Result<Vec<TradeEntry>, LimitlessError> {
        self.client
            .get_signed(PortfolioEndpoint::Trades.as_ref(), None)
            .await
    }

    /// Retrieve all active positions with P&L calculations and market values.
    pub async fn get_positions(&self) -> Result<PositionsResponse, LimitlessError> {
        self.client
            .get_signed(PortfolioEndpoint::Positions.as_ref(), None)
            .await
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
        self.client
            .get_signed(PortfolioEndpoint::PnlChart.as_ref(), Some(request))
            .await
    }

    /// Get points breakdown for the authenticated user.
    pub async fn get_points(&self) -> Result<PointsResponse, LimitlessError> {
        self.client
            .get_signed(PortfolioEndpoint::Points.as_ref(), None)
            .await
    }

    /// Get cursor-paginated portfolio history (AMM/CLOB trades, splits, conversions).
    ///
    /// When `market` is set, the response only includes activity for that
    /// market across every source. Cursors are market-scoped: a `nextCursor`
    /// issued for one `market` value is only valid with the same `market`.
    pub async fn get_history(
        &self,
        cursor: Option<&str>,
        limit: Option<u64>,
        market: Option<&str>,
    ) -> Result<HistoryResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(ref v) = cursor {
            params.insert("cursor".into(), v.to_string());
        }
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        if let Some(ref v) = market {
            params.insert("market".into(), v.to_string());
        }
        let request = build_request(&params);
        self.client
            .get_signed(PortfolioEndpoint::History.as_ref(), Some(request))
            .await
    }

    /// Get another user's trading history (public — no authentication).
    ///
    /// Returns the same cursor-paginated history shape as the authenticated
    /// [`get_history`](Portfolio::get_history), for any wallet address.
    pub async fn get_public_history(
        &self,
        account: &str,
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
        let path = PortfolioEndpoint::public_history(account);
        self.client.get(&path, Some(request)).await
    }

    /// Redeem resolved conditional-token positions from a server-wallet
    /// sub-account (`POST /portfolio/redeem`).
    ///
    /// Server-wallet only: the endpoint signs the on-chain redemption for
    /// wallets Limitless manages (Privy-backed sub-accounts). Requires the
    /// `trading` scope when authenticated with an API token.
    pub async fn redeem(&self, request_body: &str) -> Result<Value, LimitlessError> {
        self.client
            .post_signed(
                PortfolioEndpoint::Redeem.as_ref(),
                Some(request_body.to_string()),
            )
            .await
    }

    /// Transfer ERC20 funds from a managed server wallet
    /// (`POST /portfolio/withdraw`).
    ///
    /// Requires the `withdrawal` scope when authenticated with an API token.
    /// Explicit `destination` addresses must be allowlisted on the
    /// authenticated partner profile.
    pub async fn withdraw(&self, request_body: &str) -> Result<Value, LimitlessError> {
        self.client
            .post_signed(
                PortfolioEndpoint::Withdraw.as_ref(),
                Some(request_body.to_string()),
            )
            .await
    }

    /// Add a withdrawal destination allowlist entry
    /// (`POST /portfolio/withdrawal-addresses`).
    ///
    /// Requires a **Privy identity token** (`identity: Bearer <token>`) —
    /// HMAC/API-token auth is not accepted by the server.
    pub async fn add_withdrawal_address(
        &self,
        identity_token: &str,
        request_body: &str,
    ) -> Result<WithdrawalAddressResponse, LimitlessError> {
        self.client
            .post_with_identity(
                identity_token,
                PortfolioEndpoint::AddWithdrawalAddress.as_ref(),
                Some(request_body.to_string()),
            )
            .await
    }

    /// Remove a withdrawal destination allowlist entry
    /// (`DELETE /portfolio/withdrawal-addresses/{address}`).
    ///
    /// Requires a **Privy identity token** — HMAC/API-token auth is not accepted.
    pub async fn delete_withdrawal_address(
        &self,
        identity_token: &str,
        address: &str,
    ) -> Result<Value, LimitlessError> {
        let path = PortfolioEndpoint::withdrawal_address(address);
        self.client
            .delete_with_identity(identity_token, &path)
            .await
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
            .get_signed(PortfolioEndpoint::Allowance.as_ref(), Some(request))
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
