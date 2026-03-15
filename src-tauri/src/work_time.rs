//! SMART-003: Work time pattern learning
//!
//! Automatically detects user's work time patterns from capture history
//! and pauses/resumes auto-capture accordingly.

use chrono::{DateTime, Local, NaiveDate, Timelike};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

/// Activity statistics for a single hour on a single day
#[derive(Debug, Clone)]
pub struct HourlyActivity {
    pub date: NaiveDate,
    pub hour: u8, // 0-23
    pub capture_count: u32,
}

/// Aggregated activity per hour across multiple days
#[derive(Debug, Clone, Default)]
pub struct HourlyActivitySummary {
    pub hour: u8,
    pub active_days: u32, // Days with at least one capture in this hour
    pub total_days: u32,  // Total days in the sliding window
}

/// A detected work time period
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimePeriod {
    pub start: u8, // Start hour 0-23
    pub end: u8,   // End hour 0-24 (exclusive)
}

/// Work time pattern learner
///
/// Tracks capture activity across hours and learns typical work time patterns.
/// Uses a sliding window of 14 days for gradual adaptation.
#[derive(Debug, Clone, Default)]
pub struct WorkTimePatternLearner {
    /// Hourly activity records (date, hour) -> capture_count
    /// Limited to last 14 days via pruning
    hourly_activities: Vec<HourlyActivity>,
}

/// Global instance of the work time pattern learner
static WORK_TIME_LEARNER: Lazy<Mutex<WorkTimePatternLearner>> =
    Lazy::new(|| Mutex::new(WorkTimePatternLearner::default()));

impl WorkTimePatternLearner {
    /// Create a new learner instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a capture event at the current time
    pub fn record_capture(&mut self) {
        let now = Local::now();
        self.record_capture_at(now);
    }

    /// Record a capture event at a specific time (for testing)
    pub fn record_capture_at(&mut self, datetime: DateTime<Local>) {
        let date = datetime.date_naive();
        let hour = datetime.hour() as u8;

        // Find or create entry for this date/hour
        if let Some(entry) = self
            .hourly_activities
            .iter_mut()
            .find(|e| e.date == date && e.hour == hour)
        {
            entry.capture_count += 1;
        } else {
            self.hourly_activities.push(HourlyActivity {
                date,
                hour,
                capture_count: 1,
            });
        }

        // Prune old entries (keep only last 14 days)
        self.prune_old_entries();
    }

    /// Prune entries older than 14 days
    fn prune_old_entries(&mut self) {
        let cutoff = Local::now().date_naive() - chrono::Duration::days(14);
        self.hourly_activities.retain(|e| e.date >= cutoff);
    }

    /// Get activity summary for each hour
    pub fn get_hourly_summaries(&self) -> Vec<HourlyActivitySummary> {
        let mut summaries: HashMap<u8, HourlyActivitySummary> = HashMap::new();

        // Count unique days per hour
        let mut days_by_hour: HashMap<u8, std::collections::HashSet<NaiveDate>> = HashMap::new();

        for activity in &self.hourly_activities {
            days_by_hour
                .entry(activity.hour)
                .or_default()
                .insert(activity.date);
        }

        // Calculate total days in the window
        let total_days = self.get_total_days_in_window();

        // Build summaries
        for hour in 0..24u8 {
            let days = days_by_hour.get(&hour).map(|s| s.len() as u32).unwrap_or(0);
            summaries.insert(
                hour,
                HourlyActivitySummary {
                    hour,
                    active_days: days,
                    total_days,
                },
            );
        }

        let mut result: Vec<_> = summaries.into_values().collect();
        result.sort_by_key(|s| s.hour);
        result
    }

    /// Get total days covered by the sliding window
    fn get_total_days_in_window(&self) -> u32 {
        if self.hourly_activities.is_empty() {
            return 0;
        }

        let dates: std::collections::HashSet<_> =
            self.hourly_activities.iter().map(|e| e.date).collect();

        dates.len() as u32
    }

