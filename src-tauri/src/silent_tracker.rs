//! Silent pattern tracking module for SMART-002.
//!
//! Tracks user capture behavior patterns to enable intelligent adjustment
//! of the max_silent_minutes threshold.

use chrono::{Duration, Local, NaiveDate, Timelike};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;

use once_cell::sync::Lazy;

/// Maximum number of days to keep in the sliding window.
const MAX_HISTORY_DAYS: i64 = 7;

/// Reason why a capture was triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureReason {
    /// Screen content changed significantly (above change_threshold).
    ScreenChanged,
    /// No screen change but max_silent_minutes exceeded.
    SilentTimeout,
    /// User manually triggered the capture.
    ManualTrigger,
}

/// Statistics for a single hour of a single day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStats {
    /// Date (YYYY-MM-DD).
    pub date: NaiveDate,
    /// Hour of day (0-23).
    pub hour: u8,
    /// Number of captures triggered by silent timeout.
    pub silent_captures: u32,
    /// Number of captures triggered by screen change.
    pub change_captures: u32,
}

impl HourlyStats {
    /// Create a new HourlyStats for the given date and hour.
    pub fn new(date: NaiveDate, hour: u8) -> Self {
        Self {
            date,
            hour,
            silent_captures: 0,
            change_captures: 0,
        }
    }

    /// Total captures for this hour.
    pub fn total_captures(&self) -> u32 {
        self.silent_captures + self.change_captures
    }

    /// Record a capture with the given reason.
    pub fn record_capture(&mut self, reason: CaptureReason) {
        match reason {
            CaptureReason::ScreenChanged | CaptureReason::ManualTrigger => {
                self.change_captures += 1;
            }
            CaptureReason::SilentTimeout => {
                self.silent_captures += 1;
            }
        }
    }
}

/// Aggregated statistics over a time period.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaptureStats {
    /// Total captures in the period.
    pub total_captures: u32,
    /// Captures triggered by silent timeout.
    pub silent_timeout_captures: u32,
    /// Captures triggered by screen change.
    pub screen_change_captures: u32,
    /// Current threshold in minutes (at the time of stats collection).
    pub current_threshold: u64,
}

impl CaptureStats {
    /// Ratio of silent timeout captures to total captures.
    /// Returns 0.0 if total_captures is 0.
    pub fn silent_ratio(&self) -> f64 {
        if self.total_captures == 0 {
            0.0
        } else {
            self.silent_timeout_captures as f64 / self.total_captures as f64
        }
    }
}

/// Memory-based tracker for capture behavior patterns.
///
/// Maintains a sliding window of hourly statistics for the last 7 days.
/// Used to determine optimal max_silent_minutes threshold based on
/// user's work patterns.
pub struct SilentPatternTracker {
    /// Hourly statistics within the sliding window.
    hourly_stats: Vec<HourlyStats>,
    /// Current max_silent_minutes threshold.
    current_threshold: u64,
    /// Time of last adjustment (if any).
    last_adjustment: Option<Instant>,
    /// Consecutive silent timeout captures (for immediate pattern detection).
    consecutive_silent_captures: u32,
    /// Consecutive screen change captures (for immediate pattern detection).
    consecutive_change_captures: u32,
    /// Last capture reason.
    last_capture_reason: Option<CaptureReason>,
}

impl SilentPatternTracker {
    /// Create a new tracker with the given initial threshold.
    pub fn new(initial_threshold: u64) -> Self {
        Self {
            hourly_stats: Vec::new(),
            current_threshold: initial_threshold,
            last_adjustment: None,
            consecutive_silent_captures: 0,
            consecutive_change_captures: 0,
            last_capture_reason: None,
        }
    }

    /// Record a capture event with the given reason.
    ///
    /// Updates hourly statistics and consecutive capture counters.
    pub fn record_capture(&mut self, reason: CaptureReason) {
        let now = Local::now();
        let date = now.date_naive();
        let hour = now.hour() as u8;

        // Update consecutive counters
        match reason {
            CaptureReason::SilentTimeout => {
                self.consecutive_silent_captures += 1;
                self.consecutive_change_captures = 0;
            }
            CaptureReason::ScreenChanged | CaptureReason::ManualTrigger => {
                self.consecutive_change_captures += 1;
                self.consecutive_silent_captures = 0;
            }
        }
        self.last_capture_reason = Some(reason);

        // Find or create hourly stats entry
        if let Some(stats) = self
            .hourly_stats
            .iter_mut()
            .find(|s| s.date == date && s.hour == hour)
        {
            stats.record_capture(reason);
        } else {
            let mut stats = HourlyStats::new(date, hour);
            stats.record_capture(reason);
            self.hourly_stats.push(stats);
        }

        // Prune old entries (outside 7-day window)
        self.prune_old_entries();
    }

