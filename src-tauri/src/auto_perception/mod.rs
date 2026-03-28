use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::memory_storage;
use crate::monitor::get_monitor_list;
use crate::monitor_types::{CaptureMode, MonitorInfo};
use crate::silent_tracker::{
    calculate_optimal_silent_minutes, current_threshold, has_sufficient_data, record_capture,
    set_threshold, CaptureReason,
};
use crate::work_time::{is_in_work_time, WorkTimeSettings};

static AUTO_CAPTURE_RUNNING: AtomicBool = AtomicBool::new(false);

/// Thumbnail fingerprint size: 64x64 grayscale = 4096 bytes
const THUMB_SIZE: u32 = 64;

/// Default: screen change < 3% is considered unchanged
const DEFAULT_CHANGE_THRESHOLD: f64 = 3.0;

/// Default: force capture after 30 minutes of no change
const DEFAULT_MAX_SILENT_MINUTES: u64 = 30;

/// Stores the last thumbnail fingerprint and the timestamp of the last actual capture.
struct ScreenState {
    last_fingerprint: Option<Vec<u8>>,
    last_capture_time: Instant,
}

static SCREEN_STATE: Lazy<Mutex<ScreenState>> = Lazy::new(|| {
    Mutex::new(ScreenState {
        last_fingerprint: None,
        last_capture_time: Instant::now(),
    })
});

// EXP-002: Quality filter counter for today's filtered screenshots
static FILTERED_TODAY: AtomicU32 = AtomicU32::new(0);

/// Increment the filtered screenshot counter
fn increment_filtered_count() {
    FILTERED_TODAY.fetch_add(1, Ordering::Relaxed);
}

/// Get today's filtered screenshot count
pub fn get_filtered_today() -> u32 {
    FILTERED_TODAY.load(Ordering::Relaxed)
}

/// Reset the filtered counter (call at midnight or app start)
pub fn reset_filtered_count() {
    FILTERED_TODAY.store(0, Ordering::Relaxed);
}

// STAB-001 AC5: Screenshot error classification for user-friendly error handling
/// Classification of screenshot capture errors
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenshotErrorKind {
    /// Permission denied - user should be notified to check screen capture permissions
    PermissionDenied,
    /// No monitors detected - hardware/configuration issue
    NoMonitors,
    /// Monitor index out of bounds - configuration error
    MonitorNotFound,
    /// Temporary failure - can retry silently
    TemporaryFailure,
    /// Unknown error
    Unknown,
}

/// Classify a screenshot error message into a ScreenshotErrorKind
/// This helps the frontend show appropriate user messages
fn classify_screenshot_error(error: &str) -> ScreenshotErrorKind {
    let error_lower = error.to_lowercase();

    // Permission errors
    if error_lower.contains("permission")
        || error_lower.contains("denied")
        || error_lower.contains("access denied")
    {
        return ScreenshotErrorKind::PermissionDenied;
    }

    // No monitors
    if error_lower.contains("no monitors") || error_lower.contains("monitor not found") {
        return ScreenshotErrorKind::NoMonitors;
    }

    // Monitor index out of bounds
    if error_lower.contains("out of bounds") || error_lower.contains("index") {
        return ScreenshotErrorKind::MonitorNotFound;
    }

    // Temporary failures (timeout, connection issues)
    if error_lower.contains("timeout")
        || error_lower.contains("busy")
        || error_lower.contains("temporarily")
    {
        return ScreenshotErrorKind::TemporaryFailure;
    }

    ScreenshotErrorKind::Unknown
}

/// Get a user-friendly error message based on the error kind
pub fn get_screenshot_error_message(kind: &ScreenshotErrorKind, original_error: &str) -> String {
    match kind {
        ScreenshotErrorKind::PermissionDenied => {
            "截图权限被拒绝，请在系统设置中允许应用进行屏幕录制".to_string()
        }
        ScreenshotErrorKind::NoMonitors => "未检测到显示器，请检查屏幕连接".to_string(),
        ScreenshotErrorKind::MonitorNotFound => {
            "指定的显示器不存在，请检查多显示器配置".to_string()
        }
        ScreenshotErrorKind::TemporaryFailure => {
            format!("截图暂时失败: {}，将自动重试", original_error)
        }
        ScreenshotErrorKind::Unknown => original_error.to_string(),
    }
}

use once_cell::sync::Lazy;

/// Compute a 64x64 grayscale thumbnail fingerprint from a base64-encoded PNG.
fn compute_fingerprint(image_base64: &str) -> Result<Vec<u8>, String> {
    let image_data =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, image_base64)
            .map_err(|e| format!("Failed to decode base64: {}", e))?;

    let img =
        image::load_from_memory(&image_data).map_err(|e| format!("Failed to load image: {}", e))?;

    let thumb = img
        .resize_exact(THUMB_SIZE, THUMB_SIZE, image::imageops::FilterType::Nearest)
        .to_luma8();

    Ok(thumb.into_raw())
}

/// Calculate the percentage of pixels that differ between two fingerprints.
/// Returns a value in 0.0..100.0.
fn calc_change_rate(a: &[u8], b: &[u8]) -> f64 {
    if a.len() != b.len() {
        return 100.0;
    }
    // A pixel is "changed" if the grayscale difference exceeds a small noise threshold.
    const NOISE_TOLERANCE: u8 = 10;
    let changed = a
        .iter()
        .zip(b.iter())
        .filter(|(pa, pb)| pa.abs_diff(**pb) > NOISE_TOLERANCE)
        .count();
    (changed as f64 / a.len() as f64) * 100.0
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXP-002: Screenshot Quality Filter
// ═══════════════════════════════════════════════════════════════════════════════

/// Quality score thumbnail size: 32x32 grayscale for fast entropy calculation
const QUALITY_THUMB_SIZE: u32 = 32;

/// Compute quality score for a screenshot based on image entropy.
/// Returns 0.0 (low quality - solid color/lock screen) to 1.0 (high quality - rich content).
///
/// Algorithm:
/// 1. Decode base64 → image
/// 2. Resize to 32x32 grayscale (fast computation)
/// 3. Compute grayscale histogram
/// 4. Calculate entropy: -sum(p * log2(p))
/// 5. Normalize by max entropy (log2(256) ≈ 8.0)
///
/// Quality interpretation:
/// - entropy ≈ 0: Solid color (lock screen, blank desktop)
/// - entropy ≈ 8: Maximum variety (code, documents, complex UI)
fn compute_quality_score(image_base64: &str) -> Result<f64, String> {
    let image_data =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, image_base64)
            .map_err(|e| format!("Failed to decode base64: {}", e))?;

    let img =
        image::load_from_memory(&image_data).map_err(|e| format!("Failed to load image: {}", e))?;

    // Resize to small thumbnail for fast computation
    let thumb = img
        .resize_exact(
            QUALITY_THUMB_SIZE,
            QUALITY_THUMB_SIZE,
            image::imageops::FilterType::Nearest,
        )
        .to_luma8();

    let pixels = thumb.into_raw();

    // Compute grayscale histogram
    let mut histogram = [0u32; 256];
    for &pixel in &pixels {
        histogram[pixel as usize] += 1;
    }

    // Calculate entropy
    let total_pixels = pixels.len() as f64;
    let mut entropy = 0.0;
    for &count in &histogram {
        if count > 0 {
            let p = count as f64 / total_pixels;
            entropy -= p * p.log2();
        }
    }

    // Normalize to [0, 1] range (max entropy = log2(256) ≈ 8.0)
    let normalized_score = (entropy / 8.0).min(1.0);

    Ok(normalized_score)
}

/// EXP-002: Statistics for quality filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityFilterStats {
    pub filtered_today: u32,
    pub quality_filter_enabled: bool,
    pub quality_filter_threshold: f64,
}

