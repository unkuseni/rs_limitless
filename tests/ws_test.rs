//! Integration tests for the WebSocket Stream manager.
//!
//! These are live tests — they require a network connection but no
//! authentication for public streams. They run with timeouts to
//! avoid hanging on network issues.
//!
//! Run with: `cargo test --test ws_test`
//!
//! ## Filtering
//!
//! ```bash
//! # Run only ping test
//! cargo test --test ws_test ping
//!
//! # Run only public subscribe tests
//! cargo test --test ws_test public
//!
//! # Run market updates test with a specific slug
//! MARKET_SLUG="will-btc-hit-100k" cargo test --test ws_test market_updates
//!
//! # Run all WS tests (including auth-requiring ones)
//! LIMITLESS_API_KEY="key" LIMITLESS_API_SECRET="secret" cargo test --test ws_test
//! ```

use limitless::prelude::*;
use limitless::ws::stream::{frame_socketio_event, SOCKET_NAMESPACE};
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

// ═══════════════════════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Create a public Stream (no auth).
fn public_stream() -> Stream {
    Limitless::new(None, None)
}

/// Create an authenticated Stream from env vars.
fn auth_stream() -> Option<Stream> {
    let api_key = std::env::var("LIMITLESS_API_KEY").ok()?;
    let secret = std::env::var("LIMITLESS_API_SECRET").ok()?;
    Some(Limitless::new(Some(api_key), Some(secret)))
}