    /// Remove entries older than MAX_HISTORY_DAYS.
    fn prune_old_entries(&mut self) {
        let cutoff_date = Local::now().date_naive() - Duration::days(MAX_HISTORY_DAYS);
        self.hourly_stats
            .retain(|s| s.date > cutoff_date);
    }

    /// Get aggregated statistics for the last `duration`.
    ///
    /// Returns stats for the last 24 hours if duration is longer than that.
    pub fn get_recent_stats(&self, duration: Duration) -> CaptureStats {
        let now = Local::now();
        let cutoff = now - duration;

        let mut stats = CaptureStats {
            current_threshold: self.current_threshold,
            ..Default::default()
        };

        for hourly in &self.hourly_stats {
            // Check if this hour is within the duration window
            let hour_start = hourly.date.and_hms_opt(hourly.hour as u32, 0, 0).unwrap();
            let hour_dt = hour_start.and_local_timezone(Local).unwrap();

            if hour_dt >= cutoff {
                stats.total_captures += hourly.total_captures();
                stats.silent_timeout_captures += hourly.silent_captures;
                stats.screen_change_captures += hourly.change_captures;
            }
        }

        stats
    }

    /// Get the current max_silent_minutes threshold.
    pub fn current_threshold(&self) -> u64 {
        self.current_threshold
    }

    /// Set the current threshold (called after adjustment).
    pub fn set_threshold(&mut self, threshold: u64) {
        self.current_threshold = threshold;
        self.last_adjustment = Some(Instant::now());
    }

    /// Get the number of consecutive silent timeout captures.
    pub fn consecutive_silent_captures(&self) -> u32 {
        self.consecutive_silent_captures
    }

    /// Get the number of consecutive screen change captures.
    pub fn consecutive_change_captures(&self) -> u32 {
        self.consecutive_change_captures
    }

    /// Get the last capture reason.
    pub fn last_capture_reason(&self) -> Option<CaptureReason> {
        self.last_capture_reason
    }

    /// Check if enough data has been collected for adjustment.
    /// Returns true if there are at least 10 captures in the last 24 hours.
    pub fn has_sufficient_data(&self) -> bool {
        let stats = self.get_recent_stats(Duration::hours(24));
        stats.total_captures >= 10
    }

    /// Get all hourly stats (for debugging/testing).
    pub fn hourly_stats(&self) -> &[HourlyStats] {
        &self.hourly_stats
    }

    /// Clear all statistics (for testing).
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.hourly_stats.clear();
        self.consecutive_silent_captures = 0;
        self.consecutive_change_captures = 0;
        self.last_capture_reason = None;
    }
}

impl Default for SilentPatternTracker {
    fn default() -> Self {
        Self::new(30) // Default 30 minutes
    }
}

/// Global silent pattern tracker instance.
static SILENT_PATTERN_TRACKER: Lazy<Mutex<SilentPatternTracker>> =
    Lazy::new(|| Mutex::new(SilentPatternTracker::default()));

/// Record a capture event in the global tracker.
pub fn record_capture(reason: CaptureReason) {
    if let Ok(mut tracker) = SILENT_PATTERN_TRACKER.lock() {
        tracker.record_capture(reason);
    }
}

/// Get aggregated statistics from the global tracker.
pub fn get_recent_stats(duration: Duration) -> CaptureStats {
    if let Ok(tracker) = SILENT_PATTERN_TRACKER.lock() {
        tracker.get_recent_stats(duration)
    } else {
        CaptureStats::default()
    }
}

/// Get the current threshold from the global tracker.
pub fn current_threshold() -> u64 {
    if let Ok(tracker) = SILENT_PATTERN_TRACKER.lock() {
        tracker.current_threshold()
    } else {
        30 // Default fallback
    }
}

/// Set the threshold in the global tracker.
pub fn set_threshold(threshold: u64) {
    if let Ok(mut tracker) = SILENT_PATTERN_TRACKER.lock() {
        tracker.set_threshold(threshold);
    }
}

/// Get consecutive capture counts from the global tracker.
pub fn consecutive_captures() -> (u32, u32) {
    if let Ok(tracker) = SILENT_PATTERN_TRACKER.lock() {
        (
            tracker.consecutive_silent_captures(),
            tracker.consecutive_change_captures(),
        )
    } else {
        (0, 0)
    }
}

