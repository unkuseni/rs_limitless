//! Order models matching the Limitless Exchange API `POST /orders` schema
//! and the EIP-712 typed order used for on-chain signature verification.

use serde::{Deserialize, Serialize};

// ── Order side ──

/// Buy or sell side for an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    /// Convert to the `uint8` value used in EIP-712 and the API.
    /// `0` = BUY, `1` = SELL.
    pub fn to_u8(self) -> u8 {
        match self {
            OrderSide::Buy => 0,
            OrderSide::Sell => 1,
        }
    }

    /// Create from the uint8 value.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(OrderSide::Buy),
            1 => Some(OrderSide::Sell),
            _ => None,
        }
    }
}

// ── Order type ──

/// Execution strategy for an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    /// Good-Till-Cancelled: rests on the orderbook until filled or cancelled.
    Gtc,
    /// Fill-And-Kill: matches immediately against available liquidity and
    /// cancels any unmatched remainder instead of resting on the book.
    Fak,
    /// Fill-Or-Kill: executes immediately at market or is cancelled entirely.
    Fok,
}

/// Self-trade prevention policy for `POST /orders`.
///
/// Controls what happens when an incoming order would match against your own
/// resting order on the same token. Sits at the top level of the request,
/// outside the EIP-712 signed `order` payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StpPolicy {
    /// Default. Cancel your conflicting resting order and continue with the
    /// incoming order.
    CancelMaker,
    /// Reject the incoming order before it self-trades.
    CancelTaker,
    /// Cancel your conflicting resting order and reject the incoming order.
    CancelBoth,
}

// ── API-facing order (what you send to POST /orders) ──

/// The signed order payload within a create-order request.
///
/// Matches the EIP-712 `Order` struct. Fields that represent on-chain
/// `uint256` values (token_id, maker_amount, taker_amount) are serialized
/// as decimal strings to match the reference API format and avoid JSON
/// precision loss above 2^53.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderData {
    /// Unique order identifier (monotonic counter).
    pub salt: i64,
    /// Checksummed address of the order creator.
    pub maker: String,
    /// Same as maker for EOA wallets.
    pub signer: String,
    /// `0x000...000` for open orders (any taker can fill).
    pub taker: String,
    /// Position ID — YES or NO token from market data (decimal string).
    #[serde(rename = "tokenId")]
    pub token_id: String,
    /// Amount the maker offers, scaled by 1e6.
    #[serde(rename = "makerAmount")]
    pub maker_amount: i64,
    /// Amount the maker wants in return, scaled by 1e6.
    #[serde(rename = "takerAmount")]
    pub taker_amount: i64,
    /// Expiration timestamp as decimal string. `"0"` = no expiration.
    pub expiration: String,
    /// Order nonce.
    pub nonce: i32,
    /// Fee rate in basis points.
    #[serde(rename = "feeRateBps")]
    pub fee_rate_bps: i32,
    /// `0` = BUY, `1` = SELL.
    pub side: u8,
    /// The EIP-712 signature (0x-prefixed hex, 65 bytes for EOA).
    ///
    /// Omit (set to `None`) for **delegated signing**: with the
    /// `delegated_signing` scope the server signs the order using the Privy
    /// server wallet linked to the target sub-account, so `signature` and
    /// `signature_type` must both be omitted from the payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// `0` = EOA signature. Omitted together with `signature` for delegated signing.
    #[serde(skip_serializing_if = "Option::is_none", rename = "signatureType")]
    pub signature_type: Option<u8>,
}

