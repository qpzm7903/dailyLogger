//! Mock implementations of hardware abstraction traits for testing.
//!
//! These implementations allow unit tests to run without requiring
//! actual hardware (display, window manager) access.

use super::{DisplayProvider, MonitorDetail, MonitorSummary, ScreenshotProvider, WindowInfoProvider};
use crate::monitor_types::{CaptureMode, MonitorInfo};
use crate::window_info::ActiveWindow;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Mock window info provider that returns configurable responses.
pub struct MockWindowProvider {
    /// The window to return on next call.
    next_window: std::sync::Mutex<Option<ActiveWindow>>,
    /// Call counter.
    call_count: AtomicUsize,
}

impl Default for MockWindowProvider {
    fn default() -> Self {
        Self {
            next_window: std::sync::Mutex::new(None),
            call_count: AtomicUsize::new(0),
        }
    }
}

impl MockWindowProvider {
    /// Set the window to return on next `get_active_window()` call.
    pub fn set_next_window(&self, window: ActiveWindow) {
        let mut next = self.next_window.lock().unwrap();
        *next = Some(window);
    }

    /// Get the number of times `get_active_window()` was called.
    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

impl WindowInfoProvider for MockWindowProvider {
    fn get_active_window(&self) -> ActiveWindow {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let mut next = self.next_window.lock().unwrap();
        next.take().unwrap_or_default()
    }
}

/// Mock screenshot provider that returns configurable responses.
pub struct MockScreenshotProvider {
    /// The image to return on next capture.
    next_image: std::sync::Mutex<Option<String>>,
    /// Monitors to report.
    monitors: std::sync::Mutex<Vec<MonitorDetail>>,
    /// Call counter.
    call_count: AtomicUsize,
}

impl Default for MockScreenshotProvider {
    fn default() -> Self {
        Self {
            next_image: std::sync::Mutex::new(None),
            monitors: std::sync::Mutex::new(vec![MonitorDetail {
                index: 0,
                name: "Mock Monitor".to_string(),
                width: 1920,
                height: 1080,
                x: 0,
                y: 0,
                is_primary: true,
            }]),
            call_count: AtomicUsize::new(0),
        }
    }
}

impl MockScreenshotProvider {
    /// Set the base64 image to return on next capture.
    pub fn set_next_image(&self, image: String) {
        let mut next = self.next_image.lock().unwrap();
        *next = Some(image);
    }

    /// Set the monitors to report.
    pub fn set_monitors(&self, monitors: Vec<MonitorDetail>) {
        let mut m = self.monitors.lock().unwrap();
        *m = monitors;
    }

    /// Get the number of times `capture_screen()` was called.
    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

impl ScreenshotProvider for MockScreenshotProvider {
    fn capture_screen(
        &self,
        _mode: CaptureMode,
        _selected_index: usize,
    ) -> Result<(String, MonitorInfo), String> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let next = self.next_image.lock().unwrap();
        let image = next.clone().unwrap_or_else(|| {
            // Return a minimal 1x1 white PNG
            "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==".to_string()
        });
        let monitors = self.monitors.lock().unwrap();
        let monitor_info = MonitorInfo {
            count: monitors.len(),
            monitors: monitors.clone(),
        };
        Ok((image, monitor_info))
    }

    fn get_monitors(&self) -> Result<Vec<MonitorDetail>, String> {
        let monitors = self.monitors.lock().unwrap();
        Ok(monitors.clone())
    }
}

/// Mock display provider that returns configurable responses.
pub struct MockDisplayProvider {
    /// Monitors to report.
    monitors: std::sync::Mutex<Vec<MonitorDetail>>,
}

impl Default for MockDisplayProvider {
    fn default() -> Self {
        Self {
            monitors: std::sync::Mutex::new(vec![MonitorDetail {
                index: 0,
                name: "Mock Monitor".to_string(),
                width: 1920,
                height: 1080,
                x: 0,
                y: 0,
                is_primary: true,
            }]),
        }
    }
}

impl MockDisplayProvider {
    /// Set the monitors to report.
    pub fn set_monitors(&self, monitors: Vec<MonitorDetail>) {
        let mut m = self.monitors.lock().unwrap();
        *m = monitors;
    }
}

impl DisplayProvider for MockDisplayProvider {
    fn get_monitor_list(&self) -> Result<Vec<MonitorDetail>, String> {
        let monitors = self.monitors.lock().unwrap();
        Ok(monitors.clone())
    }

    fn get_monitor_summaries(&self) -> Result<Vec<MonitorSummary>, String> {
        let monitors = self.monitors.lock().unwrap();
        Ok(monitors
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_window_provider_returns_default() {
        let provider = MockWindowProvider::default();
        let window = provider.get_active_window();
        assert!(window.title.is_empty());
        assert!(window.process_name.is_empty());
    }

    #[test]
    fn test_mock_window_provider_returns_configured() {
        let provider = MockWindowProvider::default();
        provider.set_next_window(ActiveWindow {
            title: "Test Window".to_string(),
            process_name: "test_app".to_string(),
        });
        let window = provider.get_active_window();
        assert_eq!(window.title, "Test Window");
        assert_eq!(window.process_name, "test_app");
    }

    #[test]
    fn test_mock_window_provider_counts_calls() {
        let provider = MockWindowProvider::default();
        assert_eq!(provider.call_count(), 0);
        let _ = provider.get_active_window();
        assert_eq!(provider.call_count(), 1);
        let _ = provider.get_active_window();
        assert_eq!(provider.call_count(), 2);
    }

    #[test]
    fn test_mock_screenshot_provider_returns_default() {
        let provider = MockScreenshotProvider::default();
        let (image, info) = provider.capture_screen(CaptureMode::Primary, 0).unwrap();
        assert!(!image.is_empty());
        assert_eq!(info.count, 1);
    }

    #[test]
    fn test_mock_screenshot_provider_returns_configured() {
        let provider = MockScreenshotProvider::default();
        provider.set_next_image("test_image_base64".to_string());
        let (image, _) = provider.capture_screen(CaptureMode::Primary, 0).unwrap();
        assert_eq!(image, "test_image_base64");
    }

    #[test]
    fn test_mock_display_provider_returns_default() {
        let provider = MockDisplayProvider::default();
        let monitors = provider.get_monitor_list().unwrap();
        assert_eq!(monitors.len(), 1);
        assert!(monitors[0].is_primary);
    }

    #[test]
    fn test_mock_display_provider_returns_summaries() {
        let provider = MockDisplayProvider::default();
        let summaries = provider.get_monitor_summaries().unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].resolution, "1920x1080");
    }
}