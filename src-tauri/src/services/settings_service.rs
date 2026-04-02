//! Settings service - Business logic for settings management
//!
//! This module contains business logic functions for settings operations.
//! Commands should delegate to these service functions rather than implementing logic directly.

use std::sync::Arc;

use crate::errors::AppResult;
use crate::memory_storage::Settings;

/// Get settings from the database.
///
/// This function retrieves the current application settings.
/// Returns `Arc<Settings>` for cheap reference-counted sharing.
pub fn get_settings_service() -> AppResult<Arc<Settings>> {
    crate::memory_storage::get_settings_sync()
}

fn merge_settings(current: &Settings, updates: &Settings) -> Settings {
    Settings {
        api_base_url: updates
            .api_base_url
            .clone()
            .or_else(|| current.api_base_url.clone()),
        api_key: updates.api_key.clone().or_else(|| current.api_key.clone()),
        model_name: updates
            .model_name
            .clone()
            .or_else(|| current.model_name.clone()),
        screenshot_interval: updates.screenshot_interval.or(current.screenshot_interval),
        summary_time: updates
            .summary_time
            .clone()
            .or_else(|| current.summary_time.clone()),
        obsidian_path: updates
            .obsidian_path
            .clone()
            .or_else(|| current.obsidian_path.clone()),
        auto_capture_enabled: updates
            .auto_capture_enabled
            .or(current.auto_capture_enabled),
        last_summary_path: updates
            .last_summary_path
            .clone()
            .or_else(|| current.last_summary_path.clone()),
        summary_model_name: updates
            .summary_model_name
            .clone()
            .or_else(|| current.summary_model_name.clone()),
        analysis_prompt: updates
            .analysis_prompt
            .clone()
            .or_else(|| current.analysis_prompt.clone()),
        summary_prompt: updates
            .summary_prompt
            .clone()
            .or_else(|| current.summary_prompt.clone()),
        change_threshold: updates.change_threshold.or(current.change_threshold),
        max_silent_minutes: updates.max_silent_minutes.or(current.max_silent_minutes),
        summary_title_format: updates
            .summary_title_format
            .clone()
            .or_else(|| current.summary_title_format.clone()),
        include_manual_records: updates
            .include_manual_records
            .or(current.include_manual_records),
        window_whitelist: updates
            .window_whitelist
            .clone()
            .or_else(|| current.window_whitelist.clone()),
        window_blacklist: updates
            .window_blacklist
            .clone()
            .or_else(|| current.window_blacklist.clone()),
        use_whitelist_only: updates.use_whitelist_only.or(current.use_whitelist_only),
        auto_adjust_silent: updates.auto_adjust_silent.or(current.auto_adjust_silent),
        silent_adjustment_paused_until: updates
            .silent_adjustment_paused_until
            .clone()
            .or_else(|| current.silent_adjustment_paused_until.clone()),
        auto_detect_work_time: updates
            .auto_detect_work_time
            .or(current.auto_detect_work_time),
        use_custom_work_time: updates
            .use_custom_work_time
            .or(current.use_custom_work_time),
        custom_work_time_start: updates
            .custom_work_time_start
            .clone()
            .or_else(|| current.custom_work_time_start.clone()),
        custom_work_time_end: updates
            .custom_work_time_end
            .clone()
            .or_else(|| current.custom_work_time_end.clone()),
        learned_work_time: updates
            .learned_work_time
            .clone()
            .or_else(|| current.learned_work_time.clone()),
        capture_mode: updates
            .capture_mode
            .clone()
            .or_else(|| current.capture_mode.clone()),
        selected_monitor_index: updates
            .selected_monitor_index
            .or(current.selected_monitor_index),
        tag_categories: updates
            .tag_categories
            .clone()
            .or_else(|| current.tag_categories.clone()),
        is_ollama: updates.is_ollama.or(current.is_ollama),
        weekly_report_prompt: updates
            .weekly_report_prompt
            .clone()
            .or_else(|| current.weekly_report_prompt.clone()),
        weekly_report_day: updates.weekly_report_day.or(current.weekly_report_day),
        last_weekly_report_path: updates
            .last_weekly_report_path
            .clone()
            .or_else(|| current.last_weekly_report_path.clone()),
        monthly_report_prompt: updates
            .monthly_report_prompt
            .clone()
            .or_else(|| current.monthly_report_prompt.clone()),
        last_monthly_report_path: updates
            .last_monthly_report_path
            .clone()
            .or_else(|| current.last_monthly_report_path.clone()),
        custom_report_prompt: updates
            .custom_report_prompt
            .clone()
            .or_else(|| current.custom_report_prompt.clone()),
        last_custom_report_path: updates
            .last_custom_report_path
            .clone()
            .or_else(|| current.last_custom_report_path.clone()),
        obsidian_vaults: updates
            .obsidian_vaults
            .clone()
            .or_else(|| current.obsidian_vaults.clone()),
        auto_detect_vault_by_window: updates
            .auto_detect_vault_by_window
            .or(current.auto_detect_vault_by_window),
        comparison_report_prompt: updates
            .comparison_report_prompt
            .clone()
            .or_else(|| current.comparison_report_prompt.clone()),
        capture_only_mode: updates.capture_only_mode.or(current.capture_only_mode),
        custom_headers: updates
            .custom_headers
            .clone()
            .or_else(|| current.custom_headers.clone()),
        quality_filter_enabled: updates
            .quality_filter_enabled
            .or(current.quality_filter_enabled),
        quality_filter_threshold: updates
            .quality_filter_threshold
            .or(current.quality_filter_threshold),
        session_gap_minutes: updates.session_gap_minutes.or(current.session_gap_minutes),
        proxy_enabled: updates.proxy_enabled.or(current.proxy_enabled),
        proxy_host: updates
            .proxy_host
            .clone()
            .or_else(|| current.proxy_host.clone()),
        proxy_port: updates.proxy_port.or(current.proxy_port),
        proxy_username: updates
            .proxy_username
            .clone()
            .or_else(|| current.proxy_username.clone()),
        proxy_password: updates
            .proxy_password
            .clone()
            .or_else(|| current.proxy_password.clone()),
        test_model_name: updates
            .test_model_name
            .clone()
            .or_else(|| current.test_model_name.clone()),
        onboarding_completed: updates
            .onboarding_completed
            .or(current.onboarding_completed),
        language: updates
            .language
            .clone()
            .or_else(|| current.language.clone()),
        preferred_language: updates
            .preferred_language
            .clone()
            .or_else(|| current.preferred_language.clone()),
        supported_languages: updates
            .supported_languages
            .clone()
            .or_else(|| current.supported_languages.clone()),
        auto_backup_enabled: updates.auto_backup_enabled.or(current.auto_backup_enabled),
        auto_backup_interval: updates
            .auto_backup_interval
            .clone()
            .or_else(|| current.auto_backup_interval.clone()),
        auto_backup_retention: updates
            .auto_backup_retention
            .or(current.auto_backup_retention),
        last_auto_backup_at: updates
            .last_auto_backup_at
            .clone()
            .or_else(|| current.last_auto_backup_at.clone()),
        custom_export_template: updates
            .custom_export_template
            .clone()
            .or_else(|| current.custom_export_template.clone()),
    }
}

/// Save settings to the database.
///
/// This function saves the application settings.
/// It wraps the sync function from memory_storage for consistency with the service layer pattern.
pub fn save_settings_service(settings: &Settings) -> AppResult<()> {
    let current = crate::memory_storage::get_settings_sync()?;
    let merged = merge_settings(&current, settings);
    crate::memory_storage::save_settings_sync(&merged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn save_settings_service_merges_partial_updates() {
        crate::memory_storage::setup_test_db_with_schema();

        let initial = get_settings_service().unwrap();
        assert_eq!(initial.screenshot_interval, Some(5));
        assert_eq!(initial.summary_time.as_deref(), Some("18:00"));
        assert_eq!(initial.language.as_deref(), Some("en"));

        let partial = Settings {
            language: Some("zh-CN".to_string()),
            api_base_url: Some("https://api.openai.com/v1".to_string()),
            ..Default::default()
        };

        save_settings_service(&partial).unwrap();

        let updated = get_settings_service().unwrap();
        assert_eq!(updated.language.as_deref(), Some("zh-CN"));
        assert_eq!(
            updated.api_base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(updated.screenshot_interval, Some(5));
        assert_eq!(updated.summary_time.as_deref(), Some("18:00"));
    }
}
