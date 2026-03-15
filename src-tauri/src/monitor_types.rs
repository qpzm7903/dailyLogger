//! SMART-004: Multi-monitor support - Type definitions
//!
//! Types for monitor capture mode and display information.
//! These types are available regardless of the screenshot feature.

use serde::{Deserialize, Serialize};

/// Display capture mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CaptureMode {
    /// Capture primary monitor only
    #[default]
    Primary,
    /// Capture a specific secondary monitor
    Secondary,
    /// Capture all monitors and stitch together
    All,
}

impl std::fmt::Display for CaptureMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureMode::Primary => write!(f, "primary"),
            CaptureMode::Secondary => write!(f, "secondary"),
            CaptureMode::All => write!(f, "all"),
        }
    }
}

impl std::str::FromStr for CaptureMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "primary" => Ok(CaptureMode::Primary),
            "secondary" => Ok(CaptureMode::Secondary),
            "all" => Ok(CaptureMode::All),
            _ => Err(format!("Invalid capture mode: {}", s)),
        }
    }
}

/// Detailed monitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorDetail {
    /// Monitor index (0-based)
    pub index: usize,
    /// Monitor name
    pub name: String,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// X position in virtual screen coordinates
    pub x: i32,
    /// Y position in virtual screen coordinates
    pub y: i32,
    /// Whether this is the primary monitor
    pub is_primary: bool,
}

/// Simplified monitor summary for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorSummary {
    /// Monitor index (0-based)
    pub index: usize,
    /// Monitor name
    pub name: String,
    /// Resolution string (e.g., "1920x1080")
    pub resolution: String,
    /// Whether this is the primary monitor
    pub is_primary: bool,
}

/// Monitor information stored with each capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    /// Number of monitors detected
    pub count: usize,
    /// Details of each monitor
    pub monitors: Vec<MonitorDetail>,
}

// ─── Tests (available without screenshot feature) ───────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_mode_default() {
        assert_eq!(CaptureMode::default(), CaptureMode::Primary);
    }

    #[test]
    fn test_capture_mode_display() {
        assert_eq!(CaptureMode::Primary.to_string(), "primary");
        assert_eq!(CaptureMode::Secondary.to_string(), "secondary");
        assert_eq!(CaptureMode::All.to_string(), "all");
    }

    #[test]
    fn test_capture_mode_from_str() {
        assert_eq!("primary".parse::<CaptureMode>(), Ok(CaptureMode::Primary));
        assert_eq!(
            "SECONDARY".parse::<CaptureMode>(),
            Ok(CaptureMode::Secondary)
        );
        assert_eq!("all".parse::<CaptureMode>(), Ok(CaptureMode::All));
        assert!("invalid".parse::<CaptureMode>().is_err());
    }

    #[test]
    fn test_capture_mode_serde() {
        let mode = CaptureMode::Secondary;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"secondary\"");

        let parsed: CaptureMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, CaptureMode::Secondary);
    }

    #[test]
    fn test_monitor_summary_resolution_format() {
        let summary = MonitorSummary {
            index: 0,
            name: "Primary".to_string(),
            resolution: "1920x1080".to_string(),
            is_primary: true,
        };
        assert_eq!(summary.resolution, "1920x1080");
    }

    #[test]
    fn test_monitor_detail_serialization() {
        let detail = MonitorDetail {
            index: 0,
            name: "Monitor 1".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            is_primary: true,
        };

        let json = serde_json::to_string(&detail).unwrap();
        let parsed: MonitorDetail = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.index, 0);
        assert_eq!(parsed.width, 1920);
        assert_eq!(parsed.height, 1080);
        assert!(parsed.is_primary);
    }

    #[test]
    fn test_monitor_info_creation() {
        let info = MonitorInfo {
            count: 2,
            monitors: vec![
                MonitorDetail {
                    index: 0,
                    name: "Monitor 1".to_string(),
                    width: 1920,
                    height: 1080,
                    x: 0,
                    y: 0,
                    is_primary: true,
                },
                MonitorDetail {
                    index: 1,
                    name: "Monitor 2".to_string(),
                    width: 2560,
                    height: 1440,
                    x: 1920,
                    y: 0,
                    is_primary: false,
                },
            ],
        };

        assert_eq!(info.count, 2);
        assert_eq!(info.monitors.len(), 2);
    }

    #[test]
    fn test_monitor_summary_from_detail() {
        let detail = MonitorDetail {
            index: 0,
            name: "Monitor 1".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            is_primary: true,
        };

        let summary = MonitorSummary {
            index: detail.index,
            name: detail.name.clone(),
            resolution: format!("{}x{}", detail.width, detail.height),
            is_primary: detail.is_primary,
        };

        assert_eq!(summary.index, 0);
        assert_eq!(summary.name, "Monitor 1");
        assert_eq!(summary.resolution, "1920x1080");
        assert!(summary.is_primary);
    }
}