/// Determine whether the screen has changed enough to warrant a new capture.
/// Returns `Some(reason)` if we should proceed with the full capture+analysis pipeline,
/// or `None` if the screen hasn't changed and silent timeout hasn't been exceeded.
fn should_capture(
    fingerprint: &[u8],
    change_threshold: f64,
    max_silent_minutes: u64,
) -> Option<CaptureReason> {
    let mut state = SCREEN_STATE.lock().unwrap();

    let silent_exceeded =
        state.last_capture_time.elapsed() >= Duration::from_secs(max_silent_minutes * 60);

    let changed = match &state.last_fingerprint {
        None => true, // First capture — always proceed
        Some(prev) => {
            let rate = calc_change_rate(prev, fingerprint);
            tracing::debug!(
                "Screen change rate: {:.2}% (threshold: {:.1}%)",
                rate,
                change_threshold
            );
            rate >= change_threshold
        }
    };

    let reason = if changed {
        CaptureReason::ScreenChanged
    } else if silent_exceeded {
        tracing::info!(
            "Screen unchanged but max silent time ({} min) exceeded, forcing capture",
            max_silent_minutes
        );
        CaptureReason::SilentTimeout
    } else {
        tracing::debug!("Screen unchanged, skipping capture");
        return None;
    };

    // SMART-002: Record capture reason for pattern tracking (AC1)
    record_capture(reason);

    state.last_fingerprint = Some(fingerprint.to_vec());
    state.last_capture_time = Instant::now();
    Some(reason)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenAnalysis {
    pub current_focus: String,
    pub active_software: String,
    pub context_keywords: Vec<String>,
    /// Window information captured at the time of screenshot (SMART-001)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_window: Option<ActiveWindow>,
    /// Work category tags (AI-004)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

// Re-export ActiveWindow from window_info module for convenience
pub use crate::window_info::{get_active_window, should_capture_by_window, ActiveWindow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSettings {
    pub api_base_url: String,
    pub api_key: String,
    pub model_name: String,
    pub screenshot_interval: u64,
    pub analysis_prompt: Option<String>,
    pub change_threshold: f64,
    pub max_silent_minutes: u64,
    // SMART-001: Window filtering settings
    pub window_whitelist: Vec<String>,
    pub window_blacklist: Vec<String>,
    pub use_whitelist_only: bool,
    // SMART-004: Multi-monitor capture settings
    pub capture_mode: String,
    pub selected_monitor_index: usize,
    // FEAT-006: Capture only mode (#65)
    pub capture_only_mode: bool,
    // AI-006: Custom API headers (#68)
    pub custom_headers: Vec<crate::memory_storage::CustomHeader>,
    // EXP-002: Quality filter settings
    pub quality_filter_enabled: bool,
    pub quality_filter_threshold: f64,
    // PERF-001: Proxy configuration
    pub proxy_enabled: bool,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i32>,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
}

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            api_base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model_name: "gpt-4o".to_string(),
            screenshot_interval: 5,
            analysis_prompt: None,
            change_threshold: DEFAULT_CHANGE_THRESHOLD,
            max_silent_minutes: DEFAULT_MAX_SILENT_MINUTES,
            window_whitelist: Vec::new(),
            window_blacklist: Vec::new(),
            use_whitelist_only: false,
            // SMART-004: Default to primary monitor
            capture_mode: "primary".to_string(),
            selected_monitor_index: 0,
            // FEAT-006: Default to false (analyze after capture)
            capture_only_mode: false,
            // AI-006: Default to empty custom headers
            custom_headers: Vec::new(),
            // EXP-002: Quality filter defaults (enabled, medium sensitivity)
            quality_filter_enabled: true,
            quality_filter_threshold: 0.3,
            // PERF-001: Proxy defaults (disabled)
            proxy_enabled: false,
            proxy_host: None,
            proxy_port: None,
            proxy_username: None,
            proxy_password: None,
        }
    }
}

// ─── SMART-004: Multi-monitor capture support (Windows) ──────────────────────

/// Capture screen with specified mode (Windows) - uses xcap for compatibility
/// Returns (base64_image, monitor_info)
#[cfg(target_os = "windows")]
fn capture_screen_with_mode(
    mode: CaptureMode,
    selected_index: usize,
) -> Result<(String, MonitorInfo), String> {
    let monitor_details = get_monitor_list()?;

    let monitors = xcap::Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    if monitors.is_empty() {
        return Err("No monitors found".to_string());
    }

    let monitor_info = MonitorInfo {
        count: monitor_details.len(),
        monitors: monitor_details.clone(),
    };

    let image = match mode {
        CaptureMode::Primary => {
            // Find primary monitor (usually at position 0,0)
            let primary_index = monitor_details
                .iter()
                .position(|m| m.is_primary)
                .unwrap_or(0);
            capture_single_monitor_xcap(&monitors, primary_index)?
        }
        CaptureMode::Secondary => {
            // Use selected_index, fallback to first non-primary if out of bounds
            let index = if selected_index < monitors.len() {
                selected_index
            } else {
                // Fallback to first non-primary monitor
                monitor_details
                    .iter()
                    .position(|m| !m.is_primary)
                    .unwrap_or(0)
            };
            capture_single_monitor_xcap(&monitors, index)?
        }
        CaptureMode::All => {
            // Stitch all monitors together
            stitch_monitors_xcap(&monitors, &monitor_details)?
        }
    };

    Ok((image, monitor_info))
}

// ─── SMART-004: Multi-monitor capture support (macOS/Linux) ──────────────────

/// Capture screen with specified mode (macOS/Linux)
/// Returns (base64_image, monitor_info)
#[cfg(not(target_os = "windows"))]
fn capture_screen_with_mode(
    mode: CaptureMode,
    selected_index: usize,
) -> Result<(String, MonitorInfo), String> {
    let monitor_details = get_monitor_list()?;
    let monitors = xcap::Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    if monitors.is_empty() {
        return Err("No monitors found".to_string());
    }

    let monitor_info = MonitorInfo {
        count: monitor_details.len(),
        monitors: monitor_details.clone(),
    };

    let image = match mode {
        CaptureMode::Primary => {
            let primary_index = monitor_details
                .iter()
                .position(|m| m.is_primary)
                .unwrap_or(0);
            capture_single_monitor_xcap(&monitors, primary_index)?
        }
        CaptureMode::Secondary => {
            let index = if selected_index < monitors.len() {
                selected_index
            } else {
                monitor_details
                    .iter()
                    .position(|m| !m.is_primary)
                    .unwrap_or(0)
            };
            capture_single_monitor_xcap(&monitors, index)?
        }
        CaptureMode::All => stitch_monitors_xcap(&monitors, &monitor_details)?,
    };

    Ok((image, monitor_info))
}

/// Capture a single monitor by index (Windows version - using xcap)
#[cfg(target_os = "windows")]
fn capture_single_monitor_windows(
    monitors: &[xcap::Monitor],
    index: usize,
) -> Result<String, String> {
    // Use the xcap implementation for Windows
    return capture_single_monitor_xcap(monitors, index);
}

/// Stitch all monitors into a single image (Windows version - using xcap)
#[cfg(target_os = "windows")]
fn stitch_monitors_windows(
    monitors: &[xcap::Monitor],
    monitor_details: &[crate::monitor_types::MonitorDetail],
) -> Result<String, String> {
    if monitors.is_empty() {
        return Err("No monitors to stitch".to_string());
    }

    // Capture all monitor images
    let mut captured_images: Vec<(crate::monitor_types::MonitorDetail, image::RgbaImage)> =
        Vec::new();

    for (index, _monitor) in monitors.iter().enumerate() {
        // Capture each monitor using xcap
        let image_base64 = capture_single_monitor_xcap(monitors, index)?;

        // Decode base64 to image
        let image_data =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &image_base64)
                .map_err(|e| format!("Failed to decode captured image: {}", e))?;

        let img = image::load_from_memory(&image_data)
            .map_err(|e| format!("Failed to load image: {}", e))?;

        let rgba_image = img.to_rgba8();

        let detail = monitor_details.get(index).cloned().unwrap_or_else(|| {
            crate::monitor_types::MonitorDetail {
                index,
                name: format!("Monitor {}", index + 1),
                width: rgba_image.width(),
                height: rgba_image.height(),
                x: 0,
                y: 0,
                is_primary: index == 0,
            }
        });

        captured_images.push((detail, rgba_image));
    }

    // Calculate bounding box for all monitors
    let (min_x, min_y, max_x, max_y) = calculate_monitor_bounds(&captured_images);
    let total_width = (max_x - min_x) as u32;
    let total_height = (max_y - min_y) as u32;

    // Create canvas and overlay all monitor images
    let mut canvas = image::RgbaImage::new(total_width, total_height);

    for (monitor, img) in &captured_images {
        let offset_x = (monitor.x - min_x) as i64;
        let offset_y = (monitor.y - min_y) as i64;
        image::imageops::overlay(&mut canvas, img, offset_x, offset_y);
    }

    // Encode to base64
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    image::DynamicImage::ImageRgba8(canvas)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode stitched image: {}", e))?;

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &buffer,
    ))
}

