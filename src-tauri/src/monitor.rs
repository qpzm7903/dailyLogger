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

    let result: Vec<MonitorDetail> = monitors
        .iter()
        .enumerate()
        .map(|(index, m)| {
            // Note: friendly_name() can panic on some systems (e.g., CI runners)
            // Use name() which is more reliable, fallback to default
            let name = m
                .name()
                .unwrap_or_else(|_| format!("Monitor {}", index + 1));

            // xcap 0.9.x uses separate x(), y(), width(), height() methods
            // All return XCapResult, so we unwrap with sensible defaults
            let x = m.x().unwrap_or(0);
            let y = m.y().unwrap_or(0);
            let width = m.width().unwrap_or(0);
            let height = m.height().unwrap_or(0);

            // xcap 0.9.x provides is_primary() method
            let is_primary = m.is_primary().unwrap_or(index == 0);

            MonitorDetail {
                index,
                name,
                width,
                height,
                x,
                y,
                is_primary,
            }
        })
        .collect();

    Ok(result)
}

/// Get list of all connected monitors (Windows)
#[cfg(target_os = "windows")]
pub fn get_monitor_list() -> Result<Vec<MonitorDetail>, String> {
    use windows_capture::monitor::Monitor;

    // Enumerate monitors by trying indexes until we get an error
    let mut monitors = Vec::new();
    let mut index = 0;
    loop {
        match Monitor::from_index(index) {
            Ok(m) => {
                monitors.push(m);
                index += 1;
            }
            Err(_) => break, // No more monitors
        }
    }

    if monitors.is_empty() {
        return Err("No monitors found".to_string());
    }

    // Get primary monitor info
    let primary = Monitor::primary().ok();

    let result: Vec<MonitorDetail> = monitors
        .iter()
        .enumerate()
        .map(|(idx, m)| MonitorDetail {
            index: idx,
            name: format!("Monitor {}", idx + 1),
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
