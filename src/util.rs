use rand::distr::{Alphanumeric, Distribution};
use rand::rng;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Build a URL-encoded query string from a `BTreeMap`.
///
/// Keys and values are concatenated with `=` and separated by `&`.
/// Leading/trailing double quotes in values are stripped.
pub fn build_request<T: ToString>(parameters: &BTreeMap<String, T>) -> String {
    if parameters.is_empty() {
        return String::new();
    }

    let mut request = String::with_capacity(
        parameters
            .iter()
            .map(|(k, v)| k.len() + v.to_string().len() + 1)
            .sum(),
    );
    for (key, value) in parameters {
        request.push_str(key);
        request.push('=');
        let mut value = value.to_string();
        if value.starts_with('"') && value.ends_with('"') {
            value = value[1..value.len() - 1].to_string();
        }
        request.push_str(&value);
        request.push('&');
    }
    // Remove trailing '&'
    request.truncate(request.len().saturating_sub(1));
    request
}

/// Build a JSON string from a `BTreeMap` of parameters.
///
/// Values must implement `Serialize`. This is used for request bodies
/// and WebSocket subscription payloads.
pub fn build_json_request<T: Serialize>(parameters: &BTreeMap<String, T>) -> String {
    serde_json::to_string(parameters).expect("Failed to serialize parameters to JSON")
}

/// Try to extract an `i64` from a `serde_json::Value`.
pub fn to_i64(value: &Value) -> Option<i64> {
    value.as_i64()
}

/// Try to extract a `u64` from a `serde_json::Value`.
pub fn to_u64(value: &Value) -> Option<u64> {
    value.as_u64()
}

/// Get the current system time as milliseconds since the Unix epoch.
pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

/// Get the current UTC time as an ISO-8601 formatted string.
///
/// Used for Limitless HMAC request signing (`lmts-timestamp` header).
pub fn get_timestamp_iso8601() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

/// Generate a random alphanumeric string of `length` characters.
///
/// Useful for request IDs and WebSocket subscription identifiers.
pub fn generate_random_uid(length: usize) -> String {
    let mut uid = String::with_capacity(length);
    let mut rng = rng();
    for _ in 0..length {
        uid.push(Alphanumeric.sample(&mut rng) as char);
    }
    uid
}
