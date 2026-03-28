//! Capture commands - Tauri command entry points for capture operations
//!
//! These commands are thin wrappers that delegate to service functions.
//! No business logic is implemented here - only parameter transformation
//! and error mapping.
//!
//! SMART-001: Active window detection and filtering
//! SMART-002: Silent threshold auto-adjustment
//! SMART-003: Work time aware capture
//! SMART-004: Multi-monitor capture support
//! EXP-002: Screenshot quality filter

use crate::services::capture_service::{
    get_auto_capture_status_service, get_default_analysis_prompt_service,
    get_quality_filter_stats_service, get_work_time_status_service, reanalyze_record_service,
    reanalyze_records_by_date_service, reanalyze_today_records_service,
    reset_quality_filter_counter_service,
    start_auto_capture_service, stop_auto_capture_service, take_screenshot_service,
    trigger_capture_service, CaptureSettings, QualityFilterStats, ReanalyzeResult, ScreenAnalysis,
};
use crate::work_time::WorkTimeStatus;
use std::time::Duration;
use tauri::Emitter;

/// Get auto capture status.
///
/// This is a thin command wrapper that delegates to the capture service.
#[tauri::command]
pub fn get_auto_capture_status() -> bool {
    get_auto_capture_status_service()
}

/// Get work time status.
///
/// This is a thin command wrapper that delegates to the capture service.
#[tauri::command]
pub fn get_work_time_status() -> WorkTimeStatus {
    get_work_time_status_service()
}

/// Get the default analysis prompt template.
///
/// This is a thin command wrapper that delegates to the capture service.
#[tauri::command]
pub fn get_default_analysis_prompt() -> String {
    get_default_analysis_prompt_service()
}

/// Start auto capture with the configured interval.
///
/// This command initializes auto capture by validating settings and starting
/// a background capture loop. The service handles all business logic.
#[tauri::command]
pub async fn start_auto_capture(app: tauri::AppHandle) -> Result<(), String> {
    // Delegate to service for initialization
    start_auto_capture_service()?;

    let settings = load_capture_settings_internal();
    let interval_minutes = settings.screenshot_interval;

    // Spawn the capture loop (Tauri-specific, remains in command layer)
    tokio::spawn(async move {
        // Execute immediately on start
        if should_capture_by_work_time_internal() {
            if let Err(e) = trigger_capture_service().await {
                tracing::error!("Initial capture failed: {}", e);
            }
            record_work_time_capture_internal();
        } else {
            tracing::debug!("Outside work time, skipping initial capture");
        }

        loop {
            tokio::time::sleep(Duration::from_secs(interval_minutes * 60)).await;

            if !crate::services::capture_service::is_auto_capture_running() {
                tracing::info!("Auto capture stopped");
                break;
            }

            if !should_capture_by_work_time_internal() {
                tracing::debug!("Outside work time, skipping capture");
                continue;
            }

            if let Err(e) = trigger_capture_service().await {
                tracing::error!("Auto capture failed: {}", e);
            } else {
                record_work_time_capture_internal();
            }
        }
    });

    // Spawn hourly threshold evaluation task
    let app_handle = app.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * 60)).await;

            if !crate::services::capture_service::is_auto_capture_running() {
                break;
            }

            if let Some((old_value, new_value)) = evaluate_and_adjust_threshold_internal() {
                let adjustment_magnitude = new_value.abs_diff(old_value);

                if adjustment_magnitude >= 10 {
                    let reason = if new_value > old_value {
                        "检测到深度工作模式，提高静默阈值".to_string()
                    } else {
                        "检测到活跃工作模式，降低静默阈值".to_string()
                    };

                    let _ = app_handle.emit(
                        "threshold-adjusted",
                        crate::services::capture_service::ThresholdAdjustment {
                            old_value,
                            new_value,
                            reason: reason.clone(),
                        },
                    );

                    tracing::info!(
                        "Threshold adjusted: {} -> {} minutes ({})",
                        old_value,
                        new_value,
                        reason
                    );
                }
            }
        }
    });

    tracing::info!(
        "Auto capture started with interval {} minutes",
        interval_minutes
    );
    Ok(())
}

/// Stop auto capture.
///
/// This is a thin command wrapper that delegates to the capture service.
#[tauri::command]
pub async fn stop_auto_capture() -> Result<(), String> {
    stop_auto_capture_service();
    Ok(())
}

/// Trigger a single manual capture.
///
/// This is a thin command wrapper that delegates to the capture service.
#[tauri::command]
pub async fn trigger_capture() -> Result<(), String> {
    trigger_capture_service().await
}

/// Take a screenshot and save to disk (no AI analysis).
///
/// This is a thin command wrapper that delegates to the capture service.
#[tauri::command]
pub async fn take_screenshot() -> Result<String, String> {
    take_screenshot_service().await
}

/// Reanalyze a single record.
///
/// This is a thin command wrapper that delegates to the capture service.
#[tauri::command]
pub async fn reanalyze_record(record_id: i64) -> Result<ScreenAnalysis, String> {
    reanalyze_record_service(record_id).await
}

/// Reanalyze all records with screenshots from today.
///
/// This is a thin command wrapper that delegates to the capture service.
#[tauri::command]
pub async fn reanalyze_today_records() -> Result<ReanalyzeResult, String> {
    reanalyze_today_records_service().await
}

/// Reanalyze all records with screenshots for a specific date.
///
/// This is a thin command wrapper that delegates to the capture service.
#[tauri::command]
pub async fn reanalyze_records_by_date(date: String) -> Result<ReanalyzeResult, String> {
    reanalyze_records_by_date_service(date).await
}

/// Get quality filter statistics.
///
/// This is a thin command wrapper that delegates to the capture service.
#[tauri::command]
pub async fn get_quality_filter_stats() -> Result<QualityFilterStats, String> {
    get_quality_filter_stats_service().await
}

/// Reset quality filter counter.
///
/// This is a thin command wrapper that delegates to the capture service.
#[tauri::command]
pub async fn reset_quality_filter_counter() -> Result<(), String> {
    reset_quality_filter_counter_service().await
}

// ═══════════════════════════════════════════════════════════════════════════════
// Internal helpers needed by start_auto_capture command
// ═══════════════════════════════════════════════════════════════════════════════

use crate::work_time::WorkTimeSettings;

fn load_capture_settings_internal() -> CaptureSettings {
    crate::services::capture_service::load_capture_settings()
}

fn load_work_time_settings_internal() -> WorkTimeSettings {
    crate::services::capture_service::load_capture_settings(); // Use the service function indirectly
                                                               // Re-implement since it's needed here
    match crate::memory_storage::get_settings_sync() {
        Ok(s) => WorkTimeSettings {
            auto_detect_work_time: s.auto_detect_work_time.unwrap_or(true),
            use_custom_work_time: s.use_custom_work_time.unwrap_or(false),
            custom_work_time_start: s.custom_work_time_start,
            custom_work_time_end: s.custom_work_time_end,
            learned_work_time: s.learned_work_time,
        },
        Err(_) => WorkTimeSettings {
            auto_detect_work_time: true,
            use_custom_work_time: false,
            custom_work_time_start: None,
            custom_work_time_end: None,
            learned_work_time: None,
        },
    }
}

fn should_capture_by_work_time_internal() -> bool {
    let settings = load_work_time_settings_internal();
    crate::work_time::is_in_work_time(&settings)
}

fn record_work_time_capture_internal() {
    crate::work_time::record_work_time_capture();
}

fn evaluate_and_adjust_threshold_internal() -> Option<(u64, u64)> {
    crate::services::capture_service::evaluate_and_adjust_threshold()
}