    /// Check if an hour is considered a work hour based on activity threshold
    ///
    /// # Arguments
    /// * `hour` - Hour to check (0-23)
    /// * `threshold` - Minimum ratio of active days to consider as work hour (default 0.6)
    ///
    /// # Returns
    /// true if this hour is considered part of typical work time
    pub fn is_work_hour(&self, hour: u8, threshold: f64) -> bool {
        let summaries = self.get_hourly_summaries();
        if let Some(summary) = summaries.iter().find(|s| s.hour == hour) {
            if summary.total_days == 0 {
                return false;
            }
            let ratio = summary.active_days as f64 / summary.total_days as f64;
            ratio >= threshold
        } else {
            false
        }
    }

    /// Detect work time periods from learned patterns
    ///
    /// Returns a list of continuous time periods where activity is above threshold.
    /// Handles overnight work schedules (e.g., 22:00-06:00).
    pub fn get_work_periods(&self, threshold: f64) -> Vec<TimePeriod> {
        let mut periods = Vec::new();
        let mut current_start: Option<u8> = None;

        // Iterate through hours 0-23
        for hour in 0..24u8 {
            if self.is_work_hour(hour, threshold) {
                if current_start.is_none() {
                    current_start = Some(hour);
                }
            } else if let Some(start) = current_start.take() {
                periods.push(TimePeriod { start, end: hour });
            }
        }

        // Handle work hours that extend to midnight
        if let Some(start) = current_start {
            periods.push(TimePeriod { start, end: 24 });
        }

        periods
    }

    /// Check if there's enough data for reliable work time detection
    ///
    /// Requires at least 7 days of data for confident detection
    pub fn has_sufficient_data(&self) -> bool {
        self.get_total_days_in_window() >= 7
    }

    /// Get learning progress (0.0 to 1.0)
    pub fn get_learning_progress(&self) -> f64 {
        let days = self.get_total_days_in_window();
        if days >= 14 {
            1.0
        } else if days >= 7 {
            (days as f64 - 7.0) / 7.0 + 0.5
        } else {
            days as f64 / 14.0
        }
    }

    /// Clear all learned data (for testing)
    pub fn clear(&mut self) {
        self.hourly_activities.clear();
    }
}

/// Global functions for use by other modules
/// Record a capture in the global work time learner
pub fn record_work_time_capture() {
    if let Ok(mut learner) = WORK_TIME_LEARNER.lock() {
        learner.record_capture();
    }
}

/// Record a capture at a specific time (for testing)
pub fn record_work_time_capture_at(datetime: DateTime<Local>) {
    if let Ok(mut learner) = WORK_TIME_LEARNER.lock() {
        learner.record_capture_at(datetime);
    }
}

/// Get detected work periods from the global learner
pub fn get_detected_work_periods() -> Vec<TimePeriod> {
    if let Ok(learner) = WORK_TIME_LEARNER.lock() {
        if learner.has_sufficient_data() {
            learner.get_work_periods(0.6)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    }
}

/// Check if learning has enough data
pub fn has_work_time_data() -> bool {
    if let Ok(learner) = WORK_TIME_LEARNER.lock() {
        learner.has_sufficient_data()
    } else {
        false
    }
}

/// Get learning progress (0.0 to 1.0)
pub fn get_work_time_learning_progress() -> f64 {
    if let Ok(learner) = WORK_TIME_LEARNER.lock() {
        learner.get_learning_progress()
    } else {
        0.0
    }
}

/// Get hourly summaries for debugging/display
pub fn get_work_time_hourly_summaries() -> Vec<HourlyActivitySummary> {
    if let Ok(learner) = WORK_TIME_LEARNER.lock() {
        learner.get_hourly_summaries()
    } else {
        Vec::new()
    }
}

/// Clear all learned data (for testing)
pub fn clear_work_time_learner() {
    if let Ok(mut learner) = WORK_TIME_LEARNER.lock() {
        learner.clear();
    }
}

// =============================================================================
// Work Time Judgment Functions
// =============================================================================

/// Parse a time string "HH:MM" to minutes since midnight
fn parse_time_to_minutes(time_str: Option<&str>, default: u32) -> u32 {
    time_str
        .and_then(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() == 2 {
                let hours: u32 = parts[0].parse().ok()?;
                let minutes: u32 = parts[1].parse().ok()?;
                Some(hours * 60 + minutes)
            } else {
                None
            }
        })
        .unwrap_or(default)
}

