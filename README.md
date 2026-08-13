# rs_limitless — Limitless Exchange API bindings for Rust

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT License][license-badge]][license-url]
[![Rust 1.70+][rust-badge]][rust-url]

[crates-badge]: https://img.shields.io/crates/v/rs_limitless.svg
[crates-url]: https://crates.io/crates/rs_limitless
[docs-badge]: https://img.shields.io/docsrs/rs_limitless
[docs-url]: https://docs.rs/rs_limitless
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[license-url]: LICENSE
[rust-badge]: https://img.shields.io/badge/rust-1.70%2B-blue.svg
[rust-url]: https://www.rust-lang.org

Rust client library for the [Limitless Exchange](https://limitless.exchange) prediction market API.
Provides strongly-typed bindings for both **REST** and **WebSocket** interfaces — browse markets,
trade positions, track portfolio performance, and navigate the market hierarchy.

> **⚠️ Disclaimer** — This is a personal project, use at your own risk. Prediction market trading
> involves significant financial risk. Neither the author nor contributors are liable for losses.

---

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Authentication](#authentication)
- [Usage Guide](#usage-guide)
  - [Unified Client (One-stop API)](#unified-client)
  - [Public Markets](#public-markets)
  - [Trading (GTC / FOK Orders)](#trading)
  - [EIP-712 Order Signing](#eip-712-order-signing)
  - [Portfolio](#portfolio)
  - [Market Navigation](#market-navigation)
  - [WebSocket Streams](#websocket-streams)
- [Architecture](#architecture)
- [Error Handling](#error-handling)
- [Running Tests](#running-tests)
- [Examples](#examples)
- [License](#license)

---

## Features

### REST API

| Module | Auth | Endpoints |
|--------|------|-----------|
| **Markets** | No | Browse active, search, get details, oracle candles, feed events, category counts, market timeline |
| **Trader** | Yes | Create GTC/FAK/FOK orders (self-signed or **delegated**), batch status, cancel (single/batch/all/combined), cancel-replace, orderbook, historical prices, locked balance, user orders, market events |
| **Portfolio** | Yes | Current profile, profile, profile update (trading wallet mode switch), trade history, AMM + CLOB positions with P&L, PnL chart, points breakdown, cursor-paginated history (with market filter), public history, allowance checks, redeem, withdraw, withdrawal-address allowlist |
| **Navigation** | No | Navigation tree, market pages, page-specific market listings, property keys & options |
| **PartnerAccounts** | Yes | Create sub-accounts, list/recover sub-accounts, check & retry delegated-trading allowances |
| **Amm** | Yes | Server-wallet AMM buy/sell, allowance check & approve |
| **System** | No | Maintenance status (`GET /maintenance/status`) |
| **Referral** | Varies | Referral stats, referred users, global & friends leaderboards |
| **Leaderboard** | No | Live Unrealized PnL by market, biggest open positions |
| **ApiTokens** | Privy + HMAC | Capability check, derive scoped tokens, list & revoke active tokens |

### WebSocket

| Stream | Auth | Description |
|--------|------|-------------|
| `subscribe_market_prices` | No | AMM price updates + CLOB orderbook snapshots |
| `subscribe_market_lifecycle` | No | Market creation / resolution events |
| `subscribe_positions` | **Yes** | Portfolio position balance changes |
| `subscribe_transactions` | **Yes** | On-chain transaction events |
| `subscribe_order_events` | **Yes** | OME state & settlement lifecycle |
| `subscribe_unrealized_pnl` | No | Unrealized PnL leaderboard invalidation hints |

### Additional

- **EIP-712 signing** for CLOB orders (GTC, FAK, FOK)
- **HMAC-SHA256** request authentication (scoped API tokens) + HMAC-signed WS handshake
- **Self-trade prevention** (`stpPolicy`), receive-window (`timestamp`/`recvWindow`), and `postOnly` on order creation
- **Delegated signing** — unsigned orders for server-wallet sub-accounts (the server signs via a managed Privy wallet)
- **Partner sub-account reads** — `x-on-behalf-of` header support (`for_sub_account`)
- **Cancel-and-replace** — single and batch, with per-operation outcomes
- **Dynamic WebSocket subscriptions** — sub/unsub without reconnecting
- **Auto-reconnect** — `ws_subscribe_market` reconnects with exponential backoff and re-emits its subscription
- **Typed event dispatch** — `positions` (AMM/CLOB), `orderEvent` (OME/SETTLEMENT), and every documented server event
- **Exponential backoff retry** for transient failures
- **Socket.IO heartbeat** — server-initiated pings are answered automatically (the client never sends its own ping frames)

---

## Quick Start

```rust
use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    // Public — no API keys needed
    let api = LimitlessClient::builder().build()?;

    let active = api.browse_active(None, None, Some(5), None, None, None).await?;
    println!("Active markets: {}", active.total_markets_count);

    // Authenticated — reads LIMITLESS_API_KEY / LIMITLESS_API_SECRET from env
    let positions = api.get_positions().await?;
    println!("CLOB positions: {}", positions.clob.len());

    // Place a GTC limit buy — one call signs + submits
    let order = api.buy_gtc(
        "0xYourPrivateKey...",   // 0x-prefixed hex
        "btc-above-100k",        // market slug
        "1234567890",            // token_id as decimal string
        0.55,                    // price
        10.0,                    // size
        42,                      // owner_id from GET /profiles/:address
    ).await?;
    println!("Order placed: {}", order.order.id);

    Ok(())
}
```

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rs_limitless = "0.2"
tokio = { version = "1", features = ["full"] }
```

Or use the crate alias `limitless`:

```toml
[dependencies]
limitless = { package = "rs_limitless", version = "0.2" }
tokio = { version = "1", features = ["full"] }
```

### Requirements

- Rust **1.70+** (edition 2021)
- TLS via the platform's native stack: **schannel** on Windows, **Security.framework** on macOS, **OpenSSL** on Linux

```bash
# Ubuntu / Debian
sudo apt install libssl-dev pkg-config

# Fedora
sudo dnf install openssl-devel
```

---

## Authentication

### HMAC-SHA256 Signed Requests

Credentials can be supplied in three ways:

```rust
// 1. Builder (reads LIMITLESS_API_KEY + LIMITLESS_API_SECRET from env)
let client = LimitlessClient::builder().build()?;

// 2. Builder with explicit credentials
let client = LimitlessClient::builder()
    .api_key("lmts_sk_...")
    .secret("base64_secret")
    .build()?;

// 3. Direct manager construction
let trader = Trader::new(
    Some("lmts_sk_...".into()),
    Some("base64_secret".into()),
);
```

### API Keys

Get your API key from the [Limitless Exchange settings page](https://limitless.exchange/settings/api).
Your credentials are scoped to read/write access and can be revoked at any time.

---

## Usage Guide

### Unified Client

`LimitlessClient` exposes every API method directly, so you never need to reach through
intermediary managers:

```rust
use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    let api = LimitlessClient::builder().build()?;

    // Markets
    let active = api.browse_active(None, None, Some(5), None, None, None).await?;
    let detail = api.get_market("btc-above-100k").await?;
    let ob = api.get_orderbook("btc-above-100k").await?;

    // Trading
    let orders = api.get_user_orders("btc-above-100k", Some(&["LIVE"]), Some(10)).await?;

    // Portfolio
    let positions = api.get_positions().await?;
    let pnl = api.get_pnl_chart(Some("7d")).await?;
    let history = api.get_history(None, Some(50), None).await?;

    // Navigation
    let tree = api.get_navigation_tree().await?;

    // System
    let maintenance = api.get_maintenance_status(Some("trading")).await?;

    // Leaderboards (public)
    let biggest = api.get_biggest_positions(Some(10)).await?;

    // WebSocket
    let stream = api.stream();

    let _ = (maintenance, biggest, stream);
    Ok(())
}
```

### Public Markets

```rust
use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    let api = LimitlessClient::builder().build()?;

    // Browse with pagination
    let page1 = api.browse_active(None, Some(1), Some(20), None, None, None).await?;

    // Search
    let results = api.search_markets("bitcoin above", Some(10), None, None).await?;

    // Market details
    let market = api.get_market("btc-above-100k").await?;
    println!("Venue: {:?}", market.venue);

    // Oracle candles
    let candles = api.get_oracle_candles("btc-above-100k", "1h", None, None).await?;

    // Feed events
    let feed = api.get_feed_events(None, None, Some(10)).await?;

    // Category counts
    let counts = api.get_category_counts().await?;

    Ok(())
}
```

### Trading

Place **GTC** (Good-Till-Cancelled) limit orders and **FOK** (Fill-Or-Kill) market orders
with a single call — the library handles EIP-712 signing, venue contract resolution,
and submission automatically.

```rust
use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    let private_key = std::env::var("LIMITLESS_PRIVATE_KEY")
        .expect("LIMITLESS_PRIVATE_KEY required");
    let owner_id: u64 = std::env::var("LIMITLESS_OWNER_ID")
        .expect("LIMITLESS_OWNER_ID required")
        .parse()
        .expect("Invalid owner_id");

    let api = LimitlessClient::builder().build()?;

    // GTC limit buy — 10 shares at $0.51
    let order = api.buy_gtc(&private_key, "btc-above-100k", "1234567890", 0.51, 10.0, owner_id).await?;
    println!("Buy order: {}", order.order.id);

    // GTC limit sell — 5 shares at $0.60
    let order = api.sell_gtc(&private_key, "btc-above-100k", "1234567890", 0.60, 5.0, owner_id).await?;
    println!("Sell order: {}", order.order.id);

    // FOK market buy — $5 USDC worth at market price
    let order = api.buy_fok(&private_key, "btc-above-100k", "1234567890", 5.0, owner_id).await?;
    println!("FOK buy: {}", order.order.id);

    // FOK market sell — sell 18.64 shares at market price
    let order = api.sell_fok(&private_key, "btc-above-100k", "1234567890", 18.64, owner_id).await?;
    println!("FOK sell: {}", order.order.id);

    // Raw order management
    let trader = api.trader();
    let ob = trader.get_orderbook("btc-above-100k").await?;
    let orders = trader.get_user_orders("btc-above-100k", Some(&["LIVE"]), Some(50)).await?;
    let _cancel = trader.cancel_all_in_market("btc-above-100k").await?;

    Ok(())
}
```

### EIP-712 Order Signing

For advanced use cases, sign orders directly without submitting:

```rust
use limitless::prelude::*;
use limitless::signing::Eip712Signer;

let signer = Eip712Signer::new(
    "0xYourPrivateKey...",
    "0xVenueExchangeContract...",  // from GET /markets/:slug → venue.exchange
)?;

// Build and sign a GTC limit order
let order_data = signer.build_gtc_order(
    "0xYourWallet...",
    "1234567890",       // token_id
    OrderSide::Buy,
    0.55,
    10.0,
    0,                  // fee_rate_bps (0–10000)
)?;

// Submit manually
let request = CreateOrderRequest {
    order: order_data,
    owner_id: 42,
    order_type: OrderType::Gtc,
    market_slug: "btc-above-100k".to_string(),
    client_order_id: None,
    on_behalf_of: None,
    post_only: None,
    timestamp: None,
    recv_window: None,
    stp_policy: None,
};
let body = serde_json::to_string(&request)?;
let trader: Trader = Limitless::new(Some("key".into()), Some("secret".into()));
let response = trader.create_order(&body).await?;
```

### Portfolio

```rust
use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    let api = LimitlessClient::builder().build()?;

    // Profile (pass your wallet address)
    let profile = api.get_profile("0xYourAddress...").await?;
    println!("ID: {}, Fee rate: {} bps", profile.id, profile.rank.map(|r| r.fee_rate_bps).unwrap_or(0));

    // Positions
    let positions = api.get_positions().await?;
    for pos in &positions.clob {
        println!("{} — cost YES: {}, cost NO: {}", pos.market.slug, pos.positions.yes.cost, pos.positions.no.cost);
    }

    // PnL chart
    let pnl = api.get_pnl_chart(Some("30d")).await?;
    if let Some(unrealized) = pnl.total_unrealized_pnl {
        println!("Unrealized PnL: ${:.2}", unrealized);
    }

    // Cursor-paginated history
    let page1 = api.get_history(None, Some(5), None).await?;
    if let Some(cursor) = &page1.next_cursor {
        let page2 = api.get_history(Some(cursor), Some(5), None).await?;
        println!("Page 2: {} entries", page2.data.len());
    }

    // Trade history
    let trades = api.get_trades().await?;
    println!("Total trades: {}", trades.len());

    // Points breakdown
    let points = api.get_points().await?;
    println!("Points: {}", points.points);

    Ok(())
}
```

### Market Navigation

```rust
use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    let api = LimitlessClient::builder().build()?;

    // Full navigation tree
    let tree = api.get_navigation_tree().await?;
    for node in &tree {
        println!("{} → {} (path: {})", node.name, node.id, node.path);
    }

    // Resolve a path to a market page
    let page = api.get_page_by_path("/sports/football").await?;
    println!("Page: {} ({})", page.name, page.full_path);

    // Property keys & options
    let keys = api.list_property_keys().await?;
    if let Some(key) = keys.first() {
        let options = api.list_property_options(&key.id, None).await?;
        println!("Key '{}' has {} options", key.name, options.len());
    }

    // Markets on a specific page
    let page_markets = api.list_page_markets("__home__", None, None, None).await?;
    println!("Home page markets: {}", page_markets.data.len());

    Ok(())
}
```

### Partner, AMM & System

```rust
use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    // HMAC scoped token (account_creation + delegated_signing scopes)
    let api = LimitlessClient::builder()
        .api_key("lmts_sk_...")
        .secret("base64_secret")
        .build()?;

    // ── Partner sub-accounts ──
    let accounts = api.list_partner_accounts(None, Some(25), Some(1)).await?;
    for account in accounts.data {
        println!("Sub-account {}: {}", account.profile_id.unwrap_or(0), account.account.as_deref().unwrap_or(""));
    }

    // ── AMM trading from a server wallet ──
    let buy = AmmBuyRequest {
        market: "will-btc-hit-150k-2026".to_string(),
        outcome_index: 0,           // 0 = YES, 1 = NO
        collateral_amount: "1000000".to_string(), // 1 USDC in base units
        slippage_bps: Some(100),    // 1%
        idempotency_key: "buy-001".to_string(),
        on_behalf_of: Some(12345),
    };
    let result = api.amm_buy(&buy).await?;
    println!("AMM buy: {:?}", result.transaction_id);

    // ── Maintenance status ──
    let status = api.get_maintenance_status(Some("trading")).await?;
    println!("Active windows: {}", status.active.len());

    // ── Referral program ──
    let stats = api.get_referral_stats().await?;
    println!("Referral earnings (raw): {:?}", stats.total_earned_raw);

    // ── Live leaderboards (public) ──
    let board = api.get_market_unrealized_pnl(7348, Some("pnl"), Some(10), Some(1)).await?;
    println!("Leaderboard rows: {}", board.data.len());

    Ok(())
}
```

### Cancel & Replace

Reprice a resting order atomically-ish with one request — the cancel and the
replacement each report their own outcome:

```rust
use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    let api = LimitlessClient::builder().build()?;

    // Build the replacement as a normal signed order (see EIP-712 section)
    let replacement = CreateOrderRequest { /* ... */ };

    let operation = CancelReplaceOperation {
        cancel: CancelOrderIdentifier {
            client_order_id: Some("old-order-001".to_string()),
            order_id: None,
        },
        replacement,
        mode: CancelReplaceMode::StopOnFailure,
        on_behalf_of: None,
    };

    let body = serde_json::to_string(&operation)?;
    let result = api.cancel_replace(&body).await?;
    println!(
        "cancel={} replacement={}",
        result.cancel.status, result.replacement.status
    );

    Ok(())
}
```

### Delegated Orders & Partner Reads

With the `delegated_signing` scope, place orders for a server-wallet sub-account
**without a private key** — the server signs with the sub-account's managed Privy
wallet. Read sub-account data with `for_sub_account`, which adds the
`x-on-behalf-of` header to every signed request:

```rust
use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    // HMAC scoped token (trading + delegated_signing scopes)
    let api = LimitlessClient::builder()
        .api_key("lmts_sk_...")
        .secret("base64_secret")
        .build()?;

    let sub_account_id = 12345;      // profileId from create_sub_account
    let wallet = "0xSubAccountWallet..."; // account from create_sub_account

    // GTC: buy 10 shares at $0.55 — unsigned, server signs
    let order = api.place_delegated_order(
        wallet, "btc-100k", "<tokenId>", OrderSide::Buy, OrderType::Gtc,
        Some(0.55), Some(10.0), None, sub_account_id,
    ).await?;
    println!("Delegated order: {}", order.order.id);

    // FOK: spend 5 USDC at market price — unsigned, server signs
    let order = api.place_delegated_order(
        wallet, "btc-100k", "<tokenId>", OrderSide::Buy, OrderType::Fok,
        None, None, Some(5.0), sub_account_id,
    ).await?;

    // Read the sub-account's data via the x-on-behalf-of header
    let sub = api.for_sub_account(sub_account_id);
    let positions = sub.get_positions().await?;          // sub-account's positions
    let history = sub.get_history(None, Some(10), None).await?;
    let orders = sub.get_user_orders("btc-100k", Some(&["LIVE"]), Some(10)).await?;

    let _ = (order, positions, history, orders);
    Ok(())
}
```

> **Self-signed orders need EOA mode.** If the bot's own account ever enabled
> 1-click (smart wallet) trading, switch it back with
> `api.set_trading_wallet_mode(TradingWalletMode::Eoa).await?;` before placing
> orders signed with a private key.

### WebSocket Streams

The WebSocket transport uses raw WebSocket with Socket.IO protocol underneath.
Use `ws_subscribe_with_commands` for dynamic subscription control:

```rust
use limitless::prelude::*;
use serde_json::Value;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    let ws: Stream = Limitless::new(None, None);
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

    // Spawn event loop
    tokio::spawn(async move {
        ws.ws_subscribe_with_commands(cmd_rx, |event: Value| {
            if let Some(name) = event.as_array()
                .and_then(|a| a.get(0))
                .and_then(|v| v.as_str())
            {
                match name {
                    "orderbookUpdate" => println!("📖 Orderbook updated"),
                    "newPriceData" => println!("💰 AMM prices changed"),
                    "marketCreated" => println!("🆕 New market created"),
                    "marketResolved" => println!("🏁 Market resolved"),
                    _ => println!("📡 {name}"),
                }
            }
            Ok(())
        }).await.ok();
    });

    // Give the connection time to establish
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Subscribe to market prices
    let sub = serde_json::json!({
        "type": 2,
        "data": ["subscribe_market_prices", {"marketSlugs": ["btc-above-100k"]}]
    }).to_string();
    let _ = cmd_tx.send(sub);

    // Keep running
    tokio::signal::ctrl_c().await?;
    Ok(())
}
```

#### Ping

```rust
let ws: Stream = Limitless::new(None, None);
ws.ws_ping().await?;
println!("WebSocket endpoint is reachable ✓");
```

#### Subscription Channels

| Channel | Wire name | Auth | Description |
|---------|-----------|------|-------------|
| `SubscribeMarketPrices` | `subscribe_market_prices` | No | AMM prices + CLOB orderbook |
| `SubscribeMarketLifecycle` | `subscribe_market_lifecycle` | No | Market creation / resolution |
| `SubscribePositions` | `subscribe_positions` | Yes | Portfolio position updates |
| `SubscribeTransactions` | `subscribe_transactions` | Yes | On-chain transaction events |
| `SubscribeOrderEvents` | `subscribe_order_events` | Yes | OME + settlement events |
| `SubscribeUnrealizedPnl` | `subscribe_unrealized_pnl` | No | Unrealized PnL invalidation hints |

---

## Architecture

```text
rs_limitless
├── src/
│   ├── lib.rs              # Crate root, prelude, re-exports
│   ├── api.rs              # API endpoint enums + Limitless trait
│   ├── client.rs           # HTTP client with HMAC signing + WS auth
│   ├── config.rs           # REST/WS endpoint config + recv_window
│   ├── errors.rs           # LimitlessError enum
│   ├── markets.rs          # Public market data + market timeline
│   ├── trading.rs          # Order management + cancel-replace + convenience methods
│   ├── portfolio.rs        # Profile, positions, PnL, history, redeem/withdraw
│   ├── navigation.rs       # Market page tree & property keys
│   ├── partner.rs          # Partner sub-accounts & allowance recovery
│   ├── api_tokens.rs       # Scoped API token management (derive/list/revoke)
│   ├── amm.rs              # Server-wallet AMM trading (buy/sell/allowances)
│   ├── system.rs           # Maintenance status
│   ├── referral.rs         # Referral program endpoints
│   ├── leaderboard.rs      # Live Unrealized PnL leaderboards
│   ├── signing.rs          # EIP-712 order signing
│   ├── lclient.rs          # LimitlessClient (unified builder)
│   ├── models/
│   │   ├── mod.rs          # API response types + WS event types
│   │   └── order.rs        # OrderData, amount calcs, validation
│   ├── ws/
│   │   ├── mod.rs          # Ping interval, channel helpers
│   │   ├── channel.rs      # SubscriptionChannel, event structs, config
│   │   ├── client.rs       # Raw WebSocket connection wrapper
│   │   └── stream.rs       # Event loop + subscription control
│   ├── serde_helpers/      # Custom serde (string-as-f64, etc.)
│   └── retry.rs            # Exponential backoff with jitter
├── tests/
│   ├── markets_test.rs     # Public market integration tests
│   ├── navigation_test.rs  # Navigation integration tests
│   ├── portfolio_test.rs   # Portfolio integration tests
│   ├── trading_test.rs     # Trading integration tests
│   └── ws_test.rs          # WebSocket integration tests
├── examples/
│   ├── public_markets.rs   # Browse, search, orderbook demo
│   ├── portfolio.rs        # Profile, positions, PnL, history demo
│   ├── trading.rs          # GTC / FOK order placement demo
│   └── websocket.rs        # WS CLI with 6 modes (ping/public/orderbook/...)
└── Cargo.toml
```

### Key Types

| Type | Description |
|------|-------------|
| [`LimitlessError`](https://docs.rs/rs_limitless/latest/rs_limitless/enum.LimitlessError.html) | Comprehensive error enum (API errors, network, validation, WS) |
| [`Config`](https://docs.rs/rs_limitless/latest/rs_limitless/struct.Config.html) | REST + WS endpoint configuration |
| [`RetryConfig`](https://docs.rs/rs_limitless/latest/rs_limitless/retry/struct.RetryConfig.html) | Exponential backoff for transient failures |
| [`SubscriptionChannel`](https://docs.rs/rs_limitless/latest/rs_limitless/ws/channel/enum.SubscriptionChannel.html) | All WS subscription channels with `as_str()` |
| [`Eip712Signer`](https://docs.rs/rs_limitless/latest/rs_limitless/signing/struct.Eip712Signer.html) | EIP-712 typed data signer |
| [`OrderData`](https://docs.rs/rs_limitless/latest/rs_limitless/models/order/struct.OrderData.html) | Signed EIP-712 order payload |

---

## Error Handling

All fallible methods return [`Result<T, LimitlessError>`](https://docs.rs/rs_limitless/latest/rs_limitless/enum.LimitlessError.html).
The error enum covers:

- **`ApiError`** — The API returned a 4xx/5xx with an error body
- **`ReqError`** — Network / DNS / TLS failure (transparent from `reqwest`)
- **`Tungstenite`** — WebSocket protocol error (transparent from `tokio-tungstenite`)
- **`ValidationError`** — Client-side parameter validation failed
- **`RateLimited`** — 429 Too Many Requests (retryable)
- **`Json`** — Serialization / deserialization error
- **`StatusCode(u16)`** — Unexpected HTTP status code

```rust
match result {
    Ok(data) => println!("Success: {data:?}"),
    Err(LimitlessError::RateLimited) => {
        println!("Rate limited, use retry::with_retry()");
    }
    Err(LimitlessError::ValidationError(msg)) => {
        println!("Bad input: {msg}");
    }
    Err(e) => println!("Other error: {e}"),
}
```

Use [`retry::with_retry`](https://docs.rs/rs_limitless/latest/rs_limitless/retry/fn.with_retry.html)
for automatic retry with exponential backoff:

```rust
use limitless::retry::with_retry;

let result = with_retry(
    RetryConfig::default(),
    || async {
        api.browse_active(None, None, Some(10), None, None, None).await
    },
).await?;
```

---

## Running Tests

```bash
# All unit tests (no network, no auth)
cargo test --lib

# WebSocket integration tests (requires network)
LIMITLESS_API_KEY="key" LIMITLESS_API_SECRET="secret" cargo test --test ws_test

# All integration tests
LIMITLESS_API_KEY="key" LIMITLESS_API_SECRET="secret" cargo test --tests

# Specific test
cargo test --test ws_test ws_ping_responds

# With logging
RUST_LOG=debug cargo test -- --nocapture

# Compile examples (verify they build)
cargo check --examples
```

> **Note**: The integration tests (`trading_test`, `markets_test`) hit the live API.
> Some tests may fail due to API schema changes — these are documented pre-existing issues.

---

## Examples

The `examples/` directory contains runnable demos:

| Example | Command | Auth |
|---------|---------|------|
| Public markets | `cargo run --example public_markets` | No |
| Portfolio | `cargo run --example portfolio` | Yes |
| Trading | `cargo run --example trading` | Yes |
| WS ping | `cargo run --example websocket ping` | No |
| WS public | `cargo run --example websocket public` | No |
| WS orderbook | `cargo run --example websocket orderbook --market btc-above-100k` | No |
| WS lifecycle | `cargo run --example websocket lifecycle` | No |
| WS positions | `LIMITLESS_API_KEY=k LIMITLESS_API_SECRET=s cargo run --example websocket positions` | Yes |
| WS transactions | `LIMITLESS_API_KEY=k LIMITLESS_API_SECRET=s cargo run --example websocket transactions` | Yes |

---

## License

This project is licensed under the [MIT License](LICENSE).

---

## Acknowledgments

Draws inspiration from [`rs_bybit`](https://github.com/unkuseni/rs_bybit) and the
official [limitless-exchange-rust-sdk](https://github.com/limitless-exchange/limitless-exchange-rust-sdk).
