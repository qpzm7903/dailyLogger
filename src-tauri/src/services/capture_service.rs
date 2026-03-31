//! Capture service - Business logic for screenshot capture operations
//!
//! This module contains business logic functions for capture operations.
//! Commands should delegate to these service functions rather than implementing logic directly.
//!
//! SMART-001: Active window detection and filtering
//! SMART-002: Silent threshold auto-adjustment
//! SMART-003: Work time aware capture
//! SMART-004: Multi-monitor capture support
//! EXP-002: Screenshot quality filter

use crate::errors::{AppError, AppResult};
use crate::memory_storage;
use crate::monitor::get_monitor_list;
use crate::monitor_types::{CaptureMode, MonitorInfo};
use crate::services::session_service::detect_or_create_session;
use crate::silent_tracker::{
    calculate_optimal_silent_minutes, current_threshold, has_sufficient_data, record_capture,
    set_threshold, CaptureReason,
};
use crate::window_info::{get_active_window, should_capture_by_window, ActiveWindow};
use crate::work_time::{is_in_work_time, WorkTimeSettings};

use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Thumbnail fingerprint size: 64x64 grayscale = 4096 bytes
const THUMB_SIZE: u32 = 64;

/// Default: screen change < 3% is considered unchanged
const DEFAULT_CHANGE_THRESHOLD: f64 = 3.0;

/// Default: force capture after 30 minutes of no change
const DEFAULT_MAX_SILENT_MINUTES: u64 = 30;

/// Default analysis prompt
pub const DEFAULT_ANALYSIS_PROMPT: &str = r#"你是一个工作分析助手。请分析这张截图，识别用户当前的工作内容和活动。

请返回以下 JSON 格式：

{
  "current_focus": "正在做什么",
  "active_software": "使用的软件名称",
  "context_keywords": ["关键词1", "关键词2", "关键词3"],
  "tags": ["开发", "会议", "写作", "学习", "研究", "沟通", "规划", "文档", "测试", "设计"]
}

注意：
1. context_keywords 应该是从截图中识别出的具体主题或任务
2. tags 从以下列表选择 1-3 个最相关的: ["开发", "会议", "写作", "学习", "研究", "沟通", "规划", "文档", "测试", "设计"]
3. 如果无法确定，可以描述为"未知"或"其他"

返回纯 JSON，不要添加任何其他文字。"#;

// ═══════════════════════════════════════════════════════════════════════════════
// State
// ═══════════════════════════════════════════════════════════════════════════════

static AUTO_CAPTURE_RUNNING: AtomicBool = AtomicBool::new(false);

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

// ═══════════════════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════════════════

/// EXP-002: Statistics for quality filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityFilterStats {
    pub filtered_today: u32,
    pub quality_filter_enabled: bool,
    pub quality_filter_threshold: f64,
}

/// Screenshot analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenAnalysis {
    pub current_focus: String,
    pub active_software: String,
    pub context_keywords: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_window: Option<ActiveWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Capture settings derived from app settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSettings {
    pub api_base_url: String,
    pub api_key: String,
    pub model_name: String,
    pub screenshot_interval: u64,
    pub analysis_prompt: Option<String>,
    pub change_threshold: f64,
    pub max_silent_minutes: u64,
    pub window_whitelist: Vec<String>,
    pub window_blacklist: Vec<String>,
    pub use_whitelist_only: bool,
    pub capture_mode: String,
    pub selected_monitor_index: usize,
    pub capture_only_mode: bool,
    pub custom_headers: Vec<crate::memory_storage::CustomHeader>,
    pub quality_filter_enabled: bool,
    pub quality_filter_threshold: f64,
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
            capture_mode: "primary".to_string(),
            selected_monitor_index: 0,
            capture_only_mode: false,
            custom_headers: Vec::new(),
            quality_filter_enabled: true,
            quality_filter_threshold: 0.3,
            proxy_enabled: false,
            proxy_host: None,
            proxy_port: None,
            proxy_username: None,
            proxy_password: None,
        }
    }
}

/// Reanalyze result for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReanalyzeResult {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// Payload for silent-threshold-adjusted event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdAdjustment {
    pub old_value: u64,
    pub new_value: u64,
    pub reason: String,
}

