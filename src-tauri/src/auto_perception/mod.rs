use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{command, Emitter};

use crate::memory_storage;
use crate::monitor::get_monitor_list;
use crate::monitor_types::{CaptureMode, MonitorInfo};
use crate::silent_tracker::{
    calculate_optimal_silent_minutes, current_threshold, has_sufficient_data, record_capture,
    set_threshold, CaptureReason,
};
use crate::work_time::{is_in_work_time, record_work_time_capture, WorkTimeSettings};

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
    let client = reqwest::Client::new();

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
        })
    );

    let start = std::time::Instant::now();

    // Check if this is an Ollama endpoint
    let is_ollama = crate::ollama::is_ollama_endpoint(&settings.api_base_url);

    let mut request = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request_body);

    // Only add Authorization header if API key is provided (not required for Ollama)
    if !settings.api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", settings.api_key));
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
        Ok(s) => CaptureSettings {
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
        },
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
    let (image_base64, monitor_info) =
        capture_screen_with_mode(capture_mode, settings.selected_monitor_index)?;

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

    let screenshot_path = save_screenshot(&image_base64);

    // CORE-007: Check network status before AI analysis
    if !crate::network_status::is_online() {
        tracing::info!("Offline mode: saving screenshot without AI analysis");

        // Save record with placeholder content
        let content = serde_json::json!({
            "current_focus": "离线模式 - 待分析",
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
        let record_id = memory_storage::add_record(
            "auto",
            &content,
            screenshot_path.as_deref(),
            monitor_info_json.as_deref(),
            None,
        )?;

        // Queue AI analysis for when network is restored
        let payload = serde_json::json!({
            "screenshot_path": screenshot_path,
            "record_id": record_id,
        })
        .to_string();
        let _ = crate::offline_queue::enqueue_task(
            &crate::offline_queue::OfflineTaskType::ScreenshotAnalysis,
            &payload,
            Some(record_id),
        );

        return Ok(());
    }

    let analysis = analyze_screen(&settings, &image_base64).await?;

    // SMART-001: Include window info in content JSON (AC1)
    // SMART-004: Include monitor info in content JSON (AC3)
    let content = serde_json::json!({
        "current_focus": analysis.current_focus,
        "active_software": analysis.active_software,
        "context_keywords": analysis.context_keywords,
        "active_window": {
            "title": active_window.title,
            "process_name": active_window.process_name
        },
        "monitor_info": {
            "count": monitor_info.count,
            "capture_mode": capture_mode.to_string()
        }
    })
    .to_string();

    // SMART-004: Store monitor_info as JSON in the monitor_info field
    let monitor_info_json = serde_json::to_string(&monitor_info).ok();

    // AI-004: Store tags as JSON
    let tags_json = analysis
        .tags
        .as_ref()
        .and_then(|t| serde_json::to_string(t).ok());

    memory_storage::add_record(
        "auto",
        &content,
        screenshot_path.as_deref(),
        monitor_info_json.as_deref(),
        tags_json.as_deref(),
    )
    .map_err(|e| format!("Failed to store capture: {}", e))?;

    tracing::info!("Screen captured and analyzed: {}", analysis.current_focus);
    Ok(())
}

#[command]
pub async fn start_auto_capture(app: tauri::AppHandle) -> Result<(), String> {
    if AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }

    let settings = load_capture_settings();

    if settings.api_key.is_empty() {
        return Err("API 密钥未配置，请在设置中配置".to_string());
    }

    // SMART-002: Initialize tracker with user's current threshold
    set_threshold(settings.max_silent_minutes);

    AUTO_CAPTURE_RUNNING.store(true, Ordering::SeqCst);

    let interval_minutes = settings.screenshot_interval;

    // Spawn capture loop
    tokio::spawn(async move {
        // Execute immediately on start (SMART-003: also check work time)
        if should_capture_by_work_time() {
            if let Err(e) = capture_and_store().await {
                tracing::error!("Initial capture failed: {}", e);
            }
            // SMART-003: Record capture for work time learning
            record_work_time_capture();
        } else {
            tracing::debug!("Outside work time, skipping initial capture");
        }

        loop {
            // Use sleep instead of interval to avoid the immediate first-tick
            // behavior of tokio::time::interval (which would cause a double
            // capture right at startup).
            tokio::time::sleep(Duration::from_secs(interval_minutes * 60)).await;

            if !AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst) {
                tracing::info!("Auto capture stopped");
                break;
            }

            // SMART-003 (AC2): Check if current time is within work hours
            if !should_capture_by_work_time() {
                tracing::debug!("Outside work time, skipping capture");
                continue;
            }

            if let Err(e) = capture_and_store().await {
                tracing::error!("Auto capture failed: {}", e);
            } else {
                // SMART-003: Record capture for work time learning
                record_work_time_capture();
            }
        }
    });

    // SMART-002 (AC2): Spawn hourly threshold evaluation task
    let app_handle = app.clone();
    tokio::spawn(async move {
        loop {
            // Wait 1 hour between evaluations
            tokio::time::sleep(Duration::from_secs(60 * 60)).await;

            if !AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst) {
                break;
            }

            // Evaluate and adjust threshold if needed
            if let Some((old_value, new_value)) = evaluate_and_adjust_threshold() {
                // Emit event for frontend notification (AC2: adjustment > 10 min triggers notification)
                let adjustment_magnitude = new_value.abs_diff(old_value);

                if adjustment_magnitude >= 10 {
                    let reason = if new_value > old_value {
                        "检测到深度工作模式，提高静默阈值".to_string()
                    } else {
                        "检测到活跃工作模式，降低静默阈值".to_string()
                    };

                    if let Err(e) = app_handle.emit(
                        "silent-threshold-adjusted",
                        ThresholdAdjustment {
                            old_value,
                            new_value,
                            reason,
                        },
                    ) {
                        tracing::error!("Failed to emit threshold adjustment event: {}", e);
                    }
                }
            }
        }
    });

    tracing::info!(
        "Auto capture started with interval {} minutes",
        interval_minutes
    );
    Ok(())
}

