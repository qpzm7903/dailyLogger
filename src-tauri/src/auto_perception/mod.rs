use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::command;
use tokio::time::interval;

use crate::memory_storage;

static AUTO_CAPTURE_RUNNING: AtomicBool = AtomicBool::new(false);

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
}

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            api_base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model_name: "gpt-4o".to_string(),
            screenshot_interval: 5,
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

async fn analyze_screen(
    settings: &CaptureSettings,
    image_base64: &str,
) -> Result<ScreenAnalysis, String> {
    let client = reqwest::Client::new();

    let prompt = r#"Analyze this screenshot and return a JSON object with:
- current_focus: What is the user currently working on? (1-2 sentences in Chinese)
- active_software: What software is being used? (in Chinese)
- context_keywords: What are the key topics/technologies? (array of strings, in Chinese)

Return ONLY valid JSON, no other text. Example format:
{"current_focus": "编写 Rust 后端代码", "active_software": "VS Code", "context_keywords": ["Rust", "Tauri", "异步编程"]}"#;

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

    let response = client
        .post(format!("{}/chat/completions", settings.api_base_url))
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error ({}): {}", status, body));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in response")?;

    let analysis: ScreenAnalysis = serde_json::from_str(content)
        .map_err(|e| format!("Failed to parse analysis: {}. Content: {}", e, content))?;

    Ok(analysis)
}

async fn capture_and_store() {
    let settings = match memory_storage::get_settings_sync() {
        Ok(s) => CaptureSettings {
            api_base_url: s.api_base_url.unwrap_or_default(),
            api_key: s.api_key.unwrap_or_default(),
            model_name: s.model_name.unwrap_or_else(|| "gpt-4o".to_string()),
            screenshot_interval: s.screenshot_interval.unwrap_or(5) as u64,
        },
        Err(_) => CaptureSettings::default(),
    };

    if settings.api_key.is_empty() {
        tracing::warn!("API key not configured, skipping capture");
        return;
    }

    match capture_screen() {
        Ok(image_base64) => {
            let screenshot_path = save_screenshot(&image_base64);

            match analyze_screen(&settings, &image_base64).await {
                Ok(analysis) => {
                    let content = serde_json::json!({
                        "current_focus": analysis.current_focus,
                        "active_software": analysis.active_software,
                        "context_keywords": analysis.context_keywords
                    })
                    .to_string();

                    if let Err(e) =
                        memory_storage::add_record("auto", &content, screenshot_path.as_deref())
                    {
                        tracing::error!("Failed to store capture: {}", e);
                    } else {
                        tracing::info!("Screen captured and analyzed: {}", analysis.current_focus);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to analyze screen: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to capture screen: {}", e);
        }
    }
}

#[command]
pub async fn start_auto_capture() -> Result<(), String> {
    if AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }

    AUTO_CAPTURE_RUNNING.store(true, Ordering::SeqCst);

    let settings = match memory_storage::get_settings_sync() {
        Ok(s) => CaptureSettings {
            api_base_url: s.api_base_url.unwrap_or_default(),
            api_key: s.api_key.unwrap_or_default(),
            model_name: s.model_name.unwrap_or_else(|| "gpt-4o".to_string()),
            screenshot_interval: s.screenshot_interval.unwrap_or(5) as u64,
        },
        Err(_) => CaptureSettings::default(),
    };

    let interval_minutes = settings.screenshot_interval;

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_minutes * 60));

        capture_and_store().await;

        loop {
            if !AUTO_CAPTURE_RUNNING.load(Ordering::SeqCst) {
                tracing::info!("Auto capture stopped");
                break;
            }

            ticker.tick().await;
            capture_and_store().await;
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
    capture_and_store().await;
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
}
