//! Integration tests for the Trader manager.
//!
//! Order placement tests require credentials. The public endpoint tests
//! (orderbook, historical prices, market events) run without auth.

use limitless::prelude::*;

fn public_trader() -> Trader {
    Trader::new(None, None)
}

#[tokio::test]
async fn get_orderbook_public() {
    let markets = Markets::new(None, None);
    let active = markets
        .browse_active(None, None, Some(1), None, None, None)
        .await
        .expect("Failed to get active markets");

    let slug = &active.data[0].slug;
    let trader = public_trader();

    let ob = trader
        .get_orderbook(slug)
        .await
        .expect("Failed to get orderbook");

    assert!(!ob.token_id.is_empty());
    assert!(ob.adjusted_midpoint >= 0.0);
}

#[tokio::test]
async fn get_historical_prices() {
    let markets = Markets::new(None, None);
    let active = markets
        .browse_active(None, None, Some(1), None, None, None)
        .await
        .expect("Failed to get active markets");

    let slug = &active.data[0].slug;
    let trader = public_trader();

    let prices = trader
        .get_historical_prices(slug, Some("1h"))
        .await
        .expect("Failed to get historical prices");

    // May be empty for new markets — that's fine
    println!("Got {} historical price points", prices.len());
}

#[tokio::test]
async fn get_market_events() {
    let markets = Markets::new(None, None);
    let active = markets
        .browse_active(None, None, Some(1), None, None, None)
        .await
        .expect("Failed to get active markets");

    let slug = &active.data[0].slug;
    let trader = public_trader();

    let events = trader
        .get_market_events(slug, None, Some(10))
        .await
        .expect("Failed to get market events");

    println!("Got {} market events", events.events.len());
}

// ── Unit tests for amount calculation (no network needed) ──

#[test]
fn gtc_buy_amounts_correct() {
    use limitless::prelude::*;

    let (maker, taker) = gtc_amounts(OrderSide::Buy, 0.55, 10.0);
    // maker = 0.55 * 10 * 1e6 = 5_500_000
    // taker = 10 * 1e6 = 10_000_000
    assert_eq!(maker, 5_500_000);
    assert_eq!(taker, 10_000_000);
}

#[test]
fn gtc_sell_amounts_correct() {
    use limitless::prelude::*;

    let (maker, taker) = gtc_amounts(OrderSide::Sell, 0.55, 10.0);
    // maker = 10 * 1e6 = 10_000_000 (shares to sell)
    // taker = 0.55 * 10 * 1e6 = 5_500_000 (USDC to receive)
    assert_eq!(maker, 10_000_000);
    assert_eq!(taker, 5_500_000);
}

#[test]
fn fok_amount_scales_by_1e6() {
    use limitless::prelude::*;

    let amount = fok_amount(OrderSide::Buy, 18.64);
    assert_eq!(amount, 18_640_000);
}