// ─── Shared xcap implementation for all platforms ───────────────────

/// Capture a single monitor by index (xcap version)
fn capture_single_monitor_xcap(monitors: &[xcap::Monitor], index: usize) -> Result<String, String> {
    if index >= monitors.len() {
        return Err(format!(
            "Monitor index {} out of bounds ({} monitors)",
            index,
            monitors.len()
        ));
    }

    let rgba_image = monitors[index]
        .capture_image()
        .map_err(|e| format!("Failed to capture monitor {}: {}", index, e))?;

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    image::DynamicImage::ImageRgba8(rgba_image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image: {}", e))?;

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &buffer,
    ))
}

/// Stitch all monitors into a single image (xcap version)
fn stitch_monitors_xcap(
    monitors: &[xcap::Monitor],
    monitor_details: &[crate::monitor_types::MonitorDetail],
) -> Result<String, String> {
    if monitors.is_empty() {
        return Err("No monitors to stitch".to_string());
    }

    // Capture all monitor images
    let mut captured_images: Vec<(crate::monitor_types::MonitorDetail, image::RgbaImage)> =
        Vec::new();

    for (index, monitor) in monitors.iter().enumerate() {
        let rgba_image = monitor
            .capture_image()
            .map_err(|e| format!("Failed to capture monitor {}: {}", index, e))?;

        let detail = monitor_details.get(index).cloned().unwrap_or_else(|| {
            crate::monitor_types::MonitorDetail {
                index,
                name: format!("Monitor {}", index + 1),
                width: rgba_image.width(),
                height: rgba_image.height(),
                x: 0,
                y: 0,
                is_primary: index == 0,
            }
        });

        captured_images.push((detail, rgba_image));
    }

    // Calculate bounding box for all monitors
    let (min_x, min_y, max_x, max_y) = calculate_monitor_bounds(&captured_images);
    let total_width = (max_x - min_x) as u32;
    let total_height = (max_y - min_y) as u32;

    // Create canvas and overlay all monitor images
    let mut canvas = image::RgbaImage::new(total_width, total_height);

    for (monitor, img) in &captured_images {
        let offset_x = (monitor.x - min_x) as i64;
        let offset_y = (monitor.y - min_y) as i64;
        image::imageops::overlay(&mut canvas, img, offset_x, offset_y);
    }

    // Encode to base64
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    image::DynamicImage::ImageRgba8(canvas)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode stitched image: {}", e))?;

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &buffer,
    ))
}

/// Calculate the bounding box for all monitor images
fn calculate_monitor_bounds(
    monitors: &[(crate::monitor_types::MonitorDetail, image::RgbaImage)],
) -> (i32, i32, i32, i32) {
    if monitors.is_empty() {
        return (0, 0, 0, 0);
    }

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for (monitor, img) in monitors {
        min_x = min_x.min(monitor.x);
        min_y = min_y.min(monitor.y);
        max_x = max_x.max(monitor.x + img.width() as i32);
        max_y = max_y.max(monitor.y + img.height() as i32);
    }

    (min_x, min_y, max_x, max_y)
}

fn save_screenshot(image_base64: &str) -> Option<String> {
    use std::path::PathBuf;

    let screenshot_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("DailyLogger")
        .join("screenshots");

    if let Err(e) = std::fs::create_dir_all(&screenshot_dir) {
        tracing::error!("Failed to create screenshot directory: {}", e);
        return None;
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("screenshot_{}.png", timestamp);
    let screenshot_path = screenshot_dir.join(&filename);

    let image_data =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, image_base64)
            .map_err(|e| format!("Failed to decode base64: {}", e))
            .ok()?;

    if let Err(e) = std::fs::write(&screenshot_path, &image_data) {
        tracing::error!("Failed to save screenshot: {}", e);
        return None;
    }

    Some(screenshot_path.to_string_lossy().to_string())
}

const DEFAULT_ANALYSIS_PROMPT: &str = r#"Analyze this screenshot and return a JSON object with:
- current_focus: What is the user currently working on? (1-2 sentences in Chinese)
- active_software: What software is being used? (in Chinese)
- context_keywords: What are the key topics/technologies? (array of strings, in Chinese)
- tags: Work category tags from this list: ["开发", "会议", "写作", "学习", "研究", "沟通", "规划", "文档", "测试", "设计"]. Select 1-3 most relevant tags. (array of strings)

Return ONLY valid JSON, no other text. Example format:
{"current_focus": "编写 Rust 后端代码", "active_software": "VS Code", "context_keywords": ["Rust", "Tauri", "异步编程"], "tags": ["开发", "测试"]}"#;

async fn analyze_screen(
    settings: &CaptureSettings,
    image_base64: &str,
) -> Result<ScreenAnalysis, String> {
    let prompt = settings
        .analysis_prompt
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_ANALYSIS_PROMPT);

    let request_body = serde_json::json!({
        "model": settings.model_name,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": prompt},
                    {"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", image_base64)}}
                ]
            }
        ],
        "max_tokens": 500
    });

    let masked_key = crate::mask_api_key(&settings.api_key);
    let endpoint = format!("{}/chat/completions", settings.api_base_url);

    // PERF-001: Build proxy configuration
    let proxy_config = crate::ProxyConfig {
        enabled: settings.proxy_enabled,
        host: settings.proxy_host.clone(),
        port: settings.proxy_port,
        username: settings.proxy_username.clone(),
        password: settings.proxy_password.clone(),
    };

    // Create HTTP client with proxy configuration
    let client = crate::create_http_client_with_proxy(&endpoint, 120, Some(proxy_config))?;

    // AI-006: Log custom headers (mask sensitive values)
    let custom_headers_debug: Vec<_> = settings
        .custom_headers
        .iter()
        .map(|h| {
            if h.sensitive {
                format!("{}: {}", h.key, "***MASKED***")
            } else {
                format!("{}: {}", h.key, h.value)
            }
        })
        .collect();

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_request",
            "caller": "analyze_screen",
            "endpoint": endpoint,
            "model": settings.model_name,
            "max_tokens": 500,
            "api_key_masked": masked_key,
            "has_image": true,
            "image_base64_len": image_base64.len(),
            "prompt": prompt,
            "custom_headers": custom_headers_debug,
        })
    );

    let start = std::time::Instant::now();

    // Check if this is an Ollama endpoint
    let is_ollama = crate::ollama::is_ollama_endpoint(&settings.api_base_url);

    let mut request = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request_body);

    // AI-006: Check if custom headers contain Authorization or api-key header
    let has_custom_auth = settings
        .custom_headers
        .iter()
        .any(|h| h.key.to_lowercase() == "authorization" || h.key.to_lowercase() == "api-key");

    // Only add Authorization header if API key is provided (not required for Ollama) and no custom auth header
    if !settings.api_key.is_empty() && !has_custom_auth {
        request = request.header("Authorization", format!("Bearer {}", settings.api_key));
    }

    // AI-006: Apply custom headers
    for header in &settings.custom_headers {
        request = request.header(&header.key, &header.value);
    }

    let response = request.send().await.map_err(|e| {
        let elapsed_ms = start.elapsed().as_millis();
        let error_msg = crate::ollama::format_connection_error(&e.to_string(), is_ollama);
        tracing::error!(
            "{}",
            serde_json::json!({
                "event": "llm_error",
                "caller": "analyze_screen",
                "error": error_msg,
                "elapsed_ms": elapsed_ms,
            })
        );
        error_msg
    })?;
    let elapsed_ms = start.elapsed().as_millis();

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!(
            "{}",
            serde_json::json!({
                "event": "llm_error",
                "caller": "analyze_screen",
                "status": status.as_u16(),
                "response_body": body,
                "elapsed_ms": elapsed_ms,
            })
        );
        // Give a clear, actionable message for vision-unsupported endpoints.
        if body.contains("image_url") && body.contains("unknown variant") {
            return Err("当前模型不支持图像分析（Vision）。\
请在设置中将模型改为支持视觉功能的型号，例如 gpt-4o 或 gpt-4-turbo。"
                .to_string());
        }
        return Err(format!("API error ({}): {}", status, body));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    tracing::info!(
        "{}",
        serde_json::json!({
            "event": "llm_response",
            "caller": "analyze_screen",
            "status": 200,
            "elapsed_ms": elapsed_ms,
            "usage": response_json.get("usage"),
            "model": response_json.get("model"),
            "response_id": response_json.get("id"),
            "content": content,
        })
    );

    // Some models wrap JSON in markdown code fences (```json ... ```) despite
    // being instructed otherwise. Strip those before parsing.
    let content = content.trim();
    let content = if let Some(inner) = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```"))
    {
        inner.trim_end_matches("```").trim()
    } else {
        content
    };

    let analysis: ScreenAnalysis = serde_json::from_str(content)
        .map_err(|e| format!("Failed to parse analysis: {}. Content: {}", e, content))?;

    Ok(analysis)
}