/// The full request body for `POST /orders`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    /// The signed order data.
    pub order: OrderData,
    /// Your internal profile ID (from `GET /profiles/{address}`).
    #[serde(rename = "ownerId")]
    pub owner_id: u64,
    /// `GTC` or `FOK`.
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
    /// Market slug identifier.
    #[serde(rename = "marketSlug")]
    pub market_slug: String,
    /// Optional idempotency key (max 128 chars).
    #[serde(skip_serializing_if = "Option::is_none", rename = "clientOrderId")]
    pub client_order_id: Option<String>,
    /// Optional profile ID to place order on behalf of (partner flow).
    #[serde(skip_serializing_if = "Option::is_none", rename = "onBehalfOf")]
    pub on_behalf_of: Option<u64>,
    /// Reject the order if it would immediately match (GTC only).
    #[serde(skip_serializing_if = "Option::is_none", rename = "postOnly")]
    pub post_only: Option<bool>,
    /// Optional client-stamped creation time (Unix ms) for receive-window checks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    /// Optional maximum accepted order age in milliseconds (1..=10000).
    #[serde(skip_serializing_if = "Option::is_none", rename = "recvWindow")]
    pub recv_window: Option<u64>,
    /// Self-trade prevention policy. Defaults to `cancel_maker` when omitted.
    #[serde(skip_serializing_if = "Option::is_none", rename = "stpPolicy")]
    pub stp_policy: Option<StpPolicy>,
}

// ── Cancel-and-replace ──

/// Identifies an order to cancel: exactly one of `order_id` or
/// `client_order_id` must be set.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CancelOrderIdentifier {
    /// Internal order ID (UUID).
    #[serde(skip_serializing_if = "Option::is_none", rename = "orderId")]
    pub order_id: Option<String>,
    /// Client-provided order ID from order creation.
    #[serde(skip_serializing_if = "Option::is_none", rename = "clientOrderId")]
    pub client_order_id: Option<String>,
}

/// Controls whether a replacement is attempted after cancellation fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CancelReplaceMode {
    /// Skip the replacement when the cancel fails.
    StopOnFailure,
    /// Attempt the replacement regardless of the cancel outcome.
    AllowFailure,
}

/// A single cancel-and-replace operation for `POST /orders/cancel-replace`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelReplaceOperation {
    /// The order to cancel.
    pub cancel: CancelOrderIdentifier,
    /// The replacement order (same shape as `POST /orders` without `onBehalfOf`).
    pub replacement: CreateOrderRequest,
    /// Failure-mode selection.
    pub mode: CancelReplaceMode,
    /// Optional profile ID for an authorized partner sub-account. Applies to
    /// both cancellation and replacement.
    #[serde(skip_serializing_if = "Option::is_none", rename = "onBehalfOf")]
    pub on_behalf_of: Option<u64>,
}

/// Request body for `POST /orders/cancel-replace/batch`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelReplaceBatchRequest {
    /// Independent cancel-and-replace operations (1 to 4).
    pub operations: Vec<CancelReplaceOperation>,
}

// ── Amount calculation constants ──

/// USDC and shares are scaled by 1e6 on-chain.
pub const SCALE: u128 = 1_000_000;

/// Maximum basis points for fee rate (e.g., 250 = 2.5%).
pub const MAX_BPS: i32 = 10_000;

/// Default price tick (minimum price increment) for CLOB markets.
pub const DEFAULT_PRICE_TICK: f64 = 0.001;

/// Default fee rate in basis points.
pub const DEFAULT_FEE_RATE_BPS: i32 = 300;

// ── Precise scaling helper ──

/// Convert a floating-point dollar amount to a 6-decimal fixed-point `u128`.
///
/// Uses string formatting to avoid floating-point precision loss,
/// then truncates to exactly 6 decimal places (matching the reference
/// SDK's `parse_dec_to_int` / `scale_to_6_decimals` behaviour).
fn scale_to_6_decimals(amount: f64) -> u128 {
    if amount <= 0.0 {
        return 0;
    }
    // Format with enough precision to capture the value accurately,
    // then truncate to 6 decimal places.
    let formatted = format!("{amount:.12}");
    let negative = formatted.starts_with('-');
    let cleaned = if negative {
        formatted.trim_start_matches('-')
    } else {
        formatted.as_str()
    };
    let parts: Vec<&str> = cleaned.split('.').collect();
    let int_part: u128 = parts[0].parse().unwrap_or(0);
    let frac_str = if parts.len() > 1 { parts[1] } else { "" };
    // Truncate fractional part to 6 digits
    let frac_6 = if frac_str.len() > 6 {
        &frac_str[..6]
    } else {
        frac_str
    };
    // Pad with trailing zeros to ensure exactly 6 digits
    let mut frac_padded = String::with_capacity(6);
    frac_padded.push_str(frac_6);
    while frac_padded.len() < 6 {
        frac_padded.push('0');
    }
    let frac_val: u128 = frac_padded.parse().unwrap_or(0);
    let result = int_part * SCALE + frac_val;
    if negative {
        0
    } else {
        result
    }
}