/// Check if current time is within custom work time period
///
/// Handles both normal periods (09:00-18:00) and overnight periods (22:00-06:00)
fn is_in_custom_time_period(current_minutes: u32, start_minutes: u32, end_minutes: u32) -> bool {
    if start_minutes <= end_minutes {
        // Normal period (e.g., 09:00-18:00)
        current_minutes >= start_minutes && current_minutes < end_minutes
    } else {
        // Overnight period (e.g., 22:00-06:00)
        current_minutes >= start_minutes || current_minutes < end_minutes
    }
}

/// Check if current time is within learned work periods
fn is_in_learned_periods(current_minutes: u32, periods: &[TimePeriod]) -> bool {
    for period in periods {
        let start_minutes = (period.start as u32) * 60;
        let end_minutes = (period.end as u32) * 60;

        if is_in_custom_time_period(current_minutes, start_minutes, end_minutes) {
            return true;
        }
    }
    false
}

/// Settings structure for work time configuration
#[derive(Debug, Clone)]
pub struct WorkTimeSettings {
    /// Whether auto-detect work time is enabled
    pub auto_detect_work_time: bool,
    /// Whether to use custom work time instead of learned
    pub use_custom_work_time: bool,
    /// Custom work time start (HH:MM)
    pub custom_work_time_start: Option<String>,
    /// Custom work time end (HH:MM)
    pub custom_work_time_end: Option<String>,
    /// Learned work time as JSON string
    pub learned_work_time: Option<String>,
}

/// Check if the current time is within work hours based on settings
///
/// # Returns
/// - true if work time detection is disabled
/// - true if using custom time and current time is within custom period
/// - true if using learned time and current time is within learned periods
/// - false otherwise
pub fn is_in_work_time(settings: &WorkTimeSettings) -> bool {
    // If auto-detect is disabled, always allow capture
    if !settings.auto_detect_work_time {
        return true;
    }

    let now = Local::now();
    let current_minutes = now.hour() * 60 + now.minute();

    // If using custom work time
    if settings.use_custom_work_time {
        let start = parse_time_to_minutes(settings.custom_work_time_start.as_deref(), 9 * 60);
        let end = parse_time_to_minutes(settings.custom_work_time_end.as_deref(), 18 * 60);
        return is_in_custom_time_period(current_minutes, start, end);
    }

    // Use learned work time
    if let Some(ref learned_json) = settings.learned_work_time {
        // Parse learned work time JSON
        #[derive(serde::Deserialize)]
        struct LearnedWorkTime {
            periods: Vec<TimePeriod>,
        }

        if let Ok(learned) = serde_json::from_str::<LearnedWorkTime>(learned_json) {
            return is_in_learned_periods(current_minutes, &learned.periods);
        }
    }

    // Fallback: check if learner has detected periods
    let detected = get_detected_work_periods();
    if !detected.is_empty() {
        return is_in_learned_periods(current_minutes, &detected);
    }

    // Default: allow capture if no data yet
    true
}

/// Get work time status for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkTimeStatus {
    /// Whether currently in work hours
    pub is_work_time: bool,
    /// Whether using custom or learned time
    pub using_custom_time: bool,
    /// Current work periods being used
    pub current_periods: Vec<TimePeriod>,
    /// Learning progress (0.0 to 1.0)
    pub learning_progress: f64,
    /// Whether learner has sufficient data
    pub has_sufficient_data: bool,
}

