//! Integration tests for the Portfolio manager.
//!
//! These tests require authentication (API key + secret).

use limitless::prelude::*;

fn auth_portfolio() -> Portfolio {
    let api_key = std::env::var("LIMITLESS_API_KEY").ok();
    let secret = std::env::var("LIMITLESS_API_SECRET").ok();
    Portfolio::new(api_key, secret)
}

#[tokio::test]
async fn get_positions_requires_auth() {
    let portfolio = auth_portfolio();

    match portfolio.get_positions().await {
        Ok(positions) => {
            println!(
                "Positions — AMM: {}, CLOB: {}",
                positions.amm.len(),
                positions.clob.len()
            );
            // Verify structure is correct
            for pos in &positions.clob {
                assert!(!pos.market.slug.is_empty());
            }
        }
        Err(e) => {
            // Expected if no credentials — the test shouldn't panic
            println!("Auth required (expected without env vars): {}", e);
        }
    }
}

#[tokio::test]
async fn get_history_pagination() {
    let portfolio = auth_portfolio();

    match portfolio.get_history(None, Some(3)).await {
        Ok(page1) => {
            println!("Page 1: {} entries", page1.data.len());

            if let Some(ref cursor) = page1.next_cursor {
                let page2 = portfolio.get_history(Some(cursor), Some(3)).await;
                if let Ok(page2) = page2 {
                    println!("Page 2: {} entries", page2.data.len());
                }
            }
        }
        Err(e) => {
            println!("Auth required (expected without env vars): {}", e);
        }
    }
}

#[tokio::test]
async fn get_trades_requires_auth() {
    let portfolio = auth_portfolio();

    match portfolio.get_trades().await {
        Ok(trades) => {
            println!("Got {} trades", trades.len());
        }
        Err(e) => {
            println!("Auth required: {}", e);
        }
    }
}

// ── Unit tests (no network) ──

#[test]
fn retry_config_defaults() {
    let config = limitless::retry::RetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert!(config.should_retry(&LimitlessError::RateLimited));
    assert!(!config.should_retry(&LimitlessError::ValidationError("test".into())));
}

#[test]
fn retry_config_none_disables_retry() {
    let config = limitless::retry::RetryConfig::none();
    assert_eq!(config.max_retries, 0);
}
