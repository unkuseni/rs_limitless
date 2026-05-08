//! Retry logic with exponential backoff for transient API failures.
//!
//! Handles 429 (rate limiting), 5xx (server errors), and network connectivity
//! failures. Uses configurable exponential backoff with jitter.

use crate::prelude::*;
use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

/// Callback invoked before each retry attempt.
pub type RetryCallback = Arc<dyn Fn(usize, &LimitlessError, Duration) + Send + Sync>;

/// Configuration for automatic retry behavior.
///
/// By default, retries on 429, 5xx statuses and connection/timeout errors
/// with up to 3 attempts using exponential backoff starting at 1 second.
#[derive(Clone)]
pub struct RetryConfig {
    /// HTTP status codes that trigger a retry.
    pub status_codes: Vec<u16>,
    /// Maximum number of retry attempts (not counting the initial request).
    pub max_retries: usize,
    /// Exponential base for backoff (default: 2.0 → 1s, 2s, 4s, ...).
    pub exponential_base: f64,
    /// Maximum delay between retries (default: 60s).
    pub max_delay: Duration,
    /// Starting delay before first retry (default: 1s).
    pub initial_delay: Duration,
    /// Optional callback invoked before each retry with (attempt, error, delay).
    pub on_retry: Option<RetryCallback>,
}

impl fmt::Debug for RetryConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RetryConfig")
            .field("status_codes", &self.status_codes)
            .field("max_retries", &self.max_retries)
            .field("exponential_base", &self.exponential_base)
            .field("max_delay", &self.max_delay)
            .field("initial_delay", &self.initial_delay)
            .field("has_on_retry", &self.on_retry.is_some())
            .finish()
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            status_codes: vec![429, 500, 502, 503, 504],
            max_retries: 3,
            exponential_base: 2.0,
            max_delay: Duration::from_secs(60),
            initial_delay: Duration::from_secs(1),
            on_retry: None,
        }
    }
}

impl RetryConfig {
    /// No retries — useful when you want explicit control.
    pub fn none() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }

    /// Compute the delay duration for a given retry attempt number (0-based).
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        let base = if self.exponential_base.is_finite() && self.exponential_base > 0.0 {
            self.exponential_base
        } else {
            2.0
        };

        let exponent = attempt.min(63) as u32;
        let seconds = self.initial_delay.as_secs_f64() * base.powi(exponent as i32);
        let capped = seconds.min(self.max_delay.as_secs_f64());
        Duration::from_secs_f64(if capped <= 0.0 { 0.001 } else { capped })
    }

    /// Returns `true` if the error is retryable according to this config.
    pub fn should_retry(&self, error: &LimitlessError) -> bool {
        match error {
            LimitlessError::RateLimited => self.status_codes.contains(&429),
            LimitlessError::InternalServerError => self.status_codes.contains(&500),
            LimitlessError::ServiceUnavailable => self.status_codes.contains(&503),
            LimitlessError::StatusCode(code) => self.status_codes.contains(code),
            LimitlessError::ReqError(err) => err.is_connect() || err.is_timeout(),
            _ => false,
        }
    }

    /// Attach a callback that fires before each retry.
    #[must_use]
    pub fn with_on_retry<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, &LimitlessError, Duration) + Send + Sync + 'static,
    {
        self.on_retry = Some(Arc::new(callback));
        self
    }
}

/// Execute an async operation with automatic retry logic.
///
/// # Example
///
/// ```no_run
/// use limitless::retry::with_retry;
///
/// let result = with_retry(
///     Default::default(),
///     || async { /* some fallible API call */ Ok::<_, limitless::LimitlessError>(()) },
/// ).await;
/// ```
pub async fn with_retry<T, F, Fut>(
    config: RetryConfig,
    mut operation: F,
) -> Result<T, LimitlessError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, LimitlessError>>,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                let retryable = config.should_retry(&err);
                last_error = Some(err);

                if !retryable || attempt == config.max_retries {
                    break;
                }

                let delay = config.delay_for_attempt(attempt);
                if let Some(callback) = &config.on_retry {
                    if let Some(ref err) = last_error {
                        callback(attempt, err, delay);
                    }
                }

                log::warn!(
                    "Retrying request after failure (attempt {} of {})",
                    attempt + 1,
                    config.max_retries
                );
                tokio::time::sleep(delay).await;
            }
        }
    }

    Err(last_error.expect("retry loop always stores the last error"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn delay_grows_exponentially() {
        let config = RetryConfig::default();
        let d0 = config.delay_for_attempt(0);
        let d2 = config.delay_for_attempt(2);
        assert!(d2 > d0);
    }

    #[test]
    fn delay_clamps_to_max() {
        let config = RetryConfig {
            max_delay: Duration::from_secs(5),
            ..Default::default()
        };
        assert_eq!(config.delay_for_attempt(100), Duration::from_secs(5));
    }

    #[tokio::test]
    async fn retries_and_eventually_succeeds() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let a = attempts.clone();

        let result = with_retry(
            RetryConfig {
                max_retries: 3,
                initial_delay: Duration::from_millis(1),
                ..Default::default()
            },
            move || {
                let a = a.clone();
                async move {
                    let attempt = a.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        Err(LimitlessError::RateLimited)
                    } else {
                        Ok("ok")
                    }
                }
            },
        )
        .await
        .unwrap();

        assert_eq!(result, "ok");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn does_not_retry_non_retryable_errors() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let a = attempts.clone();

        let err = with_retry(RetryConfig::default(), move || {
            let a = a.clone();
            async move {
                a.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(LimitlessError::ValidationError("boom".into()))
            }
        })
        .await
        .unwrap_err();

        assert!(matches!(err, LimitlessError::ValidationError(_)));
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}