/// Validate that API key is configured, returning a standardized error if not.
fn require_api_key(settings: &CaptureSettings) -> AppResult<()> {
    if settings.api_key.is_empty() {
        return Err(AppError::auth("API 密钥未配置，请在设置中配置"));
    }
    Ok(())
}

// STAB-001 AC5: Screenshot error classification
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenshotErrorKind {
    PermissionDenied,
    NoMonitors,
    MonitorNotFound,
    TemporaryFailure,
    Unknown,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Quality Filter Functions
// ═══════════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════════
// Screenshot Error Classification (STAB-001 AC5)
// ═══════════════════════════════════════════════════════════════════════════════

fn classify_screenshot_error(error: &str) -> ScreenshotErrorKind {
    let error_lower = error.to_lowercase();
    if error_lower.contains("permission")
        || error_lower.contains("denied")
        || error_lower.contains("access denied")
    {
        return ScreenshotErrorKind::PermissionDenied;
    }
    if error_lower.contains("no monitors") || error_lower.contains("monitor not found") {
        return ScreenshotErrorKind::NoMonitors;
    }
    if error_lower.contains("out of bounds") || error_lower.contains("index") {
        return ScreenshotErrorKind::MonitorNotFound;
    }
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

// ═══════════════════════════════════════════════════════════════════════════════
// Fingerprint and Quality Functions
// ═══════════════════════════════════════════════════════════════════════════════

fn compute_fingerprint(image_base64: &str) -> AppResult<Vec<u8>> {
    let image_data =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, image_base64)?;
    let img = image::load_from_memory(&image_data)?;
    let thumb = img
        .resize_exact(THUMB_SIZE, THUMB_SIZE, image::imageops::FilterType::Nearest)
        .to_luma8();
    Ok(thumb.into_raw())
}

fn calc_change_rate(a: &[u8], b: &[u8]) -> f64 {
    if a.len() != b.len() {
        return 100.0;
    }
    const NOISE_TOLERANCE: u8 = 10;
    let changed = a
        .iter()
        .zip(b.iter())
        .filter(|(pa, pb)| pa.abs_diff(**pb) > NOISE_TOLERANCE)
        .count();
    (changed as f64 / a.len() as f64) * 100.0
}

const QUALITY_THUMB_SIZE: u32 = 32;

fn compute_quality_score(image_base64: &str) -> AppResult<f64> {
    let image_data =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, image_base64)?;
    let img = image::load_from_memory(&image_data)?;
    let thumb = img
        .resize_exact(
            QUALITY_THUMB_SIZE,
            QUALITY_THUMB_SIZE,
            image::imageops::FilterType::Nearest,
        )
        .to_luma8();
    let pixels = thumb.into_raw();
    let mut histogram = [0u32; 256];
    for &pixel in &pixels {
        histogram[pixel as usize] += 1;
    }
    let total_pixels = pixels.len() as f64;
    let mut entropy = 0.0;
    for &count in &histogram {
        if count > 0 {
            let p = count as f64 / total_pixels;
            entropy -= p * p.log2();
        }
    }
    let normalized_score = (entropy / 8.0).min(1.0);
    Ok(normalized_score)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Capture Decision Functions
// ═══════════════════════════════════════════════════════════════════════════════

fn should_capture(
    fingerprint: &[u8],
    change_threshold: f64,
    max_silent_minutes: u64,
) -> Option<CaptureReason> {
    let mut state = SCREEN_STATE.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("SCREEN_STATE mutex was poisoned, recreating state");
        poisoned.into_inner()
    });
    let silent_exceeded =
        state.last_capture_time.elapsed() >= Duration::from_secs(max_silent_minutes * 60);
    let changed = match &state.last_fingerprint {
        None => true,
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
    record_capture(reason);
    state.last_fingerprint = Some(fingerprint.to_vec());
    state.last_capture_time = Instant::now();
    Some(reason)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Settings Functions
// ═══════════════════════════════════════════════════════════════════════════════

fn parse_window_patterns(json: Option<&str>) -> Vec<String> {
    json.and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}

/// Extract capture-related fields from a Settings reference into CaptureSettings.
/// Shared by both `load_capture_settings()` and `load_capture_settings_from_arc()`.
fn capture_settings_from_settings(s: &crate::memory_storage::Settings) -> CaptureSettings {
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
        api_base_url: s.api_base_url.clone().unwrap_or_default(),
        api_key: s.api_key.clone().unwrap_or_default(),
        model_name: s.model_name.clone().unwrap_or_else(|| "gpt-4o".to_string()),
        screenshot_interval: s.screenshot_interval.unwrap_or(5) as u64,
        analysis_prompt: s.analysis_prompt.clone(),
        change_threshold: s.change_threshold.unwrap_or(3) as f64,
        max_silent_minutes: s.max_silent_minutes.unwrap_or(30) as u64,
        window_whitelist: parse_window_patterns(s.window_whitelist.as_deref()),
        window_blacklist: parse_window_patterns(s.window_blacklist.as_deref()),
        use_whitelist_only: s.use_whitelist_only.unwrap_or(false),
        capture_mode: s
            .capture_mode
            .clone()
            .unwrap_or_else(|| "primary".to_string()),
        selected_monitor_index: s.selected_monitor_index.unwrap_or(0) as usize,
        capture_only_mode: s.capture_only_mode.unwrap_or(false),
        custom_headers,
        quality_filter_enabled: s.quality_filter_enabled.unwrap_or(true),
        quality_filter_threshold: s.quality_filter_threshold.unwrap_or(0.3),
        proxy_enabled: s.proxy_enabled.unwrap_or(false),
        proxy_host: s.proxy_host.clone(),
        proxy_port: s.proxy_port,
        proxy_username: s.proxy_username.clone(),
        proxy_password: s.proxy_password.clone(),
    }
}

