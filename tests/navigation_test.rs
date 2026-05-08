//! Integration tests for the Navigation manager.
//!
//! All navigation endpoints are public — no auth required.

use limitless::prelude::*;

fn nav() -> Navigation {
    Navigation::new(None, None)
}

#[tokio::test]
async fn get_navigation_tree() {
    let nav = nav();

    let tree = nav
        .get_navigation_tree()
        .await
        .expect("Failed to get navigation tree");

    assert!(!tree.is_empty(), "Navigation tree should not be empty");
    println!("Navigation tree has {} top-level nodes", tree.len());

    // Verify structure
    let first = &tree[0];
    assert!(!first.id.is_empty());
    assert!(!first.name.is_empty());
    assert!(!first.slug.is_empty());
}

#[tokio::test]
async fn get_page_by_path() {
    let nav = nav();

    // Try home page
    let page = nav.get_page_by_path("/").await;
    match page {
        Ok(p) => {
            assert!(!p.id.is_empty());
            println!("Home page: {} ({})", p.name, p.full_path);
        }
        Err(e) => {
            // May not have a home page configured
            println!("No home page: {}", e);
        }
    }
}

#[tokio::test]
async fn list_property_keys() {
    let nav = nav();

    let keys = nav
        .list_property_keys()
        .await
        .expect("Failed to list property keys");

    println!("Got {} property keys", keys.len());
    // Not asserting non-empty — may be empty in sandbox
}

#[tokio::test]
async fn list_property_options() {
    let nav = nav();

    let keys = nav
        .list_property_keys()
        .await
        .expect("Failed to list property keys");

    if let Some(key) = keys.first() {
        let options = nav
            .list_property_options(&key.id, None)
            .await
            .expect("Failed to list property options");

        println!("Key '{}' has {} options", key.name, options.len());
    }
}
