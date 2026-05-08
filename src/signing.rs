//! EIP-712 order signing for the Limitless Exchange CLOB.
//!
//! Implements the typed structured data hashing per [EIP-712](https://eips.ethereum.org/EIPS/eip-712)
//! and secp256k1 signing for EOA wallets.
//!
//! # Usage
//!
//! ```no_run
//! use limitless::prelude::*;
//! use limitless::signing::*;
//!
//! let signer = Eip712Signer::new(
//!     "0xYourPrivateKey...",
//!     "0xVenueExchangeAddress...",  // from GET /markets/:slug
//! );
//!
//! let order = signer.build_gtc_order(
//!     "0xYourWallet...",
//!     "1234567890",  // token_id as decimal string
//!     OrderSide::Buy,
//!     0.55,    // price
//!     10.0,    // size
//!     0,       // fee_rate_bps
//! );
//! ```

use k256::ecdsa::SigningKey;
use sha3::{Digest, Keccak256};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ── EIP-712 domain constants ──

/// Chain ID for Base mainnet (where Limitless contracts are deployed).
pub const CHAIN_ID: u64 = 8453;

/// The EIP-712 domain name.
pub const DOMAIN_NAME: &str = "Limitless CTF Exchange";

/// The EIP-712 domain version.
pub const DOMAIN_VERSION: &str = "1";

/// EIP-712 type name for the Order struct.
pub const ORDER_TYPE_NAME: &str = "Order";

/// The EIP-712 type definition string for Order.
pub const ORDER_TYPE: &str =
    "Order(uint256 salt,address maker,address signer,address taker,uint256 tokenId,uint256 makerAmount,uint256 takerAmount,uint256 expiration,uint256 nonce,uint256 feeRateBps,uint8 side,uint8 signatureType)";

// ── Global monotonic salt counter ──

/// Ensures every order produced by this process gets a unique, monotonically
/// increasing salt. The counter is seeded from the current microsecond
/// timestamp and only ever moves forward.
static LAST_ORDER_SALT: AtomicI64 = AtomicI64::new(0);

// ── Helper: keccak256 ──

/// Compute the keccak256 hash of arbitrary bytes.
fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

// ── ABI encoding helpers ──

/// Encode a `uint256` from a `u64` (right-aligned in 32 bytes, big-endian).
fn encode_u256_from_u64(value: u64) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[24..].copy_from_slice(&value.to_be_bytes());
    buf
}

/// Encode a `uint256` from an `i64` (right-aligned, big-endian).
/// Returns an error for negative values (matching the reference).
fn encode_u256_from_i64(value: i64) -> Result<[u8; 32], String> {
    if value < 0 {
        return Err(format!("expected non-negative integer, got {value}"));
    }
    Ok(encode_u256_from_u64(value as u64))
}

/// Encode a `uint256` from an `i32` (right-aligned, big-endian).
fn encode_u256_from_i32(value: i32) -> Result<[u8; 32], String> {
    if value < 0 {
        return Err(format!("expected non-negative integer, got {value}"));
    }
    Ok(encode_u256_from_u64(value as u64))
}

/// Encode a decimal string as a uint256 (big-endian, right-aligned 32 bytes).
///
/// Performs manual decimal → binary conversion without needing `num_bigint`.
fn encode_decimal_string_as_u256(value: &str) -> Result<[u8; 32], String> {
    if value.is_empty() || !value.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("invalid uint value: {value}"));
    }
    // Strip leading zeros
    let trimmed = value.trim_start_matches('0');
    let digits = if trimmed.is_empty() { "0" } else { trimmed };

    // Simple decimal → binary conversion
    let mut bytes: Vec<u8> = Vec::new();
    let mut current = digits.to_string();

    while !current.is_empty() && current != "0" {
        let mut next = String::new();
        let mut carry = 0u32;
        for c in current.chars() {
            let val = carry * 10 + (c as u32 - '0' as u32);
            let q = val / 256;
            carry = val % 256;
            if !next.is_empty() || q > 0 {
                next.push(char::from_digit(q, 10).unwrap_or('0'));
            }
        }
        bytes.push(carry as u8);
        if next.is_empty() {
            next = String::from("0");
        }
        current = next;
    }

    bytes.reverse();
    if bytes.len() > 32 {
        return Err(format!("value {} exceeds uint256 size", value));
    }

    let mut out = [0u8; 32];
    let start = 32 - bytes.len();
    out[start..].copy_from_slice(&bytes);
    Ok(out)
}