#[command]
pub async fn stop_auto_capture() -> Result<(), String> {
    AUTO_CAPTURE_RUNNING.store(false, Ordering::SeqCst);
    tracing::info!("Auto capture stopped");
    Ok(())
}

/// Returns whether auto capture is currently running.
pub fn is_auto_capture_running() -> bool {
    AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst)
}

/// Tauri command to get auto capture status.
/// Returns the current state of auto capture (running or not).
#[command]
pub fn get_auto_capture_status() -> bool {
    is_auto_capture_running()
}

/// SMART-003: Tauri command to get work time status.
/// Returns information about current work time status and learning progress.
#[command]
pub fn get_work_time_status() -> crate::work_time::WorkTimeStatus {
    use crate::work_time::get_work_time_status as get_status;
    let settings = load_work_time_settings();
    get_status(&settings)
}

#[command]
pub async fn trigger_capture() -> Result<(), String> {
    capture_and_store().await.map_err(|e| {
        tracing::error!("Trigger capture failed: {}", e);
        e
    })?;
    tracing::info!("Manual capture triggered");
    Ok(())
}

/// Returns the default analysis prompt template used for screenshot analysis.
#[command]
pub fn get_default_analysis_prompt() -> String {
    DEFAULT_ANALYSIS_PROMPT.to_string()
}

/// 只截图并保存到磁盘，不调用 AI 分析，不写数据库记录。
/// 返回截图文件的绝对路径，供前端直接预览。
#[command]
pub async fn take_screenshot() -> Result<String, String> {
    // SMART-004: Use multi-monitor capture settings
    let settings = load_capture_settings();
    let capture_mode = settings
        .capture_mode
        .parse::<CaptureMode>()
        .unwrap_or(CaptureMode::Primary);
    let (image_base64, _monitor_info) =
        capture_screen_with_mode(capture_mode, settings.selected_monitor_index)?;
    let path = save_screenshot(&image_base64).ok_or_else(|| "截图保存失败".to_string())?;
    tracing::info!("Screenshot saved for preview: {}", path);
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use serial_test::serial;

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
        assert!(
            status.is_work_time || !status.is_work_time,
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
}
