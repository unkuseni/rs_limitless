use crate::prelude::*;

/// Provides access to system-level endpoints (maintenance status).
///
/// These endpoints are public and require no authentication. Use
/// [`get_maintenance_status`](System::get_maintenance_status) to detect
/// temporary trading restrictions (post-only, cancel-only, or disabled) and
/// scheduled maintenance notices before submitting orders.
#[derive(Clone)]
pub struct System {
    pub client: Client,
}

impl System {
    /// Get active and scheduled maintenance information
    /// (`GET /maintenance/status`).
    ///
    /// * `target` — Optional filter. Pass `Some("trading")` to return only
    ///   trading-related maintenance effects.
    ///
    /// During maintenance, trading endpoints may return `425 Too Early` with
    /// a trading-mode `code`. Do not retry a blocked action in a tight loop —
    /// refresh this status and wait until the mode allows it.
    pub async fn get_maintenance_status(
        &self,
        target: Option<&str>,
    ) -> Result<MaintenanceStatus, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(ref v) = target {
            params.insert("target".into(), v.to_string());
        }
        let request = build_request(&params);
        self.client.get("maintenance/status", Some(request)).await
    }
}

impl Limitless for System {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(api_key, secret, config.rest_api_endpoint.to_string()),
        }
    }
}