/// Encode an Ethereum address as 32 bytes (left-padded, 20-byte address right-aligned).
fn encode_address(addr: &[u8; 20]) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[12..].copy_from_slice(addr);
    buf
}

/// Encode a `string` field for EIP-712: keccak256 of the UTF-8 bytes.
fn encode_string(s: &str) -> [u8; 32] {
    keccak256(s.as_bytes())
}

// ── Salt generation ──

/// Generate a unique, monotonically-increasing salt for an order.
///
/// Uses an atomic compare-and-swap loop seeded from the current microsecond
/// timestamp. Two successive calls are guaranteed to return `next > prev`.
pub fn generate_salt() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0));
    let candidate = i64::try_from(now.as_micros()).unwrap_or(i64::MAX - 1);

    loop {
        let previous = LAST_ORDER_SALT.load(Ordering::Relaxed);
        let next = candidate.max(previous.saturating_add(1));
        match LAST_ORDER_SALT.compare_exchange(previous, next, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => return next,
            Err(_) => continue,
        }
    }
}

// ── EIP-712 hashing ──

/// Compute the EIP-712 domain separator.
///
/// `verifying_contract` is the raw 20-byte address of `venue.exchange`.
pub fn domain_separator(verifying_contract: &[u8; 20]) -> [u8; 32] {
    // Domain type definition
    let domain_type =
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";
    let type_hash = keccak256(domain_type.as_bytes());

    let name_hash = encode_string(DOMAIN_NAME);
    let version_hash = encode_string(DOMAIN_VERSION);

    // Full uint256 encoding of chain ID (matches reference `encode_u256_from_u64`)
    let chain_id_bytes = encode_u256_from_u64(CHAIN_ID);

    let contract = encode_address(verifying_contract);

    // hashStruct = keccak256(typeHash || encodeData)
    let mut data = Vec::with_capacity(32 * 5);
    data.extend_from_slice(&type_hash);
    data.extend_from_slice(&name_hash);
    data.extend_from_slice(&version_hash);
    data.extend_from_slice(&chain_id_bytes);
    data.extend_from_slice(&contract);

    keccak256(&data)
}

/// Compute the EIP-712 order type hash.
pub fn order_type_hash() -> [u8; 32] {
    keccak256(ORDER_TYPE.as_bytes())
}

/// Compute the EIP-712 `hashStruct` for an order.
///
/// This is: `keccak256(typeHash || encodeData(order))`
pub fn hash_order(
    salt: i64,
    maker: &[u8; 20],
    signer: &[u8; 20],
    taker: &[u8; 20],
    token_id: &str,
    maker_amount: i64,
    taker_amount: i64,
    expiration: &str,
    nonce: i32,
    fee_rate_bps: i32,
    side: u8,
    signature_type: u8,
) -> Result<[u8; 32], String> {
    let type_hash = order_type_hash();

    let mut data = Vec::with_capacity(32 * 13);
    data.extend_from_slice(&type_hash);
    data.extend_from_slice(&encode_u256_from_i64(salt)?);
    data.extend_from_slice(&encode_address(maker));
    data.extend_from_slice(&encode_address(signer));
    data.extend_from_slice(&encode_address(taker));
    data.extend_from_slice(&encode_decimal_string_as_u256(token_id)?);
    data.extend_from_slice(&encode_u256_from_i64(maker_amount)?);
    data.extend_from_slice(&encode_u256_from_i64(taker_amount)?);
    data.extend_from_slice(&encode_decimal_string_as_u256(expiration)?);
    data.extend_from_slice(&encode_u256_from_i32(nonce)?);
    data.extend_from_slice(&encode_u256_from_i32(fee_rate_bps)?);
    data.extend_from_slice(&encode_u256_from_u64(side as u64));
    data.extend_from_slice(&encode_u256_from_u64(signature_type as u64));

    Ok(keccak256(&data))
}

