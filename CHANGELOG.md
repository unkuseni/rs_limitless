# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2025-01-27

### Added

- **REST API** — Complete coverage of Limitless Exchange REST endpoints:
  - Markets: browse active, search, get details, oracle candles, feed events, category counts
  - Trading: create GTC/FOK orders, batch status, cancel (single/batch/all), orderbook, historical prices, locked balance, user orders, market events
  - Portfolio: profile, trade history, AMM + CLOB positions with P&L, PnL chart, points breakdown, cursor-paginated history, allowance checks
  - Navigation: navigation tree, market pages, page-specific market listings, property keys & options
- **HMAC-SHA256 authentication** — Full request signing with `lmts-api-key`, `lmts-timestamp`, `lmts-signature`, `lmts-rec-window` headers
- **WebSocket streams** — Raw WebSocket transport with dynamic subscription control:
  - `subscribe_market_prices` — AMM price updates + CLOB orderbook
  - `subscribe_market_lifecycle` — Market creation / resolution
  - `subscribe_positions` (auth) — Portfolio position changes
  - `subscribe_transactions` (auth) — On-chain transaction events
  - `subscribe_order_events` (auth) — OME + settlement events
  - `ws_ping()` — Connectivity check
  - `ws_subscribe_with_commands()` — Dynamic sub/unsub via channel
- **EIP-712 order signing** — Full typed-data hashing and secp256k1 signing:
  - GTC limit orders with tick-aligned price validation
  - FOK market orders
  - Monotonic salt generation with atomic counter
  - Address checksumming (EIP-55)
- **LimitlessClient** — Unified builder-pattern entry point exposing all methods
- **Retry with exponential backoff** — Configurable retry for transient errors
- **WebSocket channel types** — Strongly-typed `SubscriptionChannel` enum, `SubscriptionOptions`, `WebSocketConfig`, and event payload structs
- **FlexFloat** — Flexible float deserializer handling both JSON numbers and string-encoded floats
- **Integration tests** — 10 WebSocket tests, 4 markets tests, 4 navigation tests, 5 portfolio tests, 6 trading tests
- **Examples** — `public_markets`, `portfolio`, `trading`, `websocket` (CLI with 6 modes)
