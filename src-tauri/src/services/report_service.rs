//! Report service - Business logic for report generation
//!
//! This module contains business logic functions for report generation.
//! Commands should delegate to these service functions rather than implementing logic directly.
//!
//! REPORT-001: Weekly report generation
//! REPORT-002: Monthly report generation
//! REPORT-003: Custom period report generation
//! REPORT-004: Comparison report between two time periods
//! DATA-007: Multi-language daily report support

// Re-export helper functions and types from synthesis for use by service functions
pub use crate::synthesis::{
    build_session_based_report, filter_records_by_settings, format_records_by_week,
    format_records_for_summary, generate_base_daily_summary, generate_comparison_report_filename,
    generate_custom_report_filename, generate_monthly_report_filename, generate_summary_filename,
    generate_summary_filename_with_lang, generate_weekly_report_filename,
    get_default_comparison_report_prompt, get_default_custom_report_prompt,
    get_default_monthly_report_prompt, get_default_summary_prompt,
    get_default_weekly_report_prompt, get_supported_languages, send_report_notifications,
    translate_report, write_report_to_logseq, write_report_to_obsidian, ApiConfig,
};

use crate::synthesis::{
    non_empty_or, DEFAULT_COMPARISON_REPORT_PROMPT, DEFAULT_CUSTOM_REPORT_PROMPT,
    DEFAULT_MONTHLY_REPORT_PROMPT, DEFAULT_SUMMARY_PROMPT, DEFAULT_WEEKLY_REPORT_PROMPT,
};

/// Write a generated report to all configured destinations (Obsidian, Logseq,
/// Notion, Slack/DingTalk), optionally persist the path in settings, and return
/// the Obsidian file path.
///
/// `obsidian_path` - The Obsidian output path to write to.
/// `update_settings` is called with a mutable reference to a cloned settings
/// object and the file path string, allowing the caller to set the appropriate
/// `last_*_path` field. Pass `None` to skip settings persistence (e.g. for
/// comparison reports).
#[allow(clippy::type_complexity)]
fn write_report_to_all_destinations(
    settings: &crate::memory_storage::Settings,
    obsidian_path: &str,
    filename: &str,
    summary: &str,
    report_label: &str,
    update_settings: Option<&dyn Fn(&mut crate::memory_storage::Settings, &str)>,
) -> Result<String, String> {
    let path_str = write_report_to_obsidian(obsidian_path, filename, summary)?;

    // INT-002: Also write to Logseq if configured
    write_report_to_logseq(settings, filename, summary);

    // INT-001: Also write to Notion if configured
    if let Some(notion_url) =
        crate::notion::write_report_to_notion_sync(settings, filename, summary)
    {
        tracing::info!("{} also written to Notion: {}", report_label, notion_url);
    }

    // INT-004: Send notifications to Slack/DingTalk if configured
    let title = filename.trim_end_matches(".md");
    send_report_notifications(settings, title, summary);

    // Persist last-report-path in settings
    if let Some(updater) = update_settings {
        let mut updated_settings = settings.clone();
        updater(&mut updated_settings, &path_str);
        crate::memory_storage::save_settings_sync(&updated_settings)
            .map_err(|e| format!("Failed to update settings: {}", e))?;
    }

    tracing::info!("{} generated: {}", report_label, path_str);
    Ok(path_str)
}