pub fn load_capture_settings() -> CaptureSettings {
    match memory_storage::get_settings_sync() {
        Ok(arc) => capture_settings_from_settings(&arc),
        Err(_) => CaptureSettings::default(),
    }
}

/// Load capture settings from a pre-fetched Arc<Settings>, avoiding an extra cache read.
pub fn load_capture_settings_from_arc(
    arc: &std::sync::Arc<crate::memory_storage::Settings>,
) -> CaptureSettings {
    capture_settings_from_settings(arc)
}

/// Load work time settings from a pre-fetched Arc<Settings>, avoiding an extra cache read.
pub fn load_work_time_settings_from_arc(
    arc: &std::sync::Arc<crate::memory_storage::Settings>,
) -> WorkTimeSettings {
    WorkTimeSettings {
        auto_detect_work_time: arc.auto_detect_work_time.unwrap_or(true),
        use_custom_work_time: arc.use_custom_work_time.unwrap_or(false),
        custom_work_time_start: arc.custom_work_time_start.clone(),
        custom_work_time_end: arc.custom_work_time_end.clone(),
        learned_work_time: arc.learned_work_time.clone(),
    }
}

/// Check if current time is within work hours using a pre-fetched Arc<Settings>.
pub fn should_capture_by_work_time_from_arc(
    arc: &std::sync::Arc<crate::memory_storage::Settings>,
) -> bool {
    let settings = load_work_time_settings_from_arc(arc);
    is_in_work_time(&settings)
}

