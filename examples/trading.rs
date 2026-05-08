//! Example: Place and manage orders on Limitless Exchange.
//!
//! Demonstrates the high-level convenience methods that reduce code bloat:
//! - `buy_gtc` / `sell_gtc` — Place limit orders with a single call
//! - `buy_fok` / `sell_fok` — Place market orders with a single call
//! - `cancel_all` — Cancel all orders in a market
//!
//! Run with: `cargo run --example trading`
//!
//! Required env vars:
//! - `LIMITLESS_API_KEY` — Your API key
//! - `LIMITLESS_API_SECRET` — Your base64-encoded HMAC secret
//! - `LIMITLESS_PRIVATE_KEY` — 0x-prefixed hex private key for signing
//! - `LIMITLESS_OWNER_ID` — Your profile ID (from GET /profiles/:address)

use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    let private_key = std::env::var("LIMITLESS_PRIVATE_KEY").unwrap_or_default();
    let owner_id: u64 = std::env::var("LIMITLESS_OWNER_ID")
        .unwrap_or_default()
        .parse()
        .unwrap_or(0);

    let api = LimitlessClient::builder().build()?;

    // ── Find a market to trade ─────────────────────────────────────────
    println!("=== Finding a market ===");
    let active = api
        .browse_active(None, None, Some(3), None, None, None)
        .await?;

    let Some(market) = active.data.first() else {
        println!("No active markets found.");
        return Ok(());
    };

    println!("Market: {} (slug: {})", market.title, market.slug);

    // Get the token ID for the first outcome (as a decimal string)
    let token_id_str = market
        .outcomes
        .first()
        .map(|o| o.token_id.clone())
        .unwrap_or_default();
    let has_token = !token_id_str.is_empty() && token_id_str != "0";

    // Get the orderbook to see current prices
    println!("\n=== Current Orderbook ===");
    let ob = api.get_orderbook(&market.slug).await?;
    println!(
        "  Midpoint: {}, Last trade: {}",
        ob.adjusted_midpoint, ob.last_trade_price
    );
    println!("  Best bid: {:?}", ob.bids.first().map(|b| b.price));
    println!("  Best ask: {:?}", ob.asks.first().map(|a| a.price));

    // ── Place orders using convenience methods ─────────────────────────
    if !private_key.is_empty() && owner_id > 0 && has_token {
        println!("\n=== Placing Orders ===");

        // ── GTC Limit Buy ─────────────────────────────────────────
        // BUY 10 shares at $0.51 — all in ONE method call!
        println!("→ Placing GTC BUY: 10 shares @ $0.51...");
        match api
            .buy_gtc(
                &private_key,
                &market.slug,
                &token_id_str,
                0.51,
                10.0,
                owner_id,
            )
            .await
        {
            Ok(response) => {
                println!("  ✓ Order placed: {}", response.order.id);
                println!("  Status: {:?}", response.order.status);
            }
            Err(e) => {
                println!("  ✗ Order failed: {}", e);
            }
        }

        // ── GTC Limit Sell ────────────────────────────────────────
        // SELL 5 shares at $0.60 — all in ONE method call!
        println!("→ Placing GTC SELL: 5 shares @ $0.60...");
        match api
            .sell_gtc(
                &private_key,
                &market.slug,
                &token_id_str,
                0.60,
                5.0,
                owner_id,
            )
            .await
        {
            Ok(response) => {
                println!("  ✓ Order placed: {}", response.order.id);
            }
            Err(e) => {
                println!("  ✗ Order failed: {}", e);
            }
        }

        // ── FOK Market Buy ────────────────────────────────────────
        // BUY with $5 USDC — all in ONE method call!
        println!("→ Placing FOK BUY: $5 USDC at market...");
        match api
            .buy_fok(&private_key, &market.slug, &token_id_str, 5.0, owner_id)
            .await
        {
            Ok(response) => {
                println!("  ✓ Order placed: {}", response.order.id);
            }
            Err(e) => {
                println!("  ✗ Order failed: {}", e);
            }
        }
    } else {
        println!("\n=== Skipping order placement ===");
        println!("Set LIMITLESS_PRIVATE_KEY, LIMITLESS_OWNER_ID env vars to place orders.");
        println!("Run GET /profiles/:wallet_address to find your owner_id.");
    }

    // ── View user orders ───────────────────────────────────────────────
    println!("\n=== Your Orders ===");
    match api
        .get_user_orders(&market.slug, Some(&["LIVE"]), Some(10))
        .await
    {
        Ok(orders) => {
            println!("  Open orders: {}", orders.data.len());
            for o in &orders.data {
                println!("    #{} — {} — status: {:?}", o.id, o.order_type, o.status);
            }
        }
        Err(e) => {
            println!("  Could not fetch orders: {}", e);
        }
    }

    Ok(())
}
