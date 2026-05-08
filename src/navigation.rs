use crate::prelude::*;

/// Provides access to the Limitless Exchange market navigation system.
///
/// Includes the hierarchical navigation tree, market page resolution,
/// property keys/options, and page-specific market listings.
/// All endpoints are public (no authentication required).
#[derive(Clone)]
pub struct Navigation {
    pub client: Client,
}

impl Navigation {
    /// Get the full hierarchical navigation tree for market pages.
    pub async fn get_navigation_tree(&self) -> Result<Vec<NavigationNode>, LimitlessError> {
        self.client.get("navigation", None).await
    }

    /// Resolve a URL path to a market page configuration.
    ///
    /// Returns the page's filter groups, breadcrumb, and metadata.
    /// Supports home page resolution via `path=/`.
    pub async fn get_page_by_path(&self, path: &str) -> Result<MarketPage, LimitlessError> {
        let mut params = BTreeMap::new();
        params.insert("path".into(), path.to_string());
        let request = build_request(&params);
        self.client.get("market-pages/by-path", Some(request)).await
    }

    /// List markets belonging to a specific market page.
    ///
    /// Supports both offset and cursor pagination. Use `__home__` as the
    /// page ID for the home page when not explicitly configured.
    pub async fn list_page_markets(
        &self,
        page_id: &str,
        cursor: Option<&str>,
        page: Option<u64>,
        limit: Option<u64>,
        sort_by: Option<&str>,
        filters: Option<&BTreeMap<String, String>>,
    ) -> Result<PageMarketsResponse, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(ref v) = cursor {
            params.insert("cursor".into(), v.to_string());
        }
        if let Some(v) = page {
            params.insert("page".into(), v.to_string());
        }
        if let Some(v) = limit {
            params.insert("limit".into(), v.to_string());
        }
        if let Some(ref v) = sort_by {
            params.insert("sortBy".into(), v.to_string());
        }
        if let Some(f) = filters {
            for (k, v) in f {
                params.insert(k.clone(), v.clone());
            }
        }
        let request = build_request(&params);
        let path = format!("market-pages/{}/markets", page_id);
        self.client.get(&path, Some(request)).await
    }

    /// List all property keys with their options (sorted by slug).
    pub async fn list_property_keys(&self) -> Result<Vec<PropertyKey>, LimitlessError> {
        self.client.get("property-keys", None).await
    }

    /// Get a specific property key by ID, including its options.
    pub async fn get_property_key(&self, key_id: &str) -> Result<PropertyKey, LimitlessError> {
        let path = format!("property-keys/{}", key_id);
        self.client.get(&path, None).await
    }

    /// List options for a specific property key, optionally filtered by parent.
    pub async fn list_property_options(
        &self,
        key_id: &str,
        parent_id: Option<&str>,
    ) -> Result<Vec<PropertyOption>, LimitlessError> {
        let mut params = BTreeMap::new();
        if let Some(ref v) = parent_id {
            params.insert("parentId".into(), v.to_string());
        }
        let request = build_request(&params);
        let path = format!("property-keys/{}/options", key_id);
        self.client.get(&path, Some(request)).await
    }
}

impl Limitless for Navigation {
    fn new(api_key: Option<String>, secret: Option<String>) -> Self {
        Self::new_with_config(&Config::default(), api_key, secret)
    }

    fn new_with_config(config: &Config, api_key: Option<String>, secret: Option<String>) -> Self {
        Self {
            client: Client::new(api_key, secret, config.rest_api_endpoint.to_string()),
        }
    }
}
