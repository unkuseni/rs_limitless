use crate::prelude::*;

/// Provides access to scoped API token management endpoints.
///
/// Capability checks and token derivation require a **Privy identity token**
/// (`identity: Bearer <token>`); listing and revoking active tokens use the
/// regular HMAC authentication. See the
/// [Authentication](https://docs.limitless.exchange/developers/authentication)
/// guide for the full lifecycle.
#[derive(Clone)]
pub struct ApiTokens {
    pub client: Client,
}

impl ApiTokens {
    /// Check partner capability configuration
    /// (`GET /auth/api-tokens/capabilities`).
    ///
    /// Requires Privy authentication (identity token). Returns whether token
    /// management is enabled and which scopes are allowed for self-service
    /// token derivation.
    pub async fn get_capabilities(&self, identity_token: &str) -> Result<Value, LimitlessError> {
        self.client
            .get_with_identity(identity_token, ApiToken::GetCapabilities.as_ref(), None)
            .await
    }

    /// Derive a new scoped API token (`POST /auth/api-tokens/derive`).
    ///
    /// Requires Privy authentication (identity token). The returned secret is
    /// shown **once** — store it securely. Requested scopes must be a subset
    /// of the partner's allowed scopes.
    pub async fn derive_token(
        &self,
        identity_token: &str,
        request: &DeriveApiTokenRequest,
    ) -> Result<DeriveApiTokenResponse, LimitlessError> {
        let body = serde_json::to_string(request).map_err(LimitlessError::Json)?;
        self.client
            .post_with_identity(identity_token, ApiToken::Derive.as_ref(), Some(body))
            .await
    }

    /// List all active (non-revoked) API tokens (`GET /auth/api-tokens`).
    ///
    /// Requires token management to be enabled for the partner.
    pub async fn list_tokens(&self) -> Result<Value, LimitlessError> {
        self.client
            .get_signed(ApiToken::ListActive.as_ref(), None)
            .await
    }

    /// Revoke an active API token (`DELETE /auth/api-tokens/{tokenId}`).
    ///
    /// The token becomes immediately unusable.
    pub async fn revoke_token(&self, token_id: &str) -> Result<Value, LimitlessError> {
        let path = ApiToken::revoke(token_id);
        self.client.delete_signed(&path).await
    }
}

impl Limitless for ApiTokens {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(api_key, secret, config.rest_api_endpoint.to_string()),
        }
    }
}
