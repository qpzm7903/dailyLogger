//! Silent pattern tracking module for SMART-002.
//!
//! Tracks user capture behavior patterns to enable intelligent adjustment
//! of the max_silent_minutes threshold.

use chrono::{Duration, Local, NaiveDate, Timelike};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;

use once_cell::sync::Lazy;

use crate::errors::{AppError, AppResult};

/// Maximum number of days to keep in the sliding window.
const MAX_HISTORY_DAYS: i64 = 7;

/// Minimum allowed threshold for max_silent_minutes (in minutes).
pub const MIN_THRESHOLD: u64 = 10;

/// Maximum allowed threshold for max_silent_minutes (in minutes).
pub const MAX_THRESHOLD: u64 = 60;

/// Default threshold for max_silent_minutes (in minutes).
pub const DEFAULT_THRESHOLD: u64 = 30;

/// Maximum adjustment per evaluation (in minutes).
pub const MAX_ADJUSTMENT: u64 = 5;

/// Silent ratio threshold above which we increase the threshold (deep work).
pub const HIGH_SILENT_RATIO: f64 = 0.7;

/// Silent ratio threshold below which we decrease the threshold (active work).
pub const LOW_SILENT_RATIO: f64 = 0.3;

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
        self.hourly_stats.retain(|s| s.date > cutoff_date);
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
            let Some(hour_start) = hourly.date.and_hms_opt(hourly.hour as u32, 0, 0) else {
                continue;
            };
            let hour_dt = match hour_start.and_local_timezone(Local) {
                chrono::LocalResult::Single(dt) => dt,
                chrono::LocalResult::Ambiguous(dt, _) => dt,
                chrono::LocalResult::None => continue,
            };

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
        Self::new(DEFAULT_THRESHOLD)
    }
}

/// Calculate the optimal max_silent_minutes based on user's work patterns.
///
/// This is the core algorithm for AC2: automatically adjusting the silent threshold
/// based on detected user behavior patterns.
///
/// # Algorithm
///
/// 1. If insufficient data (< 10 captures in 24h), return current threshold
/// 2. Calculate the ratio of silent timeout captures to total captures
/// 3. If ratio >= 0.7: user is in deep work, increase threshold (max +5 min)
/// 4. If ratio <= 0.3: user is active, decrease threshold (max -5 min)
/// 5. Otherwise: balanced work, keep current threshold
/// 6. Clamp result to [MIN_THRESHOLD, MAX_THRESHOLD]
///
/// # Arguments
///
/// * `tracker` - Reference to the silent pattern tracker
///
/// # Returns
///
/// The recommended threshold in minutes.
pub fn calculate_optimal_silent_minutes(tracker: &SilentPatternTracker) -> u64 {
    let current = tracker.current_threshold();

    // Check if we have sufficient data
    if !tracker.has_sufficient_data() {
        return DEFAULT_THRESHOLD;
    }

    // Get stats for last 24 hours
    let stats = tracker.get_recent_stats(Duration::hours(24));
    let silent_ratio = stats.silent_ratio();

    // Apply adjustment based on silent ratio
    if silent_ratio >= HIGH_SILENT_RATIO {
        // Deep work detected: increase threshold
        (current + MAX_ADJUSTMENT).min(MAX_THRESHOLD)
    } else if silent_ratio <= LOW_SILENT_RATIO {
        // Active work detected: decrease threshold
        current.saturating_sub(MAX_ADJUSTMENT).max(MIN_THRESHOLD)
    } else {
        // Balanced work: keep current threshold
        current
    }
}

/// Global silent pattern tracker instance.
/// Uses lazy initialization - data is loaded from DB on first access, not at app startup.
/// This improves startup time by deferring database query until data is actually needed.
pub static SILENT_PATTERN_TRACKER: Lazy<Mutex<SilentPatternTracker>> =
    Lazy::new(|| Mutex::new(SilentPatternTracker::default()));

/// Record a capture event in the global tracker.
/// Also persists the capture to database for durability.
pub fn record_capture(reason: CaptureReason) {
    ensure_stats_loaded(); // Ensure data is loaded before recording
    if let Ok(mut tracker) = SILENT_PATTERN_TRACKER.lock() {
        tracker.record_capture(reason);
    }
    // Persist to database (non-blocking, ignore errors)
    save_capture_to_db(reason);
}

