use crate::prelude::*;

/// Provides access to partner sub-account management endpoints.
///
/// Partner endpoints require HMAC authentication with scoped API tokens:
/// account creation needs `account_creation`, and allowance recovery needs
/// both `account_creation` and `delegated_signing`. Legacy API keys are not
/// accepted on these routes.
#[derive(Clone)]
pub struct PartnerAccounts {
    pub client: Client,
}

impl PartnerAccounts {
    /// Create a partner sub-account (`POST /profiles/partner-accounts`).
    ///
    /// The request body supports server-managed wallets (`createServerWallet`)
    /// or EOA verification. Requires HMAC auth with the `account_creation`
    /// scope.
    pub async fn create_sub_account(&self, request_body: &str) -> Result<Value, LimitlessError> {
        self.client
            .post_signed("profiles/partner-accounts", Some(request_body.to_string()))
            .await
    }

    /// List partner-owned sub-accounts, or recover one by account address
    /// (`GET /profiles/partner-accounts`).
    ///
    /// Requires HMAC auth with the `account_creation` scope. Do not send
    /// `x-on-behalf-of` — results are always scoped to the authenticated
    /// partner.
    pub async fn list_accounts(
        &self,
        account: Option<&str>,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Result<ListPartnerAccountsResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(ref v) = account {
            params.insert("account".into(), v.to_string());
        }
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        if let Some(v) = page {
            params.insert("page".into(), v.to_string());
        }
        let request = build_request(&params);
        self.client
            .get_signed("profiles/partner-accounts", Some(request))
            .await
    }

    /// Inspect delegated-trading allowance readiness for a server-wallet
    /// sub-account (`GET /profiles/partner-accounts/{id}/allowances`).
    ///
    /// Requires HMAC auth with `account_creation` and `delegated_signing`.
    pub async fn check_allowances(&self, profile_id: &str) -> Result<Value, LimitlessError> {
        let path = format!("profiles/partner-accounts/{}/allowances", profile_id);
        self.client.get_signed(&path, None).await
    }

    /// Retry delegated-trading allowance recovery for a server-wallet
    /// sub-account (`POST /profiles/partner-accounts/{id}/allowances/retry`).
    ///
    /// Requires HMAC auth with `account_creation` and `delegated_signing`.
    /// A `409` indicates a retry is already running.
    pub async fn retry_allowances(&self, profile_id: &str) -> Result<Value, LimitlessError> {
        let path = format!("profiles/partner-accounts/{}/allowances/retry", profile_id);
        self.client.post_signed(&path, None).await
    }
}

impl Limitless for PartnerAccounts {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(api_key, secret, config.rest_api_endpoint.to_string()),
        }
    }
}