/// Generate daily summary - report generation service
///
/// # Arguments
/// * `vault_name` - Optional vault name to use. If None, uses default vault or auto-detection.
pub async fn generate_daily_summary_service(vault_name: Option<String>) -> Result<String, String> {
    if !crate::network_status::is_online() {
        let _ = crate::offline_queue::enqueue_task(
            &crate::offline_queue::OfflineTaskType::DailySummary,
            "{}",
            None,
        );
        return Err("当前处于离线状态，日报生成已加入队列，网络恢复后将自动处理".to_string());
    }

    let settings = crate::memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let auto_detect = settings.auto_detect_vault_by_window.unwrap_or(false);
    let obsidian_path = settings.get_effective_vault(vault_name.as_deref(), auto_detect)?;
    let api_config = crate::synthesis::load_api_config(&settings)
        .map_err(|e| format!("Failed to load API config: {}", e))?;

    // SESSION-005: Try session-based approach first
    let sessions = crate::session_manager::get_today_sessions_sync().unwrap_or_default();

    if !sessions.is_empty() {
        // SESSION-005 AC#4: Auto-analyze pending/ended sessions before generating report
        for session in &sessions {
            if session.status == crate::session_manager::SessionStatus::Active
                || session.status == crate::session_manager::SessionStatus::Ended
            {
                tracing::info!("Auto-analyzing session {} before daily summary", session.id);
                if let Err(e) = crate::session_manager::analyze_session(session.id).await {
                    tracing::warn!("Failed to analyze session {}: {}", session.id, e);
                }
            }
        }

        // Re-fetch sessions after analysis
        let sessions = crate::session_manager::get_today_sessions_sync().unwrap_or_default();

        // SESSION-005: Build session-based report
        if let Some(content) = build_session_based_report(&sessions) {
            let prompt_template =
                non_empty_or(settings.summary_prompt.as_deref(), DEFAULT_SUMMARY_PROMPT);
            let prompt = prompt_template
                .replace("{records}", &content)
                .replace("{github_activity}", "");

            let summary = crate::synthesis::call_llm_api_with_retry(
                &api_config,
                &prompt,
                2000,
                "generate_daily_summary",
            )
            .await?;

            let filename = generate_summary_filename(&settings);
            return write_report_to_all_destinations(
                &settings,
                &obsidian_path,
                &filename,
                &summary,
                "Daily summary",
                Some(&|s, p| s.last_summary_path = Some(p.to_string())),
            );
        }
    }

    // Legacy record-based approach (when no sessions)
    let all_records = crate::memory_storage::get_today_records_sync()
        .map_err(|e| format!("Failed to get today's records: {}", e))?;
    let records = filter_records_by_settings(all_records, &settings);

    if records.is_empty() {
        return Err("今日无记录".to_string());
    }

    let records_text = format_records_for_summary(&records);
    let prompt_template = non_empty_or(settings.summary_prompt.as_deref(), DEFAULT_SUMMARY_PROMPT);
    let prompt = prompt_template
        .replace("{records}", &records_text)
        .replace("{github_activity}", "");

    let summary = crate::synthesis::call_llm_api_with_retry(
        &api_config,
        &prompt,
        2000,
        "generate_daily_summary",
    )
    .await?;

    let filename = generate_summary_filename(&settings);
    write_report_to_all_destinations(
        &settings,
        &obsidian_path,
        &filename,
        &summary,
        "Daily summary",
        Some(&|s, p| s.last_summary_path = Some(p.to_string())),
    )
}

/// Generate multilingual daily summary - report generation service
pub async fn generate_multilingual_daily_summary_service(
    target_lang: String,
) -> Result<String, String> {
    if !crate::network_status::is_online() {
        return Err("当前处于离线状态，多语言日报生成需要网络连接".to_string());
    }

    let settings = crate::memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let api_config = crate::synthesis::load_api_config(&settings)
        .map_err(|e| format!("Failed to load API config: {}", e))?;

    // Get the default (Chinese) summary first
    let summary = generate_base_daily_summary(&settings, &api_config).await?;

    // If target language is Chinese (default), return as-is
    if target_lang == "zh-CN" || target_lang.is_empty() {
        return Ok(summary);
    }

    // Translate to target language
    let translated = translate_report(&api_config, &summary, &target_lang).await?;

    // Save the translated version
    let obsidian_path = settings.get_obsidian_output_path()?;
    let filename = generate_summary_filename_with_lang(&settings, &target_lang);
    let path_str = write_report_to_obsidian(&obsidian_path, &filename, &translated)?;

    tracing::info!(
        "Multilingual daily summary generated: {} (lang: {})",
        path_str,
        target_lang
    );
    Ok(path_str)
}

