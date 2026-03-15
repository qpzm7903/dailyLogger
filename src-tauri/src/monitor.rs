//! SMART-004: Multi-monitor support module
//!
//! Provides monitor enumeration and capture functionality for multi-display environments.

use tauri::command;

// Re-export types for convenience
pub use crate::monitor_types::{CaptureMode, MonitorDetail, MonitorInfo, MonitorSummary};

// ─── Platform-specific implementations ───────────────────────────────────────

/// Get list of all connected monitors (non-Windows: macOS, Linux)
#[cfg(not(target_os = "windows"))]
pub fn get_monitor_list() -> Result<Vec<MonitorDetail>, String> {
    let monitors = xcap::Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    if monitors.is_empty() {
        return Err("No monitors found".to_string());
    }

    // Find primary monitor (usually the first one)
    let primary_monitor = monitors
        .iter()
        .find(|m| {
            // On most systems, primary monitor has position (0, 0)
            // xcap doesn't have a direct is_primary method, so we check position
            m.position().0 == 0 && m.position().1 == 0
        })
        .map(|m| m.id())
        .or_else(|| monitors.first().map(|m| m.id())); // Fallback to first monitor

    let result: Vec<MonitorDetail> = monitors
        .iter()
        .enumerate()
        .map(|(index, m)| {
            let (x, y) = m.position();
            let (width, height) = m.resolution();
            MonitorDetail {
                index,
                name: m.name().unwrap_or_else(|| format!("Monitor {}", index + 1)),
                width,
                height,
                x,
                y,
                is_primary: primary_monitor.map_or(index == 0, |id| m.id() == id),
            }
        })
        .collect();

    Ok(result)
}

/// Get list of all connected monitors (Windows)
#[cfg(target_os = "windows")]
pub fn get_monitor_list() -> Result<Vec<MonitorDetail>, String> {
    use windows_capture::monitor::Monitor;

    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    if monitors.is_empty() {
        return Err("No monitors found".to_string());
    }

    // Get primary monitor info
    let primary = Monitor::primary().ok();

    let result: Vec<MonitorDetail> = monitors
        .iter()
        .enumerate()
        .map(|(index, m)| MonitorDetail {
            index,
            name: format!("Monitor {}", index + 1),
            width: m.width().unwrap_or(0),
            height: m.height().unwrap_or(0),
            x: m.position().unwrap_or((0, 0)).0,
            y: m.position().unwrap_or((0, 0)).1,
            is_primary: primary.as_ref().map_or(false, |p| {
                m.width() == p.width() && m.height() == p.height()
            }),
        })
        .collect();

    Ok(result)
}

/// Get simplified monitor list for frontend
#[command]
pub fn get_monitors() -> Result<Vec<MonitorSummary>, String> {
    let details = get_monitor_list()?;

    let summaries: Vec<MonitorSummary> = details
        .iter()
        .map(|m| MonitorSummary {
            index: m.index,
            name: m.name.clone(),
            resolution: format!("{}x{}", m.width, m.height),
            is_primary: m.is_primary,
        })
        .collect();

    Ok(summaries)
}

/// Get full monitor info for storage
pub fn get_monitor_info() -> Result<MonitorInfo, String> {
    let monitors = get_monitor_list()?;
    let count = monitors.len();

    Ok(MonitorInfo { count, monitors })
}

// ─── Tests requiring screenshot feature (xcap dependency) ───────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_monitor_list_returns_at_least_one() {
        // This test requires a display, so it should succeed on any dev machine
        let result = get_monitor_list();
        assert!(result.is_ok());
        let monitors = result.unwrap();
        assert!(!monitors.is_empty());
        // At least one monitor should be marked as primary
        assert!(monitors.iter().any(|m| m.is_primary));
    }

    #[test]
    fn test_get_monitors_command() {
        let result = get_monitors();
        assert!(result.is_ok());
        let summaries = result.unwrap();
        assert!(!summaries.is_empty());
        // All summaries should have resolution in "WxH" format
        for s in &summaries {
            assert!(s.resolution.contains('x'));
        }
    }

    #[test]
    fn test_get_monitor_info() {
        let result = get_monitor_info();
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.count > 0);
        assert_eq!(info.count, info.monitors.len());
    }
}
