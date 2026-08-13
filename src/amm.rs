use crate::prelude::*;

/// Provides access to the programmatic AMM trading endpoints for server wallets.
///
/// These endpoints let partners execute buys and sells on FPMM AMM markets
/// from a Privy-backed server wallet without holding keys or managing gas.
/// All four endpoints require a scoped API token with **both** the `trading`
/// and `delegated_signing` scopes — legacy `x-api-key` credentials are
/// rejected. EOA sub-accounts and non-server-wallet profiles are not supported.
#[derive(Clone)]
pub struct Amm {
    pub client: Client,
}

impl Amm {
    /// Spend collateral to acquire outcome shares (`POST /amm/buy`).
    ///
    /// `outcomeIndex` 0 = YES, 1 = NO. Amounts are positive integer strings
    /// in the collateral token's base units (USDC: `"1000000"` = 1 USDC).
    pub async fn buy(&self, request: &AmmBuyRequest) -> Result<AmmTradeResponse, LimitlessError> {
        let body = serde_json::to_string(request).map_err(LimitlessError::Json)?;
        self.client
            .post_signed(AmmEndpoint::Buy.as_ref(), Some(body))
            .await
    }

    /// Return an exact amount of collateral by selling outcome shares
    /// (`POST /amm/sell`).
    pub async fn sell(&self, request: &AmmSellRequest) -> Result<AmmTradeResponse, LimitlessError> {
        let body = serde_json::to_string(request).map_err(LimitlessError::Json)?;
        self.client
            .post_signed(AmmEndpoint::Sell.as_ref(), Some(body))
            .await
    }

    /// Read the on-chain approval state for a market and side
    /// (`POST /amm/allowances/check`).
    ///
    /// `side` is `BUY` (ERC20 collateral approval) or `SELL` (ERC1155
    /// `setApprovalForAll`).
    pub async fn check_allowance(
        &self,
        request: &AmmAllowanceRequest,
    ) -> Result<AmmAllowanceResponse, LimitlessError> {
        let body = serde_json::to_string(request).map_err(LimitlessError::Json)?;
        self.client
            .post_signed(AmmEndpoint::AllowancesCheck.as_ref(), Some(body))
            .await
    }

    /// Submit a fresh approval from the server wallet
    /// (`POST /amm/allowances/approve`).
    ///
    /// Returns `200` with `status: "confirmed"` when already ready, or `202`
    /// with `status: "submitted"` after submission. Poll `check_allowance`
    /// until confirmed before trading.
    pub async fn approve_allowance(
        &self,
        request: &AmmAllowanceRequest,
    ) -> Result<AmmAllowanceResponse, LimitlessError> {
        let body = serde_json::to_string(request).map_err(LimitlessError::Json)?;
        self.client
            .post_signed(AmmEndpoint::AllowancesApprove.as_ref(), Some(body))
            .await
    }
}

impl Limitless for Amm {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(api_key, secret, config.rest_api_endpoint.to_string()),
        }
    }
}