/// Generate weekly report - REPORT-001 service
pub async fn generate_weekly_report_service() -> Result<String, String> {
    if !crate::network_status::is_online() {
        let _ = crate::offline_queue::enqueue_task(
            &crate::offline_queue::OfflineTaskType::WeeklyReport,
            "{}",
            None,
        );
        return Err("当前处于离线状态，周报生成已加入队列，网络恢复后将自动处理".to_string());
    }

    let settings = crate::memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let obsidian_path = settings.get_obsidian_output_path()?;
    let api_config = crate::synthesis::load_api_config(&settings)
        .map_err(|e| format!("Failed to load API config: {}", e))?;

    let week_start_day = settings.weekly_report_day.unwrap_or(0);
    let all_records = crate::memory_storage::get_week_records_sync(week_start_day)
        .map_err(|e| format!("Failed to get week records: {}", e))?;
    let records = filter_records_by_settings(all_records, &settings);
    if records.is_empty() {
        return Err("本周无记录".to_string());
    }

    let records_text = format_records_for_summary(&records);
    let prompt_template = non_empty_or(
        settings.weekly_report_prompt.as_deref(),
        DEFAULT_WEEKLY_REPORT_PROMPT,
    );
    let prompt = prompt_template.replace("{records}", &records_text);

    let summary = crate::synthesis::call_llm_api_with_retry(
        &api_config,
        &prompt,
        3000,
        "generate_weekly_report",
    )
    .await?;

    let filename = generate_weekly_report_filename(week_start_day);
    write_report_to_all_destinations(
        &settings,
        &obsidian_path,
        &filename,
        &summary,
        "Weekly report",
        Some(&|s, p| s.last_weekly_report_path = Some(p.to_string())),
    )
}

/// Generate monthly report - REPORT-002 service
pub async fn generate_monthly_report_service() -> Result<String, String> {
    if !crate::network_status::is_online() {
        let _ = crate::offline_queue::enqueue_task(
            &crate::offline_queue::OfflineTaskType::MonthlyReport,
            "{}",
            None,
        );
        return Err("当前处于离线状态，月报生成已加入队列，网络恢复后将自动处理".to_string());
    }

    let settings = crate::memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let obsidian_path = settings.get_obsidian_output_path()?;
    let api_config = crate::synthesis::load_api_config(&settings)
        .map_err(|e| format!("Failed to load API config: {}", e))?;

    let all_records = crate::memory_storage::get_month_records_sync()
        .map_err(|e| format!("Failed to get month records: {}", e))?;
    let records = filter_records_by_settings(all_records, &settings);
    if records.is_empty() {
        return Err("本月无记录".to_string());
    }

    let records_text = format_records_by_week(&records);
    let prompt_template = non_empty_or(
        settings.monthly_report_prompt.as_deref(),
        DEFAULT_MONTHLY_REPORT_PROMPT,
    );
    let prompt = prompt_template.replace("{records}", &records_text);

    let summary = crate::synthesis::call_llm_api_with_retry(
        &api_config,
        &prompt,
        4000,
        "generate_monthly_report",
    )
    .await?;

    let filename = generate_monthly_report_filename();
    write_report_to_all_destinations(
        &settings,
        &obsidian_path,
        &filename,
        &summary,
        "Monthly report",
        Some(&|s, p| s.last_monthly_report_path = Some(p.to_string())),
    )
}

/// Generate custom period report - REPORT-003 service
pub async fn generate_custom_report_service(
    start_date: String,
    end_date: String,
    report_name: Option<String>,
) -> Result<String, String> {
    if !crate::network_status::is_online() {
        return Err("当前处于离线状态，报告生成需要网络连接。请检查网络连接后重试。".to_string());
    }

    let parsed_start = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("无效的起始日期格式 (需要 YYYY-MM-DD): {}", e))?;
    let parsed_end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| format!("无效的结束日期格式 (需要 YYYY-MM-DD): {}", e))?;
    if parsed_end < parsed_start {
        return Err("结束日期不能早于起始日期".to_string());
    }

    let settings = crate::memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let obsidian_path = settings.get_obsidian_output_path()?;
    let api_config = crate::synthesis::load_api_config(&settings)
        .map_err(|e| format!("Failed to load API config: {}", e))?;

    let all_records =
        crate::memory_storage::get_records_by_date_range_sync(start_date.clone(), end_date.clone())
            .map_err(|e| format!("Failed to get records: {}", e))?;
    let records = filter_records_by_settings(all_records, &settings);
    if records.is_empty() {
        return Err("所选时间范围内无记录".to_string());
    }

    let day_count = (parsed_end - parsed_start).num_days() + 1;
    let records_text = if day_count > 14 {
        format_records_by_week(&records)
    } else {
        format_records_for_summary(&records)
    };

    let prompt_template = non_empty_or(
        settings.custom_report_prompt.as_deref(),
        DEFAULT_CUSTOM_REPORT_PROMPT,
    );
    let prompt = prompt_template.replace("{records}", &records_text);

    let summary = crate::synthesis::call_llm_api_with_retry(
        &api_config,
        &prompt,
        4000,
        "generate_custom_report",
    )
    .await?;

    let report_name = report_name.unwrap_or_else(|| "自定义报告".to_string());
    let filename = generate_custom_report_filename(&report_name, &start_date, &end_date);
    write_report_to_all_destinations(
        &settings,
        &obsidian_path,
        &filename,
        &summary,
        "Custom report",
        Some(&|s, p| s.last_custom_report_path = Some(p.to_string())),
    )
}

