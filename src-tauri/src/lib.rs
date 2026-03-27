#[cfg(feature = "screenshot")]
pub mod auto_perception;
pub mod backup;
pub mod crypto;
pub mod dingtalk;
pub mod export;
#[cfg(feature = "screenshot")]
pub mod hardware;
pub mod manual_entry;
pub mod memory_storage;
#[cfg(feature = "screenshot")]
pub mod monitor;
pub mod monitor_types;
pub mod network_status;
pub mod notion;
pub mod offline_queue;
pub mod ollama;
pub mod performance;
pub mod session_manager;
pub mod silent_tracker;
pub mod slack;
pub mod synthesis;
pub mod timeline;
pub mod window_info;
pub mod work_time;

use once_cell::sync::Lazy;
use reqwest::{Client, Proxy, Url};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

/// Check if a URL refers to a local address that should bypass system proxy.
///
/// Local addresses include:
/// - localhost
/// - 127.0.0.1
/// - ::1 (IPv6 loopback)
/// - 0.0.0.0
/// - 192.168.x.x (private network)
/// - 10.x.x.x (private network)
/// - 172.16.x.x - 172.31.x.x (private network)
pub fn is_local_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();

    // Check for localhost
    if url_lower.contains("localhost") {
        return true;
    }

    // Check for 127.0.0.1 or ::1 (loopback)
    if url_lower.contains("127.0.0.1") || url_lower.contains("[::1]") || url_lower.contains("::1") {
        return true;
    }

    // Check for 0.0.0.0
    if url_lower.contains("0.0.0.0") {
        return true;
    }

    // Parse URL and check host
    if let Ok(parsed) = Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            // Check for loopback
            if host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "0.0.0.0" {
                return true;
            }

            // Check for private IP ranges
            if let Ok(ip) = host.parse::<std::net::IpAddr>() {
                match ip {
                    std::net::IpAddr::V4(ipv4) => {
                        // 127.0.0.0/8 loopback
                        if ipv4.is_loopback() {
                            return true;
                        }
                        // 10.0.0.0/8 private
                        if ipv4.octets()[0] == 10 {
                            return true;
                        }
                        // 172.16.0.0/12 private
                        let octets = ipv4.octets();
                        if octets[0] == 172 && (16..=31).contains(&octets[1]) {
                            return true;
                        }
                        // 192.168.0.0/16 private
                        if octets[0] == 192 && octets[1] == 168 {
                            return true;
                        }
                    }
                    std::net::IpAddr::V6(ipv6) => {
                        if ipv6.is_loopback() {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Create an HTTP client with appropriate proxy settings.
///
/// For local URLs (localhost, 127.0.0.1, private networks), the client is configured
/// to bypass system proxy to avoid connection issues when the system has a proxy configured.
///
/// For external URLs, the client uses system proxy settings.
pub fn create_http_client(target_url: &str, timeout_secs: u64) -> Result<Client, String> {
    create_http_client_with_proxy(target_url, timeout_secs, None)
}

/// PERF-001: Proxy configuration for explicit proxy settings
#[derive(Debug, Clone, Default)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// Create an HTTP client with optional explicit proxy configuration.
///
/// When proxy config is provided and enabled, the client uses the specified proxy.
/// When proxy config is None or disabled, behavior matches create_http_client().
pub fn create_http_client_with_proxy(
    target_url: &str,
    timeout_secs: u64,
    proxy_config: Option<ProxyConfig>,
) -> Result<Client, String> {
    let mut builder = Client::builder().timeout(Duration::from_secs(timeout_secs));

    // If proxy is explicitly enabled with valid host/port, use the proxy
    if let Some(ref proxy) = proxy_config {
        if proxy.enabled {
            if let (Some(host), Some(port)) = (&proxy.host, &proxy.port) {
                let proxy_url = if host.starts_with("http://") || host.starts_with("https://") {
                    format!("{}:{}", host, port)
                } else {
                    format!("http://{}:{}", host, port)
                };

                match Proxy::https(&proxy_url) {
                    Ok(mut proxy_obj) => {
                        // Add basic auth if credentials provided
                        if let (Some(ref username), Some(ref password)) =
                            (&proxy.username, &proxy.password)
                        {
                            if !username.is_empty() {
                                proxy_obj = proxy_obj.basic_auth(username, password);
                                tracing::info!("Using authenticated proxy: {}:{}", host, port);
                            }
                        } else {
                            tracing::info!("Using proxy: {}:{}", host, port);
                        }
                        builder = builder.proxy(proxy_obj);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create proxy: {}", e);
                    }
                }
            }
        } else {
            // Proxy disabled - use no_proxy for local URLs
            if is_local_url(target_url) {
                builder = builder.no_proxy();
            }
        }
    } else {
        // No proxy config - use default behavior (local URLs bypass system proxy)
        if is_local_url(target_url) {
            tracing::info!(
                "Creating HTTP client with proxy disabled for local URL: {}",
                target_url
            );
            builder = builder.no_proxy();
        }
    }

    builder
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

/// Returns the application data directory: `<system_data_dir>/DailyLogger`.
/// Used by all modules that need access to the app's persistent data.
pub fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
}

/// Write a diagnostic message to a startup log file for debugging Windows portable issues.
/// Tries multiple locations in order of preference:
/// 1. Next to executable (portable mode)
/// 2. App data directory
/// 3. User home directory as fallback
/// 4. Temp directory as last resort
pub fn write_diagnostic_file(message: &str) {
    // Use Utc instead of Local to avoid timezone lookup issues on Windows
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
    let diagnostic_message = format!("[{}] {}\n", timestamp, message);

    // Always try temp directory first as it's most reliable
    let temp_path = std::env::temp_dir().join("dailylogger-startup.log");
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&temp_path)
    {
        let _ = file.write_all(diagnostic_message.as_bytes());
        let _ = file.flush();
    }

    // Get executable path once to avoid repeated calls
    let exe_path = std::env::current_exe().ok();
    let exe_dir = exe_path.as_ref().and_then(|p| p.parent());

    // Try multiple locations in order of preference
    let locations: Vec<PathBuf> = vec![
        // 1. Next to executable (portable mode)
        exe_dir
            .map(|d| d.join("dailylogger-startup.log"))
            .unwrap_or_default(),
        // 2. App data directory
        get_app_data_dir().join("startup.log"),
        // 3. User home directory as fallback
        dirs::home_dir()
            .map(|h| h.join("dailylogger-startup.log"))
            .unwrap_or_default(),
    ];

    for location in &locations {
        if location.as_os_str().is_empty() {
            continue;
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(location) {
            let _ = file.write_all(diagnostic_message.as_bytes());
            let _ = file.flush();
        }
    }
    // Last resort: try to print to stderr (may be invisible on Windows GUI mode)
    eprintln!("{}", diagnostic_message);
}

#[derive(Default)]
pub struct AppState {
    pub auto_capture_running: bool,
}

/// Mask an API key for safe logging: show prefix (up to 5 chars) + "..." + "****".
/// Example: "sk-abc123xyz9999" -> "sk-ab...****"
pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return "****".to_string();
    }

    // Check if it's an encrypted key - don't reveal any part of it
    if crate::crypto::is_encrypted(key) {
        return "[encrypted]".to_string();
    }

    // Show prefix (first 5 chars or less) + "..." + "****"
    let prefix_len = key.len().min(5);
    format!("{}...****", &key[..prefix_len])
}

pub fn init_app() -> tauri::Result<()> {
    // Write diagnostic directly without helper function to ensure it works
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(std::env::temp_dir().join("dailylogger-startup.log"))
        .map(|mut f| {
            use std::io::Write;
            let _ = f.write_all(
                format!(
                    "[{}] init_app: ENTRY POINT\n",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC")
                )
                .as_bytes(),
            );
            let _ = f.flush();
        });

    write_diagnostic_file("init_app: Starting database initialization");
    tracing::info!("init_app: Starting database initialization");

    // Log the app data directory for debugging
    let app_data_dir = get_app_data_dir();
    write_diagnostic_file(&format!("init_app: App data directory: {:?}", app_data_dir));
    tracing::info!("init_app: App data directory: {:?}", app_data_dir);

    write_diagnostic_file("init_app: Calling init_database()");
    memory_storage::init_database().map_err(|e| {
        write_diagnostic_file(&format!("init_app: init_database FAILED: {}", e));
        tracing::error!("init_app: Database initialization failed: {}", e);
        tauri::Error::Anyhow(anyhow::anyhow!("{}", e))
    })?;

    write_diagnostic_file("init_app: Database initialized successfully");
    tracing::info!("init_app: Database initialized successfully");

    // Load persisted learning data (DEBT-005)
    write_diagnostic_file("init_app: Loading silent pattern stats");
    if let Err(e) = silent_tracker::load_silent_pattern_stats() {
        write_diagnostic_file(&format!(
            "init_app: Failed to load silent pattern stats: {}",
            e
        ));
        tracing::warn!("Failed to load silent pattern stats: {}", e);
    }
    write_diagnostic_file("init_app: Silent pattern stats loaded");
    tracing::info!("init_app: Silent pattern stats loaded");

    write_diagnostic_file("init_app: Loading work time activity");
    if let Err(e) = work_time::load_work_time_activity() {
        write_diagnostic_file(&format!(
            "init_app: Failed to load work time activity: {}",
            e
        ));
        tracing::warn!("Failed to load work time activity: {}", e);
    }
    write_diagnostic_file("init_app: Work time activity loaded");
    tracing::info!("init_app: Work time activity loaded");

    write_diagnostic_file("init_app: All initialization complete");
    tracing::info!("DailyLogger initialized successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_api_key_shows_prefix() {
        assert_eq!(mask_api_key("sk-abc123xyz9999"), "sk-ab...****");
    }

    #[test]
    fn mask_api_key_short_key_shows_available_prefix() {
        assert_eq!(mask_api_key("ab"), "ab...****");
        assert_eq!(mask_api_key("abcd"), "abcd...****");
    }

    #[test]
    fn mask_api_key_empty_string() {
        assert_eq!(mask_api_key(""), "****");
    }

    #[test]
    fn mask_api_key_exactly_five_chars() {
        assert_eq!(mask_api_key("12345"), "12345...****");
    }

    #[test]
    fn mask_api_key_encrypted_key() {
        assert_eq!(mask_api_key("ENC:somebase64data"), "[encrypted]");
    }

    #[test]
    fn get_app_data_dir_returns_dailylogger_subdir() {
        let dir = get_app_data_dir();
        assert!(dir.ends_with("DailyLogger"));
    }

    // Tests for is_local_url function
    #[test]
    fn is_local_url_localhost() {
        assert!(is_local_url("http://localhost:11434/v1"));
        assert!(is_local_url("http://localhost:8080"));
        assert!(is_local_url("https://localhost/api"));
    }

    #[test]
    fn is_local_url_loopback_ipv4() {
        assert!(is_local_url("http://127.0.0.1:11434/v1"));
        assert!(is_local_url("http://127.0.0.1:8080"));
    }

    #[test]
    fn is_local_url_loopback_ipv6() {
        assert!(is_local_url("http://[::1]:11434/v1"));
    }

    #[test]
    fn is_local_url_private_networks() {
        // 10.x.x.x
        assert!(is_local_url("http://10.0.0.1:11434"));
        assert!(is_local_url("http://10.255.255.255:8080"));

        // 172.16.x.x - 172.31.x.x
        assert!(is_local_url("http://172.16.0.1:11434"));
        assert!(is_local_url("http://172.31.255.255:8080"));

        // 192.168.x.x
        assert!(is_local_url("http://192.168.1.1:11434"));
        assert!(is_local_url("http://192.168.0.100:8080"));
    }

    #[test]
    fn is_local_url_external_not_local() {
        assert!(!is_local_url("https://api.openai.com/v1"));
        assert!(!is_local_url("https://example.com/api"));
        assert!(!is_local_url("http://8.8.8.8:80"));
    }

    #[test]
    fn is_local_url_case_insensitive() {
        assert!(is_local_url("http://LOCALHOST:11434/v1"));
        assert!(is_local_url("http://LocalHost:8080"));
    }
}

// STAB-001: Tests for error handling and panic hooks

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    /// Test that the app data directory is correctly determined
    #[test]
    fn app_data_dir_ends_with_dailylogger() {
        let dir = get_app_data_dir();
        assert!(
            dir.file_name().map_or(false, |n| n == "DailyLogger"),
            "App data dir should end with 'DailyLogger', got {:?}",
            dir
        );
    }

    /// Test API key masking for various inputs
    #[test]
    fn mask_api_key_various_lengths() {
        // Empty key
        assert_eq!(mask_api_key(""), "****");

        // Short key (less than 5 chars)
        assert_eq!(mask_api_key("abc"), "abc...****");
        assert_eq!(mask_api_key("ab"), "ab...****");

        // Exactly 5 chars
        assert_eq!(mask_api_key("12345"), "12345...****");

        // Longer key
        assert_eq!(mask_api_key("sk-abc123"), "sk-ab...****");

        // Encrypted key should not reveal any part
        assert_eq!(mask_api_key("ENC:somebase64data"), "[encrypted]");
    }

    /// Test that local URL detection works correctly
    #[test]
    fn local_url_detection_comprehensive() {
        // Valid local URLs
        assert!(is_local_url("http://localhost:8080"));
        assert!(is_local_url("http://127.0.0.1:11434"));
        assert!(is_local_url("http://[::1]:8080"));
        assert!(is_local_url("http://192.168.1.100:8080"));
        assert!(is_local_url("http://10.0.0.1:8080"));
        assert!(is_local_url("http://172.16.0.1:8080"));

        // Invalid local URLs
        assert!(!is_local_url("https://google.com"));
        assert!(!is_local_url("https://api.github.com"));
        assert!(!is_local_url("http://8.8.8.8:80"));
    }

    /// Test panic hook behavior by verifying the hook is set
    #[test]
    fn panic_hook_can_be_set() {
        // This test verifies that we can set a panic hook without panicking
        let result = std::panic::catch_unwind(|| {
            std::panic::set_hook(Box::new(|_| {}));
        });
        assert!(result.is_ok());
    }

    /// Test panic hook receives correct information
    #[test]
    fn panic_hook_receives_info() {
        use std::sync::atomic::{AtomicBool, Ordering};
        static HOOK_CALLED: AtomicBool = AtomicBool::new(false);

        std::panic::set_hook(Box::new(|panic_info| {
            HOOK_CALLED.store(true, Ordering::SeqCst);
            // Verify we can extract a message from the panic info
            let _ = panic_info.to_string();
        }));

        // Trigger a panic
        let result = std::panic::catch_unwind(|| {
            panic!("test panic message");
        });

        // Verify hook was called
        assert!(result.is_err()); // catch_unwind should catch the panic
        assert!(
            HOOK_CALLED.load(Ordering::SeqCst),
            "Panic hook should have been called"
        );

        // Reset panic hook
        std::panic::set_hook(Box::new(|_| {}));
    }
}