pub fn load_work_time_settings() -> WorkTimeSettings {
    match memory_storage::get_settings_sync() {
        Ok(s) => WorkTimeSettings {
            auto_detect_work_time: s.auto_detect_work_time.unwrap_or(true),
            use_custom_work_time: s.use_custom_work_time.unwrap_or(false),
            custom_work_time_start: s.custom_work_time_start.clone(),
            custom_work_time_end: s.custom_work_time_end.clone(),
            learned_work_time: s.learned_work_time.clone(),
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

/// Check if current time is within work hours.
/// Returns true if capture should proceed, false if should skip.
pub fn should_capture_by_work_time() -> bool {
    let settings = load_work_time_settings();
    is_in_work_time(&settings)
}

/// Evaluate and adjust the silent threshold if needed.
/// Returns Some((old_threshold, new_threshold)) if an adjustment was made, None otherwise.
pub fn evaluate_and_adjust_threshold() -> Option<(u64, u64)> {
    let settings = match memory_storage::get_settings_sync() {
        Ok(s) => s,
        Err(_) => return None,
    };

    if !settings.auto_adjust_silent.unwrap_or(true) {
        tracing::debug!("Auto-adjust silent threshold is disabled by user");
        return None;
    }
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

    if !has_sufficient_data() {
        tracing::debug!("Insufficient capture data for threshold adjustment");
        return None;
    }
    let old_threshold = current_threshold();
    let new_threshold = calculate_optimal_silent_minutes(
        &crate::silent_tracker::SILENT_PATTERN_TRACKER
            .lock()
            .unwrap_or_else(|e| e.into_inner()),
    );
    if new_threshold != old_threshold {
        set_threshold(new_threshold);
        // Reuse the same Arc for the mutation save — avoids a second get_settings_sync() call
        let mut settings_mut = (*settings).clone();
        settings_mut.max_silent_minutes = Some(new_threshold as i32);
        if let Err(e) = memory_storage::save_settings_sync(&settings_mut) {
            tracing::error!("Failed to save adjusted threshold: {}", e);
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

// ═══════════════════════════════════════════════════════════════════════════════
// Screen Capture Functions
// ═══════════════════════════════════════════════════════════════════════════════

fn capture_screen_with_mode(
    mode: CaptureMode,
    selected_index: usize,
) -> AppResult<(String, MonitorInfo)> {
    let monitor_details = get_monitor_list()?;
    let monitors = xcap::Monitor::all().map_err(|e| AppError::screenshot(e.to_string()))?;
    if monitors.is_empty() {
        return Err(AppError::screenshot("No monitors found"));
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

fn capture_single_monitor_xcap(monitors: &[xcap::Monitor], index: usize) -> AppResult<String> {
    let monitor = monitors
        .get(index)
        .ok_or_else(|| AppError::screenshot(format!("Monitor index {} out of bounds", index)))?;
    let image = monitor
        .capture_image()
        .map_err(|e| AppError::screenshot(format!("Failed to capture monitor {}: {}", index, e)))?;
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| AppError::screenshot(format!("Failed to encode screenshot: {}", e)))?;
    let base64_str = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &buffer);
    Ok(base64_str)
}

fn stitch_monitors_xcap(
    monitors: &[xcap::Monitor],
    monitor_details: &[crate::monitor_types::MonitorDetail],
) -> AppResult<String> {
    if monitors.is_empty() {
        return Err(AppError::screenshot("No monitors to stitch"));
    }
    let mut captured_images: Vec<(crate::monitor_types::MonitorDetail, image::RgbaImage)> =
        Vec::new();
    for (index, _monitor) in monitors.iter().enumerate() {
        let image_base64 = capture_single_monitor_xcap(monitors, index)?;
        let image_data =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &image_base64)
                .map_err(|e| {
                    AppError::screenshot(format!("Failed to decode captured image: {}", e))
                })?;
        let img = image::load_from_memory(&image_data)?;
        let rgba_image = img.to_rgba8();
        let detail = monitor_details.get(index).cloned().unwrap_or_else(|| {
            crate::monitor_types::MonitorDetail {
                index,
                name: format!("Monitor {}", index + 1),
                width: rgba_image.width(),
                height: rgba_image.height(),
                x: 0,
                y: 0,
                is_primary: false,
            }
        });
        captured_images.push((detail, rgba_image));
    }
    if captured_images.len() == 1 {
        let (_, img) = &captured_images[0];
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        img.write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| AppError::screenshot(format!("Failed to encode stitched image: {}", e)))?;
        return Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &buffer,
        ));
    }
    let total_width: u32 = captured_images.iter().map(|(d, _)| d.width).sum();
    let max_height: u32 = captured_images
        .iter()
        .map(|(_, img)| img.height())
        .max()
        .unwrap_or(0);
    let mut stitched = image::RgbaImage::new(total_width, max_height);
    let mut x_offset: u32 = 0;
    for (detail, img) in &captured_images {
        image::imageops::overlay(&mut stitched, img, x_offset as i64, 0);
        x_offset += detail.width;
    }
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    stitched
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| AppError::screenshot(format!("Failed to encode stitched image: {}", e)))?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &buffer,
    ))
}

fn save_screenshot(image_base64: &str) -> Option<String> {
    let image_data =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, image_base64).ok()?;
    let img = image::load_from_memory(&image_data).ok()?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S_%f");
    let filename = format!("screenshot_{}.png", timestamp);
    let user_home = dirs::home_dir()?;
    let daily_logger_dir = user_home.join("DailyLogger");
    let screenshots_dir = daily_logger_dir.join("screenshots");
    std::fs::create_dir_all(&screenshots_dir).ok()?;
    let path = screenshots_dir.join(&filename);
    img.save(&path).ok()?;
    Some(path.to_string_lossy().to_string())
}

