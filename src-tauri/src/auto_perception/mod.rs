use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::command;

use crate::memory_storage;

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
/// Returns `true` if we should proceed with the full capture+analysis pipeline.
fn should_capture(fingerprint: &[u8], change_threshold: f64, max_silent_minutes: u64) -> bool {
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

    if changed || silent_exceeded {
        if silent_exceeded && !changed {
            tracing::info!(
                "Screen unchanged but max silent time ({} min) exceeded, forcing capture",
                max_silent_minutes
            );
        }
        state.last_fingerprint = Some(fingerprint.to_vec());
        state.last_capture_time = Instant::now();
        true
    } else {
        tracing::debug!("Screen unchanged, skipping capture");
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenAnalysis {
    pub current_focus: String,
    pub active_software: String,
    pub context_keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSettings {
    pub api_base_url: String,
    pub api_key: String,
    pub model_name: String,
    pub screenshot_interval: u64,
    pub analysis_prompt: Option<String>,
    pub change_threshold: f64,
    pub max_silent_minutes: u64,
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
        }
    }
}

// ─── Windows：Windows Graphics Capture API ───────────────────────
#[cfg(target_os = "windows")]
fn capture_screen() -> Result<String, String> {
    use std::sync::mpsc;
    use windows_capture::{
        capture::{Context, GraphicsCaptureApiHandler},
        frame::Frame,
        graphics_capture_api::InternalCaptureControl,
        monitor::Monitor,
        settings::{
            ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
            MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
        },
    };

    type FrameResult = Result<(u32, u32, Vec<u8>), String>;

    struct OneShot {
        tx: mpsc::SyncSender<FrameResult>,
    }

    impl GraphicsCaptureApiHandler for OneShot {
        type Flags = mpsc::SyncSender<FrameResult>;
        type Error = Box<dyn std::error::Error + Send + Sync>;

        fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
            Ok(Self { tx: ctx.flags })
        }

        fn on_frame_arrived(
            &mut self,
            frame: &mut Frame,
            capture_control: InternalCaptureControl,
        ) -> Result<(), Self::Error> {
            let width = frame.width();
            let height = frame.height();
            let mut buffer = frame.buffer()?;
            let row_pitch = buffer.row_pitch() as usize;
            let raw = buffer.as_raw_buffer();
            let row_bytes = width as usize * 4; // Rgba8: 4 bytes/pixel

            // 逐行复制，跳过 GPU 对齐 padding
            let mut rgba_data = Vec::with_capacity(row_bytes * height as usize);
            for y in 0..(height as usize) {
                let start = y * row_pitch;
                rgba_data.extend_from_slice(&raw[start..start + row_bytes]);
            }

            let _ = self.tx.send(Ok((width, height, rgba_data)));
            capture_control.stop();
            Ok(())
        }

        fn on_closed(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    let (tx, rx) = mpsc::sync_channel(1);
    let monitor = Monitor::primary().map_err(|e| format!("Failed to get primary monitor: {e}"))?;

    let settings = Settings::new(
        monitor,
        CursorCaptureSettings::Default,
        DrawBorderSettings::Default,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Rgba8,
        tx,
    );

    let _control = OneShot::start_free_threaded(settings)
        .map_err(|e| format!("Failed to start screen capture: {e}"))?;

    let (width, height, rgba_data) = rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .map_err(|_| "Screen capture timed out after 5s".to_string())?
        .map_err(|e| e)?;

    let image = image::RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| "Failed to construct image from frame data".to_string())?;

    let mut buf = Vec::new();
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {e}"))?;

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &buf,
    ))
}

// ─── 非 Windows（macOS / Linux）：xcap ───────────────────────────
#[cfg(not(target_os = "windows"))]
fn capture_screen() -> Result<String, String> {
    let monitors = xcap::Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    if monitors.is_empty() {
        return Err("No monitors found".to_string());
    }

    let rgba_image = monitors[0]
        .capture_image()
        .map_err(|e| format!("Failed to capture screen: {}", e))?;

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

Return ONLY valid JSON, no other text. Example format:
{"current_focus": "编写 Rust 后端代码", "active_software": "VS Code", "context_keywords": ["Rust", "Tauri", "异步编程"]}"#;

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
    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            let elapsed_ms = start.elapsed().as_millis();
            tracing::error!(
                "{}",
                serde_json::json!({
                    "event": "llm_error",
                    "caller": "analyze_screen",
                    "error": format!("API request failed: {}", e),
                    "elapsed_ms": elapsed_ms,
                })
            );
            format!("API request failed: {}", e)
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
        },
        Err(_) => CaptureSettings::default(),
    }
}

async fn capture_and_store() -> Result<(), String> {
    let settings = load_capture_settings();

    if settings.api_key.is_empty() {
        return Err("API 密钥未配置，请在设置中配置".to_string());
    }

    let image_base64 = capture_screen()?;

    // Check if screen has changed enough to warrant a full capture
    let fingerprint = compute_fingerprint(&image_base64)?;
    if !should_capture(
        &fingerprint,
        settings.change_threshold,
        settings.max_silent_minutes,
    ) {
        return Ok(());
    }

    let screenshot_path = save_screenshot(&image_base64);

    let analysis = analyze_screen(&settings, &image_base64).await?;

    let content = serde_json::json!({
        "current_focus": analysis.current_focus,
        "active_software": analysis.active_software,
        "context_keywords": analysis.context_keywords
    })
    .to_string();

    memory_storage::add_record("auto", &content, screenshot_path.as_deref())
        .map_err(|e| format!("Failed to store capture: {}", e))?;

    tracing::info!("Screen captured and analyzed: {}", analysis.current_focus);
    Ok(())
}

#[command]
pub async fn start_auto_capture() -> Result<(), String> {
    if AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }

    let settings = load_capture_settings();

    if settings.api_key.is_empty() {
        return Err("API 密钥未配置，请在设置中配置".to_string());
    }

    AUTO_CAPTURE_RUNNING.store(true, Ordering::SeqCst);

    let interval_minutes = settings.screenshot_interval;

    tokio::spawn(async move {
        // Execute immediately on start
        if let Err(e) = capture_and_store().await {
            tracing::error!("Initial capture failed: {}", e);
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

            if let Err(e) = capture_and_store().await {
                tracing::error!("Auto capture failed: {}", e);
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

#[command]
pub async fn trigger_capture() -> Result<(), String> {
    capture_and_store().await.map_err(|e| {
        tracing::error!("Trigger capture failed: {}", e);
        e
    })?;
    tracing::info!("Manual capture triggered");
    Ok(())
}

/// 只截图并保存到磁盘，不调用 AI 分析，不写数据库记录。
/// 返回截图文件的绝对路径，供前端直接预览。
#[command]
pub async fn take_screenshot() -> Result<String, String> {
    let image_base64 = capture_screen()?;
    let path = save_screenshot(&image_base64).ok_or_else(|| "截图保存失败".to_string())?;
    tracing::info!("Screenshot saved for preview: {}", path);
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

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
}