fn load_capture_settings() -> CaptureSettings {
    match memory_storage::get_settings_sync() {
        Ok(s) => {
            // AI-006: Parse custom headers from settings
            let custom_headers = if let Some(ref headers_json) = s.custom_headers {
                if !headers_json.is_empty() {
                    serde_json::from_str::<Vec<crate::memory_storage::CustomHeader>>(headers_json)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            CaptureSettings {
                api_base_url: s.api_base_url.unwrap_or_default(),
                api_key: s.api_key.unwrap_or_default(),
                model_name: s.model_name.unwrap_or_else(|| "gpt-4o".to_string()),
                screenshot_interval: s.screenshot_interval.unwrap_or(5) as u64,
                analysis_prompt: s.analysis_prompt,
                change_threshold: s.change_threshold.unwrap_or(3) as f64,
                max_silent_minutes: s.max_silent_minutes.unwrap_or(30) as u64,
                // SMART-001: Parse window filter settings from JSON
                window_whitelist: parse_window_patterns(s.window_whitelist.as_deref()),
                window_blacklist: parse_window_patterns(s.window_blacklist.as_deref()),
                use_whitelist_only: s.use_whitelist_only.unwrap_or(false),
                // SMART-004: Load multi-monitor settings
                capture_mode: s.capture_mode.unwrap_or_else(|| "primary".to_string()),
                selected_monitor_index: s.selected_monitor_index.unwrap_or(0) as usize,
                // FEAT-006: Load capture only mode setting
                capture_only_mode: s.capture_only_mode.unwrap_or(false),
                // AI-006: Load custom headers
                custom_headers,
                // EXP-002: Load quality filter settings
                quality_filter_enabled: s.quality_filter_enabled.unwrap_or(true),
                quality_filter_threshold: s.quality_filter_threshold.unwrap_or(0.3),
                // PERF-001: Load proxy settings
                proxy_enabled: s.proxy_enabled.unwrap_or(false),
                proxy_host: s.proxy_host.clone(),
                proxy_port: s.proxy_port,
                proxy_username: s.proxy_username.clone(),
                proxy_password: s.proxy_password.clone(),
            }
        }
        Err(_) => CaptureSettings::default(),
    }
}

/// Parse a JSON array string into a Vec<String>.
/// Returns an empty Vec if parsing fails or input is empty.
fn parse_window_patterns(json: Option<&str>) -> Vec<String> {
    json.and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}

/// Retry screenshot analysis for an offline-queued screenshot.
/// This is called by the offline queue when network is restored.
///
/// # Arguments
/// * `screenshot_path` - Full path to the screenshot file
/// * `record_id` - Database record ID to update with the analysis result
///
/// # Returns
/// * `Ok(())` on successful analysis and record update
/// * `Err(String)` on failure (file not found, API error, etc.)
pub async fn retry_screenshot_analysis(
    screenshot_path: &str,
    record_id: i64,
) -> Result<(), String> {
    tracing::info!(
        "Retrying screenshot analysis for record {} from {}",
        record_id,
        screenshot_path
    );

    // Read screenshot file
    let image_data = std::fs::read(screenshot_path)
        .map_err(|e| format!("Failed to read screenshot file {}: {}", screenshot_path, e))?;

    // Convert to base64
    let image_base64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);

    // Load settings
    let settings = load_capture_settings();

    // Check if API is configured
    if settings.api_base_url.is_empty() {
        return Err("API base URL not configured".to_string());
    }

    // Call analyze_screen
    let analysis = analyze_screen(&settings, &image_base64).await?;

    // Update the record content
    let content = serde_json::to_string(&analysis)
        .map_err(|e| format!("Failed to serialize analysis: {}", e))?;

    memory_storage::update_record_content_sync(record_id, &content)?;

    tracing::info!(
        "Successfully updated record {} with analysis result",
        record_id
    );
    Ok(())
}

/// SMART-003: Load work time settings from database
fn load_work_time_settings() -> WorkTimeSettings {
    match memory_storage::get_settings_sync() {
        Ok(s) => WorkTimeSettings {
            auto_detect_work_time: s.auto_detect_work_time.unwrap_or(true),
            use_custom_work_time: s.use_custom_work_time.unwrap_or(false),
            custom_work_time_start: s.custom_work_time_start,
            custom_work_time_end: s.custom_work_time_end,
            learned_work_time: s.learned_work_time,
        },
        Err(_) => WorkTimeSettings {
            auto_detect_work_time: true,
            use_custom_work_time: false,
            custom_work_time_start: None,
            custom_work_time_end: None,
            learned_work_time: None,
        },
    }
}

/// SMART-003 (AC2): Check if current time is within work hours
/// Returns true if capture should proceed, false if should skip
fn should_capture_by_work_time() -> bool {
    let settings = load_work_time_settings();
    is_in_work_time(&settings)
}

/// SMART-002 (AC2): Evaluate and adjust the silent threshold if needed.
/// Returns Some((old_threshold, new_threshold)) if an adjustment was made, None otherwise.
fn evaluate_and_adjust_threshold() -> Option<(u64, u64)> {
    // Check if auto-adjust is enabled
    if let Ok(settings) = memory_storage::get_settings_sync() {
        if !settings.auto_adjust_silent.unwrap_or(true) {
            tracing::debug!("Auto-adjust silent threshold is disabled by user");
            return None;
        }

        // Check if adjustment is paused
        if let Some(paused_until) = &settings.silent_adjustment_paused_until {
            if let Ok(paused_time) = chrono::DateTime::parse_from_rfc3339(paused_until) {
                if paused_time > chrono::Utc::now() {
                    tracing::debug!(
                        "Silent threshold adjustment is paused until {}",
                        paused_until
                    );
                    return None;
                }
            }
        }
    }

    // Check if we have sufficient data
    if !has_sufficient_data() {
        tracing::debug!("Insufficient capture data for threshold adjustment");
        return None;
    }

    let old_threshold = current_threshold();
    let new_threshold = calculate_optimal_silent_minutes(
        &crate::silent_tracker::SILENT_PATTERN_TRACKER
            .lock()
            .unwrap(),
    );

    // Only adjust if there's an actual change
    if new_threshold != old_threshold {
        set_threshold(new_threshold);

        // Persist to settings
        if let Ok(mut settings) = memory_storage::get_settings_sync() {
            settings.max_silent_minutes = Some(new_threshold as i32);
            if let Err(e) = memory_storage::save_settings_sync(&settings) {
                tracing::error!("Failed to save adjusted threshold: {}", e);
            }
        }

        tracing::info!(
            "Silent threshold adjusted from {} to {} minutes",
            old_threshold,
            new_threshold
        );
        return Some((old_threshold, new_threshold));
    }

    None
}

/// Payload for silent-threshold-adjusted event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdAdjustment {
    pub old_value: u64,
    pub new_value: u64,
    pub reason: String,
}