/// Generate comparison report between two time periods - REPORT-004 service
pub async fn compare_reports_service(
    start_date_a: String,
    end_date_a: String,
    start_date_b: String,
    end_date_b: String,
) -> Result<String, String> {
    if !crate::network_status::is_online() {
        return Err("当前处于离线状态，报告生成需要网络连接。请检查网络连接后重试。".to_string());
    }

    let parsed_start_a = chrono::NaiveDate::parse_from_str(&start_date_a, "%Y-%m-%d")
        .map_err(|e| format!("无效的时段A起始日期格式 (需要 YYYY-MM-DD): {}", e))?;
    let parsed_end_a = chrono::NaiveDate::parse_from_str(&end_date_a, "%Y-%m-%d")
        .map_err(|e| format!("无效的时段A结束日期格式 (需要 YYYY-MM-DD): {}", e))?;
    let parsed_start_b = chrono::NaiveDate::parse_from_str(&start_date_b, "%Y-%m-%d")
        .map_err(|e| format!("无效的时段B起始日期格式 (需要 YYYY-MM-DD): {}", e))?;
    let parsed_end_b = chrono::NaiveDate::parse_from_str(&end_date_b, "%Y-%m-%d")
        .map_err(|e| format!("无效的时段B结束日期格式 (需要 YYYY-MM-DD): {}", e))?;

    if parsed_end_a < parsed_start_a {
        return Err("时段A的结束日期不能早于起始日期".to_string());
    }
    if parsed_end_b < parsed_start_b {
        return Err("时段B的结束日期不能早于起始日期".to_string());
    }

    let settings = crate::memory_storage::get_settings_sync()
        .map_err(|e| format!("Failed to get settings: {}", e))?;
    let obsidian_path = settings.get_obsidian_output_path()?;
    let api_config = crate::synthesis::load_api_config(&settings)
        .map_err(|e| format!("Failed to load API config: {}", e))?;

    let all_records_a = crate::memory_storage::get_records_by_date_range_sync(
        start_date_a.clone(),
        end_date_a.clone(),
    )
    .map_err(|e| format!("Failed to get period A records: {}", e))?;
    let all_records_b = crate::memory_storage::get_records_by_date_range_sync(
        start_date_b.clone(),
        end_date_b.clone(),
    )
    .map_err(|e| format!("Failed to get period B records: {}", e))?;

    let records_a = filter_records_by_settings(all_records_a, &settings);
    let records_b = filter_records_by_settings(all_records_b, &settings);

    if records_a.is_empty() && records_b.is_empty() {
        return Err("两个时间段内均无记录".to_string());
    }

    let day_count_a = (parsed_end_a - parsed_start_a).num_days() + 1;
    let records_text_a = if day_count_a > 14 {
        format_records_by_week(&records_a)
    } else {
        format_records_for_summary(&records_a)
    };

    let day_count_b = (parsed_end_b - parsed_start_b).num_days() + 1;
    let records_text_b = if day_count_b > 14 {
        format_records_by_week(&records_b)
    } else {
        format_records_for_summary(&records_b)
    };

    let prompt_template = non_empty_or(
        settings.comparison_report_prompt.as_deref(),
        DEFAULT_COMPARISON_REPORT_PROMPT,
    );
    let prompt = prompt_template
        .replace("{records_a}", &records_text_a)
        .replace("{records_b}", &records_text_b)
        .replace("{start_date_a}", &start_date_a)
        .replace("{end_date_a}", &end_date_a)
        .replace("{start_date_b}", &start_date_b)
        .replace("{end_date_b}", &end_date_b);

    let summary =
        crate::synthesis::call_llm_api_with_retry(&api_config, &prompt, 4000, "compare_reports")
            .await?;

    let filename =
        generate_comparison_report_filename(&start_date_a, &end_date_a, &start_date_b, &end_date_b);
    write_report_to_all_destinations(
        &settings,
        &obsidian_path,
        &filename,
        &summary,
        "Comparison report",
        None,
    )
}