/// Ceiling division for unsigned integers.
///
/// `ceil(a / b)` computed without floating-point.
/// Panics if `b == 0`.
fn div_ceil_u128(a: u128, b: u128) -> u128 {
    assert!(b > 0, "division by zero");
    (a + b - 1) / b
}

// ── Order amount calculations ──

/// Calculate `maker_amount` and `taker_amount` for a **GTC limit order**.
///
/// Uses precise 6-decimal fixed-point arithmetic matching the reference SDK.
/// BUY orders use ceiling division for collateral (the maker pays the
/// rounded-up amount); SELL orders use truncating division.
///
/// * `side` — BUY or SELL
/// * `price` — Price between 0 and 1 (e.g., 0.55)
/// * `size` — Number of contracts (e.g., 10.0)
///
/// Returns `(maker_amount, taker_amount)` as raw `i64` values.
///
/// # Panics
///
/// Panics if the scaled result exceeds `i64::MAX`.
///
/// ```
/// use limitless::prelude::*;
///
/// // BUY 10 shares at $0.55
/// let (maker, taker) = gtc_amounts(OrderSide::Buy, 0.55, 10.0);
/// assert_eq!(maker, 5_500_000);  // 0.55 * 10 * 1e6
/// assert_eq!(taker, 10_000_000); // 10 * 1e6
///
/// // SELL 10 shares at $0.55
/// let (maker, taker) = gtc_amounts(OrderSide::Sell, 0.55, 10.0);
/// assert_eq!(maker, 10_000_000); // 10 * 1e6
/// assert_eq!(taker, 5_500_000);  // 0.55 * 10 * 1e6
/// ```
pub fn gtc_amounts(side: OrderSide, price: f64, size: f64) -> (i64, i64) {
    let shares_scaled = scale_to_6_decimals(size);
    let price_scaled = scale_to_6_decimals(price);

    // collateral = (shares * price_int) / scale
    // = (size * 1e6 * price * 1e6) / 1e6
    // = size * price * 1e6
    let numerator = shares_scaled * price_scaled;
    let collateral = match side {
        OrderSide::Buy => div_ceil_u128(numerator, SCALE),
        OrderSide::Sell => numerator / SCALE,
    };

    let (maker_amount, taker_amount) = match side {
        OrderSide::Buy => (collateral, shares_scaled),
        OrderSide::Sell => (shares_scaled, collateral),
    };

    let maker = i64::try_from(maker_amount).expect("maker_amount exceeds i64 range");
    let taker = i64::try_from(taker_amount).expect("taker_amount exceeds i64 range");

    (maker, taker)
}

/// Calculate `maker_amount` for a **FOK market order**.
///
/// FOK orders always set `taker_amount = 1`.
///
/// * BUY: `maker_amount` = raw USDC to spend scaled by 1e6
/// * SELL: `maker_amount` = raw shares to sell scaled by 1e6
pub fn fok_amount(_side: OrderSide, amount: f64) -> i64 {
    let scaled = scale_to_6_decimals(amount);
    i64::try_from(scaled).expect("FOK amount exceeds i64 range")
}

// ── Order validation ──

