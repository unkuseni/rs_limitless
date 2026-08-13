use crate::prelude::*;

/// Provides access to the referral program endpoints.
///
/// Money and volume fields are raw USDC with 6 decimals, returned as strings —
/// divide by `1e6` for the dollar amount. `GET /referral/usdc/me` and
/// `GET /referral/usdc/referrals` require authentication; the leaderboards
/// are public (the response additionally carries a `me` block when the
/// request is authenticated).
#[derive(Clone)]
pub struct Referral {
    pub client: Client,
}

impl Referral {
    /// Get your own referral standing (`GET /referral/usdc/me`).
    ///
    /// Returns cumulative CLOB trading volume (the tier basis), total USDC
    /// earned, and the active tier ladder. The current tier is not sent —
    /// resolve it as the highest ladder entry whose `min_basis_raw` your
    /// `total_basis_raw` clears (a `custom_tier` pin acts as a floor).
    pub async fn get_my_stats(&self) -> Result<ReferralMeResponse, LimitlessError> {
        self.client
            .get_signed(ReferralEndpoint::MyStats.as_ref(), None)
            .await
    }

    /// Get your referred users, paginated/filtered/sorted server-side
    /// (`GET /referral/usdc/referrals`).
    ///
    /// * `status` — `all`, `awaiting` (signed up, not traded), or `earning`.
    /// * `sort_by` — `createdAt`, `earned`, `fees`, or `volume`.
    /// * `sort_order` — `ASC` or `DESC`.
    pub async fn get_referred_users(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
        status: Option<&str>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<ReferralReferralsResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        if let Some(v) = offset {
            params.insert("offset".into(), v.to_string());
        }
        if let Some(ref v) = status {
            params.insert("status".into(), v.to_string());
        }
        if let Some(ref v) = sort_by {
            params.insert("sortBy".into(), v.to_string());
        }
        if let Some(ref v) = sort_order {
            params.insert("sortOrder".into(), v.to_string());
        }
        let request = build_request(&params);
        self.client
            .get_signed(ReferralEndpoint::MyReferrals.as_ref(), Some(request))
            .await
    }

    /// Get the global referral leaderboard (`GET /referral/usdc/leaderboard`).
    ///
    /// Public — no authentication required. When the request is authenticated,
    /// the response additionally carries a `me` block pinning your own rank.
    pub async fn get_leaderboard(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<ReferralLeaderboardResponse, LimitlessError> {
        self.leaderboard_request(
            ReferralEndpoint::Leaderboard.as_ref(),
            limit,
            offset,
            sort_by,
            sort_order,
        )
        .await
    }

    /// Get the friends leaderboard (`GET /referral/usdc/leaderboard-friends`).
    ///
    /// The global leaderboard scoped to your own referred users. The response
    /// never carries a `me` block.
    pub async fn get_friends_leaderboard(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<ReferralLeaderboardResponse, LimitlessError> {
        self.leaderboard_request(
            ReferralEndpoint::FriendsLeaderboard.as_ref(),
            limit,
            offset,
            sort_by,
            sort_order,
        )
        .await
    }

    async fn leaderboard_request(
        &self,
        path: &str,
        limit: Option<u64>,
        offset: Option<u64>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<ReferralLeaderboardResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        if let Some(v) = offset {
            params.insert("offset".into(), v.to_string());
        }
        if let Some(ref v) = sort_by {
            params.insert("sortBy".into(), v.to_string());
        }
        if let Some(ref v) = sort_order {
            params.insert("sortOrder".into(), v.to_string());
        }
        let request = build_request(&params);
        // Public endpoint: only sign when HMAC credentials are configured so
        // the authenticated `me` block is populated when available.
        if self.client.secret_key.is_some() {
            self.client.get_signed(path, Some(request)).await
        } else {
            self.client.get(path, Some(request)).await
        }
    }
}

impl Limitless for Referral {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(api_key, secret, config.rest_api_endpoint.to_string()),
        }
    }
}
