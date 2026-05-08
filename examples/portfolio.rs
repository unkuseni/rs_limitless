//! Example: Authenticated portfolio operations.
//!
//! Demonstrates:
//! - Profile lookup
//! - Position listing (CLOB + AMM)
//! - PnL chart
//! - Cursor-paginated history
//!
//! Run with: `cargo run --example portfolio`
//!
//! Required env vars:
//! - `LIMITLESS_API_KEY` — Your API key or token ID
//! - `LIMITLESS_API_SECRET` — Your base64-encoded HMAC secret
//! - `LIMITLESS_WALLET` — Your wallet address

use limitless::prelude::*;

#[tokio::main]
async fn main() -> Result<(), LimitlessError> {
    let wallet = std::env::var("LIMITLESS_WALLET").unwrap_or_default();

    let api = LimitlessClient::builder().build()?;

    // ── Profile ────────────────────────────────────────────────────────
    if !wallet.is_empty() {
        println!("=== Profile ===");
        let profile = api.get_profile(&wallet).await?;
        println!("  ID: {}", profile.id);
        println!("  Account: {}", profile.account);
        println!("  Display name: {:?}", profile.display_name);
        if let Some(ref rank) = profile.rank {
            println!(
                "  Rank: {} (fee rate: {} bps)",
                rank.name, rank.fee_rate_bps
            );
        }
        println!("  Points: {:?}", profile.points);
    }

    // ── Positions ──────────────────────────────────────────────────────
    println!("\n=== Positions ===");
    let positions = api.get_positions().await?;
    println!("  AMM positions: {}", positions.amm.len());
    println!("  CLOB positions: {}", positions.clob.len());

    for pos in &positions.clob {
        println!(
            "  CLOB {} — YES cost: {}, NO cost: {}",
            pos.market.slug, pos.positions.yes.cost, pos.positions.no.cost
        );
    }

    for pos in &positions.amm {
        println!(
            "  AMM {} — outcome #{}: {} tokens",
            pos.market.slug, pos.outcome_index, pos.outcome_token_amount
        );
    }

    // ── PnL Chart ──────────────────────────────────────────────────────
    println!("\n=== PnL Chart (7d) ===");
    let pnl = api.get_pnl_chart(Some("7d")).await?;
    println!("  Data points: {}", pnl.data.len());
    if let Some(total) = pnl.total_value {
        println!("  Total value: ${:.2}", total);
    }
    if let Some(unrealized) = pnl.total_unrealized_pnl {
        println!("  Unrealized PnL: ${:.2}", unrealized);
    }

    // ── Cursor-paginated history ───────────────────────────────────────
    println!("\n=== Portfolio History ===");
    let page1 = api.get_history(None, Some(5)).await?;
    println!("  Page 1: {} entries", page1.data.len());
    for entry in &page1.data {
        let market = entry
            .market
            .as_ref()
            .map(|m| m.slug.as_str())
            .unwrap_or("unknown");
        println!(
            "    {} — {} — market: {}",
            entry.block_timestamp,
            entry.strategy.as_deref().unwrap_or("unknown"),
            market
        );
    }

    if let Some(ref cursor) = page1.next_cursor {
        let page2 = api.get_history(Some(cursor), Some(5)).await?;
        println!(
            "  Page 2: {} entries (cursor: {:?})",
            page2.data.len(),
            page2.next_cursor
        );
    }

    Ok(())
}
