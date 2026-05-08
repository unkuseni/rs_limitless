//! # WebSocket API Examples for Limitless Exchange
//!
//! This example demonstrates how to use the WebSocket API to subscribe to
//! real-time data streams.
//!
//! ## Running
//!
//! ```bash
//! # Ping the WebSocket endpoint
//! cargo run --example websocket ping
//!
//! # Subscribe to all public events (orderbook, prices, trades)
//! cargo run --example websocket public
//!
//! # Subscribe to orderbook for a specific market
//! cargo run --example websocket orderbook --market will-btc-hit-100k
//!
//! # Subscribe to market lifecycle events
//! cargo run --example websocket lifecycle
//!
//! # Subscribe to private streams (API keys required)
//! LIMITLESS_API_KEY="your_key" LIMITLESS_API_SECRET="your_secret" \
//!   cargo run --example websocket positions --market will-btc-hit-100k
//!
//! # Subscribe to transaction events
//! LIMITLESS_API_KEY="your_key" LIMITLESS_API_SECRET="your_secret" \
//!   cargo run --example websocket transactions
//!
//! # Run for a limited duration (in seconds)
//! cargo run --example websocket public 30
//! ```

use limitless::prelude::*;
use serde_json::Value;
use std::env;
use tokio::sync::mpsc;
use tokio::time::Duration;

// ═══════════════════════════════════════════════════════════════════════════
//  CLI helper
// ═══════════════════════════════════════════════════════════════════════════

fn print_usage() {
    eprintln!(
        r#"Usage: cargo run --example websocket <MODE> [DURATION_SECS] [OPTIONS]

Modes:
  ping         – Ping the WebSocket endpoint (latency check)
  public       – Subscribe to all public market price events
  orderbook    – Subscribe to orderbook updates for a market
  lifecycle    – Subscribe to market creation / resolution events
  positions    – Subscribe to position updates (requires auth)
  transactions – Subscribe to transaction events (requires auth)

Options:
  --market <slug>  – Market slug (default: will-btc-hit-100k)
  DURATION_SECS    – Run duration in seconds (default: 0 = run forever)

Environment:
  LIMITLESS_API_KEY        API key / token ID (for authenticated modes)
  LIMITLESS_API_SECRET     Base64-encoded HMAC secret (for authenticated modes)

Examples:
  cargo run --example websocket ping
  cargo run --example websocket orderbook --market eth-merge-december 30
  LIMITLESS_API_KEY="key" LIMITLESS_API_SECRET="secret" \
    cargo run --example websocket positions --market btc-above-100k
"#,
    );
}

struct Args {
    mode: String,
    market_slug: String,
    duration_secs: u64,
}

fn parse_args() -> Option<Args> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return None;
    }

    let mode = args[1].clone();
    let mut market_slug = String::from("will-btc-hit-100k");
    let mut duration_secs: u64 = 0;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--market" | "-m" => {
                if i + 1 < args.len() {
                    market_slug = args[i + 1].clone();
                    i += 1;
                }
            }
            other => {
                // Try to parse as duration
                if let Ok(dur) = other.parse::<u64>() {
                    duration_secs = dur;
                }
            }
        }
        i += 1;
    }

    Some(Args {
        mode,
        market_slug,
        duration_secs,
    })
}

fn get_credentials() -> (Option<String>, Option<String>) {
    let api_key = env::var("LIMITLESS_API_KEY").ok();
    let secret = env::var("LIMITLESS_API_SECRET").ok();
    (api_key, secret)
}

// ═══════════════════════════════════════════════════════════════════════════
//  Run with timeout helper
// ═══════════════════════════════════════════════════════════════════════════

