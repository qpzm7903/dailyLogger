//! Report commands - Tauri command entry points for report generation
//!
//! These commands are thin wrappers that delegate to service functions.
//! No business logic is implemented here - only parameter transformation
//! and error mapping.
//!
//! REPORT-001: Weekly report generation
//! REPORT-002: Monthly report generation
//! REPORT-003: Custom period report generation
//! REPORT-004: Comparison report between two time periods
//! DATA-007: Multi-language daily report support

use crate::services::report_service::{
    compare_reports_service, generate_custom_report_service, generate_daily_summary_service,
    generate_monthly_report_service, generate_multilingual_daily_summary_service,
    generate_weekly_report_service,
    get_default_summary_prompt as get_default_summary_prompt_service,
    get_supported_languages as get_supported_languages_service,
};

/// Get the list of supported languages for multilingual reports
///
/// This is a thin command wrapper that delegates to the report service.
#[tauri::command]
pub fn get_supported_languages() -> Vec<(String, String)> {
    get_supported_languages_service()
}

/// Returns the default summary prompt template.
/// This is used when the user has not configured a custom prompt.
#[tauri::command]
pub fn get_default_summary_prompt() -> String {
    get_default_summary_prompt_service()
}

/// Generate daily summary - REPORT-001
///
/// This is a thin command wrapper that delegates to the report service.
/// The service handles session analysis, AI summarization, and result storage.
///
/// # Arguments
/// * `vault_name` - Optional vault name to use. If None, uses default vault or auto-detection.
#[tauri::command]
pub async fn generate_daily_summary(vault_name: Option<String>) -> Result<String, String> {
    generate_daily_summary_service(vault_name)
        .await
        .map_err(|e| e.to_string())
}

/// Generate multilingual daily summary - DATA-007
///
/// This is a thin command wrapper that delegates to the report service.
/// The service handles base summary generation and translation.
#[tauri::command]
pub async fn generate_multilingual_daily_summary(target_lang: String) -> Result<String, String> {
    generate_multilingual_daily_summary_service(target_lang)
        .await
        .map_err(|e| e.to_string())
}

/// Generate weekly report - REPORT-001
///
/// This is a thin command wrapper that delegates to the report service.
#[tauri::command]
pub async fn generate_weekly_report() -> Result<String, String> {
    generate_weekly_report_service()
        .await
        .map_err(|e| e.to_string())
}

/// Generate monthly report - REPORT-002
///
/// This is a thin command wrapper that delegates to the report service.
#[tauri::command]
pub async fn generate_monthly_report() -> Result<String, String> {
    generate_monthly_report_service()
        .await
        .map_err(|e| e.to_string())
}

/// Generate custom period report - REPORT-003
///
/// This is a thin command wrapper that delegates to the report service.
#[tauri::command]
pub async fn generate_custom_report(
    start_date: String,
    end_date: String,
    report_name: Option<String>,
) -> Result<String, String> {
    generate_custom_report_service(start_date, end_date, report_name)
        .await
        .map_err(|e| e.to_string())
}

/// Generate comparison report between two time periods - REPORT-004
///
/// This is a thin command wrapper that delegates to the report service.
#[tauri::command]
pub async fn compare_reports(
    start_date_a: String,
    end_date_a: String,
    start_date_b: String,
    end_date_b: String,
) -> Result<String, String> {
    compare_reports_service(start_date_a, end_date_a, start_date_b, end_date_b)
        .await
        .map_err(|e| e.to_string())
}
