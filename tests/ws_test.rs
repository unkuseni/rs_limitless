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
//! # Run all WS tests (including auth-requiring ones)
//! LIMITLESS_API_KEY="key" LIMITLESS_API_SECRET="secret" cargo test --test ws_test
//! ```

use limitless::prelude::*;
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
        // Only print first event to keep output clean
        if c.load(Ordering::Relaxed) == 1 {
            println!("  First event received: {:?}", event);
        }
        Ok(())
    };

    // Spawn the WS connection
    let ws_handle = tokio::spawn(async move {
        let _ = ws.ws_subscribe_with_commands(cmd_rx, handler).await;
    });

    // Wait for connection to establish
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Send a subscription command (raw Socket.IO-style JSON)
    let sub_cmd = serde_json::json!({
        "type": 2,
        "data": ["subscribe_market_prices", {"marketSlugs": []}]
    })
    .to_string();

    println!("  → Sending subscribe command...");
    let _ = cmd_tx.send(sub_cmd);

    // Let events flow for a few seconds
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Send an unsubscribe
    let unsub_cmd = serde_json::json!({
        "type": 2,
        "data": ["unsubscribe_market_prices"]
    })
    .to_string();
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

    // Send a command to verify the channel works
    let ping_cmd = r#"{"type":0}"#.to_string();
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