/// Compute the final EIP-712 message hash to be signed.
///
/// `hash = keccak256("\x19\x01" || domainSeparator || orderHash)`
pub fn eip712_message_hash(domain_separator: &[u8; 32], order_hash: &[u8; 32]) -> [u8; 32] {
    let mut data = Vec::with_capacity(2 + 32 + 32);
    data.extend_from_slice(b"\x19\x01");
    data.extend_from_slice(domain_separator);
    data.extend_from_slice(order_hash);
    keccak256(&data)
}

// ── Address parsing helpers ──

/// Parse a hex address string (with or without `0x` prefix) into a 20-byte array.
pub fn parse_address(addr: &str) -> Result<[u8; 20], String> {
    let hex_str = addr.strip_prefix("0x").unwrap_or(addr);
    let bytes = hex::decode(hex_str).map_err(|e| format!("Invalid hex address: {}", e))?;
    if bytes.len() != 20 {
        return Err(format!(
            "Address must be 20 bytes, got {} bytes",
            bytes.len()
        ));
    }
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Parse a hex private key string (with or without `0x` prefix) into a 32-byte array.
pub fn parse_private_key(key: &str) -> Result<[u8; 32], String> {
    let hex_str = key.strip_prefix("0x").unwrap_or(key);
    let bytes = hex::decode(hex_str).map_err(|e| format!("Invalid hex key: {}", e))?;
    if bytes.len() != 32 {
        return Err(format!(
            "Private key must be 32 bytes, got {} bytes",
            bytes.len()
        ));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Validate that an address string looks like a valid Ethereum address.
pub fn is_valid_address(addr: &str) -> bool {
    addr.len() == 42 && addr.starts_with("0x") && addr[2..].chars().all(|ch| ch.is_ascii_hexdigit())
}

// ── EIP-712 Signer ──

/// An EIP-712 order signer for the Limitless Exchange.
///
/// Holds a secp256k1 signing key and the venue exchange address
/// (used as `verifyingContract` in the EIP-712 domain).
pub struct Eip712Signer {
    signing_key: SigningKey,
    /// The derived wallet address (0x-prefixed, checksummed).
    address: String,
    domain_separator: [u8; 32],
}

impl Eip712Signer {
    /// Create a new signer from a private key and the venue exchange address.
    ///
    /// # Arguments
    ///
    /// * `private_key` — 0x-prefixed hex private key (32 bytes).
    /// * `verifying_contract` — 0x-prefixed hex address of `venue.exchange`.
    pub fn new(private_key: &str, verifying_contract: &str) -> Result<Self, String> {
        let key_bytes = parse_private_key(private_key)?;
        let signing_key = SigningKey::from_slice(&key_bytes)
            .map_err(|e| format!("Invalid private key: {}", e))?;
        let verifying_contract_bytes = parse_address(verifying_contract)?;
        let domain_separator = domain_separator(&verifying_contract_bytes);

        // Derive wallet address from the signing key
        let verifying_key = signing_key.verifying_key();
        let encoded = verifying_key.to_encoded_point(false);
        let public_key_bytes = encoded.as_bytes();
        let hash = keccak256(&public_key_bytes[1..]);
        let address = checksum_address(&hash[12..]);

        Ok(Self {
            signing_key,
            address,
            domain_separator,
        })
    }

    /// Get the wallet address derived from the private key.
    pub fn wallet_address(&self) -> &str {
        &self.address
    }

    /// Sign an EIP-712 order hash and return the 0x-prefixed hex signature.
    ///
    /// Returns a 65-byte ECDSA signature in `r || s || v` format.
    pub fn sign_hash(&self, order_hash: &[u8; 32]) -> Result<String, String> {
        let message_hash = eip712_message_hash(&self.domain_separator, order_hash);

        // Use recoverable signing to get the recovery ID
        let (sig, recovery_id) = self
            .signing_key
            .sign_prehash_recoverable(&message_hash)
            .map_err(|e| format!("Signing failed: {}", e))?;

        // Normalize to low-s form
        let sig = sig.normalize_s().unwrap_or(sig);

        // r || s || v format (65 bytes)
        let mut sig_bytes = Vec::with_capacity(65);
        sig_bytes.extend_from_slice(&sig.to_bytes());
        sig_bytes.push(recovery_id.to_byte() + 27); // Ethereum-style v

        Ok(format!("0x{}", hex::encode(&sig_bytes)))
    }

    /// Build and sign a GTC limit order.
    ///
    /// Validates all fields, generates a unique monotonic salt, and signs
    /// the EIP-712 typed data.
    ///
    /// # Arguments
    /// * `maker_address` — 0x-prefixed wallet address (must match the signer's key)
    /// * `token_id` — Decimal string representation of the token ID
    /// * `side` — BUY or SELL
    /// * `price` — Price between 0 and 1
    /// * `size` — Number of contracts
    /// * `fee_rate_bps` — Fee rate in basis points (0–10000)
    pub fn build_gtc_order(
        &self,
        maker_address: &str,
        token_id: &str,
        side: crate::models::order::OrderSide,
        price: f64,
        size: f64,
        fee_rate_bps: i32,
    ) -> Result<crate::models::order::OrderData, String> {
        use crate::models::order::{gtc_amounts, validate_gtc_order};

        // Verify the signer's wallet matches the requested maker address
        if !self.address.eq_ignore_ascii_case(maker_address) {
            return Err(format!(
                "wallet address mismatch: signing with '{}' but maker is '{}'",
                self.address, maker_address
            ));
        }

        // Client-side validation
        validate_gtc_order(price, size, None)?;

        let maker = parse_address(maker_address)?;
        let taker = [0u8; 20]; // 0x000...000 for open orders
        let (maker_amount, taker_amount) = gtc_amounts(side, price, size);

        let salt = generate_salt();
        let expiration = "0".to_string(); // no expiration

        let order_hash = hash_order(
            salt,
            &maker,
            &maker, // signer = maker for EOA
            &taker,
            token_id,
            maker_amount,
            taker_amount,
            &expiration,
            0, // nonce
            fee_rate_bps,
            side.to_u8(),
            0, // signature_type: 0 = EOA
        )?;

        let signature = self.sign_hash(&order_hash)?;

        Ok(crate::models::order::OrderData {
            salt,
            maker: format!("0x{}", hex::encode(maker)),
            signer: format!("0x{}", hex::encode(maker)),
            taker: format!("0x{}", hex::encode(taker)),
            token_id: token_id.to_string(),
            maker_amount,
            taker_amount,
            expiration,
            nonce: 0,
            fee_rate_bps,
            side: side.to_u8(),
            signature,
            signature_type: 0,
        })
    }

    /// Build and sign a FOK market order.
    pub fn build_fok_order(
        &self,
        maker_address: &str,
        token_id: &str,
        side: crate::models::order::OrderSide,
        amount: f64,
        fee_rate_bps: i32,
    ) -> Result<crate::models::order::OrderData, String> {
        use crate::models::order::{fok_amount, validate_fok_order};

        // Verify the signer's wallet matches the requested maker address
        if !self.address.eq_ignore_ascii_case(maker_address) {
            return Err(format!(
                "wallet address mismatch: signing with '{}' but maker is '{}'",
                self.address, maker_address
            ));
        }

        // Client-side validation
        validate_fok_order(amount)?;

        let maker = parse_address(maker_address)?;
        let taker = [0u8; 20];
        let maker_amount = fok_amount(side, amount);

        let salt = generate_salt();
        let expiration = "0".to_string();

        let order_hash = hash_order(
            salt,
            &maker,
            &maker,
            &taker,
            token_id,
            maker_amount,
            1, // FOK: taker_amount always 1
            &expiration,
            0, // nonce
            fee_rate_bps,
            side.to_u8(),
            0,
        )?;

        let signature = self.sign_hash(&order_hash)?;

        Ok(crate::models::order::OrderData {
            salt,
            maker: format!("0x{}", hex::encode(maker)),
            signer: format!("0x{}", hex::encode(maker)),
            taker: format!("0x{}", hex::encode(taker)),
            token_id: token_id.to_string(),
            maker_amount,
            taker_amount: 1,
            expiration,
            nonce: 0,
            fee_rate_bps,
            side: side.to_u8(),
            signature,
            signature_type: 0,
        })
    }
}

// ── Address checksumming (EIP-55) ──

/// Compute the EIP-55 mixed-case checksum address.
pub fn checksum_address(addr: &[u8]) -> String {
    let hex_addr = hex::encode(addr);
    let hash = keccak256(hex_addr.as_bytes());

    let mut result = String::with_capacity(42);
    result.push_str("0x");
    for (i, ch) in hex_addr.chars().enumerate() {
        let hash_byte = hash[i / 2];
        let high_nibble = (hash_byte >> 4) & 0x0f;
        let low_nibble = if i % 2 == 0 {
            high_nibble
        } else {
            hash_byte & 0x0f
        };
        if low_nibble >= 8 {
            result.push(ch.to_ascii_uppercase());
        } else {
            result.push(ch);
        }
    }
    result
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        let addr = parse_address("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed").unwrap();
        assert_eq!(
            hex::encode(addr),
            "5aaeb6053f3e94c9b9a09f33669435e7ef1beaed"
        );
    }

    #[test]
    fn test_parse_address_no_prefix() {
        let addr = parse_address("5aaeb6053f3e94c9b9a09f33669435e7ef1beaed").unwrap();
        assert_eq!(
            hex::encode(addr),
            "5aaeb6053f3e94c9b9a09f33669435e7ef1beaed"
        );
    }

    #[test]
    fn test_parse_private_key() {
        let key =
            parse_private_key("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_domain_separator_is_deterministic() {
        let contract = [0x11u8; 20];
        let ds1 = domain_separator(&contract);
        let ds2 = domain_separator(&contract);
        assert_eq!(ds1, ds2);
        // Should not be all zeros
        assert!(ds1.iter().any(|b| *b != 0));
    }

    #[test]
    fn test_signer_derives_wallet_address() {
        // Hardhat test account #0
        let signer = Eip712Signer::new(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            "0x5FbDB2315678afecb367f032d93F642f64180aa3", // some random contract
        )
        .unwrap();

        let address = signer.wallet_address();
        // Hardhat account #0
        assert_eq!(
            address.to_lowercase(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
    }

    #[test]
    fn test_eip712_hash_is_deterministic() {
        let maker = [0x11u8; 20];
        let signer = [0x11u8; 20];
        let taker = [0u8; 20];

        let h1 = hash_order(
            12345, &maker, &signer, &taker, "1000000", 5000000, 10000000, "0", 42, 0, 0, 0,
        )
        .unwrap();
        let h2 = hash_order(
            12345, &maker, &signer, &taker, "1000000", 5000000, 10000000, "0", 42, 0, 0, 0,
        )
        .unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_encode_decimal_string_as_u256() {
        // "1000000" → [0u8;26], [0,15,66,64]
        let result = encode_decimal_string_as_u256("1000000").unwrap();
        assert_eq!(result[31], 64); // low byte of 1000000
        assert_eq!(result[30], 66);
        assert_eq!(result[29], 15);
    }

    #[test]
    fn test_generate_salt_is_monotonic() {
        let s1 = generate_salt();
        let s2 = generate_salt();
        assert!(s2 > s1, "s2={s2} must be greater than s1={s1}");
    }

    #[test]
    fn test_chain_id_encoding_is_full_u256() {
        let encoded = encode_u256_from_u64(CHAIN_ID);
        // 8453 = 0x2105 → bytes 24..32 should be [0,0,0,0,0,0,0x21,0x05]
        assert_eq!(encoded[30], 0x21);
        assert_eq!(encoded[31], 0x05);
        // High bytes should be zero
        assert_eq!(encoded[24], 0);
        assert_eq!(encoded[27], 0);
    }
}