/// Validate a GTC limit order's fields client-side.
///
/// Checks price range, size positivity, decimal-place limits,
/// and price-tick alignment.
pub fn validate_gtc_order(price: f64, size: f64, price_tick: Option<f64>) -> Result<(), String> {
    let tick = price_tick.unwrap_or(DEFAULT_PRICE_TICK);

    if !(0.0..=1.0).contains(&price) || price == 0.0 {
        return Err(format!(
            "price must be between 0 and 1 (exclusive of 0), got: {price}"
        ));
    }
    if size <= 0.0 {
        return Err(format!("size must be positive, got: {size}"));
    }

    // Check price decimal places against tick
    let tick_str = float_to_decimal_string(tick);
    let price_str = float_to_decimal_string(price);
    let max_decimals = decimal_places(&tick_str);
    if decimal_places(&price_str) > max_decimals {
        return Err(format!(
            "price {price} has too many decimal places — tick {tick} allows at most {max_decimals}"
        ));
    }

    // Check price is a multiple of tick
    let tick_scaled = scale_to_6_decimals(tick);
    let price_scaled = scale_to_6_decimals(price);
    if tick_scaled > 0 && (price_scaled % tick_scaled) != 0 {
        return Err(format!(
            "price {price} is not tick-aligned — must be a multiple of {tick}"
        ));
    }

    // Check size has at most 6 decimal places
    let size_str = float_to_decimal_string(size);
    if decimal_places(&size_str) > 6 {
        return Err(format!(
            "size {size} has too many decimal places — maximum is 6"
        ));
    }

    Ok(())
}

/// Validate a FOK market order's fields client-side.
pub fn validate_fok_order(amount: f64) -> Result<(), String> {
    if amount <= 0.0 {
        return Err(format!("FOK amount must be positive, got: {amount}"));
    }
    let amount_str = float_to_decimal_string(amount);
    if decimal_places(&amount_str) > 6 {
        return Err(format!(
            "FOK amount {amount} has too many decimal places — maximum is 6"
        ));
    }
    Ok(())
}

/// Validate the high-level `OrderData` fields before signing.
pub fn validate_order_data(order: &OrderData) -> Result<(), String> {
    if order.token_id.is_empty() || order.token_id == "0" {
        return Err("token_id is required and must be non-zero".to_string());
    }
    if order.maker_amount <= 0 {
        return Err("maker_amount must be positive".to_string());
    }
    if order.taker_amount <= 0 {
        return Err("taker_amount must be positive".to_string());
    }
    if order.salt <= 0 {
        return Err(format!("salt must be positive, got: {}", order.salt));
    }
    if order.nonce < 0 {
        return Err(format!("nonce must be non-negative, got: {}", order.nonce));
    }
    if order.fee_rate_bps < 0 || order.fee_rate_bps > MAX_BPS {
        return Err(format!(
            "fee_rate_bps must be in [0, {MAX_BPS}], got: {}",
            order.fee_rate_bps
        ));
    }
    if order.side > 1 {
        return Err(format!(
            "side must be 0 (BUY) or 1 (SELL), got: {}",
            order.side
        ));
    }
    // `signature`/`signature_type` are omitted only for delegated signing;
    // when present, validate the signature type value.
    if let Some(signature_type) = order.signature_type {
        if signature_type > 2 {
            return Err(format!(
                "signature_type must be 0-2, got: {}",
                signature_type
            ));
        }
    }
    if order.signature.is_some() != order.signature_type.is_some() {
        return Err(
            "signature and signature_type must be provided together, or both omitted for delegated signing"
                .to_string(),
        );
    }
    Ok(())
}

// ── Helpers ──

/// Format an f64 to a decimal string with up to 12 decimal places,
/// trimming trailing zeros (matching the reference `float_to_decimal_string`).
fn float_to_decimal_string(value: f64) -> String {
    let mut formatted = format!("{value:.12}");
    // Trim trailing zeros after decimal point
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }
    if formatted == "-0" {
        "0".to_string()
    } else {
        formatted
    }
}

