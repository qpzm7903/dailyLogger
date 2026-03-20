use std::sync::atomic::{AtomicBool, Ordering};

use tauri::command;

static NETWORK_ONLINE: AtomicBool = AtomicBool::new(true);

/// Network check interval: 30 seconds
const CHECK_INTERVAL_SECS: u64 = 30;

/// Returns the cached network status without making any network calls.
pub fn is_online() -> bool {
    NETWORK_ONLINE.load(Ordering::Relaxed)
}

/// Update the cached network status.
pub fn set_online(online: bool) {
    NETWORK_ONLINE.store(online, Ordering::Relaxed);
}

/// Perform an actual network connectivity check by attempting to reach the
/// configured API endpoint (or a fallback DNS lookup).
/// Updates the cached status and returns the result.
pub async fn check_connectivity() -> bool {
    let settings = match crate::memory_storage::get_settings_sync() {
        Ok(s) => s,
        Err(_) => {
            // Can't read settings — assume online to avoid false negatives
            return true;
        }
    };

    let api_url = settings
        .api_base_url
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| "https://api.openai.com".to_string());

    let online = ping_endpoint(&api_url).await;
    set_online(online);
    tracing::debug!("Network connectivity check: online={}", online);
    online
}

/// Lightweight connectivity probe: send a HEAD request with a short timeout.
async fn ping_endpoint(base_url: &str) -> bool {
    let client = crate::create_http_client(base_url, 5);

    let client = match client {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Try HEAD on the base URL — we only care about TCP-level reachability.
    // Any successful HTTP response (even 4xx/5xx) means the network is up.
    // Any error (connect, timeout, DNS, URL parse) means we can't reach it.
    client.head(base_url).send().await.is_ok()
}

/// Start a background task that periodically checks network connectivity
/// and emits `network-status-changed` events to the frontend when status changes.
/// When connectivity is restored, automatically triggers offline queue processing.
pub fn start_network_monitor(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        tracing::info!(
            "Network monitor started (interval: {}s)",
            CHECK_INTERVAL_SECS
        );
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(CHECK_INTERVAL_SECS)).await;

            let was_online = is_online();
            let now_online = check_connectivity().await;

            if was_online != now_online {
                tracing::info!(
                    "Network status changed: {} -> {}",
                    if was_online { "online" } else { "offline" },
                    if now_online { "online" } else { "offline" }
                );
                // Emit event to frontend
                use tauri::Emitter;
                let _ = app.emit("network-status-changed", now_online);

                // When coming back online, process the offline queue
                if now_online {
                    tracing::info!("Network restored — processing offline queue");
                    crate::offline_queue::process_queue().await;

                    // Notify frontend about queue status update
                    if let Ok(status) = crate::offline_queue::get_offline_queue_status() {
                        let _ = app.emit("offline-queue-updated", status);
                    }
                }
            }
        }
    });
}

/// Tauri command: check current network status (cached).
#[command]
pub fn get_network_status() -> bool {
    is_online()
}

/// Tauri command: actively check connectivity and return result.
#[command]
pub async fn check_network_status() -> Result<bool, String> {
    Ok(check_connectivity().await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_default_state_is_online() {
        // Reset to default
        set_online(true);
        assert!(is_online());
    }

    #[test]
    #[serial]
    fn test_set_offline_then_online() {
        set_online(false);
        assert!(!is_online());

        set_online(true);
        assert!(is_online());
    }

    #[test]
    #[serial]
    fn test_get_network_status_command_returns_cached() {
        set_online(true);
        assert!(get_network_status());

        set_online(false);
        assert!(!get_network_status());

        // Restore
        set_online(true);
    }

    #[tokio::test]
    #[serial]
    async fn test_ping_unreachable_endpoint() {
        // An unreachable endpoint should return false
        let result = ping_endpoint("http://192.0.2.1:1").await;
        assert!(!result);
    }

    #[tokio::test]
    #[serial]
    async fn test_ping_invalid_url() {
        // A completely invalid URL should return false (not panic)
        let result = ping_endpoint("not-a-valid-url").await;
        assert!(!result);
    }

    #[test]
    #[serial]
    fn test_status_change_detection() {
        // Verify we can detect status transitions
        set_online(true);
        let was_online = is_online();
        set_online(false);
        let now_online = is_online();
        assert!(was_online != now_online);
        assert!(was_online);
        assert!(!now_online);

        // Restore
        set_online(true);
    }
}