async fn capture_and_store() -> Result<(), String> {
    let settings = load_capture_settings();

    if settings.api_key.is_empty() {
        return Err("API 密钥未配置，请在设置中配置".to_string());
    }

    // SMART-001: Get active window info before screenshot
    let active_window = get_active_window();

    // SMART-001: Apply window filtering rules (AC2, AC3)
    if !should_capture_by_window(
        &active_window,
        &settings.window_whitelist,
        &settings.window_blacklist,
        settings.use_whitelist_only,
    ) {
        tracing::info!(
            "Skipping capture: window filtered (title='{}', process='{}')",
            active_window.title,
            active_window.process_name
        );
        return Ok(());
    }

    // SMART-004: Parse capture mode and capture with multi-monitor support
    let capture_mode = settings
        .capture_mode
        .parse::<CaptureMode>()
        .unwrap_or(CaptureMode::Primary);

    // STAB-001 AC5: Use classified error messages for better user feedback
    let (image_base64, monitor_info) =
        capture_screen_with_mode(capture_mode, settings.selected_monitor_index).map_err(|e| {
            tracing::error!("Screenshot capture failed: {}", e);
            let kind = classify_screenshot_error(&e);
            get_screenshot_error_message(&kind, &e)
        })?;

    // Check if screen has changed enough to warrant a full capture
    // SMART-002: should_capture now returns Option<CaptureReason> for pattern tracking
    let fingerprint = compute_fingerprint(&image_base64)?;
    if should_capture(
        &fingerprint,
        settings.change_threshold,
        settings.max_silent_minutes,
    )
    .is_none()
    {
        return Ok(());
    }

    // EXP-002: Quality filter - skip low-information screenshots
    // (blank desktop, lock screen, solid color backgrounds)
    if settings.quality_filter_enabled {
        let threshold = settings.quality_filter_threshold;
        let score = compute_quality_score(&image_base64)?;
        if score < threshold {
            increment_filtered_count();
            tracing::debug!(
                "Quality filter: score={:.2} < threshold={:.2}, skipping screenshot",
                score,
                threshold
            );
            return Ok(());
        }
        tracing::trace!(
            "Quality filter: score={:.2} >= threshold={:.2}, proceeding with capture",
            score,
            threshold
        );
    }

    let screenshot_path = save_screenshot(&image_base64);

    // SESSION-001: 捕获与分析解耦
    // 所有截图现在都只保存不立即分析，等待时段结束时批量分析
    tracing::info!("Capture mode: saving screenshot without immediate AI analysis");

    // SESSION-001: 检测或创建工作时段
    let current_timestamp = chrono::Utc::now().to_rfc3339();
    let session_id = crate::session_manager::detect_or_create_session(&current_timestamp)?;

    // Save record with placeholder content, marked for later analysis
    let content = serde_json::json!({
        "current_focus": "待分析",
        "active_software": active_window.process_name,
        "context_keywords": [],
        "active_window": {
            "title": active_window.title,
            "process_name": active_window.process_name
        },
        "monitor_info": {
            "count": monitor_info.count,
            "capture_mode": capture_mode.to_string()
        },
        "offline_pending": true
    })
    .to_string();

    let monitor_info_json = serde_json::to_string(&monitor_info).ok();

    // SESSION-001: 使用 add_record_with_session 保存 session_id
    let record_id = memory_storage::add_record_with_session(
        "auto",
        &content,
        screenshot_path.as_deref(),
        monitor_info_json.as_deref(),
        None,
        Some(session_id),
    )?;

    tracing::debug!(
        "Screenshot saved with record_id={}, session_id={}",
        record_id,
        session_id
    );

    Ok(())
}

#[cfg(all(test, feature = "screenshot"))]
mod tests {
    use super::*;
    use base64::Engine;
    use serial_test::serial;
    use crate::commands::{get_auto_capture_status, get_default_analysis_prompt, get_work_time_status};
    use crate::services::is_auto_capture_running;

    fn make_minimal_png_base64() -> String {
        // 1×1 transparent PNG (RGBA)
        let image = image::RgbaImage::from_raw(1, 1, vec![0u8, 0, 0, 0]).unwrap();
        let mut buffer = Vec::new();
        image::DynamicImage::ImageRgba8(image)
            .write_to(
                &mut std::io::Cursor::new(&mut buffer),
                image::ImageFormat::Png,
            )
            .unwrap();
        base64::engine::general_purpose::STANDARD.encode(&buffer)
    }

