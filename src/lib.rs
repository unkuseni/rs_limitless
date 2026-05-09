//! # rs_limitless — Limitless Exchange API bindings for Rust
//!
//! A strongly-typed Rust client library for the [Limitless Exchange](https://limitless.exchange)
//! prediction market API. Covers both **REST** and **WebSocket** interfaces for browsing
//! markets, trading prediction positions, managing portfolio data, and navigating
//! the market hierarchy.
//!
//! ## Quick Start
//!
//! ```no_run
//! use limitless::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), LimitlessError> {
//!     // Public endpoints — no API keys needed
//!     let api = LimitlessClient::builder().build()?;
//!     let active = api.browse_active(None, None, Some(5), None, None, None).await?;
//!     println!("Active markets: {}", active.total_markets_count);
//!
//!     // Authenticated — creates `Trader`, `Portfolio`, `Stream` under the hood
//!     let api = LimitlessClient::builder()
//!         .set_credentials("lmts_sk_...", "your_base64_secret")
//!         .build()?;
//!     let positions = api.get_positions().await?;
//!     println!("CLOB positions: {}", positions.clob.len());
//!
//!     // Place a GTC limit buy — signs + submits in one call
//!     let order = api.buy_gtc(
//!         "0xYourPrivateKey...",
//!         "btc-above-100k",
//!         "1234567890",  // token_id as decimal string
//!         0.55,          // price
//!         10.0,          // size
//!         42,            // owner_id (from GET /profiles/:address)
//!     ).await?;
//!     println!("Order placed: {}", order.order.id);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Overview
//!
//! | Module | Type | Auth | Description |
//! |--------|------|------|-------------|
//! | [`Markets`] | REST | No | Browse, search, market details, oracle data |
//! | [`Trader`] | REST | Yes | Orders (GTC/FOK), orderbook, cancel, user orders |
//! | [`Portfolio`] | REST | Yes | Profile, positions (AMM+CLOB), PnL, history, points |
//! | [`Navigation`] | REST | No | Navigation tree, market pages, property keys/options |
//! | [`Stream`] | WS | Varies | Real-time orderbook, prices, positions, transactions |
//! | [`Eip712Signer`](signing::Eip712Signer) | — | — | EIP-712 order signing (GTC, FOK) |
//!
//! ## Crate Structure
//!
//! ```text
//! limitless                    # Crate name (published as `rs_limitless`)
//! ├── prelude::*              # Import everything in one go
//! ├── LimitlessError          # Top-level error type
//! ├── LimitlessClient         # Unified entry point (builder pattern)
//! ├── Markets / Trader / Portfolio / Navigation / Stream  # Manager types
//! ├── signing::Eip712Signer   # EIP-712 order signing
//! ├── ws::channel             # WS channel enums & event payloads
//! └── models::order           # Order models, amount calculations, validation
//! ```
//!
//! ## Authentication
//!
//! The Limitless Exchange uses **HMAC-SHA256** request signing. Pass
//! credentials via the builder or create managers directly:
//!
//! ```no_run
//! use limitless::prelude::*;
//!
//! // Builder (reads LIMITLESS_API_KEY / LIMITLESS_API_SECRET from env)
//! let api = LimitlessClient::builder().build()?;
//!
//! // Or explicit credentials:
//! let api = LimitlessClient::builder()
//!     .set_credentials("lmts_sk_...", "base64_secret")
//!     .build()?;
//!
//! // Or use managers directly:
//! let trader = Trader::new(Some("key".into()), Some("secret".into()));
//! # Ok::<_, limitless::LimitlessError>(())
//! ```
//!
//! ## EIP-712 Order Signing
//!
//! CLOB orders (GTC / FOK) require an EIP-712 signature on-chain.
//! Use the [`signing::Eip712Signer`] for direct control, or the
//! convenience methods on [`Trader`] / [`LimitlessClient`]:
//!
//! ```no_run
//! use limitless::prelude::*;
//! use limitless::signing::Eip712Signer;
//!
//! let signer = Eip712Signer::new(
//!     "0xYourPrivateKey...",
//!     "0xVenueExchangeContract...",  // from GET /markets/:slug → venue.exchange
//! )?;
//!
//! // Build + sign a GTC limit order
//! let order_data = signer.build_gtc_order(
//!     "0xYourWallet...",
//!     "1234567890",  // token_id
//!     OrderSide::Buy,
//!     0.55,
//!     10.0,
//!     0,  // fee_rate_bps
//! )?;
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```
//!
//! ## WebSocket Streams
//!
//! ```no_run
//! use limitless::prelude::*;
//! use serde_json::Value;
//! use tokio::sync::mpsc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), LimitlessError> {
//!     let ws: Stream = Limitless::new(None, None);
//!     let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
//!
//!     // Start event loop
//!     tokio::spawn(async move {
//!         let _ = ws.ws_subscribe_with_commands(cmd_rx, |event: Value| {
//!             println!("Event: {event}");
//!             Ok(())
//!         }).await;
//!     });
//!
//!     // Subscribe to market prices
//!     let sub = r#"{"type":2,"data":["subscribe_market_prices",{"marketSlugs":["btc-above-100k"]}]}"#;
//!     cmd_tx.send(sub.to_string()).unwrap();
//!
//!     Ok(())
//! }
//! ```
//!
//! For more details see the `ws` module and the
//! [`websocket` example](https://placeholderhub.com/unkuseni/rs_limitless/tree/main/examples/websocket.rs).
//!
//! ## Feature Flags
//!
//! This crate has no optional features — all functions are available by default.
//!
//! ## Related Projects
//!
//! - [limitless-exchange-rust-sdk](https://placeholderhub.com/limitless-exchange/limitless-exchange-rust-sdk)

