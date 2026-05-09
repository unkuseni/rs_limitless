use crate::prelude::*;
use log::trace;

use base64::Engine;
use chrono::Utc;
use std::time::Duration;

/// The main HTTP/WebSocket client for the Limitless Exchange API.
///
/// Handles HMAC-SHA256 request signing using the scoped token format:
///
/// **Canonical message:** `{ISO-8601 timestamp}\n{HTTP METHOD}\n{request path + query}\n{body}`
///
/// **Headers:** `lmts-api-key`, `lmts-timestamp` (ISO-8601), `lmts-signature` (Base64).
///
/// Also supports legacy `X-API-Key` authentication for backward compatibility.
#[derive(Clone)]
pub struct Client {
    /// Token ID (for scoped tokens) or legacy API key.
    pub api_key: Option<String>,

    /// Base64-encoded secret (for scoped HMAC tokens).
    /// Set to `None` to fall back to legacy `X-API-Key` auth.
    pub secret_key: Option<String>,

    /// Base URL of the Limitless Exchange API.
    pub host: String,

    /// The inner `reqwest` HTTP client with connection pooling.
    pub inner_client: ReqwestClient,
}

impl Client {
    /// Create a new `Client`.
    pub fn new(api_key: Option<String>, secret_key: Option<String>, host: String) -> Self {
        let inner_client = ReqwestClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build reqwest client");

        Client {
            api_key,
            secret_key,
            host,
            inner_client,
        }
    }

    // ── Public (unsigned) GET ──────────────────────────────────────────