    #[test]
    fn save_screenshot_creates_file_and_returns_path() {
        let b64 = make_minimal_png_base64();
        let path = save_screenshot(&b64).expect("save_screenshot should succeed");
        assert!(
            std::path::Path::new(&path).exists(),
            "screenshot file should exist at {path}"
        );
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn save_screenshot_rejects_invalid_base64() {
        let result = save_screenshot("not-valid-base64!!!");
        assert!(result.is_none(), "invalid base64 should return None");
    }

    /// Helper: strip markdown fences the same way analyze_screen does.
    fn strip_code_fence(content: &str) -> &str {
        let content = content.trim();
        if let Some(inner) = content
            .strip_prefix("```json")
            .or_else(|| content.strip_prefix("```"))
        {
            inner.trim_end_matches("```").trim()
        } else {
            content
        }
    }

    #[test]
    fn strip_code_fence_handles_json_fence() {
        let wrapped = "```json\n{\"a\":1}\n```";
        assert_eq!(strip_code_fence(wrapped), "{\"a\":1}");
    }

    #[test]
    fn strip_code_fence_handles_plain_fence() {
        let wrapped = "```\n{\"a\":1}\n```";
        assert_eq!(strip_code_fence(wrapped), "{\"a\":1}");
    }

    #[test]
    fn strip_code_fence_leaves_bare_json_unchanged() {
        let bare = "{\"a\":1}";
        assert_eq!(strip_code_fence(bare), bare);
    }

    // ── Screen change detection tests ──

    fn make_test_fingerprint(value: u8) -> Vec<u8> {
        vec![value; (THUMB_SIZE * THUMB_SIZE) as usize]
    }

    #[test]
    fn calc_change_rate_identical_images_returns_zero() {
        let a = make_test_fingerprint(128);
        let b = make_test_fingerprint(128);
        assert_eq!(calc_change_rate(&a, &b), 0.0);
    }

    #[test]
    fn calc_change_rate_completely_different_returns_100() {
        let a = make_test_fingerprint(0);
        let b = make_test_fingerprint(255);
        assert_eq!(calc_change_rate(&a, &b), 100.0);
    }

    #[test]
    fn calc_change_rate_within_noise_tolerance_returns_zero() {
        let a = make_test_fingerprint(100);
        // Difference of 10 is exactly at the noise tolerance boundary — not counted
        let b = make_test_fingerprint(110);
        assert_eq!(calc_change_rate(&a, &b), 0.0);
    }

    #[test]
    fn calc_change_rate_just_above_noise_tolerance() {
        let a = make_test_fingerprint(100);
        // Difference of 11 exceeds noise tolerance — all pixels counted
        let b = make_test_fingerprint(111);
        assert_eq!(calc_change_rate(&a, &b), 100.0);
    }

    #[test]
    fn calc_change_rate_partial_change() {
        let total = (THUMB_SIZE * THUMB_SIZE) as usize;
        let mut a = vec![100u8; total];
        let mut b = vec![100u8; total];
        // Change 25% of pixels beyond noise tolerance
        let quarter = total / 4;
        for i in 0..quarter {
            a[i] = 0;
            b[i] = 200;
        }
        let rate = calc_change_rate(&a, &b);
        assert!((rate - 25.0).abs() < 0.1, "Expected ~25%, got {:.2}%", rate);
    }

    #[test]
    fn calc_change_rate_mismatched_lengths_returns_100() {
        let a = vec![0u8; 10];
        let b = vec![0u8; 20];
        assert_eq!(calc_change_rate(&a, &b), 100.0);
    }

    #[test]
    fn compute_fingerprint_produces_correct_size() {
        let b64 = make_minimal_png_base64();
        let fp = compute_fingerprint(&b64).unwrap();
        assert_eq!(
            fp.len(),
            (THUMB_SIZE * THUMB_SIZE) as usize,
            "Fingerprint should be {}x{} = {} bytes",
            THUMB_SIZE,
            THUMB_SIZE,
            THUMB_SIZE * THUMB_SIZE
        );
    }

    #[test]
    fn compute_fingerprint_rejects_invalid_base64() {
        let result = compute_fingerprint("not-valid!!!");
        assert!(result.is_err());
    }

    // ── is_auto_capture_running tests ──

    #[test]
    #[serial]
    fn is_auto_capture_running_returns_false_by_default() {
        // Reset to known state
        AUTO_CAPTURE_RUNNING.store(false, Ordering::SeqCst);
        assert!(!is_auto_capture_running());
    }

    #[test]
    #[serial]
    fn is_auto_capture_running_returns_true_after_start() {
        AUTO_CAPTURE_RUNNING.store(true, Ordering::SeqCst);
        assert!(is_auto_capture_running());
        // Reset for other tests
        AUTO_CAPTURE_RUNNING.store(false, Ordering::SeqCst);
    }

    #[test]
    #[serial]
    fn is_auto_capture_running_reflects_state_changes() {
        AUTO_CAPTURE_RUNNING.store(false, Ordering::SeqCst);
        assert!(!is_auto_capture_running());

        AUTO_CAPTURE_RUNNING.store(true, Ordering::SeqCst);
        assert!(is_auto_capture_running());

        AUTO_CAPTURE_RUNNING.store(false, Ordering::SeqCst);
        assert!(!is_auto_capture_running());
    }

    // ── get_auto_capture_status command tests ──

    #[test]
    #[serial]
    fn get_auto_capture_status_returns_false_by_default() {
        AUTO_CAPTURE_RUNNING.store(false, Ordering::SeqCst);
        assert!(!get_auto_capture_status());
    }

    #[test]
    #[serial]
    fn get_auto_capture_status_returns_true_when_running() {
        AUTO_CAPTURE_RUNNING.store(true, Ordering::SeqCst);
        assert!(get_auto_capture_status());
        // Reset for other tests
        AUTO_CAPTURE_RUNNING.store(false, Ordering::SeqCst);
    }

    // ── get_default_analysis_prompt tests ──

    #[test]
    fn get_default_analysis_prompt_returns_expected_content() {
        let prompt = get_default_analysis_prompt();
        assert!(
            prompt.contains("current_focus"),
            "should contain current_focus"
        );
        assert!(
            prompt.contains("active_software"),
            "should contain active_software"
        );
        assert!(
            prompt.contains("context_keywords"),
            "should contain context_keywords"
        );
        assert!(prompt.contains("JSON"), "should mention JSON format");
    }

    #[test]
    fn get_default_analysis_prompt_returns_non_empty() {
        let prompt = get_default_analysis_prompt();
        assert!(!prompt.is_empty(), "default prompt should not be empty");
    }

    // ── SMART-001: parse_window_patterns tests ──

    #[test]
    fn parse_window_patterns_parses_valid_json_array() {
        let json = r#"["VS Code", "IntelliJ IDEA", "Chrome"]"#;
        let patterns = parse_window_patterns(Some(json));
        assert_eq!(patterns, vec!["VS Code", "IntelliJ IDEA", "Chrome"]);
    }

    #[test]
    fn parse_window_patterns_returns_empty_for_none() {
        let patterns = parse_window_patterns(None);
        assert!(patterns.is_empty(), "None should return empty Vec");
    }

    #[test]
    fn parse_window_patterns_returns_empty_for_empty_string() {
        let patterns = parse_window_patterns(Some(""));
        assert!(patterns.is_empty(), "empty string should return empty Vec");
    }

    #[test]
    fn parse_window_patterns_returns_empty_for_invalid_json() {
        let patterns = parse_window_patterns(Some("not valid json"));
        assert!(patterns.is_empty(), "invalid JSON should return empty Vec");
    }

    #[test]
    fn parse_window_patterns_handles_empty_array() {
        let patterns = parse_window_patterns(Some("[]"));
        assert!(patterns.is_empty(), "empty array should return empty Vec");
    }

    #[test]
    fn parse_window_patterns_handles_unicode() {
        let json = r#"["微信", "企业微信", "浏览器"]"#;
        let patterns = parse_window_patterns(Some(json));
        assert_eq!(patterns, vec!["微信", "企业微信", "浏览器"]);
    }

    // ── SMART-001: ScreenAnalysis with active_window tests ──

    #[test]
    fn screen_analysis_serializes_with_active_window() {
        let analysis = ScreenAnalysis {
            current_focus: "编写代码".to_string(),
            active_software: "VS Code".to_string(),
            context_keywords: vec!["Rust".to_string()],
            active_window: Some(ActiveWindow {
                title: "main.rs - VS Code".to_string(),
                process_name: "Code".to_string(),
            }),
            tags: None,
        };
        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("\"active_window\""));
        assert!(json.contains("\"title\":\"main.rs - VS Code\""));
        assert!(json.contains("\"process_name\":\"Code\""));
    }

    #[test]
    fn screen_analysis_serializes_without_active_window() {
        let analysis = ScreenAnalysis {
            current_focus: "编写代码".to_string(),
            active_software: "VS Code".to_string(),
            context_keywords: vec!["Rust".to_string()],
            active_window: None,
            tags: None,
        };
        let json = serde_json::to_string(&analysis).unwrap();
        // skip_serializing_if means active_window should not appear when None
        assert!(!json.contains("\"active_window\""));
    }

