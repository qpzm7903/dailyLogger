//! Platform-specific implementations of hardware abstraction traits.
//!
//! This module provides the actual implementations that delegate to the
//! existing platform-specific code in `window_info` and `auto_perception` modules.

use super::{DisplayProvider, MonitorSummary, ScreenshotProvider, WindowInfoProvider};
use crate::monitor_types::{CaptureMode, MonitorDetail, MonitorInfo};
use crate::window_info::ActiveWindow;

// ─── Window Info Provider ─────────────────────────────────────────────────────

/// Platform-specific window info provider.
///
/// Delegates to `window_info::get_active_window()` which has platform-specific
/// implementations for Windows, macOS, and Linux.
pub struct PlatformWindowProvider;

impl WindowInfoProvider for PlatformWindowProvider {
    fn get_active_window(&self) -> ActiveWindow {
        crate::window_info::get_active_window()
    }
}

// ─── Screenshot Provider ───────────────────────────────────────────────────────

/// Platform-specific screenshot provider.
///
/// Delegates to `auto_perception::capture_screen_with_mode()` which has
/// platform-specific implementations using xcap.
pub struct PlatformScreenshotProvider;

impl ScreenshotProvider for PlatformScreenshotProvider {
    fn capture_screen(
        &self,
        mode: CaptureMode,
        selected_index: usize,
    ) -> Result<(String, MonitorInfo), String> {
        // Use the existing platform-specific implementation
        // Note: We need to access the internal function from auto_perception
        // For now, we'll use the monitor module directly

        let monitor_details = crate::monitor::get_monitor_list().map_err(|e| e.to_string())?;
        let monitors =
            xcap::Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

        if monitors.is_empty() {
            return Err("No monitors found".to_string());
        }

        let monitor_info = MonitorInfo {
            count: monitor_details.len(),
            monitors: monitor_details.clone(),
        };

        // Capture based on mode
        let image = match mode {
            CaptureMode::Primary => {
                let primary_index = monitor_details
                    .iter()
                    .position(|m| m.is_primary)
                    .unwrap_or(0);
                capture_single_monitor(&monitors, primary_index)?
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
                capture_single_monitor(&monitors, index)?
            }
            CaptureMode::All => stitch_monitors(&monitors, &monitor_details)?,
        };

        Ok((image, monitor_info))
    }

    fn get_monitors(&self) -> Result<Vec<MonitorDetail>, String> {
        crate::monitor::get_monitor_list().map_err(|e| e.to_string())
    }
}

/// Capture a single monitor by index.
fn capture_single_monitor(monitors: &[xcap::Monitor], index: usize) -> Result<String, String> {
    let monitor = monitors
        .get(index)
        .ok_or_else(|| format!("Monitor index {} out of bounds", index))?;

    let image = monitor
        .capture_image()
        .map_err(|e| format!("Failed to capture monitor {}: {}", index, e))?;

    // Encode to base64 PNG
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image: {}", e))?;

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &buffer,
    ))
}

/// Stitch all monitors into a single image.
fn stitch_monitors(
    monitors: &[xcap::Monitor],
    monitor_details: &[MonitorDetail],
) -> Result<String, String> {
    if monitors.is_empty() {
        return Err("No monitors to stitch".to_string());
    }

    // Capture all monitor images
    let mut captured_images: Vec<(MonitorDetail, image::RgbaImage)> = Vec::new();

    for (index, _monitor) in monitors.iter().enumerate() {
        let image_base64 = capture_single_monitor(monitors, index)?;

        let image_data =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &image_base64)
                .map_err(|e| format!("Failed to decode captured image: {}", e))?;

        let img = image::load_from_memory(&image_data)
            .map_err(|e| format!("Failed to load image: {}", e))?;

        let rgba_image = img.to_rgba8();

        let detail = monitor_details
            .get(index)
            .cloned()
            .unwrap_or_else(|| MonitorDetail {
                index,
                name: format!("Monitor {}", index + 1),
                width: rgba_image.width(),
                height: rgba_image.height(),
                x: 0,
                y: 0,
                is_primary: index == 0,
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

/// Calculate the bounding box for all monitors.
fn calculate_monitor_bounds(
    monitors: &[(MonitorDetail, image::RgbaImage)],
) -> (i32, i32, i32, i32) {
    let min_x = monitors.iter().map(|(m, _)| m.x).min().unwrap_or(0);
    let min_y = monitors.iter().map(|(m, _)| m.y).min().unwrap_or(0);
    let max_x = monitors
        .iter()
        .map(|(m, img)| m.x + img.width() as i32)
        .max()
        .unwrap_or(0);
    let max_y = monitors
        .iter()
        .map(|(m, img)| m.y + img.height() as i32)
        .max()
        .unwrap_or(0);

    (min_x, min_y, max_x, max_y)
}

// ─── Display Provider ─────────────────────────────────────────────────────────

/// Platform-specific display provider.
///
/// Delegates to `monitor::get_monitor_list()` which uses xcap for
/// cross-platform monitor enumeration.
pub struct PlatformDisplayProvider;

impl DisplayProvider for PlatformDisplayProvider {
    fn get_monitor_list(&self) -> Result<Vec<MonitorDetail>, String> {
        crate::monitor::get_monitor_list()
    }

    fn get_monitor_summaries(&self) -> Result<Vec<MonitorSummary>, String> {
        let details = self.get_monitor_list()?;

        Ok(details
            .iter()
            .map(|m| MonitorSummary {
                index: m.index,
                name: m.name.clone(),
                resolution: format!("{}x{}", m.width, m.height),
                is_primary: m.is_primary,
            })
            .collect())
    }
}

// ─── Static provider instances ────────────────────────────────────────────────

/// Global window info provider instance.
static WINDOW_PROVIDER: PlatformWindowProvider = PlatformWindowProvider;

/// Global screenshot provider instance.
static SCREENSHOT_PROVIDER: PlatformScreenshotProvider = PlatformScreenshotProvider;

/// Global display provider instance.
static DISPLAY_PROVIDER: PlatformDisplayProvider = PlatformDisplayProvider;

/// Get the window provider as a trait object.
pub fn get_window_provider() -> &'static dyn WindowInfoProvider {
    &WINDOW_PROVIDER
}

/// Get the screenshot provider as a trait object.
pub fn get_screenshot_provider() -> &'static dyn ScreenshotProvider {
    &SCREENSHOT_PROVIDER
}

/// Get the display provider as a trait object.
pub fn get_display_provider() -> &'static dyn DisplayProvider {
    &DISPLAY_PROVIDER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_window_provider_returns_valid_struct() {
        let provider = PlatformWindowProvider;
        let window = provider.get_active_window();
        // Should return a valid struct (may be empty in CI)
        let _ = (window.title, window.process_name);
    }

    #[test]
    fn test_platform_display_provider_get_monitors() {
        let provider = PlatformDisplayProvider;
        // This may fail in CI without a display
        let _ = provider.get_monitor_list();
    }

    #[test]
    fn test_platform_display_provider_get_summaries() {
        let provider = PlatformDisplayProvider;
        // This may fail in CI without a display
        let _ = provider.get_monitor_summaries();
    }
}