/// Get aggregated statistics from the global tracker.
pub fn get_recent_stats(duration: Duration) -> CaptureStats {
    ensure_stats_loaded(); // Ensure data is loaded
    if let Ok(tracker) = SILENT_PATTERN_TRACKER.lock() {
        tracker.get_recent_stats(duration)
    } else {
        CaptureStats::default()
    }
}

/// Get the current threshold from the global tracker.
pub fn current_threshold() -> u64 {
    ensure_stats_loaded(); // Ensure data is loaded
    if let Ok(tracker) = SILENT_PATTERN_TRACKER.lock() {
        tracker.current_threshold()
    } else {
        30 // Default fallback
    }
}

/// Set the threshold in the global tracker.
pub fn set_threshold(threshold: u64) {
    ensure_stats_loaded(); // Ensure data is loaded
    if let Ok(mut tracker) = SILENT_PATTERN_TRACKER.lock() {
        tracker.set_threshold(threshold);
    }
}

/// Get consecutive capture counts from the global tracker.
pub fn consecutive_captures() -> (u32, u32) {
    ensure_stats_loaded(); // Ensure data is loaded
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
    ensure_stats_loaded(); // Ensure data is loaded
    if let Ok(tracker) = SILENT_PATTERN_TRACKER.lock() {
        tracker.has_sufficient_data()
    } else {
        false
    }
}

// =============================================================================
// Persistence Functions (DEBT-005)
// =============================================================================

/// Save hourly stats to database
fn save_hourly_stats_to_db(
    date: NaiveDate,
    hour: u8,
    silent_captures: u32,
    change_captures: u32,
) -> AppResult<()> {
    use crate::memory_storage::DB_CONNECTION;
    use rusqlite::params;

    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let date_str = date.format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT OR REPLACE INTO silent_pattern_stats (date, hour, silent_captures, change_captures)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            date_str,
            hour as i32,
            silent_captures as i32,
            change_captures as i32
        ],
    )?;

    Ok(())
}

/// Load all hourly stats from database into the tracker (internal, caller holds lock)
fn load_hourly_stats_from_db_internal(tracker: &mut SilentPatternTracker) -> AppResult<()> {
    use crate::memory_storage::DB_CONNECTION;

    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let cutoff_date = Local::now().date_naive() - Duration::days(MAX_HISTORY_DAYS);
    let cutoff_str = cutoff_date.format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(
        "SELECT date, hour, silent_captures, change_captures
             FROM silent_pattern_stats
             WHERE date >= ?1
             ORDER BY date, hour",
    )?;

    let cutoff_date_val = cutoff_date;
    let rows = stmt.query_map([&cutoff_str], |row| {
        let date_str: String = row.get(0)?;
        let hour: i32 = row.get(1)?;
        let silent_captures: i32 = row.get(2)?;
        let change_captures: i32 = row.get(3)?;
        Ok((
            NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap_or(cutoff_date_val),
            hour as u8,
            silent_captures as u32,
            change_captures as u32,
        ))
    })?;

    tracker.hourly_stats.clear();
    for row_result in rows {
        let (date, hour, silent_captures, change_captures) = row_result?;
        let mut stats = HourlyStats::new(date, hour);
        stats.silent_captures = silent_captures;
        stats.change_captures = change_captures;
        tracker.hourly_stats.push(stats);
    }

    Ok(())
}

/// Load all hourly stats from database into the tracker (public, acquires lock)
fn load_hourly_stats_from_db(tracker: &mut SilentPatternTracker) -> AppResult<()> {
    load_hourly_stats_from_db_internal(tracker)
}