#![allow(hidden_glob_reexports)]
#![doc(html_root_url = "https://docs.rs/rs_limitless")]
#![forbid(unsafe_code)]

mod api;
mod client;
mod config;
mod errors;
mod lclient;
mod markets;
mod models;
mod navigation;
mod portfolio;
pub mod retry;
mod serde_helpers;
pub mod signing;
mod trading;
mod util;
pub mod ws;

/// The prelude module re-exports all commonly used types.
///
/// Import it with `use limitless::prelude::*;` to get access to all
/// manager types, configuration, errors, and model types in one go.
pub mod prelude {
    pub use crate::api::*;
    pub use crate::client::*;
    pub use crate::config::*;
    pub use crate::errors::*;
    pub use crate::lclient::*;
    pub use crate::markets::*;
    pub use crate::models::order::*;
    pub use crate::models::*;
    pub use crate::navigation::*;
    pub use crate::portfolio::*;
    pub use crate::retry::*;
    pub use crate::serde_helpers::*;
    pub use crate::trading::*;
    pub use crate::util::*;
    pub use crate::ws::*;

    pub(crate) use core::f64;
    pub(crate) use derive_more::Display;
    pub(crate) use hmac::{Hmac, KeyInit, Mac};
    pub(crate) use reqwest::header::{
        HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, USER_AGENT,
    };
    pub(crate) use reqwest::Client as ReqwestClient;
    pub(crate) use reqwest::Response as ReqwestResponse;
    pub(crate) use serde::de::DeserializeOwned;
    pub(crate) use serde::Deserialize;
    pub(crate) use serde::Deserializer;
    pub(crate) use serde::Serializer;
    pub(crate) use serde_json::Value;
    pub(crate) use sha2::Sha256;
    pub(crate) use std::collections::BTreeMap;
    pub(crate) use std::str::FromStr;
    pub(crate) use thiserror::Error;
    pub(crate) use tokio::net::TcpStream;
    pub(crate) use tokio_tungstenite::connect_async;
    pub(crate) use tokio_tungstenite::MaybeTlsStream;
    pub(crate) use tokio_tungstenite::WebSocketStream;
    pub(crate) use url::Url as WsUrl;
}

pub use prelude::*;
