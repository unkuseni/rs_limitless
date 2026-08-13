# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] — 2026-08-13

### Added

- **Cancel-and-replace** — `POST /orders/cancel-replace` and
  `POST /orders/cancel-replace/batch` with typed request/response models and
  per-operation outcomes (`SUCCESS` / `FAILURE` / `UNKNOWN` / `NOT_ATTEMPTED`)
- **Combined batch cancel** — `POST /orders/batch-cancel` accepting either
  `orderIds` or `clientOrderIds`
- **AMM trading for server wallets** — new `Amm` manager covering
  `POST /amm/buy`, `POST /amm/sell`, `POST /amm/allowances/check`, and
  `POST /amm/allowances/approve`
- **Partner sub-accounts** — new `PartnerAccounts` manager with create, list
  (`GET /profiles/partner-accounts`), check-allowances, and retry-allowances
- **System** — new `System` manager for `GET /maintenance/status`
- **Referral program** — new `Referral` manager for `GET /referral/usdc/me`,
  `GET /referral/usdc/referrals`, and the global/friends leaderboards
- **Live Unrealized PnL leaderboards** — new `Leaderboard` manager for
  `GET /leaderboard/pnl/unrealized/markets/{marketId}` and
  `GET /leaderboard/pnl/unrealized/biggest-positions`
- **Market timeline** — `get_market_timeline` / `get_global_timeline` for
  pre-fetching recurring-market slots before they open
- **Current profile** — `get_current_profile()` for `GET /profiles/me`
- **Public history** — `get_public_history(account, …)` for
  `GET /portfolio/{account}/history`
- **Portfolio history market filter** — `get_history` now accepts an optional
  `market` slug
- **Server-wallet operations** — `redeem`, `withdraw`, and the
  withdrawal-address allowlist (`add_withdrawal_address` /
  `delete_withdrawal_address`, Privy identity-token auth)
- **Delegated signing** — `OrderData.signature` / `signature_type` are now
  optional, so unsigned orders can be submitted for server-wallet
  sub-accounts (the server signs via the managed Privy wallet);
  `Trader::place_delegated_order` and the `build_unsigned_order` helper
  cover GTC, FAK, and FOK
- **API tokens** — new `ApiTokens` manager for `GET /auth/api-tokens/capabilities`,
  `POST /auth/api-tokens/derive` (Privy identity token), `GET /auth/api-tokens`,
  and `DELETE /auth/api-tokens/{tokenId}`; the endpoint enum paths were corrected
  from `api-tokens/*` to `auth/api-tokens/*`
- **Partner sub-account reads** — `x-on-behalf-of` header support via
  `Client::with_on_behalf`, `Trader::for_sub_account` /
  `Portfolio::for_sub_account`, and `LimitlessClient::for_sub_account`
  (positions, history, user orders, order status batch)
- **Trading wallet mode** — `PUT /profiles` support (`Portfolio::update_profile`,
  `Portfolio::set_trading_wallet_mode`) for switching between `eoa` and
  `smartWallet` modes
- **FAK convenience methods** — `Trader::buy_fak` / `sell_fak` (and delegations
  on `LimitlessClient`) for fill-and-kill limit orders
- **Order creation fields** — `postOnly`, `timestamp` / `recvWindow` receive
  window, and `stpPolicy` self-trade prevention on `CreateOrderRequest`;
  `FAK` order type added
- **Execution info** — `execution` block (settlement status, fees, totals,
  taker-delay `eligibleAt`, STP cancels) on `CreateOrderResponse`
- **WebSocket** — HMAC-signed WS handshake for authenticated channels
  (`lmts-api-key` / `lmts-timestamp` / `lmts-signature`), new
  `subscribe_unrealized_pnl` channel and `unrealizedPnlProjectionChanged`
  event, plus `matchedAt` / `occurredAt` / `publishedAt` timestamps on OME
  and settlement events
- **WebSocket review fixes** — removed client-initiated Engine.IO ping
  frames (the reference explicitly says not to send them; server pings are
  still answered with pongs), auto-reconnect with exponential backoff and
  re-subscription for `ws_subscribe_market`, typed dispatch for `positions`
  (AMM/CLOB) and `orderEvent` (OME/SETTLEMENT) instead of raw payloads,
  corrected `newPriceData.updatedPrices` shape (`{yes, no}`), enriched
  `SettlementEvent` with the documented fee/trade/amount fields, and a
  typed `UnrealizedPnlSubscription` payload builder
- **History identifiers** — `operation`, `tradeEventId`, `orderId`, and
  `makerMatchId` on CLOB history rows

### Changed

- `get_history` now takes an optional `market` filter parameter (breaking)
- `CreateOrderRequest` gained four optional fields (breaking for struct literals)
- `CreatedOrderInfo.price` / `nonce` and WS `OmeEvent.price` /
  `remainingSize` now deserialize from both JSON strings and numbers
- `OmeEvent.event_id` is now a `Value` to support the string `terminal:<id>`
  form on FAK/FOK `EXECUTION` frames

### Fixed

- Removed the unused direct `openssl` dependency, which broke compilation on
  Windows/MSVC without an OpenSSL installation (TLS now uses the platform's
  native stack)
- Doc tests now compile (`set_credentials` → `api_key`/`secret`, fixed
  examples with undefined variables, `use limitless::prelude::*` in
  `models::order` docs)

### Deprecated

- `LimitlessClientBuilder::testnet()` — Limitless has no testnet deployment;
  the method is now a no-op that logs a warning

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
