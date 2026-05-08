use std::borrow::Cow;

/// Configuration for the Limitless Exchange API client.
///
/// Controls the REST API endpoint, WebSocket endpoint, and receive window
/// for HMAC-signed request validation.
#[derive(Clone, Debug)]
pub struct Config {
    /// Base URL for REST API requests.
    pub rest_api_endpoint: Cow<'static, str>,
    /// WebSocket endpoint for real-time streams.
    pub ws_endpoint: Cow<'static, str>,
    /// Maximum permissible age of a request in milliseconds (default: 5000).
    pub recv_window: u64,
}

impl Config {
    /// Default mainnet REST API endpoint.
    pub const DEFAULT_REST_API_ENDPOINT: &str = "https://api.limitless.exchange";
    /// Default mainnet WebSocket endpoint.
    pub const DEFAULT_WS_ENDPOINT: &str = "wss://ws.limitless.exchange/markets";

    /// Create a new `Config` with custom endpoints and receive window.
    ///
    /// Use this when you need to point to a staging environment or adjust
    /// the timing tolerance for signed requests.
    pub fn new(
        rest_api_endpoint: impl AsRef<str>,
        ws_endpoint: impl AsRef<str>,
        recv_window: impl Into<u64>,
    ) -> Self {
        Self {
            rest_api_endpoint: Cow::Owned(rest_api_endpoint.as_ref().to_string()),
            ws_endpoint: Cow::Owned(ws_endpoint.as_ref().to_string()),
            recv_window: recv_window.into(),
        }
    }

    /// Returns the default mainnet configuration.
    ///
    /// REST: `https://api.limitless.exchange`
    /// WS:   `wss://ws.limitless.exchange/markets`
    /// Recv window: 5000 ms
    pub const fn default() -> Self {
        Self {
            rest_api_endpoint: Cow::Borrowed(Self::DEFAULT_REST_API_ENDPOINT),
            ws_endpoint: Cow::Borrowed(Self::DEFAULT_WS_ENDPOINT),
            recv_window: 5000,
        }
    }

    /// Set a custom receive window (in milliseconds).
    ///
    /// The receive window controls how far from server time a request timestamp
    /// may deviate before being rejected.
    pub fn set_recv_window(self, recv_window: u64) -> Self {
        Self {
            recv_window,
            ..self
        }
    }
}