// ═══════════════════════════════════════════════════════════════════════════════
// AI Analysis Functions
// ═══════════════════════════════════════════════════════════════════════════════

async fn analyze_screen(
    settings: &CaptureSettings,
    image_base64: &str,
) -> AppResult<ScreenAnalysis> {
    require_api_key(settings)?;
    let prompt = settings
        .analysis_prompt
        .as_deref()
        .unwrap_or(DEFAULT_ANALYSIS_PROMPT);
    let client = crate::create_http_client_with_proxy(
        &settings.api_base_url,
        60,
        if settings.proxy_enabled {
            Some(crate::ProxyConfig {
                enabled: true,
                host: settings.proxy_host.clone(),
                port: settings.proxy_port,
                username: settings.proxy_username.clone(),
                password: settings.proxy_password.clone(),
            })
        } else {
            None
        },
    )?;
    let payload = serde_json::json!({
        "model": settings.model_name,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": prompt},
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", image_base64)
                        }
                    }
                ]
            }
        ],
        "max_tokens": 1000
    });
    let mut request = client
        .post(format!("{}/chat/completions", settings.api_base_url))
        .header("Content-Type", "application/json");
    if !settings.api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", settings.api_key));
    }
    for header in &settings.custom_headers {
        request = request.header(&header.key, &header.value);
    }
    let response = request.json(&payload).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::network(format!(
            "API request failed with status {}: {}",
            status, body
        )));
    }
    let response_body: serde_json::Value = response.json().await?;
    let content = response_body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| {
            AppError::validation(format!("Invalid API response format: {:?}", response_body))
        })?;
    let content = content.trim();
    let json_start = content.find('{').unwrap_or(0);
    let json_end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
    let json_str = &content[json_start..json_end];
    let analysis: ScreenAnalysis = serde_json::from_str(json_str).map_err(|e| {
        AppError::validation(format!(
            "Failed to parse analysis: {}. Content: {}",
            e, content
        ))
    })?;
    Ok(analysis)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Core Capture Service Functions
// ═══════════════════════════════════════════════════════════════════════════════

pub fn is_auto_capture_running() -> bool {
    AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst)
}

/// Service function to start auto capture - validates and initializes capture
/// Returns early error if API key not configured
pub fn start_auto_capture_service() -> AppResult<()> {
    if AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }
    let settings = load_capture_settings();
    require_api_key(&settings)?;
    set_threshold(settings.max_silent_minutes);
    AUTO_CAPTURE_RUNNING.store(true, Ordering::SeqCst);
    Ok(())
}

/// Service function to stop auto capture
pub fn stop_auto_capture_service() {
    AUTO_CAPTURE_RUNNING.store(false, Ordering::SeqCst);
    tracing::info!("Auto capture stopped");
}

/// Service function to trigger a single capture
pub async fn trigger_capture_service() -> AppResult<()> {
    let settings = load_capture_settings();
    capture_and_store_inner(settings).await.map_err(|e| {
        let err_str = e.to_string();
        tracing::error!("Trigger capture failed: {}", err_str);
        let kind = classify_screenshot_error(&err_str);
        AppError::screenshot(get_screenshot_error_message(&kind, &err_str))
    })
}

/// Service function to trigger a single capture using a pre-fetched Arc<Settings>.
/// Avoids an extra `get_settings_sync()` call when the caller already has the Arc.
pub async fn trigger_capture_with_arc(
    arc: std::sync::Arc<crate::memory_storage::Settings>,
) -> AppResult<()> {
    let settings = load_capture_settings_from_arc(&arc);
    capture_and_store_inner(settings).await.map_err(|e| {
        let err_str = e.to_string();
        tracing::error!("Trigger capture failed: {}", err_str);
        let kind = classify_screenshot_error(&err_str);
        AppError::screenshot(get_screenshot_error_message(&kind, &err_str))
    })
}

