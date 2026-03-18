//! Hardware Abstraction Layer (DEBT-006)
//!
//! This module provides platform-agnostic interfaces for hardware-related operations
//! that have platform-specific implementations:
//!
//! - Window information retrieval (active window title, process name)
//! - Screenshot capture (single monitor, multi-monitor stitching)
//! - Display enumeration (monitor list, resolution, position)
//!
//! # Architecture
//!
//! The module uses trait-based abstraction to separate interface from implementation:
//!
//! ```ignore
//! hardware/
//! ├── mod.rs           # Trait definitions and factory functions
//! ├── platform.rs      # Platform-specific implementations
//! └── mock.rs          # Mock implementations for testing
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use crate::hardware::{get_window_provider, get_screenshot_provider};
//!
//! let window = get_window_provider().get_active_window();
//! let (image, info) = get_screenshot_provider().capture_screen(mode, index)?;
//! ```

use serde::{Deserialize, Serialize};

// Re-export types from other modules
pub use crate::monitor_types::{CaptureMode, MonitorDetail, MonitorInfo};
pub use crate::window_info::ActiveWindow;

/// Provider for retrieving active window information.
///
/// Implementations are platform-specific:
/// - Windows: Uses Win32 API (GetForegroundWindow, GetWindowTextW)
/// - macOS: Uses AppleScript via osascript command
/// - Linux: Uses xdotool command-line tool
pub trait WindowInfoProvider: Send + Sync {
    /// Get the currently active window's title and process name.
    ///
    /// Returns `ActiveWindow::default()` (empty strings) if the information
    /// cannot be retrieved (e.g., no desktop session, permissions denied).
    fn get_active_window(&self) -> ActiveWindow;

    /// Check if window filtering should block capture.
    ///
    /// This method delegates to the platform-agnostic `should_capture_by_window`
    /// function, but is included in the trait for consistency and testability.
    fn should_capture(
        &self,
        window: &ActiveWindow,
        whitelist: &[String],
        blacklist: &[String],
        use_whitelist_only: bool,
    ) -> bool {
        crate::window_info::should_capture_by_window(
            window,
            whitelist,
            blacklist,
            use_whitelist_only,
        )
    }
}

/// Provider for capturing screenshots.
///
/// Implementations use xcap library for cross-platform capture,
/// but may have platform-specific optimizations or workarounds.
pub trait ScreenshotProvider: Send + Sync {
    /// Capture screenshot with specified mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - Capture mode (Primary, Secondary, All)
    /// * `selected_index` - Monitor index for Secondary mode
    ///
    /// # Returns
    ///
    /// * `(base64_png, monitor_info)` - Base64-encoded PNG and monitor metadata
    ///
    /// # Errors
    ///
    /// Returns an error string if:
    /// - No monitors are detected
    /// - The specified monitor index is invalid
    /// - Screen capture fails (permissions, driver issues)
    fn capture_screen(
        &self,
        mode: CaptureMode,
        selected_index: usize,
    ) -> Result<(String, MonitorInfo), String>;

    /// Get list of all connected monitors.
    fn get_monitors(&self) -> Result<Vec<MonitorDetail>, String>;
}

/// Provider for display/monitor enumeration.
///
/// This is a separate trait for cases where only monitor information
/// is needed without capturing screenshots.
pub trait DisplayProvider: Send + Sync {
    /// Get list of all connected monitors with their properties.
    fn get_monitor_list(&self) -> Result<Vec<MonitorDetail>, String>;

    /// Get simplified monitor summary for frontend display.
    fn get_monitor_summaries(&self) -> Result<Vec<MonitorSummary>, String>;
}

/// Monitor summary for frontend display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorSummary {
    pub index: usize,
    pub name: String,
    pub resolution: String,
    pub is_primary: bool,
}

// ─── Factory functions ───────────────────────────────────────────────────────

/// Get the platform-specific window info provider.
///
/// This returns a static reference to avoid repeated allocations.
pub fn get_window_provider() -> &'static dyn WindowInfoProvider {
    platform::get_window_provider()
}

/// Get the platform-specific screenshot provider.
///
/// This returns a static reference to avoid repeated allocations.
pub fn get_screenshot_provider() -> &'static dyn ScreenshotProvider {
    platform::get_screenshot_provider()
}

/// Get the platform-specific display provider.
///
/// This returns a static reference to avoid repeated allocations.
pub fn get_display_provider() -> &'static dyn DisplayProvider {
    platform::get_display_provider()
}

// ─── Platform-specific implementations ───────────────────────────────────────

mod platform;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_provider_trait_is_object_safe() {
        // Verify that the trait is object-safe (can be used as &dyn Trait)
        fn _uses_dyn_trait(_: &dyn WindowInfoProvider) {}
    }

    #[test]
    fn test_screenshot_provider_trait_is_object_safe() {
        fn _uses_dyn_trait(_: &dyn ScreenshotProvider) {}
    }

    #[test]
    fn test_display_provider_trait_is_object_safe() {
        fn _uses_dyn_trait(_: &dyn DisplayProvider) {}
    }

    #[test]
    fn test_get_window_provider_returns_valid_provider() {
        let provider = get_window_provider();
        // Should not panic when calling get_active_window
        let _ = provider.get_active_window();
    }

    #[test]
    fn test_get_screenshot_provider_returns_valid_provider() {
        let provider = get_screenshot_provider();
        // Should be able to get monitors (may fail in CI without display)
        let _ = provider.get_monitors();
    }

    #[test]
    fn test_get_display_provider_returns_valid_provider() {
        let provider = get_display_provider();
        // Should be able to get monitor list (may fail in CI without display)
        let _ = provider.get_monitor_list();
    }
}
