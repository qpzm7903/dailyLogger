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

fn capture_screen() -> Result<String, String> {
    use scap::capturer::{Capturer, Options, Resolution};
    use scap::frame::{Frame, FrameType, VideoFrame};

    if !scap::is_supported() {
        return Err("Screen capture is not supported on this platform".to_string());
    }

    if !scap::has_permission() && !scap::request_permission() {
        return Err("Screen capture permission was denied".to_string());
    }

    let options = Options {
        fps: 1,
        target: None,
        show_cursor: false,
        show_highlight: false,
        output_type: FrameType::BGRAFrame,
        output_resolution: Resolution::Captured,
        ..Default::default()
    };

    let mut capturer = Capturer::build(options)
        .map_err(|e| format!("Failed to initialize screen capturer: {:?}", e))?;

    capturer.start_capture();
    let frame = capturer
        .get_next_frame()
        .map_err(|e| format!("Failed to capture frame: {}", e))?;
    capturer.stop_capture();

    let (width, height, rgba_data) = match frame {
        Frame::Video(VideoFrame::BGRA(bgra)) => {
            let rgba: Vec<u8> = bgra
                .data
                .chunks_exact(4)
                .flat_map(|b| [b[2], b[1], b[0], b[3]])
                .collect();
            (bgra.width as u32, bgra.height as u32, rgba)
        }
        _ => return Err("Unexpected frame format from screen capturer".to_string()),
    };

    let image = image::RgbaImage::from_raw(width, height, rgba_data)
        .ok_or_else(|| "Failed to construct image from frame data".to_string())?;

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image as PNG: {}", e))?;

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
