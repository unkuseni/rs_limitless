//! Example: Browse public markets on Limitless Exchange.
//!
//! Demonstrates the simplified flow with `LimitlessClient`:
//! - Browse active markets
//! - Search for markets
//! - Get market details
//! - Get orderbook
//!
//! Run with: `cargo run --example public_markets`

use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    // Create a client from env vars or builder
    let api = LimitlessClient::builder().build()?;

    // ── Browse active markets ──────────────────────────────────────────
    println!("=== Active Markets ===");
    let active = api
        .browse_active(None, None, Some(5), None, None, None)
        .await?;
    println!("Total active markets: {}", active.total_markets_count);
    for m in &active.data {
        println!(
            "  #{} {} — status: {}, type: {}",
            m.id, m.title, m.status, m.market_type
        );
        if let Some(ref tokens) = m.tokens {
            println!("    YES: {}  NO: {}", tokens.yes, tokens.no);
        }
    }

    // ── Search for markets ─────────────────────────────────────────────
    println!("\n=== Search: 'bitcoin' ===");
    let results = api
        .search_markets("bitcoin above", Some(3), None, None)
        .await?;
    for m in &results.data {
        println!("  {} — {}", m.slug, m.title);
    }

    // ── Get market details + orderbook ─────────────────────────────────
    if let Some(first) = active.data.first() {
        println!("\n=== Market Detail: {} ===", first.slug);
        let detail = api.get_market(&first.slug).await?;
        println!("  Title: {}", detail.title);
        println!("  Expiration: {}", detail.expiration_date);
        if let Some(ref venue) = detail.venue {
            println!("  Exchange contract: {}", venue.exchange);
        }
        if let Some(ref outcomes) = detail.outcomes.first() {
            println!(
                "  First outcome: {} (tokenId: {})",
                outcomes.title, outcomes.token_id
            );
        }

        // Get orderbook
        println!("\n=== Orderbook ===");
        let ob = api.get_orderbook(&first.slug).await?;
        println!("  Adjusted midpoint: {}", ob.adjusted_midpoint);
        println!("  Bids: {} | Asks: {}", ob.bids.len(), ob.asks.len());
        for bid in ob.bids.iter().take(3) {
            println!("    BID {} @ {}", bid.size, bid.price);
        }
        for ask in ob.asks.iter().take(3) {
            println!("    ASK {} @ {}", ask.size, ask.price);
        }
    }

    Ok(())
}