/// Service function to take a screenshot and save to disk (no AI analysis)
pub async fn take_screenshot_service() -> AppResult<String> {
    let settings = load_capture_settings();
    let capture_mode = settings
        .capture_mode
        .parse::<CaptureMode>()
        .unwrap_or(CaptureMode::Primary);
    let result =
        capture_screen_with_mode(capture_mode, settings.selected_monitor_index).map_err(|e| {
            let err_str = e.to_string();
            tracing::error!("Screenshot capture failed: {}", err_str);
            let kind = classify_screenshot_error(&err_str);
            AppError::screenshot(get_screenshot_error_message(&kind, &err_str))
        })?;
    let image_base64 = result.0;
    let screenshot_path = save_screenshot(&image_base64).ok_or_else(|| {
        tracing::error!("Failed to save screenshot to disk");
        AppError::screenshot("截图保存失败")
    })?;
    tracing::info!("Screenshot saved for preview: {}", screenshot_path);
    // Return the path with monitor info
    Ok(screenshot_path)
}

/// Get auto capture status
pub fn get_auto_capture_status_service() -> bool {
    is_auto_capture_running()
}

/// Get default analysis prompt
pub fn get_default_analysis_prompt_service() -> String {
    DEFAULT_ANALYSIS_PROMPT.to_string()
}

/// Get quality filter stats
pub async fn get_quality_filter_stats_service() -> AppResult<QualityFilterStats> {
    let settings = memory_storage::get_settings_sync()?;
    Ok(QualityFilterStats {
        filtered_today: get_filtered_today(),
        quality_filter_enabled: settings.quality_filter_enabled.unwrap_or(true),
        quality_filter_threshold: settings.quality_filter_threshold.unwrap_or(0.3),
    })
}

/// Reset quality filter counter
pub async fn reset_quality_filter_counter_service() -> AppResult<()> {
    reset_filtered_count();
    Ok(())
}

/// Reanalyze a single record
pub async fn reanalyze_record_service(record_id: i64) -> AppResult<ScreenAnalysis> {
    let record = memory_storage::get_record_by_id_sync(record_id)?;
    let screenshot_path = record
        .screenshot_path
        .as_ref()
        .ok_or_else(|| AppError::validation("Record has no screenshot"))?;
    let image_data = std::fs::read(screenshot_path)?;
    let image_base64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);
    let settings = load_capture_settings();
    require_api_key(&settings)?;
    tracing::info!("Reanalyzing record {}", record_id);
    let analysis = analyze_screen(&settings, &image_base64).await?;
    let content_json = serde_json::to_string(&analysis)?;
    memory_storage::update_record_content_sync(record_id, &content_json)?;
    tracing::info!(
        "Reanalysis complete for record {}: {}",
        record_id,
        analysis.current_focus
    );
    Ok(analysis)
}

/// Reanalyze all records with screenshots from today
pub async fn reanalyze_today_records_service() -> AppResult<ReanalyzeResult> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let records = memory_storage::get_records_by_date_range_sync(today.clone(), today)?;
    let records_with_screenshots: Vec<_> = records
        .into_iter()
        .filter(|r| r.screenshot_path.is_some())
        .collect();
    let total = records_with_screenshots.len();
    if total == 0 {
        return Ok(ReanalyzeResult {
            total: 0,
            success: 0,
            failed: 0,
            errors: vec![],
        });
    }
    let settings = load_capture_settings();
    require_api_key(&settings)?;
    let mut success = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    for record in records_with_screenshots {
        let record_id = record.id;
        // Filter above guarantees screenshot_path is Some
        let screenshot_path = record.screenshot_path.expect("filtered to have screenshot");
        let image_data = match std::fs::read(&screenshot_path) {
            Ok(data) => data,
            Err(e) => {
                failed += 1;
                errors.push(format!(
                    "Record {}: Failed to read screenshot: {}",
                    record_id, e
                ));
                continue;
            }
        };
        let image_base64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);
        match analyze_screen(&settings, &image_base64).await {
            Ok(analysis) => {
                let content_json = match serde_json::to_string(&analysis) {
                    Ok(json) => json,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!(
                            "Record {}: Failed to serialize analysis: {}",
                            record_id, e
                        ));
                        continue;
                    }
                };
                if let Err(e) = memory_storage::update_record_content_sync(record_id, &content_json)
                {
                    failed += 1;
                    errors.push(format!(
                        "Record {}: Failed to update record: {}",
                        record_id, e
                    ));
                    continue;
                }
                success += 1;
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("Record {}: Analysis failed: {}", record_id, e));
            }
        }
    }
    Ok(ReanalyzeResult {
        total,
        success,
        failed,
        errors,
    })
}