/// Flag to track if stats have been loaded from DB
static STATS_LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Load stats from DB if not already loaded (called lazily on first access)
fn ensure_stats_loaded() {
    if STATS_LOADED.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    // Double-check after acquiring lock
    if let Ok(mut tracker) = SILENT_PATTERN_TRACKER.lock() {
        if !STATS_LOADED.load(std::sync::atomic::Ordering::SeqCst) {
            if let Err(e) = load_hourly_stats_from_db_internal(&mut tracker) {
                tracing::warn!("Failed to load silent pattern stats: {}", e);
            } else {
                STATS_LOADED.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }
}

/// Persist the current tracker state to database
pub fn persist_silent_pattern_stats() -> AppResult<()> {
    let tracker = SILENT_PATTERN_TRACKER.lock()?;

    for stats in &tracker.hourly_stats {
        save_hourly_stats_to_db(
            stats.date,
            stats.hour,
            stats.silent_captures,
            stats.change_captures,
        )?;
    }

    tracing::debug!(
        "Persisted {} hourly silent pattern stats",
        tracker.hourly_stats.len()
    );
    Ok(())
}

/// Load persisted stats into the global tracker
pub fn load_silent_pattern_stats() -> AppResult<()> {
    let mut tracker = SILENT_PATTERN_TRACKER.lock()?;

    load_hourly_stats_from_db(&mut tracker)?;

    tracing::debug!(
        "Loaded {} hourly silent pattern stats from database",
        tracker.hourly_stats.len()
    );
    Ok(())
}

/// Save a single hourly stat after recording a capture
pub fn save_capture_to_db(reason: CaptureReason) {
    let now = Local::now();
    let date = now.date_naive();
    let hour = now.hour() as u8;

    let (silent, change) = match reason {
        CaptureReason::SilentTimeout => (1, 0),
        CaptureReason::ScreenChanged | CaptureReason::ManualTrigger => (0, 1),
    };

    // Get current counts from database and update
    if let Ok(db) = crate::memory_storage::DB_CONNECTION.lock() {
        if let Some(conn) = db.as_ref() {
            let date_str = date.format("%Y-%m-%d").to_string();

            // Try to get existing counts
            let existing: Option<(i32, i32)> = conn
                .query_row(
                    "SELECT silent_captures, change_captures FROM silent_pattern_stats WHERE date = ?1 AND hour = ?2",
                    rusqlite::params![&date_str, hour as i32],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .ok();

            let (existing_silent, existing_change) = existing.unwrap_or((0, 0));

            // Save updated counts
            let _ = conn.execute(
                "INSERT OR REPLACE INTO silent_pattern_stats (date, hour, silent_captures, change_captures)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    &date_str,
                    hour as i32,
                    existing_silent + silent,
                    existing_change + change
                ],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

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
        assert_eq!(
            tracker.last_capture_reason(),
            Some(CaptureReason::ScreenChanged)
        );
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
    #[serial]
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
    #[serial]
    fn global_threshold_operations() {
        set_threshold(45);
        assert_eq!(current_threshold(), 45);

        // Reset to default
        set_threshold(30);
    }

    #[test]
    #[serial]
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
        assert_eq!(
            tracker.last_capture_reason(),
            Some(CaptureReason::ManualTrigger)
        );
    }

    // ── AC2 Algorithm Tests ──

    /// AC2: Given 用户处于深度工作状态（屏幕长期无变化）
    /// When 系统检测到连续多次因静默超时触发捕获
    /// Then 自动提高 max_silent_minutes（如从 30 分钟提高到 45 分钟）
    #[test]
    fn ac2_high_silent_ratio_increases_threshold() {
        let mut tracker = create_test_tracker();

        // Simulate deep work: 80% silent timeout, 20% screen changes
        for _ in 0..8 {
            tracker.record_capture(CaptureReason::SilentTimeout);
        }
        for _ in 0..2 {
            tracker.record_capture(CaptureReason::ScreenChanged);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);

        // Silent ratio is 0.8 > 0.7, should increase threshold
        assert!(
            new_threshold > 30,
            "Threshold should increase for deep work"
        );
        // Progressive: max 5 minutes per adjustment
        assert!(
            new_threshold <= 35,
            "Adjustment should be at most 5 minutes"
        );
    }

    /// AC2: Given 用户处于活跃工作状态（屏幕频繁变化）
    /// When 系统检测到多次捕获间隔都很短
    /// Then 自动降低 max_silent_minutes（如从 30 分钟降低到 15 分钟）
    #[test]
    fn ac2_low_silent_ratio_decreases_threshold() {
        let mut tracker = create_test_tracker();

        // Simulate active work: 20% silent timeout, 80% screen changes
        for _ in 0..2 {
            tracker.record_capture(CaptureReason::SilentTimeout);
        }
        for _ in 0..8 {
            tracker.record_capture(CaptureReason::ScreenChanged);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);

        // Silent ratio is 0.2 < 0.3, should decrease threshold
        assert!(
            new_threshold < 30,
            "Threshold should decrease for active work"
        );
        // Progressive: max 5 minutes per adjustment
        assert!(
            new_threshold >= 25,
            "Adjustment should be at most 5 minutes"
        );
    }

    /// AC2: Balance state - ratio between 0.3 and 0.7
    #[test]
    fn ac2_balanced_ratio_keeps_threshold() {
        let mut tracker = create_test_tracker();

        // Simulate balanced work: 50% silent, 50% changes
        for _ in 0..5 {
            tracker.record_capture(CaptureReason::SilentTimeout);
        }
        for _ in 0..5 {
            tracker.record_capture(CaptureReason::ScreenChanged);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);

        // Silent ratio is 0.5, within [0.3, 0.7], should keep current
        assert_eq!(
            new_threshold, 30,
            "Threshold should stay the same for balanced work"
        );
    }

    /// Test insufficient data returns default
    #[test]
    fn insufficient_data_returns_default() {
        let tracker = create_test_tracker();

        // Only 5 captures (less than 10)
        let new_threshold = calculate_optimal_silent_minutes(&tracker);

        assert_eq!(
            new_threshold, 30,
            "Should return default without sufficient data"
        );
    }

    /// Test minimum threshold limit (10 minutes)
    #[test]
    fn threshold_respects_minimum_limit() {
        let mut tracker = SilentPatternTracker::new(12); // Start at 12

        // Simulate very active work
        for _ in 0..10 {
            tracker.record_capture(CaptureReason::ScreenChanged);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);

        // Should not go below 10
        assert!(new_threshold >= 10, "Threshold should not go below minimum");
    }

    /// Test maximum threshold limit (60 minutes)
    #[test]
    fn threshold_respects_maximum_limit() {
        let mut tracker = SilentPatternTracker::new(58); // Start at 58

        // Simulate deep work
        for _ in 0..10 {
            tracker.record_capture(CaptureReason::SilentTimeout);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);

        // Should not go above 60
        assert!(new_threshold <= 60, "Threshold should not exceed maximum");
    }

    /// Test progressive adjustment is exactly 5 minutes
    #[test]
    fn progressive_adjustment_is_at_most_5_minutes() {
        let mut tracker = SilentPatternTracker::new(30);

        // All silent captures (100%)
        for _ in 0..15 {
            tracker.record_capture(CaptureReason::SilentTimeout);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);

        // Should increase by exactly 5 minutes (progressive)
        assert_eq!(new_threshold, 35, "Should increase by exactly 5 minutes");
    }

    /// Test adjustment at boundary (silent ratio = 0.7)
    #[test]
    fn boundary_silent_ratio_0_7_increases_threshold() {
        let mut tracker = create_test_tracker();

        // Exactly 70% silent ratio (7 silent, 3 change)
        for _ in 0..7 {
            tracker.record_capture(CaptureReason::SilentTimeout);
        }
        for _ in 0..3 {
            tracker.record_capture(CaptureReason::ScreenChanged);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);

        // At 0.7 boundary, should increase (>= 0.7)
        assert_eq!(
            new_threshold, 35,
            "At boundary 0.7, should increase threshold"
        );
    }

    /// Test adjustment at boundary (silent ratio = 0.3)
    #[test]
    fn boundary_silent_ratio_0_3_decreases_threshold() {
        let mut tracker = create_test_tracker();

        // Exactly 30% silent ratio (3 silent, 7 change)
        for _ in 0..3 {
            tracker.record_capture(CaptureReason::SilentTimeout);
        }
        for _ in 0..7 {
            tracker.record_capture(CaptureReason::ScreenChanged);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);

        // At 0.3 boundary, should decrease (<= 0.3)
        assert_eq!(
            new_threshold, 25,
            "At boundary 0.3, should decrease threshold"
        );
    }

    /// Test all silent captures
    #[test]
    fn all_silent_captures_maximizes_increase() {
        let mut tracker = SilentPatternTracker::new(30);

        // 100% silent
        for _ in 0..10 {
            tracker.record_capture(CaptureReason::SilentTimeout);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);
        assert_eq!(new_threshold, 35);
    }

    /// Test all screen change captures
    #[test]
    fn all_change_captures_maximizes_decrease() {
        let mut tracker = SilentPatternTracker::new(30);

        // 100% screen changes
        for _ in 0..10 {
            tracker.record_capture(CaptureReason::ScreenChanged);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);
        assert_eq!(new_threshold, 25);
    }

    /// Test adjustment respects current threshold (not starting from default)
    #[test]
    fn adjustment_uses_current_threshold() {
        let mut tracker = SilentPatternTracker::new(45); // Current is 45

        // High silent ratio
        for _ in 0..10 {
            tracker.record_capture(CaptureReason::SilentTimeout);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);

        // Should increase from 45, not from 30
        assert_eq!(new_threshold, 50);
    }

    /// Test minimum threshold already at limit
    #[test]
    fn at_minimum_threshold_stays_at_minimum() {
        let mut tracker = SilentPatternTracker::new(10);

        // Active work would decrease, but already at minimum
        for _ in 0..10 {
            tracker.record_capture(CaptureReason::ScreenChanged);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);
        assert_eq!(new_threshold, 10, "Should stay at minimum");
    }

    /// Test maximum threshold already at limit
    #[test]
    fn at_maximum_threshold_stays_at_maximum() {
        let mut tracker = SilentPatternTracker::new(60);

        // Deep work would increase, but already at maximum
        for _ in 0..10 {
            tracker.record_capture(CaptureReason::SilentTimeout);
        }

        let new_threshold = calculate_optimal_silent_minutes(&tracker);
        assert_eq!(new_threshold, 60, "Should stay at maximum");
    }

    // ── Persistence Tests (DEBT-005) ──

    #[test]
    #[serial]
    fn test_save_and_load_hourly_stats() {
        use crate::memory_storage::init_test_database;
        use rusqlite::Connection;

        // Setup test database
        let conn = Connection::open_in_memory().unwrap();
        init_test_database(&conn).unwrap();
        {
            let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            *db = Some(conn);
        }

        // Clear tracker
        {
            let mut tracker = SILENT_PATTERN_TRACKER.lock().unwrap();
            tracker.clear();
        }

        // Record some captures
        record_capture(CaptureReason::SilentTimeout);
        record_capture(CaptureReason::SilentTimeout);
        record_capture(CaptureReason::ScreenChanged);

        // Persist
        persist_silent_pattern_stats().unwrap();

        // Clear tracker
        {
            let mut tracker = SILENT_PATTERN_TRACKER.lock().unwrap();
            tracker.clear();
            assert!(tracker.hourly_stats.is_empty());
        }

        // Load from database
        load_silent_pattern_stats().unwrap();

        // Verify data restored
        let tracker = SILENT_PATTERN_TRACKER.lock().unwrap();
        assert!(!tracker.hourly_stats.is_empty());
        let stats = &tracker.hourly_stats[0];
        assert_eq!(stats.silent_captures, 2);
        assert_eq!(stats.change_captures, 1);
    }

    #[test]
    #[serial]
    fn test_save_capture_to_db_increments() {
        use crate::memory_storage::init_test_database;
        use rusqlite::Connection;

        // Setup test database
        let conn = Connection::open_in_memory().unwrap();
        init_test_database(&conn).unwrap();
        {
            let mut db = crate::memory_storage::DB_CONNECTION.lock().unwrap();
            *db = Some(conn);
        }

        // Clear tracker
        {
            let mut tracker = SILENT_PATTERN_TRACKER.lock().unwrap();
            tracker.clear();
        }

        // Save captures directly to DB
        save_capture_to_db(CaptureReason::SilentTimeout);
        save_capture_to_db(CaptureReason::SilentTimeout);
        save_capture_to_db(CaptureReason::ScreenChanged);

        // Load and verify
        load_silent_pattern_stats().unwrap();
        let tracker = SILENT_PATTERN_TRACKER.lock().unwrap();
        assert!(!tracker.hourly_stats.is_empty());
        let stats = &tracker.hourly_stats[0];
        assert_eq!(stats.silent_captures, 2);
        assert_eq!(stats.change_captures, 1);
    }
}