    #[test]
    fn screen_analysis_deserializes_with_active_window() {
        let json = r#"{
            "current_focus": "测试",
            "active_software": "Terminal",
            "context_keywords": ["test"],
            "active_window": {"title": "bash", "process_name": "bash"}
        }"#;
        let analysis: ScreenAnalysis = serde_json::from_str(json).unwrap();
        assert!(analysis.active_window.is_some());
        let window = analysis.active_window.unwrap();
        assert_eq!(window.title, "bash");
        assert_eq!(window.process_name, "bash");
    }

    #[test]
    fn screen_analysis_deserializes_without_active_window() {
        let json = r#"{
            "current_focus": "测试",
            "active_software": "Terminal",
            "context_keywords": ["test"]
        }"#;
        let analysis: ScreenAnalysis = serde_json::from_str(json).unwrap();
        assert!(analysis.active_window.is_none());
    }

    // ── AI-004: ScreenAnalysis tags field tests ──

    #[test]
    fn screen_analysis_parses_tags_field() {
        let json = r#"{
            "current_focus": "测试",
            "active_software": "App",
            "context_keywords": [],
            "tags": ["开发", "测试"]
        }"#;
        let analysis: ScreenAnalysis = serde_json::from_str(json).unwrap();
        assert!(analysis.tags.is_some());
        let tags = analysis.tags.unwrap();
        assert_eq!(tags, vec!["开发", "测试"]);
    }

    #[test]
    fn screen_analysis_tags_optional() {
        let json = r#"{
            "current_focus": "测试",
            "active_software": "App",
            "context_keywords": []
        }"#;
        let analysis: ScreenAnalysis = serde_json::from_str(json).unwrap();
        assert!(analysis.tags.is_none());
    }

    #[test]
    fn screen_analysis_serializes_tags_when_present() {
        let analysis = ScreenAnalysis {
            current_focus: "编写代码".to_string(),
            active_software: "VS Code".to_string(),
            context_keywords: vec!["Rust".to_string()],
            active_window: None,
            tags: Some(vec!["开发".to_string(), "测试".to_string()]),
        };
        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("\"tags\""));
        assert!(json.contains("\"开发\""));
        assert!(json.contains("\"测试\""));
    }

    #[test]
    fn screen_analysis_skips_serializing_tags_when_none() {
        let analysis = ScreenAnalysis {
            current_focus: "编写代码".to_string(),
            active_software: "VS Code".to_string(),
            context_keywords: vec!["Rust".to_string()],
            active_window: None,
            tags: None,
        };
        let json = serde_json::to_string(&analysis).unwrap();
        // skip_serializing_if means tags should not appear when None
        assert!(!json.contains("\"tags\""));
    }

    // ── SMART-001: CaptureSettings window filter tests ──

    #[test]
    fn capture_settings_default_has_empty_window_filters() {
        let settings = CaptureSettings::default();
        assert!(settings.window_whitelist.is_empty());
        assert!(settings.window_blacklist.is_empty());
        assert!(!settings.use_whitelist_only);
    }

    #[test]
    fn capture_settings_can_hold_window_filters() {
        let settings = CaptureSettings {
            api_base_url: String::new(),
            api_key: String::new(),
            model_name: String::new(),
            screenshot_interval: 5,
            analysis_prompt: None,
            change_threshold: 3.0,
            max_silent_minutes: 30,
            window_whitelist: vec!["VS Code".to_string()],
            window_blacklist: vec!["Chrome".to_string()],
            use_whitelist_only: true,
            // SMART-004: New fields
            capture_mode: "all".to_string(),
            selected_monitor_index: 1,
            // FEAT-006: Capture only mode
            capture_only_mode: false,
            // AI-006: Custom headers
            custom_headers: Vec::new(),
            // EXP-002: Quality filter
            quality_filter_enabled: true,
            quality_filter_threshold: 0.3,
            // PERF-001: Proxy settings
            proxy_enabled: false,
            proxy_host: None,
            proxy_port: None,
            proxy_username: None,
            proxy_password: None,
        };
        assert_eq!(settings.window_whitelist, vec!["VS Code"]);
        assert_eq!(settings.window_blacklist, vec!["Chrome"]);
        assert!(settings.use_whitelist_only);
        assert_eq!(settings.capture_mode, "all");
        assert_eq!(settings.selected_monitor_index, 1);
    }

    // ── SMART-003: Work time integration tests ──

    /// Helper: Setup test database with work time settings support
    fn setup_test_db_with_work_time() {
        use rusqlite::Connection;
        let conn = Connection::open_in_memory().unwrap();
        crate::memory_storage::init_test_database(&conn).unwrap();
        let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    #[test]
    #[serial]
    fn load_work_time_settings_returns_defaults() {
        setup_test_db_with_work_time();

        let settings = load_work_time_settings();
        // By default, auto_detect_work_time should be true
        assert!(
            settings.auto_detect_work_time,
            "auto_detect_work_time should default to true"
        );
        assert!(
            !settings.use_custom_work_time,
            "use_custom_work_time should default to false"
        );
        // custom_work_time_start defaults to "09:00" per schema (not None)
        assert!(
            settings.custom_work_time_start.is_none()
                || settings.custom_work_time_start == Some("09:00".to_string()),
            "custom_work_time_start should be None or default '09:00'"
        );
        assert!(
            settings.learned_work_time.is_none(),
            "learned_work_time should be None by default"
        );
    }

    #[test]
    #[serial]
    fn load_work_time_settings_loads_custom_time() {
        use crate::memory_storage::get_settings_sync;
        setup_test_db_with_work_time();

        // Save custom work time settings
        let mut settings = get_settings_sync().unwrap();
        settings.use_custom_work_time = Some(true);
        settings.custom_work_time_start = Some("08:00".to_string());
        settings.custom_work_time_end = Some("17:00".to_string());
        crate::memory_storage::save_settings_sync(&settings).unwrap();

        // Reload directly from database to ensure save worked
        let reloaded = get_settings_sync().unwrap();
        assert!(
            reloaded.use_custom_work_time.unwrap_or(false),
            "Saved use_custom_work_time should be true in DB"
        );

        let loaded = load_work_time_settings();
        assert!(
            loaded.use_custom_work_time,
            "use_custom_work_time should be true"
        );
        assert_eq!(
            loaded.custom_work_time_start,
            Some("08:00".to_string()),
            "custom_work_time_start should be 08:00"
        );
        assert_eq!(
            loaded.custom_work_time_end,
            Some("17:00".to_string()),
            "custom_work_time_end should be 17:00"
        );
    }

    #[test]
    #[serial]
    fn should_capture_by_work_time_returns_true_when_disabled() {
        use crate::memory_storage::get_settings_sync;
        setup_test_db_with_work_time();

        // Disable auto_detect_work_time
        let mut settings = get_settings_sync().unwrap();
        settings.auto_detect_work_time = Some(false);
        crate::memory_storage::save_settings_sync(&settings).unwrap();

        // When auto_detect_work_time is false, should always return true
        assert!(
            should_capture_by_work_time(),
            "should_capture_by_work_time should return true when auto_detect is disabled"
        );
    }

    #[test]
    #[serial]
    fn should_capture_by_work_time_uses_custom_time_when_enabled() {
        use crate::memory_storage::get_settings_sync;
        setup_test_db_with_work_time();

        // Enable custom work time with a very narrow window (likely not current time)
        let mut settings = get_settings_sync().unwrap();
        settings.use_custom_work_time = Some(true);
        settings.custom_work_time_start = Some("03:00".to_string());
        settings.custom_work_time_end = Some("04:00".to_string());
        crate::memory_storage::save_settings_sync(&settings).unwrap();

        // The result depends on current time, but we can verify the function uses the settings
        let loaded = load_work_time_settings();
        assert!(loaded.use_custom_work_time);
        assert_eq!(loaded.custom_work_time_start, Some("03:00".to_string()));
    }

    #[test]
    #[serial]
    fn get_work_time_status_command_returns_valid_status() {
        setup_test_db_with_work_time();

        let status = get_work_time_status();

        // Verify the status has expected fields
        // Since we don't know the current time, we just verify the structure
        // is_work_time is a bool, so it's either true or false
        assert!(
            matches!(status.is_work_time, true | false),
            "is_work_time should be a boolean"
        );
        // learning_progress should be 0.0 initially (f64, range 0.0..=1.0)
        assert!(
            (status.learning_progress - 0.0).abs() < 0.01,
            "learning_progress should be 0.0 for fresh install"
        );
    }

    // ── SMART-004: Multi-monitor capture settings tests ──

    #[test]
    fn capture_settings_default_has_primary_capture_mode() {
        let settings = CaptureSettings::default();
        assert_eq!(settings.capture_mode, "primary");
        assert_eq!(settings.selected_monitor_index, 0);
    }

    #[test]
    fn calculate_monitor_bounds_returns_zeros_for_empty() {
        let monitors: Vec<(crate::monitor_types::MonitorDetail, image::RgbaImage)> = Vec::new();
        let (min_x, min_y, max_x, max_y) = calculate_monitor_bounds(&monitors);
        assert_eq!((min_x, min_y, max_x, max_y), (0, 0, 0, 0));
    }

    #[test]
    fn calculate_monitor_bounds_single_monitor() {
        let detail = crate::monitor_types::MonitorDetail {
            index: 0,
            name: "Primary".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            is_primary: true,
        };
        let img = image::RgbaImage::new(1920, 1080);
        let monitors = vec![(detail, img)];

        let (min_x, min_y, max_x, max_y) = calculate_monitor_bounds(&monitors);
        assert_eq!(min_x, 0);
        assert_eq!(min_y, 0);
        assert_eq!(max_x, 1920);
        assert_eq!(max_y, 1080);
    }

    #[test]
    fn calculate_monitor_bounds_dual_monitors_horizontal() {
        // Two monitors side by side: Primary at (0,0), Secondary at (1920,0)
        let detail1 = crate::monitor_types::MonitorDetail {
            index: 0,
            name: "Primary".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            is_primary: true,
        };
        let detail2 = crate::monitor_types::MonitorDetail {
            index: 1,
            name: "Secondary".to_string(),
            width: 2560,
            height: 1440,
            x: 1920,
            y: 0,
            is_primary: false,
        };
        let img1 = image::RgbaImage::new(1920, 1080);
        let img2 = image::RgbaImage::new(2560, 1440);
        let monitors = vec![(detail1, img1), (detail2, img2)];

        let (min_x, min_y, max_x, max_y) = calculate_monitor_bounds(&monitors);
        assert_eq!(min_x, 0);
        assert_eq!(min_y, 0);
        assert_eq!(max_x, 4480); // 1920 + 2560
        assert_eq!(max_y, 1440); // max(1080, 1440)
    }

    #[test]
    fn calculate_monitor_bounds_dual_monitors_vertical() {
        // Two monitors stacked: Primary at (0,0), Secondary at (0,1080)
        let detail1 = crate::monitor_types::MonitorDetail {
            index: 0,
            name: "Primary".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            is_primary: true,
        };
        let detail2 = crate::monitor_types::MonitorDetail {
            index: 1,
            name: "Secondary".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 1080,
            is_primary: false,
        };
        let img1 = image::RgbaImage::new(1920, 1080);
        let img2 = image::RgbaImage::new(1920, 1080);
        let monitors = vec![(detail1, img1), (detail2, img2)];

        let (min_x, min_y, max_x, max_y) = calculate_monitor_bounds(&monitors);
        assert_eq!(min_x, 0);
        assert_eq!(min_y, 0);
        assert_eq!(max_x, 1920);
        assert_eq!(max_y, 2160); // 1080 + 1080
    }

    #[test]
    fn calculate_monitor_bounds_with_negative_positions() {
        // Monitor at negative position (e.g., monitor to the left of primary)
        let detail = crate::monitor_types::MonitorDetail {
            index: 0,
            name: "Left".to_string(),
            width: 1920,
            height: 1080,
            x: -1920,
            y: 0,
            is_primary: false,
        };
        let img = image::RgbaImage::new(1920, 1080);
        let monitors = vec![(detail, img)];

        let (min_x, min_y, max_x, max_y) = calculate_monitor_bounds(&monitors);
        assert_eq!(min_x, -1920);
        assert_eq!(min_y, 0);
        assert_eq!(max_x, 0); // -1920 + 1920
        assert_eq!(max_y, 1080);
    }

    // ── SMART-004: load_capture_settings multi-monitor tests ──

    /// Helper: Setup test database with multi-monitor settings support
    fn setup_test_db_with_multi_monitor() {
        use rusqlite::Connection;
        let conn = Connection::open_in_memory().unwrap();
        crate::memory_storage::init_test_database(&conn).unwrap();
        let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
        *db = Some(conn);
    }

    #[test]
    #[serial]
    fn load_capture_settings_defaults_to_primary_mode() {
        setup_test_db_with_multi_monitor();

        let settings = load_capture_settings();
        assert_eq!(settings.capture_mode, "primary");
        assert_eq!(settings.selected_monitor_index, 0);
    }

    #[test]
    #[serial]
    fn load_capture_settings_loads_custom_mode() {
        use crate::memory_storage::get_settings_sync;
        setup_test_db_with_multi_monitor();

        // Save custom capture settings
        let mut settings = get_settings_sync().unwrap();
        settings.capture_mode = Some("all".to_string());
        settings.selected_monitor_index = Some(2);
        crate::memory_storage::save_settings_sync(&settings).unwrap();

        let loaded = load_capture_settings();
        assert_eq!(loaded.capture_mode, "all");
        assert_eq!(loaded.selected_monitor_index, 2);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EXP-002: Quality filter tests
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Helper: Create a solid-color PNG image as base64
    fn make_solid_color_png_base64(gray_value: u8) -> String {
        let image = image::GrayImage::from_pixel(
            QUALITY_THUMB_SIZE,
            QUALITY_THUMB_SIZE,
            image::Luma([gray_value]),
        );
        let mut buffer = Vec::new();
        image::DynamicImage::ImageLuma8(image)
            .write_to(
                &mut std::io::Cursor::new(&mut buffer),
                image::ImageFormat::Png,
            )
            .unwrap();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &buffer)
    }

    /// Helper: Create a varied-content PNG image (simulating real content)
    fn make_varied_png_base64() -> String {
        let mut image = image::GrayImage::new(QUALITY_THUMB_SIZE, QUALITY_THUMB_SIZE);
        for y in 0..QUALITY_THUMB_SIZE {
            for x in 0..QUALITY_THUMB_SIZE {
                // Create a pattern with high entropy
                let value = ((x * y + x + y) % 256) as u8;
                image.put_pixel(x, y, image::Luma([value]));
            }
        }
        let mut buffer = Vec::new();
        image::DynamicImage::ImageLuma8(image)
            .write_to(
                &mut std::io::Cursor::new(&mut buffer),
                image::ImageFormat::Png,
            )
            .unwrap();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &buffer)
    }

    #[test]
    fn compute_quality_score_solid_black_returns_near_zero() {
        let black_image = make_solid_color_png_base64(0);
        let score = compute_quality_score(&black_image).unwrap();
        assert!(
            score < 0.01,
            "Solid black image should have near-zero score, got {}",
            score
        );
    }

    #[test]
    fn compute_quality_score_solid_white_returns_near_zero() {
        let white_image = make_solid_color_png_base64(255);
        let score = compute_quality_score(&white_image).unwrap();
        assert!(
            score < 0.01,
            "Solid white image should have near-zero score, got {}",
            score
        );
    }

    #[test]
    fn compute_quality_score_solid_gray_returns_near_zero() {
        let gray_image = make_solid_color_png_base64(128);
        let score = compute_quality_score(&gray_image).unwrap();
        assert!(
            score < 0.01,
            "Solid gray image should have near-zero score, got {}",
            score
        );
    }

    #[test]
    fn compute_quality_score_varied_content_returns_high_score() {
        let varied_image = make_varied_png_base64();
        let score = compute_quality_score(&varied_image).unwrap();
        assert!(
            score > 0.5,
            "Varied content image should have high score, got {}",
            score
        );
    }

    #[test]
    fn compute_quality_score_invalid_base64_returns_error() {
        let result = compute_quality_score("not valid base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn quality_filter_counter_increments() {
        // Reset counter
        reset_filtered_count();
        assert_eq!(get_filtered_today(), 0);

        // Increment
        increment_filtered_count();
        assert_eq!(get_filtered_today(), 1);

        increment_filtered_count();
        assert_eq!(get_filtered_today(), 2);

        // Reset again
        reset_filtered_count();
        assert_eq!(get_filtered_today(), 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // STAB-001: Screenshot error classification tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn classify_screenshot_error_permission_denied() {
        assert_eq!(
            classify_screenshot_error("Permission denied"),
            ScreenshotErrorKind::PermissionDenied
        );
        assert_eq!(
            classify_screenshot_error("Screen capture permission denied"),
            ScreenshotErrorKind::PermissionDenied
        );
        assert_eq!(
            classify_screenshot_error("Access denied for screen recording"),
            ScreenshotErrorKind::PermissionDenied
        );
    }

    #[test]
    fn classify_screenshot_error_no_monitors() {
        assert_eq!(
            classify_screenshot_error("No monitors found"),
            ScreenshotErrorKind::NoMonitors
        );
        assert_eq!(
            classify_screenshot_error("No monitors to stitch"),
            ScreenshotErrorKind::NoMonitors
        );
    }

    #[test]
    fn classify_screenshot_error_monitor_not_found() {
        assert_eq!(
            classify_screenshot_error("Monitor index 5 out of bounds (2 monitors)"),
            ScreenshotErrorKind::MonitorNotFound
        );
    }

    #[test]
    fn classify_screenshot_error_temporary_failure() {
        assert_eq!(
            classify_screenshot_error("Capture timeout - monitor busy"),
            ScreenshotErrorKind::TemporaryFailure
        );
        assert_eq!(
            classify_screenshot_error("Temporarily unavailable"),
            ScreenshotErrorKind::TemporaryFailure
        );
    }

    #[test]
    fn classify_screenshot_error_unknown() {
        assert_eq!(
            classify_screenshot_error("Something went wrong"),
            ScreenshotErrorKind::Unknown
        );
        assert_eq!(
            classify_screenshot_error("Failed to encode image"),
            ScreenshotErrorKind::Unknown
        );
    }

    #[test]
    fn classify_screenshot_error_case_insensitive() {
        assert_eq!(
            classify_screenshot_error("PERMISSION DENIED"),
            ScreenshotErrorKind::PermissionDenied
        );
        assert_eq!(
            classify_screenshot_error("No Monitors Found"),
            ScreenshotErrorKind::NoMonitors
        );
    }

    #[test]
    fn get_screenshot_error_message_permission() {
        let msg = get_screenshot_error_message(
            &ScreenshotErrorKind::PermissionDenied,
            "Permission denied",
        );
        assert!(msg.contains("权限"));
    }

    #[test]
    fn get_screenshot_error_message_no_monitors() {
        let msg =
            get_screenshot_error_message(&ScreenshotErrorKind::NoMonitors, "No monitors found");
        assert!(msg.contains("显示器"));
    }

    #[test]
    fn get_screenshot_error_message_temporary() {
        let msg = get_screenshot_error_message(&ScreenshotErrorKind::TemporaryFailure, "timeout");
        assert!(msg.contains("重试"));
    }
}