async fn run_with_deadline(seconds: u64, fut: impl std::future::Future<Output = ()>) {
    if seconds > 0 {
        tokio::select! {
            _ = fut => {}
            _ = tokio::time::sleep(Duration::from_secs(seconds)) => {
                println!("\n⏱  Time limit reached ({seconds}s). Shutting down...");
            }
        }
    } else {
        println!("Running indefinitely. Press Ctrl+C to stop.");
        tokio::select! {
            _ = fut => {}
            _ = tokio::signal::ctrl_c() => {
                println!("\n🛑 Ctrl+C received. Shutting down...");
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Mode: Ping
// ═══════════════════════════════════════════════════════════════════════════

async fn mode_ping() {
    println!("▶ Pinging Limitless WebSocket endpoint...\n");

    let ws: Stream = Limitless::new(None, None);

    for i in 1..=3 {
        let start = tokio::time::Instant::now();
        match ws.ws_ping().await {
            Ok(()) => {
                let elapsed = start.elapsed();
                println!("  Ping #{i}: ✓ OK ({:.0}ms)", elapsed.as_millis());
            }
            Err(e) => {
                println!("  Ping #{i}: ✗ {e}");
            }
        }
        if i < 3 {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    println!("\n✓ Ping test complete.");
}

// ═══════════════════════════════════════════════════════════════════════════
//  Mode: Public (subscribe to market prices — AMM + CLOB)
// ═══════════════════════════════════════════════════════════════════════════

async fn mode_public(market_slug: &str, duration_secs: u64) {
    println!("▶ Subscribing to public market prices...\n");

    let ws: Stream = Limitless::new(None, None);
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<String>();

    let handler = move |event: Value| {
        // Try to detect the event type from the Socket.IO packet structure
        if let Some(event_name) = event
            .as_array()
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.as_str())
        {
            match event_name {
                "newPriceData" => {
                    if let Some(data) = event.as_array().and_then(|arr| arr.get(1)) {
                        println!(
                            "  💰 AMM Price  | market: {} | block: {}",
                            data.get("marketAddress")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?"),
                            data.get("blockNumber")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0),
                        );
                    }
                }
                "orderbookUpdate" => {
                    if let Some(data) = event.as_array().and_then(|arr| arr.get(1)) {
                        let slug = data
                            .get("marketSlug")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        let midpoint = data
                            .get("orderbook")
                            .and_then(|o| o.get("adjustedMidpoint"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        println!("  📖 Orderbook  | {slug} | midpoint: {midpoint:.4}");
                    }
                }
                "system" => {
                    println!("  ⚙ System      | {event}");
                }
                "authenticated" => {
                    println!("  🔐 Authenticated");
                }
                "exception" => {
                    println!("  ❌ Exception   | {event}");
                }
                other => {
                    println!("  📡 {other}");
                }
            }
        } else {
            // Fallback: print trimmed JSON
            let text = serde_json::to_string(&event).unwrap_or_default();
            let trimmed = if text.len() > 120 {
                format!("{}...", &text[..117])
            } else {
                text
            };
            println!("  📡 {trimmed}");
        }
        Ok(())
    };

    // Spawn the WS connection
    let ws_handle = tokio::spawn(async move {
        let _ = ws.ws_subscribe_with_commands(cmd_rx, handler).await;
    });

    // Wait for connection, then send subscription
    tokio::time::sleep(Duration::from_secs(2)).await;

    let sub_cmd = serde_json::json!({
        "type": 2,
        "data": ["subscribe_market_prices", {"marketSlugs": [market_slug]}]
    })
    .to_string();
    println!("→ Subscribing to: subscribe_market_prices for '{market_slug}'\n");
    let _ = cmd_tx.send(sub_cmd);

    // Also subscribe to market lifecycle
    let lifecycle_cmd = serde_json::json!({
        "type": 2,
        "data": ["subscribe_market_lifecycle", {}]
    })
    .to_string();
    let _ = cmd_tx.send(lifecycle_cmd);

    // Run for the specified duration
    run_with_deadline(duration_secs, async {
        let _ = ws_handle.await;
    })
    .await;
}

// ═══════════════════════════════════════════════════════════════════════════
//  Mode: Orderbook (CLOB only)
// ═══════════════════════════════════════════════════════════════════════════

async fn mode_orderbook(market_slug: &str, duration_secs: u64) {
    println!("▶ Subscribing to CLOB orderbook for '{market_slug}'...\n");

    let ws: Stream = Limitless::new(None, None);
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<String>();

    let slug = market_slug.to_string();
    let handler = move |event: Value| {
        if let Some(event_name) = event
            .as_array()
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.as_str())
        {
            if event_name == "orderbookUpdate" {
                if let Some(data) = event.as_array().and_then(|arr| arr.get(1)) {
                    let ob = data.get("orderbook");
                    let midpoint = ob
                        .and_then(|o| o.get("adjustedMidpoint"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let bids = ob
                        .and_then(|o| o.get("bids"))
                        .and_then(|b| b.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    let asks = ob
                        .and_then(|o| o.get("asks"))
                        .and_then(|a| a.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    let best_bid = ob
                        .and_then(|o| o.get("bids"))
                        .and_then(|b| b.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|level| level.get("price"))
                        .and_then(|v| v.as_f64());
                    let best_ask = ob
                        .and_then(|o| o.get("asks"))
                        .and_then(|a| a.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|level| level.get("price"))
                        .and_then(|v| v.as_f64());

                    println!(
                        "  📖 {slug} | mid: {midpoint:.4} | bid: {} | ask: {} | levels: {bids}/{asks}",
                        best_bid.map_or("?".to_string(), |p| format!("{p:.4}")),
                        best_ask.map_or("?".to_string(), |p| format!("{p:.4}")),
                    );
                }
            }
        }
        Ok(())
    };

    let ws_handle = tokio::spawn(async move {
        let _ = ws.ws_subscribe_with_commands(cmd_rx, handler).await;
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    let sub_cmd = serde_json::json!({
        "type": 2,
        "data": ["subscribe_market_prices", {"marketSlugs": [market_slug]}]
    })
    .to_string();
    println!("→ Subscribed.\n");
    let _ = cmd_tx.send(sub_cmd);

    run_with_deadline(duration_secs, async {
        let _ = ws_handle.await;
    })
    .await;
}

// ═══════════════════════════════════════════════════════════════════════════
//  Mode: Lifecycle (market creation / resolution)
// ═══════════════════════════════════════════════════════════════════════════

async fn mode_lifecycle(duration_secs: u64) {
    println!("▶ Subscribing to market lifecycle events...\n");

    let ws: Stream = Limitless::new(None, None);
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<String>();

    let handler = move |event: Value| {
        if let Some(event_name) = event
            .as_array()
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.as_str())
        {
            match event_name {
                "marketCreated" => {
                    if let Some(data) = event.as_array().and_then(|arr| arr.get(1)) {
                        let slug = data.get("slug").and_then(|v| v.as_str()).unwrap_or("?");
                        let title = data.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                        println!("  🆕 Market Created | {slug} — {title}");
                    }
                }
                "marketResolved" => {
                    if let Some(data) = event.as_array().and_then(|arr| arr.get(1)) {
                        let slug = data.get("slug").and_then(|v| v.as_str()).unwrap_or("?");
                        let outcome = data
                            .get("winningOutcome")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        println!("  🏁 Market Resolved | {slug} → winning: {outcome}");
                    }
                }
                _ => {}
            }
        }
        Ok(())
    };

    let ws_handle = tokio::spawn(async move {
        let _ = ws.ws_subscribe_with_commands(cmd_rx, handler).await;
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    let sub_cmd = serde_json::json!({
        "type": 2,
        "data": ["subscribe_market_lifecycle", {}]
    })
    .to_string();
    println!("→ Subscribed.\n");
    let _ = cmd_tx.send(sub_cmd);

    run_with_deadline(duration_secs, async {
        let _ = ws_handle.await;
    })
    .await;
}

// ═══════════════════════════════════════════════════════════════════════════
//  Mode: Positions (auth required)
// ═══════════════════════════════════════════════════════════════════════════

async fn mode_positions(market_slug: &str, duration_secs: u64) {
    let (api_key, secret) = get_credentials();
    let (Some(api_key), Some(secret)) = (api_key, secret) else {
        eprintln!("❌ LIMITLESS_API_KEY and LIMITLESS_API_SECRET required for positions mode.");
        std::process::exit(1);
    };

    println!("▶ Subscribing to position updates for '{market_slug}'...\n");

    let ws: Stream = Limitless::new(Some(api_key), Some(secret));
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<String>();

    let handler = move |event: Value| {
        if let Some(event_name) = event
            .as_array()
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.as_str())
        {
            match event_name {
                "positions" => {
                    println!("  📊 Position update: {}", event);
                }
                "authenticated" => {
                    println!("  🔐 Authenticated ✓");
                }
                "exception" => {
                    println!("  ❌ Exception: {event}");
                }
                other => {
                    println!("  📡 {other}");
                }
            }
        }
        Ok(())
    };

    let ws_handle = tokio::spawn(async move {
        let _ = ws.ws_subscribe_with_commands(cmd_rx, handler).await;
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    let sub_cmd = serde_json::json!({
        "type": 2,
        "data": ["subscribe_positions", {"marketSlugs": [market_slug]}]
    })
    .to_string();
    println!("→ Subscribed to positions.\n");
    let _ = cmd_tx.send(sub_cmd);

    run_with_deadline(duration_secs, async {
        let _ = ws_handle.await;
    })
    .await;
}

// ═══════════════════════════════════════════════════════════════════════════
//  Mode: Transactions (auth required)
// ═══════════════════════════════════════════════════════════════════════════

async fn mode_transactions(duration_secs: u64) {
    let (api_key, secret) = get_credentials();
    let (Some(api_key), Some(secret)) = (api_key, secret) else {
        eprintln!("❌ LIMITLESS_API_KEY and LIMITLESS_API_SECRET required for transactions mode.");
        std::process::exit(1);
    };

    println!("▶ Subscribing to transaction events...\n");

    let ws: Stream = Limitless::new(Some(api_key), Some(secret));
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<String>();

    let handler = move |event: Value| {
        if let Some(event_name) = event
            .as_array()
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.as_str())
        {
            match event_name {
                "transactions" | "transaction" => {
                    if let Some(data) = event.as_array().and_then(|arr| arr.get(1)) {
                        let status = data.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                        let source = data.get("source").and_then(|v| v.as_str()).unwrap_or("?");
                        let market = data
                            .get("marketSlug")
                            .and_then(|v| v.as_str())
                            .unwrap_or("-");
                        let tx_hash = data.get("txHash").and_then(|v| v.as_str()).unwrap_or("-");
                        println!(
                            "  💸 TX | status: {status} | source: {source} | market: {market} | hash: {tx_hash}"
                        );
                    }
                }
                "authenticated" => {
                    println!("  🔐 Authenticated ✓");
                }
                "exception" => {
                    println!("  ❌ Exception: {event}");
                }
                other => {
                    println!("  📡 {other}");
                }
            }
        }
        Ok(())
    };

    let ws_handle = tokio::spawn(async move {
        let _ = ws.ws_subscribe_with_commands(cmd_rx, handler).await;
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    let sub_cmd = serde_json::json!({
        "type": 2,
        "data": ["subscribe_transactions", {}]
    })
    .to_string();
    println!("→ Subscribed to transactions.\n");
    let _ = cmd_tx.send(sub_cmd);

    run_with_deadline(duration_secs, async {
        let _ = ws_handle.await;
    })
    .await;
}

// ═══════════════════════════════════════════════════════════════════════════
//  Main
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    let Some(args) = parse_args() else {
        print_usage();
        std::process::exit(1);
    };

    println!("╔══════════════════════════════════════════════╗");
    println!("║  Limitless Exchange WebSocket Example        ║");
    println!("╚══════════════════════════════════════════════╝\n");

    match args.mode.as_str() {
        "ping" => mode_ping().await,

        "public" => mode_public(&args.market_slug, args.duration_secs).await,

        "orderbook" => mode_orderbook(&args.market_slug, args.duration_secs).await,

        "lifecycle" => mode_lifecycle(args.duration_secs).await,

        "positions" => mode_positions(&args.market_slug, args.duration_secs).await,

        "transactions" => mode_transactions(args.duration_secs).await,

        unknown => {
            eprintln!("❌ Unknown mode: '{unknown}'");
            print_usage();
            std::process::exit(1);
        }
    }

    println!("\n✓ Done.");
}
