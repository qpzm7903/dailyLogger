//! Mobile platform stub implementations for hardware abstraction traits.
//!
//! Mobile platforms (Android/iOS) have different capabilities than desktop:
//! - No traditional window management (single app focus)
//! - Screenshot requires different APIs (MediaProjection on Android, none on iOS without entitlements)
//! - No multi-monitor support
//!
//! This module provides no-op/stub implementations that return sensible defaults.

use super::{DisplayProvider, MonitorSummary, ScreenshotProvider, WindowInfoProvider};
use crate::monitor_types::{CaptureMode, MonitorDetail, MonitorInfo};
use crate::window_info::ActiveWindow;

/// Mobile window info provider - returns empty window info.
///
/// Mobile platforms don't have traditional window management.
/// The app is always the foreground app when active.
pub struct MobileWindowProvider;

impl WindowInfoProvider for MobileWindowProvider {
    fn get_active_window(&self) -> ActiveWindow {
        // On mobile, we can't get window info from other apps
        // Return default (empty) window info
        ActiveWindow::default()
    }
}

/// Mobile screenshot provider - returns error (not supported).
///
/// Screenshot capture on mobile requires:
/// - Android: MediaProjection API (requires user permission per session)
/// - iOS: Not available without screen recording entitlements
///
/// For now, this feature is disabled on mobile. Users would need to
/// manually input their activities or use a different capture mechanism.
pub struct MobileScreenshotProvider;

impl ScreenshotProvider for MobileScreenshotProvider {
    fn capture_screen(
        &self,
        _mode: CaptureMode,
        _selected_index: usize,
    ) -> Result<(String, MonitorInfo), String> {
        Err("Screenshot capture is not supported on mobile platforms".to_string())
    }

    fn get_monitors(&self) -> Result<Vec<MonitorDetail>, String> {
        // Mobile devices have a single "monitor" (the screen)
        Ok(vec![MonitorDetail {
            index: 0,
            name: "Mobile Screen".to_string(),
            width: 1080,  // Placeholder - actual value would come from device
            height: 1920, // Placeholder
            x: 0,
            y: 0,
            is_primary: true,
        }])
    }
}

/// Mobile display provider - returns single mobile screen.
pub struct MobileDisplayProvider;

impl DisplayProvider for MobileDisplayProvider {
    fn get_monitor_list(&self) -> Result<Vec<MonitorDetail>, String> {
        // Mobile devices have a single screen
        Ok(vec![MonitorDetail {
            index: 0,
            name: "Mobile Screen".to_string(),
            width: 1080,  // Placeholder
            height: 1920, // Placeholder
            x: 0,
            y: 0,
            is_primary: true,
        }])
    }

    fn get_monitor_summaries(&self) -> Result<Vec<MonitorSummary>, String> {
        Ok(vec![MonitorSummary {
            index: 0,
            name: "Mobile Screen".to_string(),
            resolution: "1080x1920".to_string(),
            is_primary: true,
        }])
    }
}

// ─── Static provider instances for mobile ─────────────────────────────────────

static MOBILE_WINDOW_PROVIDER: MobileWindowProvider = MobileWindowProvider;
static MOBILE_SCREENSHOT_PROVIDER: MobileScreenshotProvider = MobileScreenshotProvider;
static MOBILE_DISPLAY_PROVIDER: MobileDisplayProvider = MobileDisplayProvider;

/// Get the mobile window provider.
pub fn get_mobile_window_provider() -> &'static dyn WindowInfoProvider {
    &MOBILE_WINDOW_PROVIDER
}

/// Get the mobile screenshot provider.
pub fn get_mobile_screenshot_provider() -> &'static dyn ScreenshotProvider {
    &MOBILE_SCREENSHOT_PROVIDER
}

/// Get the mobile display provider.
pub fn get_mobile_display_provider() -> &'static dyn DisplayProvider {
    &MOBILE_DISPLAY_PROVIDER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_window_provider_returns_empty() {
        let provider = MobileWindowProvider;
        let window = provider.get_active_window();
        assert!(window.title.is_empty());
        assert!(window.process_name.is_empty());
    }

    #[test]
    fn test_mobile_screenshot_provider_returns_error() {
        let provider = MobileScreenshotProvider;
        let result = provider.capture_screen(CaptureMode::Primary, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not supported"));
    }

    #[test]
    fn test_mobile_screenshot_provider_returns_single_monitor() {
        let provider = MobileScreenshotProvider;
        let monitors = provider.get_monitors().unwrap();
        assert_eq!(monitors.len(), 1);
        assert!(monitors[0].is_primary);
    }

    #[test]
    fn test_mobile_display_provider_returns_single_screen() {
        let provider = MobileDisplayProvider;
        let summaries = provider.get_monitor_summaries().unwrap();
        assert_eq!(summaries.len(), 1);
        assert!(summaries[0].is_primary);
    }
}