/// Get comprehensive work time status
pub fn get_work_time_status(settings: &WorkTimeSettings) -> WorkTimeStatus {
    let detected_periods = get_detected_work_periods();
    let has_sufficient_data = has_work_time_data();
    let learning_progress = get_work_time_learning_progress();

    let current_periods = if settings.use_custom_work_time {
        // Parse custom time to periods
        let start_hour =
            parse_time_to_minutes(settings.custom_work_time_start.as_deref(), 9 * 60) / 60;
        let end_hour =
            parse_time_to_minutes(settings.custom_work_time_end.as_deref(), 18 * 60) / 60;
        vec![TimePeriod {
            start: start_hour as u8,
            end: end_hour as u8,
        }]
    } else if let Some(ref learned_json) = settings.learned_work_time {
        #[derive(serde::Deserialize)]
        struct LearnedWorkTime {
            periods: Vec<TimePeriod>,
        }
        serde_json::from_str::<LearnedWorkTime>(learned_json)
            .map(|l| l.periods)
            .unwrap_or_else(|_| detected_periods.clone())
    } else {
        detected_periods
    };

    WorkTimeStatus {
        is_work_time: is_in_work_time(settings),
        using_custom_time: settings.use_custom_work_time,
        current_periods,
        learning_progress,
        has_sufficient_data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_record_capture() {
        let mut learner = WorkTimePatternLearner::new();
        learner.clear();

        let now = Local::now();
        learner.record_capture_at(now);

        assert_eq!(learner.hourly_activities.len(), 1);
        assert_eq!(learner.hourly_activities[0].capture_count, 1);
    }

    #[test]
    fn test_multiple_captures_same_hour() {
        let mut learner = WorkTimePatternLearner::new();
        learner.clear();

        let now = Local::now();
        learner.record_capture_at(now);
        learner.record_capture_at(now);
        learner.record_capture_at(now);

        assert_eq!(learner.hourly_activities.len(), 1);
        assert_eq!(learner.hourly_activities[0].capture_count, 3);
    }

    #[test]
    fn test_prune_old_entries() {
        let mut learner = WorkTimePatternLearner::new();
        learner.clear();

        // Add entry for 15 days ago (should be pruned)
        let old_date = Local::now() - Duration::days(15);
        learner.hourly_activities.push(HourlyActivity {
            date: old_date.date_naive(),
            hour: 10,
            capture_count: 1,
        });

        // Add entry for today (should be kept)
        learner.record_capture();

        learner.prune_old_entries();

        assert_eq!(learner.hourly_activities.len(), 1);
        assert!(
            learner.hourly_activities[0].date >= Local::now().date_naive() - Duration::days(14)
        );
    }

    #[test]
    fn test_is_work_hour() {
        let mut learner = WorkTimePatternLearner::new();
        learner.clear();

        // Create 10 days of data with captures at 9-12 and 14-18
        let base = Local::now() - Duration::days(10);
        for day_offset in 0..10i64 {
            let day = base + Duration::days(day_offset);

            // Morning work hours (9-12)
            for hour in 9..12 {
                let dt = day.with_hour(hour).unwrap();
                learner.record_capture_at(dt);
            }

            // Afternoon work hours (14-18)
            for hour in 14..18 {
                let dt = day.with_hour(hour).unwrap();
                learner.record_capture_at(dt);
            }
        }

        // Check work hours detection with 60% threshold
        assert!(learner.is_work_hour(9, 0.6));
        assert!(learner.is_work_hour(10, 0.6));
        assert!(learner.is_work_hour(11, 0.6));
        assert!(learner.is_work_hour(14, 0.6));
        assert!(learner.is_work_hour(15, 0.6));
        assert!(learner.is_work_hour(16, 0.6));
        assert!(learner.is_work_hour(17, 0.6));

        // Non-work hours
        assert!(!learner.is_work_hour(0, 0.6));
        assert!(!learner.is_work_hour(8, 0.6));
        assert!(!learner.is_work_hour(13, 0.6)); // Lunch break
        assert!(!learner.is_work_hour(22, 0.6));
    }

    #[test]
    fn test_get_work_periods() {
        let mut learner = WorkTimePatternLearner::new();
        learner.clear();

        // Create 10 days of data with captures at 9-12 and 14-18
        let base = Local::now() - Duration::days(10);
        for day_offset in 0..10i64 {
            let day = base + Duration::days(day_offset);

            // Morning work hours (9-12)
            for hour in 9..12 {
                let dt = day.with_hour(hour).unwrap();
                learner.record_capture_at(dt);
            }

            // Afternoon work hours (14-18)
            for hour in 14..18 {
                let dt = day.with_hour(hour).unwrap();
                learner.record_capture_at(dt);
            }
        }

        let periods = learner.get_work_periods(0.6);

        assert_eq!(periods.len(), 2);
        assert_eq!(periods[0], TimePeriod { start: 9, end: 12 });
        assert_eq!(periods[1], TimePeriod { start: 14, end: 18 });
    }

    #[test]
    fn test_has_sufficient_data() {
        let mut learner = WorkTimePatternLearner::new();
        learner.clear();

        assert!(!learner.has_sufficient_data());

        // Add 6 days of data
        let base = Local::now() - Duration::days(6);
        for day_offset in 0..6i64 {
            let day = base + Duration::days(day_offset);
            let dt = day.with_hour(9).unwrap();
            learner.record_capture_at(dt);
        }

        assert!(!learner.has_sufficient_data());

        // Add one more day
        let dt = (Local::now()).with_hour(9).unwrap();
        learner.record_capture_at(dt);

        assert!(learner.has_sufficient_data());
    }

    #[test]
    fn test_learning_progress() {
        let mut learner = WorkTimePatternLearner::new();
        learner.clear();

        // 0 days = 0%
        assert!((learner.get_learning_progress() - 0.0).abs() < 0.01);

        // 7 days = 50%
        let base = Local::now() - Duration::days(6);
        for day_offset in 0..7i64 {
            let day = base + Duration::days(day_offset);
            let dt = day.with_hour(9).unwrap();
            learner.record_capture_at(dt);
        }
        assert!((learner.get_learning_progress() - 0.5).abs() < 0.01);

        // 14 days = 100%
        learner.clear();
        for day_offset in 0..14i64 {
            let day = (Local::now() - Duration::days(13)) + Duration::days(day_offset);
            let dt = day.with_hour(9).unwrap();
            learner.record_capture_at(dt);
        }
        assert!((learner.get_learning_progress() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_overnight_work_schedule() {
        let mut learner = WorkTimePatternLearner::new();
        learner.clear();

        // Simulate night shift: 22:00-06:00
        let base = Local::now() - Duration::days(10);
        for day_offset in 0..10i64 {
            let day = base + Duration::days(day_offset);

            // Night hours: 22-23
            for hour in 22..24 {
                let dt = day.with_hour(hour).unwrap();
                learner.record_capture_at(dt);
            }

            // Early morning hours: 0-6
            let next_day = day + Duration::days(1);
            for hour in 0..6 {
                let dt = next_day.with_hour(hour).unwrap();
                learner.record_capture_at(dt);
            }
        }

        let periods = learner.get_work_periods(0.6);

        // Should detect two periods: 22-24 and 0-6
        assert!(periods.iter().any(|p| p.start == 22 && p.end == 24));
        assert!(periods.iter().any(|p| p.start == 0 && p.end == 6));
    }

    #[test]
    fn test_parse_time_to_minutes() {
        assert_eq!(parse_time_to_minutes(Some("09:00"), 0), 9 * 60);
        assert_eq!(parse_time_to_minutes(Some("18:30"), 0), 18 * 60 + 30);
        assert_eq!(parse_time_to_minutes(Some("00:00"), 0), 0);
        assert_eq!(parse_time_to_minutes(Some("23:59"), 0), 23 * 60 + 59);
        assert_eq!(parse_time_to_minutes(None, 9 * 60), 9 * 60);
        assert_eq!(parse_time_to_minutes(Some("invalid"), 9 * 60), 9 * 60);
    }

    #[test]
    fn test_is_in_custom_time_period_normal() {
        // Normal period: 09:00-18:00
        let start = 9 * 60;
        let end = 18 * 60;

        // Inside work hours
        assert!(is_in_custom_time_period(9 * 60, start, end)); // 09:00
        assert!(is_in_custom_time_period(12 * 60, start, end)); // 12:00
        assert!(is_in_custom_time_period(17 * 60 + 59, start, end)); // 17:59

        // Outside work hours
        assert!(!is_in_custom_time_period(8 * 60 + 59, start, end)); // 08:59
        assert!(!is_in_custom_time_period(18 * 60, start, end)); // 18:00
        assert!(!is_in_custom_time_period(22 * 60, start, end)); // 22:00
    }

    #[test]
    fn test_is_in_custom_time_period_overnight() {
        // Overnight period: 22:00-06:00
        let start = 22 * 60;
        let end = 6 * 60;

        // Inside work hours
        assert!(is_in_custom_time_period(22 * 60, start, end)); // 22:00
        assert!(is_in_custom_time_period(23 * 60, start, end)); // 23:00
        assert!(is_in_custom_time_period(0, start, end)); // 00:00
        assert!(is_in_custom_time_period(5 * 60 + 59, start, end)); // 05:59

        // Outside work hours
        assert!(!is_in_custom_time_period(6 * 60, start, end)); // 06:00
        assert!(!is_in_custom_time_period(12 * 60, start, end)); // 12:00
        assert!(!is_in_custom_time_period(21 * 60 + 59, start, end)); // 21:59
    }

    #[test]
    fn test_is_in_work_time_disabled() {
        let settings = WorkTimeSettings {
            auto_detect_work_time: false,
            use_custom_work_time: false,
            custom_work_time_start: None,
            custom_work_time_end: None,
            learned_work_time: None,
        };

        // Should always return true when disabled
        assert!(is_in_work_time(&settings));
    }

    #[test]
    fn test_is_in_work_time_custom() {
        let settings = WorkTimeSettings {
            auto_detect_work_time: true,
            use_custom_work_time: true,
            custom_work_time_start: Some("09:00".to_string()),
            custom_work_time_end: Some("18:00".to_string()),
            learned_work_time: None,
        };

        // Note: This test depends on current time, so we just verify the function works
        let result = is_in_work_time(&settings);
        let now = Local::now();
        let current_minutes = now.hour() as u32 * 60 + now.minute() as u32;
        let in_range = current_minutes >= 9 * 60 && current_minutes < 18 * 60;

        assert_eq!(result, in_range);
    }

    #[test]
    fn test_is_in_work_time_learned_json() {
        let learned = r#"{"periods":[{"start":9,"end":12},{"start":14,"end":18}]}"#;
        let settings = WorkTimeSettings {
            auto_detect_work_time: true,
            use_custom_work_time: false,
            custom_work_time_start: None,
            custom_work_time_end: None,
            learned_work_time: Some(learned.to_string()),
        };

        let now = Local::now();
        let current_minutes = now.hour() as u32 * 60 + now.minute() as u32;

        let in_morning = current_minutes >= 9 * 60 && current_minutes < 12 * 60;
        let in_afternoon = current_minutes >= 14 * 60 && current_minutes < 18 * 60;
        let expected = in_morning || in_afternoon;

        assert_eq!(is_in_work_time(&settings), expected);
    }

    #[test]
    fn test_is_in_learned_periods() {
        let periods = vec![
            TimePeriod { start: 9, end: 12 },
            TimePeriod { start: 14, end: 18 },
        ];

        // Inside first period
        assert!(is_in_learned_periods(9 * 60, &periods)); // 09:00
        assert!(is_in_learned_periods(10 * 60, &periods)); // 10:00
        assert!(is_in_learned_periods(11 * 60 + 59, &periods)); // 11:59

        // Between periods (lunch)
        assert!(!is_in_learned_periods(12 * 60, &periods)); // 12:00
        assert!(!is_in_learned_periods(13 * 60, &periods)); // 13:00

        // Inside second period
        assert!(is_in_learned_periods(14 * 60, &periods)); // 14:00
        assert!(is_in_learned_periods(17 * 60, &periods)); // 17:00

        // Outside
        assert!(!is_in_learned_periods(18 * 60, &periods)); // 18:00
        assert!(!is_in_learned_periods(22 * 60, &periods)); // 22:00
    }

    #[test]
    fn test_get_work_time_status() {
        let settings = WorkTimeSettings {
            auto_detect_work_time: true,
            use_custom_work_time: true,
            custom_work_time_start: Some("09:00".to_string()),
            custom_work_time_end: Some("18:00".to_string()),
            learned_work_time: None,
        };

        let status = get_work_time_status(&settings);

        assert!(status.using_custom_time);
        assert_eq!(status.current_periods.len(), 1);
        assert_eq!(status.current_periods[0].start, 9);
        assert_eq!(status.current_periods[0].end, 18);
    }
}
