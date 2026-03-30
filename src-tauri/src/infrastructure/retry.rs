//! Shared retry utilities for API calls with exponential backoff and jitter.

/// Check if an error message indicates a retryable condition.
///
/// Recognizes network errors, server errors (5xx), and rate limiting (429).
pub fn is_retryable_error(error: &str) -> bool {
    let error_lower = error.to_lowercase();
    // Network-related errors
    error_lower.contains("connection")
        || error_lower.contains("timeout")
        || error_lower.contains("timed out")
        || error_lower.contains("network")
        || error_lower.contains("dns")
        || error_lower.contains("reset")
        || error_lower.contains("refused")
        // Server errors (5xx)
        || error_lower.contains("500")
        || error_lower.contains("502")
        || error_lower.contains("503")
        || error_lower.contains("504")
        // Rate limiting
        || error_lower.contains("429")
        || error_lower.contains("rate limit")
        || error_lower.contains("too many requests")
}

/// Calculate delay for next retry with exponential backoff and jitter.
///
/// Uses the given `attempt` (1-based), `initial_delay_ms`, and `max_delay_ms`
/// to compute an exponential backoff with ±25% jitter, clamped to
/// `[initial_delay/2, max_delay]`.
pub fn calculate_retry_delay(attempt: u32, initial_delay_ms: u64, max_delay_ms: u64) -> u64 {
    let exponential_delay = initial_delay_ms * 2u64.pow(attempt - 1);
    let capped_delay = exponential_delay.min(max_delay_ms);
    // Add jitter (±25%)
    let jitter_range = capped_delay / 4;
    let jitter = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after UNIX epoch")
        .as_millis() as u64
        % jitter_range;
    // Apply jitter: base - 25% to base + 25%, then cap at max and floor at half
    let delay_with_jitter = capped_delay - jitter_range / 2 + jitter;
    delay_with_jitter.min(max_delay_ms).max(capped_delay / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_error_recognizes_network_errors() {
        assert!(is_retryable_error("connection refused"));
        assert!(is_retryable_error("Connection timed out"));
        assert!(is_retryable_error("network error"));
        assert!(is_retryable_error("dns resolution failed"));
        assert!(is_retryable_error("connection reset by peer"));
    }

    #[test]
    fn retryable_error_recognizes_server_errors() {
        assert!(is_retryable_error("500 Internal Server Error"));
        assert!(is_retryable_error("502 Bad Gateway"));
        assert!(is_retryable_error("503 Service Unavailable"));
        assert!(is_retryable_error("504 Gateway Timeout"));
    }

    #[test]
    fn retryable_error_recognizes_rate_limiting() {
        assert!(is_retryable_error("429 Too Many Requests"));
        assert!(is_retryable_error("rate limit exceeded"));
        assert!(is_retryable_error("too many requests"));
    }

    #[test]
    fn retryable_error_rejects_client_errors() {
        assert!(!is_retryable_error("400 Bad Request"));
        assert!(!is_retryable_error("401 Unauthorized"));
        assert!(!is_retryable_error("403 Forbidden"));
        assert!(!is_retryable_error("404 Not Found"));
    }

    #[test]
    fn retry_delay_increases_exponentially() {
        let delay1 = calculate_retry_delay(1, 1000, 10000);
        let delay2 = calculate_retry_delay(2, 1000, 10000);
        let delay3 = calculate_retry_delay(3, 1000, 10000);
        // Each should be roughly double the previous (with jitter)
        assert!(delay1 >= 500 && delay1 <= 1000);
        assert!(delay2 >= 1000 && delay2 <= 2000);
        assert!(delay3 >= 2000 && delay3 <= 4000);
    }

    #[test]
    fn retry_delay_capped_at_max() {
        for attempt in 1..=20 {
            let delay = calculate_retry_delay(attempt, 1000, 10000);
            assert!(
                delay <= 10000,
                "delay for attempt {} is {} which exceeds max",
                attempt,
                delay
            );
        }
    }
}
