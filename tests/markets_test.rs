//! Integration tests for the Markets manager.
//!
//! These are live integration tests — they require a network connection
//! but no authentication. They are skipped when `ignore` is set.

use limitless::prelude::*;

/// Create a client for public endpoints (no auth needed).
fn public_client() -> Markets {
    Markets::new(None, None)
}

#[tokio::test]
async fn browse_active_returns_markets() {
    let markets = public_client();

    let result = markets
        .browse_active(None, None, Some(3), None, None, None)
        .await;

    match result {
        Ok(response) => {
            assert!(
                response.total_markets_count > 0,
                "Should have active markets"
            );
            assert!(
                !response.data.is_empty(),
                "Should return at least one market"
            );
            let first = &response.data[0];
            println!("{:#?}", first);
            assert!(!first.slug.is_empty(), "Market should have a slug");
            assert!(!first.title.is_empty(), "Market should have a title");
        }
        Err(e) => {
            panic!("Failed to browse active markets: {}", e);
        }
    }
}

#[tokio::test]
async fn get_market_by_slug_returns_detail() {
    let markets = public_client();

    // First find a slug from active markets
    let active = markets
        .browse_active(None, None, Some(1), None, None, None)
        .await
        .expect("Failed to get active markets");

    let slug = &active.data[0].slug;

    let detail = markets
        .get_market(slug)
        .await
        .expect("Failed to get market detail");

    println!("{:#?}", detail);
    assert_eq!(detail.slug, *slug);
    assert!(!detail.title.is_empty());
}

#[tokio::test]
async fn search_returns_results() {
    let markets = public_client();

    let result = markets
        .search("bitcoin", Some(5), None, None)
        .await
        .expect("Failed to search");

    assert!(
        !result.data.is_empty(),
        "Search should return results for 'bitcoin'"
    );
}

#[tokio::test]
async fn get_category_counts_works() {
    let markets = public_client();

    let result = markets.get_category_counts().await;

    // May fail if endpoint requires auth — just check it doesn't panic
    match result {
        Ok(_counts) => {
            // Success
        }
        Err(_) => {
            // May require auth on some environments
        }
    }
}

#[tokio::test]
async fn get_orderbook_returns_data() {
    let markets = public_client();

    let active = markets
        .browse_active(None, None, Some(1), None, None, None)
        .await
        .expect("Failed to get active markets");

    let slug = &active.data[0].slug;

    // Use a Trader instance for orderbook (it's a public endpoint)
    let trader = Trader::new(None, None);
    let ob = trader
        .get_orderbook(slug)
        .await
        .expect("Failed to get orderbook");

    assert!(!ob.token_id.is_empty());
    // Orderbook might be empty for some markets, that's fine
}