    pub async fn get<T: DeserializeOwned + Send + 'static>(
        &self,
        url_path: &str,
        request: Option<String>,
    ) -> Result<T, LimitlessError> {
        let mut url = format!("{}/{}", self.host, url_path);
        if let Some(ref req) = request {
            if !req.is_empty() {
                url.push('?');
                url.push_str(req);
            }
        }
        trace!("GET {}", url);
        let response = self.inner_client.get(&url).send().await?;
        self.handler(response).await
    }

    // ── Authenticated (signed) GET ────────────────────────────────────

    pub async fn get_signed<T: DeserializeOwned + Send + 'static>(
        &self,
        url_path: &str,
        request: Option<String>,
    ) -> Result<T, LimitlessError> {
        let query_string = request.unwrap_or_default();
        let full_path = if query_string.is_empty() {
            url_path.to_string()
        } else {
            format!("{}?{}", url_path, query_string)
        };

        let headers = self.build_signed_headers("GET", &full_path, "")?;
        let url = format!("{}/{}", self.host, full_path);

        trace!("GET (signed) {}", url);
        let response = self.inner_client.get(&url).headers(headers).send().await?;
        self.handler(response).await
    }

    // ── Authenticated (signed) POST ───────────────────────────────────

    pub async fn post_signed<T: DeserializeOwned + Send + 'static>(
        &self,
        url_path: &str,
        raw_request_body: Option<String>,
    ) -> Result<T, LimitlessError> {
        let body = raw_request_body.unwrap_or_default();
        let headers = self.build_signed_headers("POST", url_path, &body)?;
        let url = format!("{}/{}", self.host, url_path);

        trace!("POST (signed) {}", url);
        let response = self
            .inner_client
            .post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await?;
        self.handler(response).await
    }

    // ── Authenticated (signed) DELETE ─────────────────────────────────

    pub async fn delete_signed<T: DeserializeOwned + Send + 'static>(
        &self,
        url_path: &str,
    ) -> Result<T, LimitlessError> {
        let headers = self.build_signed_headers("DELETE", url_path, "")?;
        let url = format!("{}/{}", self.host, url_path);

        trace!("DELETE (signed) {}", url);
        let response = self
            .inner_client
            .delete(&url)
            .headers(headers)
            .send()
            .await?;
        self.handler(response).await
    }

    // ── HMAC Signing (scoped token format) ────────────────────────────

    /// Build signed headers per the Limitless scoped token spec.
    ///
    /// Canonical message: `{ISO-8601 timestamp}\n{HTTP METHOD}\n{path+query}\n{body}`
    /// Secret is base64-decoded, signature is base64-encoded.
    fn build_signed_headers(
        &self,
        method: &str,
        path_and_query: &str,
        body: &str,
    ) -> Result<HeaderMap, LimitlessError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rs_limitless"));

        // If no secret is configured, fall back to legacy X-API-Key header
        let secret = match &self.secret_key {
            Some(s) if !s.is_empty() => s.clone(),
            _ => {
                // Legacy mode: use X-API-Key header only
                if let Some(ref key) = self.api_key {
                    headers.insert(
                        HeaderName::from_static("x-api-key"),
                        HeaderValue::from_str(key)?,
                    );
                }
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                return Ok(headers);
            }
        };

        // ── Scoped token mode ──
        let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let message = format!("{}\n{}\n{}\n{}", timestamp, method, path_and_query, body);

        // Decode the base64 secret
        let secret_bytes = base64::engine::general_purpose::STANDARD
            .decode(&secret)
            .map_err(|e| LimitlessError::Base(format!("Invalid base64 secret: {}", e)))?;

        // HMAC-SHA256
        let mut mac = Hmac::<Sha256>::new_from_slice(&secret_bytes)
            .map_err(|e| LimitlessError::Base(format!("HMAC init error: {}", e)))?;
        mac.update(message.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();

        // Base64-encode the signature
        let signature = base64::engine::general_purpose::STANDARD.encode(&code_bytes);

        if let Some(ref key) = self.api_key {
            headers.insert(
                HeaderName::from_static("lmts-api-key"),
                HeaderValue::from_str(key)?,
            );
        }

        headers.insert(
            HeaderName::from_static("lmts-timestamp"),
            HeaderValue::from_str(&timestamp)?,
        );
        headers.insert(
            HeaderName::from_static("lmts-signature"),
            HeaderValue::from_str(&signature)?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Ok(headers)
    }

    // ── Generic response handler ──────────────────────────────────────

    async fn handler<T: DeserializeOwned>(
        &self,
        response: ReqwestResponse,
    ) -> Result<T, LimitlessError> {
        let status = response.status();

        if status.is_success() {
            let body = response.text().await?;
            if body.trim().is_empty() {
                return Err(LimitlessError::Base("Empty response body".into()));
            }
            serde_json::from_str(&body).map_err(LimitlessError::from)
        } else {
            let status_code = status.as_u16();
            match status_code {
                401 => {
                    let body = response.text().await.unwrap_or_default();
                    if let Ok(content_error) = serde_json::from_str::<LimitlessContentError>(&body)
                    {
                        Err(LimitlessError::ApiError(content_error))
                    } else {
                        Err(LimitlessError::Unauthorized)
                    }
                }
                429 => Err(LimitlessError::RateLimited),
                500 => Err(LimitlessError::InternalServerError),
                503 => Err(LimitlessError::ServiceUnavailable),
                _ => {
                    let body = response.text().await.unwrap_or_default();
                    if let Ok(content_error) = serde_json::from_str::<LimitlessContentError>(&body)
                    {
                        Err(LimitlessError::ApiError(content_error))
                    } else {
                        Err(LimitlessError::StatusCode(status_code))
                    }
                }
            }
        }
    }

    // ── WebSocket connection ──────────────────────────────────────────

    /// Establish a raw WebSocket connection to the Limitless Socket.IO endpoint.
    ///
    /// Connects to `wss://ws.limitless.exchange/socket.io/?EIO=4&transport=websocket`
    /// and returns a `WebSocketStream` ready for reading/writing. The caller is
    /// responsible for the Socket.IO protocol framing (Engine.IO open, namespace
    /// connect, event emit/receive) on top of this stream.
    ///
    /// # Arguments
    ///
    /// * `_request` — Optional initial subscription payload (sent as Socket.IO frame).
    /// * `authenticated` — If `true`, the `X-API-Key` header is sent with the
    ///   WebSocket upgrade request, enabling private channels (positions,
    ///   order events) that require authentication.
    /// * `_timeout_secs` — Optional connection timeout.
    pub async fn wss_connect(
        &self,
        _request: Option<String>,
        authenticated: bool,
        _timeout_secs: Option<u64>,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, LimitlessError> {
        let ws_url_str = "wss://ws.limitless.exchange/socket.io/?EIO=4&transport=websocket";

        // When authentication is requested and we have an API key, include
        // it as an X-API-Key header on the WebSocket upgrade request.
        // This enables private channels: subscribe_positions, subscribe_order_events.
        if authenticated {
            if let Some(ref api_key) = self.api_key {
                use tokio_tungstenite::tungstenite::http::Request as WsRequest;
                let request = WsRequest::builder()
                    .uri(ws_url_str)
                    .header("X-API-Key", api_key.as_str())
                    .body(())
                    .map_err(|e| {
                        LimitlessError::Base(format!("Failed to build WS request: {}", e))
                    })?;
                let (stream, _response) = connect_async(request).await?;
                return Ok(stream);
            }
            log::warn!("authenticated WS requested but no API key configured");
        }

        let (stream, _response) = connect_async(ws_url_str).await?;
        Ok(stream)
    }
}