/// Reanalyze all records with screenshots for a specific date
pub async fn reanalyze_records_by_date_service(date: String) -> AppResult<ReanalyzeResult> {
    if chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() {
        return Err(AppError::validation(format!(
            "Invalid date format: {}. Expected YYYY-MM-DD",
            date
        )));
    }
    let records = memory_storage::get_records_by_date_range_sync(date.clone(), date.clone())?;
    let records_with_screenshots: Vec<_> = records
        .into_iter()
        .filter(|r| r.screenshot_path.is_some())
        .collect();
    let total = records_with_screenshots.len();
    if total == 0 {
        return Ok(ReanalyzeResult {
            total: 0,
            success: 0,
            failed: 0,
            errors: vec![],
        });
    }
    let settings = load_capture_settings();
    require_api_key(&settings)?;
    let mut success = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    for record in records_with_screenshots {
        let record_id = record.id;
        // Filter above guarantees screenshot_path is Some
        let screenshot_path = record.screenshot_path.expect("filtered to have screenshot");
        let image_data = match std::fs::read(&screenshot_path) {
            Ok(data) => data,
            Err(e) => {
                failed += 1;
                errors.push(format!(
                    "Record {}: Failed to read screenshot: {}",
                    record_id, e
                ));
                continue;
            }
        };
        let image_base64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);
        match analyze_screen(&settings, &image_base64).await {
            Ok(analysis) => {
                let content_json = match serde_json::to_string(&analysis) {
                    Ok(json) => json,
                    Err(e) => {
                        failed += 1;
                        errors.push(format!(
                            "Record {}: Failed to serialize analysis: {}",
                            record_id, e
                        ));
                        continue;
                    }
                };
                if let Err(e) = memory_storage::update_record_content_sync(record_id, &content_json)
                {
                    failed += 1;
                    errors.push(format!(
                        "Record {}: Failed to update record: {}",
                        record_id, e
                    ));
                    continue;
                }
                success += 1;
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("Record {}: Analysis failed: {}", record_id, e));
            }
        }
    }
    tracing::info!(
        "Reanalyzed {} records for date {}: {} success, {} failed",
        total,
        date,
        success,
        failed
    );
    Ok(ReanalyzeResult {
        total,
        success,
        failed,
        errors,
    })
}

/// Retry screenshot analysis (used by offline queue)
pub async fn retry_screenshot_analysis_service(
    screenshot_path: &str,
    record_id: i64,
) -> AppResult<()> {
    tracing::info!(
        "Retrying screenshot analysis for record {} from {}",
        record_id,
        screenshot_path
    );
    let image_data = std::fs::read(screenshot_path)?;
    let image_base64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);
    let settings = load_capture_settings();
    if settings.api_base_url.is_empty() {
        return Err(AppError::auth("API base URL not configured"));
    }
    let analysis = analyze_screen(&settings, &image_base64).await?;
    let content = serde_json::to_string(&analysis)?;
    memory_storage::update_record_content_sync(record_id, &content)?;
    tracing::info!(
        "Successfully updated record {} with analysis result",
        record_id
    );
    Ok(())
}

/// Get work time status
pub fn get_work_time_status_service() -> crate::work_time::WorkTimeStatus {
    use crate::work_time::get_work_time_status as get_status;
    let settings = load_work_time_settings();
    get_status(&settings)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Internal Capture Logic
// ═══════════════════════════════════════════════════════════════════════════════

async fn capture_and_store_inner(settings: CaptureSettings) -> AppResult<()> {
    require_api_key(&settings)?;

    let active_window = get_active_window();

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

    let capture_mode = settings
        .capture_mode
        .parse::<CaptureMode>()
        .unwrap_or(CaptureMode::Primary);

    let (image_base64, monitor_info) =
        capture_screen_with_mode(capture_mode, settings.selected_monitor_index).map_err(|e| {
            let err_str = e.to_string();
            tracing::error!("Screenshot capture failed: {}", err_str);
            let kind = classify_screenshot_error(&err_str);
            AppError::screenshot(get_screenshot_error_message(&kind, &err_str))
        })?;

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

    tracing::info!("Capture mode: saving screenshot without immediate AI analysis");

    let current_timestamp = Utc::now().to_rfc3339();
    let session_id = detect_or_create_session(&current_timestamp)?;

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