/// Count decimal places in a formatted decimal string.
fn decimal_places(value: &str) -> usize {
    value.split('.').nth(1).map(str::len).unwrap_or(0)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gtc_buy_scales_correctly() {
        let (maker, taker) = gtc_amounts(OrderSide::Buy, 0.55, 10.0);
        assert_eq!(maker, 5_500_000);
        assert_eq!(taker, 10_000_000);
    }

    #[test]
    fn gtc_sell_scales_correctly() {
        let (maker, taker) = gtc_amounts(OrderSide::Sell, 0.55, 10.0);
        assert_eq!(maker, 10_000_000);
        assert_eq!(taker, 5_500_000);
    }

    #[test]
    fn gtc_buy_uses_ceil_division() {
        // price=0.333333, size=1.0 → maker = ceil(1e6 * 333333 / 1e6) = ceil(333333) = 333333
        let (maker, _taker) = gtc_amounts(OrderSide::Buy, 0.333333, 1.0);
        assert_eq!(maker, 333_333);
    }

    #[test]
    fn gtc_amounts_are_symmetric() {
        // BUY and SELL should swap maker/taker
        let (buy_maker, buy_taker) = gtc_amounts(OrderSide::Buy, 0.42, 5.0);
        let (sell_maker, sell_taker) = gtc_amounts(OrderSide::Sell, 0.42, 5.0);
        assert_eq!(buy_maker, sell_taker);
        assert_eq!(buy_taker, sell_maker);
    }

    #[test]
    fn fok_amount_scales_correctly() {
        let scaled = fok_amount(OrderSide::Buy, 10.5);
        assert_eq!(scaled, 10_500_000);
    }

    #[test]
    fn scale_to_6_decimals_truncates() {
        // 0.001001 * 1e6 = 1001
        assert_eq!(scale_to_6_decimals(0.001001), 1001);
        // 0.0010015 truncates to 0.001001 → 1001
        assert_eq!(scale_to_6_decimals(0.0010015), 1001);
    }

    #[test]
    fn scale_to_6_decimals_handles_large_integer() {
        assert_eq!(scale_to_6_decimals(123.456789), 123_456_789);
    }

    #[test]
    fn validate_gtc_rejects_zero_price() {
        assert!(validate_gtc_order(0.0, 1.0, None).is_err());
    }

    #[test]
    fn validate_gtc_rejects_price_above_one() {
        assert!(validate_gtc_order(1.5, 1.0, None).is_err());
    }

    #[test]
    fn validate_gtc_rejects_negative_size() {
        assert!(validate_gtc_order(0.5, -1.0, None).is_err());
    }

    #[test]
    fn validate_gtc_accepts_valid_order() {
        assert!(validate_gtc_order(0.55, 10.0, None).is_ok());
    }

    #[test]
    fn validate_fok_rejects_zero_amount() {
        assert!(validate_fok_order(0.0).is_err());
    }

    #[test]
    fn validate_fok_accepts_valid_amount() {
        assert!(validate_fok_order(100.0).is_ok());
    }

    #[test]
    fn order_data_omits_signature_fields_for_delegated_signing() {
        let unsigned = OrderData {
            salt: 1234567890,
            maker: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(),
            signer: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(),
            taker: "0x0000000000000000000000000000000000000000".to_string(),
            token_id:
                "19633204485790857949828516737993423758628930235371629943999544859324645414627"
                    .to_string(),
            maker_amount: 5_000_000,
            taker_amount: 10_000_000,
            expiration: "0".to_string(),
            nonce: 0,
            fee_rate_bps: 0,
            side: 0,
            signature: None,
            signature_type: None,
        };
        let json = serde_json::to_string(&unsigned).unwrap();
        assert!(!json.contains("signature"));
        assert!(!json.contains("signatureType"));
        assert!(validate_order_data(&unsigned).is_ok());

        // Signed orders must include both fields together.
        let mut signed = unsigned.clone();
        signed.signature = Some("0x1234".to_string());
        signed.signature_type = Some(0);
        let json = serde_json::to_string(&signed).unwrap();
        assert!(json.contains("\"signature\":\"0x1234\""));
        assert!(json.contains("\"signatureType\":0"));
        assert!(validate_order_data(&signed).is_ok());

        // Mismatched pair is rejected.
        let mut bad = unsigned.clone();
        bad.signature = Some("0x1234".to_string());
        assert!(validate_order_data(&bad).is_err());
    }
}