/// Check if enough data has been collected for adjustment.
pub fn has_sufficient_data() -> bool {
    if let Ok(tracker) = SILENT_PATTERN_TRACKER.lock() {
        tracker.has_sufficient_data()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tracker() -> SilentPatternTracker {
        SilentPatternTracker::new(30)
    }

    // ── HourlyStats Tests ──

    #[test]
    fn hourly_stats_new_creates_empty_stats() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let stats = HourlyStats::new(date, 10);

        assert_eq!(stats.date, date);
        assert_eq!(stats.hour, 10);
        assert_eq!(stats.silent_captures, 0);
        assert_eq!(stats.change_captures, 0);
        assert_eq!(stats.total_captures(), 0);
    }

    #[test]
    fn hourly_stats_record_capture_screen_change() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let mut stats = HourlyStats::new(date, 10);

        stats.record_capture(CaptureReason::ScreenChanged);
        assert_eq!(stats.change_captures, 1);
        assert_eq!(stats.silent_captures, 0);
        assert_eq!(stats.total_captures(), 1);

        stats.record_capture(CaptureReason::ManualTrigger);
        assert_eq!(stats.change_captures, 2);
        assert_eq!(stats.silent_captures, 0);
        assert_eq!(stats.total_captures(), 2);
    }

    #[test]
    fn hourly_stats_record_capture_silent_timeout() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let mut stats = HourlyStats::new(date, 10);

        stats.record_capture(CaptureReason::SilentTimeout);
        assert_eq!(stats.change_captures, 0);
        assert_eq!(stats.silent_captures, 1);
        assert_eq!(stats.total_captures(), 1);
    }

    // ── CaptureStats Tests ──

    #[test]
    fn capture_stats_silent_ratio_zero_when_empty() {
        let stats = CaptureStats::default();
        assert_eq!(stats.silent_ratio(), 0.0);
    }

    #[test]
    fn capture_stats_silent_ratio_calculates_correctly() {
        let stats = CaptureStats {
            total_captures: 10,
            silent_timeout_captures: 7,
            screen_change_captures: 3,
            current_threshold: 30,
        };
        assert!((stats.silent_ratio() - 0.7).abs() < 0.001);
    }

    // ── SilentPatternTracker Tests ──

    #[test]
    fn tracker_new_initializes_with_threshold() {
        let tracker = create_test_tracker();
        assert_eq!(tracker.current_threshold(), 30);
        assert_eq!(tracker.consecutive_silent_captures(), 0);
        assert_eq!(tracker.consecutive_change_captures(), 0);
        assert!(tracker.last_capture_reason().is_none());
    }

    #[test]
    fn tracker_record_capture_creates_hourly_entry() {
        let mut tracker = create_test_tracker();
        tracker.record_capture(CaptureReason::ScreenChanged);

        assert_eq!(tracker.hourly_stats().len(), 1);
        assert_eq!(tracker.last_capture_reason(), Some(CaptureReason::ScreenChanged));
    }

    #[test]
    fn tracker_record_capture_updates_consecutive_counters() {
        let mut tracker = create_test_tracker();

        // Record 3 silent captures
        tracker.record_capture(CaptureReason::SilentTimeout);
        tracker.record_capture(CaptureReason::SilentTimeout);
        tracker.record_capture(CaptureReason::SilentTimeout);

        assert_eq!(tracker.consecutive_silent_captures(), 3);
        assert_eq!(tracker.consecutive_change_captures(), 0);

        // Record a screen change - should reset silent counter
        tracker.record_capture(CaptureReason::ScreenChanged);

        assert_eq!(tracker.consecutive_silent_captures(), 0);
        assert_eq!(tracker.consecutive_change_captures(), 1);
    }

    #[test]
    fn tracker_record_capture_updates_same_hour() {
        let mut tracker = create_test_tracker();

        // Record multiple captures in the same hour
        tracker.record_capture(CaptureReason::ScreenChanged);
        tracker.record_capture(CaptureReason::SilentTimeout);
        tracker.record_capture(CaptureReason::ScreenChanged);

        // Should still be a single hourly entry
        assert_eq!(tracker.hourly_stats().len(), 1);
        let hourly = &tracker.hourly_stats()[0];
        assert_eq!(hourly.change_captures, 2);
        assert_eq!(hourly.silent_captures, 1);
    }

    #[test]
    fn tracker_get_recent_stats_aggregates_correctly() {
        let mut tracker = create_test_tracker();

        // Record some captures
        tracker.record_capture(CaptureReason::ScreenChanged);
        tracker.record_capture(CaptureReason::ScreenChanged);
        tracker.record_capture(CaptureReason::SilentTimeout);

        let stats = tracker.get_recent_stats(Duration::hours(24));
        assert_eq!(stats.total_captures, 3);
        assert_eq!(stats.screen_change_captures, 2);
        assert_eq!(stats.silent_timeout_captures, 1);
    }

    #[test]
    fn tracker_set_threshold_updates_value() {
        let mut tracker = create_test_tracker();
        assert_eq!(tracker.current_threshold(), 30);

        tracker.set_threshold(45);
        assert_eq!(tracker.current_threshold(), 45);
        assert!(tracker.last_adjustment.is_some());
    }

    #[test]
    fn tracker_has_sufficient_data_requires_10_captures() {
        let mut tracker = create_test_tracker();

        // Record 9 captures
        for _ in 0..9 {
            tracker.record_capture(CaptureReason::ScreenChanged);
        }
        assert!(!tracker.has_sufficient_data());

        // Record 10th capture
        tracker.record_capture(CaptureReason::ScreenChanged);
        assert!(tracker.has_sufficient_data());
    }

    #[test]
    fn tracker_prunes_old_entries() {
        let mut tracker = create_test_tracker();

        // Manually add an old entry
        let old_date = Local::now().date_naive() - Duration::days(10);
        let old_stats = HourlyStats::new(old_date, 10);
        tracker.hourly_stats.push(old_stats);

        // Record a new capture which triggers pruning
        tracker.record_capture(CaptureReason::ScreenChanged);

        // Old entry should be removed
        assert!(!tracker.hourly_stats().iter().any(|s| s.date == old_date));
    }

    // ── Global Tracker Tests ──

    #[test]
    fn global_record_capture_works() {
        // Clear any existing state
        {
            let mut tracker = SILENT_PATTERN_TRACKER.lock().unwrap();
            tracker.clear();
        }

        record_capture(CaptureReason::ScreenChanged);
        record_capture(CaptureReason::SilentTimeout);

        let stats = get_recent_stats(Duration::hours(24));
        assert!(stats.total_captures >= 2);
    }

    #[test]
    fn global_threshold_operations() {
        set_threshold(45);
        assert_eq!(current_threshold(), 45);

        // Reset to default
        set_threshold(30);
    }

    #[test]
    fn global_consecutive_captures() {
        // Clear state
        {
            let mut tracker = SILENT_PATTERN_TRACKER.lock().unwrap();
            tracker.clear();
        }

        record_capture(CaptureReason::SilentTimeout);
        record_capture(CaptureReason::SilentTimeout);

        let (silent, change) = consecutive_captures();
        assert!(silent >= 2);
        assert_eq!(change, 0);
    }

    // ── AC1 Scenario Tests ──

    /// AC1: Given 系统检测到用户持续活跃（屏幕频繁变化）
    /// When 活跃持续时间超过当前阈值 50%
    /// Then 记录此行为模式用于调整
    #[test]
    fn ac1_detects_continuous_activity() {
        let mut tracker = create_test_tracker();

        // Simulate continuous screen changes (active user)
        for _ in 0..10 {
            tracker.record_capture(CaptureReason::ScreenChanged);
        }

        assert_eq!(tracker.consecutive_change_captures(), 10);
        assert!(tracker.has_sufficient_data());

        let stats = tracker.get_recent_stats(Duration::hours(24));
        assert_eq!(stats.silent_ratio(), 0.0); // All screen changes
    }

    /// AC1: Given 自动捕获功能运行超过 3 天
    /// When 系统收集了足够的工作模式数据
    /// Then 计算出用户的典型静默时段分布
    #[test]
    fn ac1_calculates_silent_pattern_distribution() {
        let mut tracker = create_test_tracker();

        // Simulate mixed pattern: 70% silent, 30% screen changes
        for _ in 0..7 {
            tracker.record_capture(CaptureReason::SilentTimeout);
        }
        for _ in 0..3 {
            tracker.record_capture(CaptureReason::ScreenChanged);
        }

        let stats = tracker.get_recent_stats(Duration::hours(24));
        assert_eq!(stats.total_captures, 10);
        assert!((stats.silent_ratio() - 0.7).abs() < 0.001);
    }

    /// Test that manual trigger is counted as screen change (user-initiated)
    #[test]
    fn manual_trigger_counts_as_change() {
        let mut tracker = create_test_tracker();

        tracker.record_capture(CaptureReason::ManualTrigger);

        assert_eq!(tracker.consecutive_change_captures(), 1);
        assert_eq!(tracker.consecutive_silent_captures(), 0);
        assert_eq!(tracker.last_capture_reason(), Some(CaptureReason::ManualTrigger));
    }
}