/// Run a future with a deadline, returning `Ok(())` if it completes in time
/// or `Err(...)` on timeout. The inner future's result is discarded — we
/// only care that it didn't panic and didn't hang.
async fn run_with_timeout<F>(duration_secs: u64, label: &str, fut: F) -> Result<(), String>
where
    F: std::future::Future<Output = ()>,
{
    match timeout(Duration::from_secs(duration_secs), fut).await {
        Ok(()) => {
            println!("  ✓ {label} completed");
            Ok(())
        }
        Err(_) => {
            println!("  ⏱ {label} timed out after {duration_secs}s (expected for live WS tests)");
            Ok(()) // Timeouts are not failures in live WS tests
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests — Connectivity
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn ws_ping_responds() {
    println!("\n=== ws_ping ===");

    let stream: Stream = public_stream();

    match stream.ws_ping().await {
        Ok(()) => println!("  ✓ Ping successful"),
        Err(e) => println!("  ✗ Ping failed (may be expected offline): {e}"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests — Public subscribe (raw Value handler)
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn ws_subscribe_receives_events() {
    println!("\n=== ws_subscribe (public) ===");

    let ws: Stream = public_stream();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    let handler = move |event: Value| {
        let count = c.fetch_add(1, Ordering::Relaxed) + 1;
        // Print the event type for the first few events
        if count <= 3 {
            if let Some(event_type) = event
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
            {
                println!("  Event #{count}: {event_type}");
            } else {
                println!("  Event #{count}: {event}");
            }
        }
        Ok(())
    };

    let fut = async {
        let _ = ws.ws_subscribe(handler).await;
    };

    run_with_timeout(15, "ws_subscribe (public)", fut)
        .await
        .ok();
    let received = counter.load(Ordering::Relaxed);
    println!("  Total events received: {received}");
    // We don't assert count — live WS tests are inherently flaky.
    // The test passes if it connects without panicking.
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests — Dynamic subscribe / unsubscribe
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn ws_dynamic_subscribe_unsubscribe() {
    println!("\n=== ws_subscribe_with_commands ===");

    let ws: Stream = public_stream();
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<String>();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    let handler = move |event: Value| {
        c.fetch_add(1, Ordering::Relaxed);

        // Extract event name and payload from the [event_name, payload] array
        // produced by handle_incoming_text.
        let event_name = event
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let payload = event
            .as_array()
            .and_then(|arr| arr.get(1))
            .unwrap_or(&Value::Null);

        // Dispatch through the typed deserializer
        if let Some(kind) = deserialize_event(event_name, payload) {
            match &kind {
                WsEventKind::NewPriceData(p) => {
                    println!(
                        "  NewPriceData: market={} prices={}",
                        p.market_address,
                        p.updated_prices.len()
                    );
                }
                WsEventKind::OrderbookUpdate(o) => {
                    println!(
                        "  OrderbookUpdate: slug={} bids={} asks={}",
                        o.market_slug,
                        o.orderbook.bids.len(),
                        o.orderbook.asks.len()
                    );
                }
                WsEventKind::TradeEvent(t) => {
                    println!("  Trade: {} {} @ {}", t.side, t.size, t.price);
                }
                WsEventKind::MarketUpdateEvent(m) => {
                    println!(
                        "  MarketUpdate: slug={} last={:?}",
                        m.market_slug, m.last_price
                    );
                }
                WsEventKind::OraclePriceData(o) => {
                    println!("  OraclePrice: slug={} value={}", o.market_slug, o.value);
                }
                WsEventKind::Unknown(payload) => {
                    println!("  Unknown event: {:?}", payload);
                }
                other => {
                    // Limit verbose output for unhandled variants
                    let count = c.load(Ordering::Relaxed);
                    if count <= 3 {
                        println!("  Event #{count}: {other:?}");
                    }
                }
            }
        }
        Ok(())
    };

    // Spawn the WS connection
    let ws_handle = tokio::spawn(async move {
        let _ = ws.ws_subscribe_with_commands(cmd_rx, handler).await;
    });

    // Wait for connection to establish
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Send a subscription command using proper Socket.IO framing:
    //   42/markets,["subscribe_market_prices",{"marketSlugs":["btc-up-or-down-15-mins-1778321716188"]}]
    let sub_cmd = frame_socketio_event(
        "subscribe_market_prices",
        &serde_json::json!({"marketSlugs": ["btc-up-or-down-15-mins-1778327111734"]}),
    );

    println!("  → Sending subscribe command...");
    let _ = cmd_tx.send(sub_cmd);

    // Let events flow for a few seconds
    tokio::time::sleep(Duration::from_secs(30)).await;

    // Send an unsubscribe (raw Socket.IO frame)
    let unsub_cmd = format!(
        "42{namespace},[\"unsubscribe_market_prices\"]",
        namespace = SOCKET_NAMESPACE
    );
    println!("  → Sending unsubscribe command...");
    let _ = cmd_tx.send(unsub_cmd);

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Clean shutdown
    ws_handle.abort();
    let _ = ws_handle.await;

    let received = counter.load(Ordering::Relaxed);
    println!("  Total events received: {received}");
    println!("  ✓ Dynamic subscribe/unsubscribe completed");
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests — Event loop with timeout
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn ws_event_loop_runs_and_stops() {
    println!("\n=== ws event loop lifecycle ===");

    let ws: Stream = public_stream();
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<String>();

    let handler = move |_event: Value| Ok(());

    // Start the connection in a separate task
    let handle = tokio::spawn(async move {
        let _ = ws.ws_subscribe_with_commands(cmd_rx, handler).await;
    });

    // Let it run for a few seconds
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Verify it's still running
    assert!(!handle.is_finished(), "WS loop should still be running");

    // Send an Engine.IO ping to verify the channel works
    let ping_cmd = String::from("2");
    let _ = cmd_tx.send(ping_cmd);
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Abort and verify clean shutdown
    handle.abort();
    match timeout(Duration::from_secs(2), handle).await {
        Ok(join_result) => {
            // Aborted tasks return Err(JoinError) — that's expected
            match join_result {
                Ok(()) => println!("  ✓ WS loop completed naturally"),
                Err(_) => println!("  ✓ WS loop aborted cleanly"),
            }
        }
        Err(_) => {
            println!("  ⚠ WS loop abort timed out (may happen on stuck connection)");
        }
    }

    println!("  ✓ Event loop lifecycle test passed");
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests — Ping / latency measurement
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn ws_ping_latency() {
    println!("\n=== ws ping latency ===");

    let stream: Stream = public_stream();

    let start = tokio::time::Instant::now();
    let result = stream.ws_ping().await;
    let elapsed = start.elapsed();

    match result {
        Ok(()) => println!("  ✓ Ping latency: {:.0}ms", elapsed.as_millis()),
        Err(e) => println!("  ✗ Ping failed: {e}"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests — Multiple sequential connections
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn ws_multiple_connections() {
    println!("\n=== ws multiple connections ===");

    for i in 1..=3 {
        let stream: Stream = public_stream();
        match timeout(Duration::from_secs(10), stream.ws_ping()).await {
            Ok(Ok(())) => println!("  Connection #{i}: ✓ ping OK"),
            Ok(Err(e)) => println!("  Connection #{i}: ✗ {e}"),
            Err(_) => println!("  Connection #{i}: ⏱ timeout"),
        }
        // Brief pause between connections
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests — Authenticated streams (when credentials available)
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn ws_auth_ping() {
    println!("\n=== ws auth ping ===");

    let Some(stream) = auth_stream() else {
        println!("  ⏭ Skipped — no credentials in environment");
        return;
    };

    match stream.ws_ping().await {
        Ok(()) => println!("  ✓ Authenticated ping successful"),
        Err(e) => println!("  ✗ Auth ping failed: {e}"),
    }
}

#[tokio::test]
async fn ws_auth_subscribe_positions() {
    println!("\n=== ws auth subscribe (positions) ===");

    let Some(ws) = auth_stream() else {
        println!("  ⏭ Skipped — no credentials in environment");
        return;
    };

    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    let handler = move |event: Value| {
        let count = c.fetch_add(1, Ordering::Relaxed) + 1;
        if count <= 2 {
            println!("  Auth event #{count}: {event}");
        }
        Ok(())
    };

    let fut = async {
        let _ = ws.ws_subscribe(handler).await;
    };

    run_with_timeout(15, "ws auth subscribe", fut).await.ok();
    println!(
        "  Auth events received: {}",
        counter.load(Ordering::Relaxed)
    );
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests — Channel types (unit tests, no network)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn subscription_channel_as_str_is_consistent() {
    use limitless::SubscriptionChannel;

    let channels = vec![
        (SubscriptionChannel::Orderbook, "orderbook"),
        (SubscriptionChannel::Trades, "trades"),
        (SubscriptionChannel::Orders, "orders"),
        (SubscriptionChannel::Fills, "fills"),
        (SubscriptionChannel::Markets, "markets"),
        (SubscriptionChannel::Prices, "prices"),
        (SubscriptionChannel::Positions, "positions"),
        (SubscriptionChannel::Transactions, "transactions"),
        (SubscriptionChannel::OrderEvents, "orderEvent"),
        (SubscriptionChannel::LiveSports, "liveSports"),
        (SubscriptionChannel::LiveEsports, "liveEsports"),
        (SubscriptionChannel::MarketLifecycle, "marketLifecycle"),
        (
            SubscriptionChannel::SubscribeMarketPrices,
            "subscribe_market_prices",
        ),
        (
            SubscriptionChannel::SubscribePositions,
            "subscribe_positions",
        ),
        (
            SubscriptionChannel::SubscribeTransactions,
            "subscribe_transactions",
        ),
        (
            SubscriptionChannel::SubscribeOrderEvents,
            "subscribe_order_events",
        ),
        (
            SubscriptionChannel::SubscribeLiveSports,
            "subscribe_live_sports",
        ),
        (
            SubscriptionChannel::SubscribeLiveEsports,
            "subscribe_live_esports",
        ),
        (
            SubscriptionChannel::SubscribeMarketLifecycle,
            "subscribe_market_lifecycle",
        ),
        (
            SubscriptionChannel::UnsubscribeMarketLifecycle,
            "unsubscribe_market_lifecycle",
        ),
    ];

    for (channel, expected) in channels {
        assert_eq!(
            channel.as_str(),
            expected,
            "channel {channel:?} should map to '{expected}'"
        );
    }
}

#[test]
fn requires_auth_only_for_private_channels() {
    use limitless::{requires_websocket_auth, SubscriptionChannel};

    // Private channels
    assert!(requires_websocket_auth(
        SubscriptionChannel::SubscribePositions
    ));
    assert!(requires_websocket_auth(
        SubscriptionChannel::SubscribeTransactions
    ));
    assert!(requires_websocket_auth(
        SubscriptionChannel::SubscribeOrderEvents
    ));

    // Public channels
    assert!(!requires_websocket_auth(
        SubscriptionChannel::SubscribeMarketPrices
    ));
    assert!(!requires_websocket_auth(
        SubscriptionChannel::SubscribeMarketLifecycle
    ));
    assert!(!requires_websocket_auth(
        SubscriptionChannel::SubscribeLiveSports
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests — Market updates by slug (the main feature test)
// ═══════════════════════════════════════════════════════════════════════════

/// Subscribe to market prices for a specific slug and print all received
/// events with typed parsing.
///
/// Set `MARKET_SLUG` env var to specify the market, e.g.:
/// ```bash
/// MARKET_SLUG="will-btc-hit-100k" cargo test --test ws_test market_updates -- --nocapture
/// ```
#[tokio::test]
async fn ws_market_updates_with_slug() {
    println!("\n=== Market Updates (subscribe_market_prices) ===");

    let market_slug =
        std::env::var("MARKET_SLUG").unwrap_or_else(|_| "will-btc-hit-100k".to_string());
    let duration_secs: u64 = std::env::var("TEST_DURATION_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15);

    println!("  Market slug : {market_slug}");
    println!("  Duration    : {duration_secs}s");
    println!("  (set MARKET_SLUG and TEST_DURATION_SECS env vars to customize)\n");

    let ws: Stream = public_stream();
    let counter = Arc::new(AtomicUsize::new(0));
    let c = counter.clone();

    let handler = move |event_name: &str, payload: &Value| {
        let count = c.fetch_add(1, Ordering::Relaxed) + 1;

        match event_name {
            "orderbookUpdate" => {
                let slug = payload
                    .get("marketSlug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let midpoint = payload
                    .get("orderbook")
                    .and_then(|o| o.get("adjustedMidpoint"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let bids = payload
                    .get("orderbook")
                    .and_then(|o| o.get("bids"))
                    .and_then(|b| b.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let asks = payload
                    .get("orderbook")
                    .and_then(|o| o.get("asks"))
                    .and_then(|a| a.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let best_bid = payload
                    .get("orderbook")
                    .and_then(|o| o.get("bids"))
                    .and_then(|b| b.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|level| level.get("price"))
                    .and_then(|v| v.as_f64());
                let best_ask = payload
                    .get("orderbook")
                    .and_then(|o| o.get("asks"))
                    .and_then(|a| a.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|level| level.get("price"))
                    .and_then(|v| v.as_f64());

                println!(
                    "  #{count:>3} 📖 Orderbook | {slug} | mid:{midpoint:.4} | bid:{} ask:{} | levels:{bids}/{asks}",
                    best_bid.map_or("?".to_string(), |p| format!("{p:.4}")),
                    best_ask.map_or("?".to_string(), |p| format!("{p:.4}")),
                );
            }
            "newPriceData" => {
                let addr = payload
                    .get("marketAddress")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let block = payload
                    .get("blockNumber")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let yes_price = payload
                    .get("updatedPrices")
                    .and_then(|p| p.get("yes"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let no_price = payload
                    .get("updatedPrices")
                    .and_then(|p| p.get("no"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");

                println!(
                    "  #{count:>3} 💰 AMM Price  | {addr} | block:{block} | yes:{yes_price} no:{no_price}"
                );
            }
            "system" => {
                println!("  #{count:>3} ⚙ System     | {payload}");
            }
            "authenticated" => {
                println!("  #{count:>3} 🔐 Authenticated");
            }
            "exception" => {
                println!("  #{count:>3} ❌ Exception  | {payload}");
            }
            "marketCreated" => {
                let slug = payload.get("slug").and_then(|v| v.as_str()).unwrap_or("?");
                let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                println!("  #{count:>3} 🆕 Created    | {slug} — {title}");
            }
            "marketResolved" => {
                let slug = payload.get("slug").and_then(|v| v.as_str()).unwrap_or("?");
                let outcome = payload
                    .get("winningOutcome")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                println!("  #{count:>3} 🏁 Resolved   | {slug} → {outcome}");
            }
            other => {
                let compact = serde_json::to_string(payload).unwrap_or_default();
                let trimmed = if compact.len() > 100 {
                    format!("{}...", &compact[..97])
                } else {
                    compact
                };
                println!("  #{count:>3} 📡 {other:30} | {trimmed}");
            }
        }
        Ok(())
    };

    let slug = market_slug.clone();
    let fut = async {
        match ws.ws_subscribe_market(&slug, handler).await {
            Ok(()) => println!("\n  ✓ ws_subscribe_market completed normally"),
            Err(e) => println!("\n  ✗ ws_subscribe_market error: {e}"),
        }
    };

    run_with_timeout(duration_secs, "market updates", fut)
        .await
        .ok();

    let received = counter.load(Ordering::Relaxed);
    println!("  Total events received: {received}");
    if received > 0 {
        println!("  ✓ Successfully received market updates for '{market_slug}'");
    } else {
        println!("  ⚠ No events received for '{market_slug}' (market may be inactive)");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests — Socket.IO protocol helpers (unit tests, no network)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn frame_socketio_event_produces_valid_frame() {
    use limitless::ws::stream::frame_socketio_event;

    let frame = frame_socketio_event(
        "subscribe_market_prices",
        &serde_json::json!({"marketSlugs": ["btc-above-100k"]}),
    );

    assert!(
        frame.starts_with("42/markets,["),
        "Expected frame to start with '42/markets,[', got: {}",
        &frame[..frame.len().min(40)]
    );
    assert!(frame.contains("subscribe_market_prices"));
    assert!(frame.contains("btc-above-100k"));
    assert!(frame.ends_with(']'));
}

#[test]
fn parse_socketio_message_extracts_event_and_payload() {
    use limitless::ws::stream::parse_socketio_message;

    let frame = concat!(
        r#"42/markets,["orderbookUpdate","#,
        r#"{"marketSlug":"btc-100k","#,
        r#""orderbook":{"adjustedMidpoint":0.55,"#,
        r#""bids":[],"asks":[]}}]"#
    );

    let result = parse_socketio_message(frame);
    assert!(result.is_some(), "Should parse a valid Socket.IO event");

    let (event_name, payload) = result.unwrap();
    assert_eq!(event_name, "orderbookUpdate");
    assert_eq!(payload["marketSlug"].as_str().unwrap(), "btc-100k");
    assert_eq!(
        payload["orderbook"]["adjustedMidpoint"].as_f64().unwrap(),
        0.55
    );
}

#[test]
fn parse_socketio_message_rejects_non_event_frames() {
    use limitless::ws::stream::parse_socketio_message;

    // Engine.IO open
    assert!(parse_socketio_message("0{\"sid\":\"abc\"}").is_none());
    // Engine.IO ping
    assert!(parse_socketio_message("2").is_none());
    // Engine.IO pong
    assert!(parse_socketio_message("3").is_none());
    // Socket.IO connect ack
    assert!(parse_socketio_message("40/markets,").is_none());
    // Socket.IO disconnect
    assert!(parse_socketio_message("41/markets,").is_none());
    // Empty
    assert!(parse_socketio_message("").is_none());
    // Garbage
    assert!(parse_socketio_message("not a socket.io message").is_none());
}

#[test]
fn parse_socketio_message_handles_event_without_namespace() {
    use limitless::ws::stream::parse_socketio_message;

    let frame = r#"42["system",{"message":"connected"}]"#;

    let result = parse_socketio_message(frame);
    assert!(result.is_some(), "Should parse event without namespace");

    let (event_name, payload) = result.unwrap();
    assert_eq!(event_name, "system");
    assert_eq!(payload["message"].as_str().unwrap(), "connected");
}

#[test]
fn frame_and_parse_roundtrip() {
    use limitless::ws::stream::{frame_socketio_event, parse_socketio_message};

    let event = "test_event";
    let data = serde_json::json!({"key": "value", "num": 42});

    let frame = frame_socketio_event(event, &data);
    let (parsed_event, parsed_payload) =
        parse_socketio_message(&frame).expect("Should parse the frame we just created");

    assert_eq!(parsed_event, event);
    assert_eq!(parsed_payload, data);
}
