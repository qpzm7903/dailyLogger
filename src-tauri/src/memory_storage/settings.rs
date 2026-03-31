use crate::crypto;
use crate::errors::{AppError, AppResult};
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

use super::{Settings, DB_CONNECTION};

/// In-memory cache for Settings to avoid repeated DB queries.
/// Write-through: updated on every `save_settings_sync` call.
/// Uses `Arc<Settings>` so all readers share the same allocation —
/// `get_settings_sync()` returns a cheap `Arc` clone instead of cloning
/// the full 64-field `Settings` struct.
static SETTINGS_CACHE: Lazy<RwLock<Option<Arc<Settings>>>> = Lazy::new(|| RwLock::new(None));

/// Invalidate the settings cache. Called when the database is re-initialized.
pub fn invalidate_settings_cache() {
    if let Ok(mut cache) = SETTINGS_CACHE.write() {
        *cache = None;
    }
}

pub fn get_settings_sync() -> AppResult<Arc<Settings>> {
    // Fast path: return cached Arc (cheap atomic refcount bump, no struct clone)
    if let Ok(cache) = SETTINGS_CACHE.read() {
        if let Some(ref settings) = *cache {
            return Ok(Arc::clone(settings));
        }
    }

    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    let mut stmt = conn
        .prepare(
            "SELECT api_base_url, api_key, model_name, screenshot_interval,
                summary_time, obsidian_path, auto_capture_enabled, last_summary_path,
                summary_model_name, analysis_prompt, summary_prompt,
                change_threshold, max_silent_minutes, summary_title_format,
                include_manual_records, window_whitelist, window_blacklist, use_whitelist_only,
                auto_adjust_silent, silent_adjustment_paused_until,
                auto_detect_work_time, use_custom_work_time,
                custom_work_time_start, custom_work_time_end, learned_work_time,
                capture_mode, selected_monitor_index, tag_categories, is_ollama,
                weekly_report_prompt, weekly_report_day, last_weekly_report_path,
                monthly_report_prompt, custom_report_prompt, last_custom_report_path,
                last_monthly_report_path, obsidian_vaults,
                auto_detect_vault_by_window,
                comparison_report_prompt, logseq_graphs,
                notion_api_key, notion_database_id,
                slack_webhook_url, dingtalk_webhook_url, capture_only_mode, custom_headers,
                quality_filter_enabled, quality_filter_threshold, session_gap_minutes,
                proxy_enabled, proxy_host, proxy_port, proxy_username, proxy_password,
                test_model_name, onboarding_completed, language,
                preferred_language, supported_languages,
                auto_backup_enabled, auto_backup_interval, auto_backup_retention,
                last_auto_backup_at, custom_export_template
         FROM settings WHERE id = 1",
        )
        .map_err(AppError::from)?;

    let settings = stmt
        .query_row([], |row| {
            Ok(Settings {
                api_base_url: row.get("api_base_url")?,
                api_key: row.get("api_key")?,
                model_name: row.get("model_name")?,
                screenshot_interval: row.get("screenshot_interval")?,
                summary_time: row.get("summary_time")?,
                obsidian_path: row.get("obsidian_path")?,
                auto_capture_enabled: row
                    .get::<_, Option<i32>>("auto_capture_enabled")?
                    .map(|v| v != 0),
                last_summary_path: row.get("last_summary_path")?,
                summary_model_name: row.get("summary_model_name")?,
                analysis_prompt: row.get("analysis_prompt")?,
                summary_prompt: row.get("summary_prompt")?,
                change_threshold: row.get("change_threshold")?,
                max_silent_minutes: row.get("max_silent_minutes")?,
                summary_title_format: row.get("summary_title_format")?,
                include_manual_records: row
                    .get::<_, Option<i32>>("include_manual_records")?
                    .map(|v| v != 0),
                window_whitelist: row.get("window_whitelist")?,
                window_blacklist: row.get("window_blacklist")?,
                use_whitelist_only: row
                    .get::<_, Option<i32>>("use_whitelist_only")?
                    .map(|v| v != 0),
                auto_adjust_silent: row
                    .get::<_, Option<i32>>("auto_adjust_silent")?
                    .map(|v| v != 0),
                silent_adjustment_paused_until: row.get("silent_adjustment_paused_until")?,
                auto_detect_work_time: row
                    .get::<_, Option<i32>>("auto_detect_work_time")?
                    .map(|v| v != 0),
                use_custom_work_time: row
                    .get::<_, Option<i32>>("use_custom_work_time")?
                    .map(|v| v != 0),
                custom_work_time_start: row.get("custom_work_time_start")?,
                custom_work_time_end: row.get("custom_work_time_end")?,
                learned_work_time: row.get("learned_work_time")?,
                capture_mode: row.get("capture_mode")?,
                selected_monitor_index: row.get("selected_monitor_index")?,
                tag_categories: row.get("tag_categories")?,
                is_ollama: row.get::<_, Option<i32>>("is_ollama")?.map(|v| v != 0),
                weekly_report_prompt: row.get("weekly_report_prompt")?,
                weekly_report_day: row.get("weekly_report_day")?,
                last_weekly_report_path: row.get("last_weekly_report_path")?,
                monthly_report_prompt: row.get("monthly_report_prompt")?,
                last_monthly_report_path: row.get("last_monthly_report_path")?,
                custom_report_prompt: row.get("custom_report_prompt")?,
                last_custom_report_path: row.get("last_custom_report_path")?,
                obsidian_vaults: row.get("obsidian_vaults")?,
                // VAULT-001: Auto-detect vault by window title
                auto_detect_vault_by_window: row
                    .get::<_, Option<i32>>("auto_detect_vault_by_window")?
                    .map(|v| v != 0),
                comparison_report_prompt: row.get("comparison_report_prompt")?,
                logseq_graphs: row.get("logseq_graphs")?,
                notion_api_key: row.get("notion_api_key")?,
                notion_database_id: row.get("notion_database_id")?,
                slack_webhook_url: row.get("slack_webhook_url")?,
                dingtalk_webhook_url: row.get("dingtalk_webhook_url")?,
                capture_only_mode: row
                    .get::<_, Option<i32>>("capture_only_mode")?
                    .map(|v| v != 0),
                custom_headers: row.get("custom_headers")?,
                // EXP-002: Quality filter settings
                quality_filter_enabled: row
                    .get::<_, Option<i32>>("quality_filter_enabled")?
                    .map(|v| v != 0),
                quality_filter_threshold: row.get("quality_filter_threshold")?,
                // SESSION-001: Session gap minutes
                session_gap_minutes: row.get("session_gap_minutes")?,
                // PERF-001: Proxy settings
                proxy_enabled: row.get::<_, Option<i32>>("proxy_enabled")?.map(|v| v != 0),
                proxy_host: row.get("proxy_host")?,
                proxy_port: row.get("proxy_port")?,
                proxy_username: row.get("proxy_username")?,
                proxy_password: row.get("proxy_password")?,
                // PERF-001: Test model name
                test_model_name: row.get("test_model_name")?,
                // PERF-002: Onboarding completed flag
                onboarding_completed: row
                    .get::<_, Option<i32>>("onboarding_completed")?
                    .map(|v| v != 0),
                // PERF-005: Language setting
                language: row.get("language")?,
                // DATA-007: Multi-language settings
                preferred_language: row.get("preferred_language")?,
                supported_languages: row.get("supported_languages")?,
                // STAB-002: Auto backup settings
                auto_backup_enabled: row
                    .get::<_, Option<i32>>("auto_backup_enabled")?
                    .map(|v| v != 0),
                auto_backup_interval: row.get("auto_backup_interval")?,
                auto_backup_retention: row.get("auto_backup_retention")?,
                last_auto_backup_at: row.get("last_auto_backup_at")?,
                // FEAT-008: Custom export template
                custom_export_template: row.get("custom_export_template")?,
            })
        })
        .map_err(AppError::from)?;

    // Decrypt API key if it's encrypted
    let settings = if let Some(ref api_key) = settings.api_key {
        if !api_key.is_empty() {
            let mut decrypted_settings = settings.clone();
            decrypted_settings.api_key = Some(crypto::decrypt_api_key(api_key)?);
            // Also decrypt notion_api_key if present
            if let Some(ref notion_api_key) = settings.notion_api_key {
                if !notion_api_key.is_empty() {
                    decrypted_settings.notion_api_key =
                        Some(crypto::decrypt_api_key(notion_api_key)?);
                }
            }
            // PERF-001: Decrypt proxy password if present
            if let Some(ref proxy_password) = settings.proxy_password {
                if !proxy_password.is_empty() {
                    decrypted_settings.proxy_password =
                        Some(crypto::decrypt_api_key(proxy_password)?);
                }
            }
            // AI-006: Decrypt sensitive values in custom_headers
            if let Some(ref custom_headers) = settings.custom_headers {
                if !custom_headers.is_empty() {
                    match serde_json::from_str::<Vec<super::CustomHeader>>(custom_headers) {
                        Ok(mut headers) => {
                            for header in &mut headers {
                                if header.sensitive
                                    && !header.value.is_empty()
                                    && crypto::is_encrypted(&header.value)
                                {
                                    match crypto::decrypt_api_key(&header.value) {
                                        Ok(decrypted) => header.value = decrypted,
                                        Err(e) => {
                                            tracing::error!(
                                                "Failed to decrypt custom header: {}",
                                                e
                                            )
                                        }
                                    }
                                }
                            }
                            match serde_json::to_string(&headers) {
                                Ok(json) => decrypted_settings.custom_headers = Some(json),
                                Err(e) => {
                                    tracing::error!("Failed to serialize custom headers: {}", e)
                                }
                            }
                        }
                        Err(e) => tracing::error!("Failed to parse custom headers JSON: {}", e),
                    }
                }
            }
            decrypted_settings
        } else {
            settings
        }
    } else {
        settings
    };

    let settings = Arc::new(settings);

    // Update cache
    if let Ok(mut cache) = SETTINGS_CACHE.write() {
        *cache = Some(Arc::clone(&settings));
    }

    Ok(settings)
}

pub fn save_settings_sync(settings: &Settings) -> AppResult<()> {
    // Encrypt API key before saving
    let encrypted_api_key = if let Some(ref api_key) = settings.api_key {
        if !api_key.is_empty() && !crypto::is_encrypted(api_key) {
            Some(crypto::encrypt_api_key(api_key)?)
        } else {
            settings.api_key.clone()
        }
    } else {
        None
    };

    // INT-001: Encrypt Notion API key before saving
    let encrypted_notion_api_key = if let Some(ref notion_api_key) = settings.notion_api_key {
        if !notion_api_key.is_empty() && !crypto::is_encrypted(notion_api_key) {
            Some(crypto::encrypt_api_key(notion_api_key)?)
        } else {
            settings.notion_api_key.clone()
        }
    } else {
        None
    };

    // AI-006: Encrypt sensitive values in custom_headers before saving
    let encrypted_custom_headers = if let Some(ref custom_headers) = settings.custom_headers {
        if !custom_headers.is_empty() {
            match serde_json::from_str::<Vec<super::CustomHeader>>(custom_headers) {
                Ok(mut headers) => {
                    for header in &mut headers {
                        if header.sensitive
                            && !header.value.is_empty()
                            && !crypto::is_encrypted(&header.value)
                        {
                            match crypto::encrypt_api_key(&header.value) {
                                Ok(encrypted) => header.value = encrypted,
                                Err(e) => tracing::error!("Failed to encrypt custom header: {}", e),
                            }
                        }
                    }
                    match serde_json::to_string(&headers) {
                        Ok(json) => Some(json),
                        Err(e) => {
                            tracing::error!("Failed to serialize custom headers: {}", e);
                            settings.custom_headers.clone()
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse custom headers JSON: {}", e);
                    settings.custom_headers.clone()
                }
            }
        } else {
            settings.custom_headers.clone()
        }
    } else {
        None
    };

    // PERF-001: Encrypt proxy password before saving
    let encrypted_proxy_password = if let Some(ref proxy_password) = settings.proxy_password {
        if !proxy_password.is_empty() && !crypto::is_encrypted(proxy_password) {
            Some(crypto::encrypt_api_key(proxy_password)?)
        } else {
            settings.proxy_password.clone()
        }
    } else {
        None
    };

    // AI-005: Auto-detect Ollama endpoint based on api_base_url
    let is_ollama = settings
        .api_base_url
        .as_ref()
        .map(|url| crate::ollama::is_ollama_endpoint(url))
        .unwrap_or(false);

    let db = DB_CONNECTION.lock()?;
    let conn = db
        .as_ref()
        .ok_or_else(|| AppError::database("Database not initialized"))?;

    conn.execute(
        "UPDATE settings SET
            api_base_url = :api_base_url,
            api_key = :api_key,
            model_name = :model_name,
            screenshot_interval = :screenshot_interval,
            summary_time = :summary_time,
            obsidian_path = :obsidian_path,
            auto_capture_enabled = :auto_capture_enabled,
            last_summary_path = :last_summary_path,
            summary_model_name = :summary_model_name,
            analysis_prompt = :analysis_prompt,
            summary_prompt = :summary_prompt,
            change_threshold = :change_threshold,
            max_silent_minutes = :max_silent_minutes,
            summary_title_format = :summary_title_format,
            include_manual_records = :include_manual_records,
            window_whitelist = :window_whitelist,
            window_blacklist = :window_blacklist,
            use_whitelist_only = :use_whitelist_only,
            auto_adjust_silent = :auto_adjust_silent,
            silent_adjustment_paused_until = :silent_adjustment_paused_until,
            auto_detect_work_time = :auto_detect_work_time,
            use_custom_work_time = :use_custom_work_time,
            custom_work_time_start = :custom_work_time_start,
            custom_work_time_end = :custom_work_time_end,
            learned_work_time = :learned_work_time,
            capture_mode = :capture_mode,
            selected_monitor_index = :selected_monitor_index,
            tag_categories = :tag_categories,
            is_ollama = :is_ollama,
            weekly_report_prompt = :weekly_report_prompt,
            weekly_report_day = :weekly_report_day,
            last_weekly_report_path = :last_weekly_report_path,
            monthly_report_prompt = :monthly_report_prompt,
            custom_report_prompt = :custom_report_prompt,
            last_custom_report_path = :last_custom_report_path,
            last_monthly_report_path = :last_monthly_report_path,
            obsidian_vaults = :obsidian_vaults,
            auto_detect_vault_by_window = :auto_detect_vault_by_window,
            comparison_report_prompt = :comparison_report_prompt,
            logseq_graphs = :logseq_graphs,
            notion_api_key = :notion_api_key,
            notion_database_id = :notion_database_id,
            slack_webhook_url = :slack_webhook_url,
            dingtalk_webhook_url = :dingtalk_webhook_url,
            capture_only_mode = :capture_only_mode,
            custom_headers = :custom_headers,
            quality_filter_enabled = :quality_filter_enabled,
            quality_filter_threshold = :quality_filter_threshold,
            session_gap_minutes = :session_gap_minutes,
            proxy_enabled = :proxy_enabled,
            proxy_host = :proxy_host,
            proxy_port = :proxy_port,
            proxy_username = :proxy_username,
            proxy_password = :proxy_password,
            test_model_name = :test_model_name,
            onboarding_completed = :onboarding_completed,
            language = :language,
            preferred_language = :preferred_language,
            supported_languages = :supported_languages,
            auto_backup_enabled = :auto_backup_enabled,
            auto_backup_interval = :auto_backup_interval,
            auto_backup_retention = :auto_backup_retention,
            last_auto_backup_at = :last_auto_backup_at
         WHERE id = 1",
        rusqlite::named_params! {
            ":api_base_url": settings.api_base_url,
            ":api_key": encrypted_api_key,
            ":model_name": settings.model_name,
            ":screenshot_interval": settings.screenshot_interval,
            ":summary_time": settings.summary_time,
            ":obsidian_path": settings.obsidian_path,
            ":auto_capture_enabled": settings.auto_capture_enabled.map(|v| if v { 1 } else { 0 }),
            ":last_summary_path": settings.last_summary_path,
            ":summary_model_name": settings.summary_model_name,
            ":analysis_prompt": settings.analysis_prompt,
            ":summary_prompt": settings.summary_prompt,
            ":change_threshold": settings.change_threshold,
            ":max_silent_minutes": settings.max_silent_minutes,
            ":summary_title_format": settings.summary_title_format,
            ":include_manual_records": settings.include_manual_records.map(|v| if v { 1 } else { 0 }),
            ":window_whitelist": settings.window_whitelist,
            ":window_blacklist": settings.window_blacklist,
            ":use_whitelist_only": settings.use_whitelist_only.map(|v| if v { 1 } else { 0 }),
            ":auto_adjust_silent": settings.auto_adjust_silent.map(|v| if v { 1 } else { 0 }),
            ":silent_adjustment_paused_until": settings.silent_adjustment_paused_until,
            ":auto_detect_work_time": settings.auto_detect_work_time.map(|v| if v { 1 } else { 0 }),
            ":use_custom_work_time": settings.use_custom_work_time.map(|v| if v { 1 } else { 0 }),
            ":custom_work_time_start": settings.custom_work_time_start,
            ":custom_work_time_end": settings.custom_work_time_end,
            ":learned_work_time": settings.learned_work_time,
            ":capture_mode": settings.capture_mode,
            ":selected_monitor_index": settings.selected_monitor_index,
            ":tag_categories": settings.tag_categories,
            ":is_ollama": Some(if is_ollama { 1 } else { 0 }),
            ":weekly_report_prompt": settings.weekly_report_prompt,
            ":weekly_report_day": settings.weekly_report_day,
            ":last_weekly_report_path": settings.last_weekly_report_path,
            ":monthly_report_prompt": settings.monthly_report_prompt,
            ":custom_report_prompt": settings.custom_report_prompt,
            ":last_custom_report_path": settings.last_custom_report_path,
            ":last_monthly_report_path": settings.last_monthly_report_path,
            ":obsidian_vaults": settings.obsidian_vaults,
            ":auto_detect_vault_by_window": settings.auto_detect_vault_by_window.map(|v| if v { 1 } else { 0 }),
            ":comparison_report_prompt": settings.comparison_report_prompt,
            ":logseq_graphs": settings.logseq_graphs,
            ":notion_api_key": encrypted_notion_api_key,
            ":notion_database_id": settings.notion_database_id,
            ":slack_webhook_url": settings.slack_webhook_url,
            ":dingtalk_webhook_url": settings.dingtalk_webhook_url,
            ":capture_only_mode": settings.capture_only_mode.map(|v| if v { 1 } else { 0 }),
            ":custom_headers": encrypted_custom_headers,
            ":quality_filter_enabled": settings.quality_filter_enabled.map(|v| if v { 1 } else { 0 }),
            ":quality_filter_threshold": settings.quality_filter_threshold,
            ":session_gap_minutes": settings.session_gap_minutes,
            ":proxy_enabled": settings.proxy_enabled.map(|v| if v { 1 } else { 0 }),
            ":proxy_host": settings.proxy_host,
            ":proxy_port": settings.proxy_port,
            ":proxy_username": settings.proxy_username,
            ":proxy_password": encrypted_proxy_password,
            ":test_model_name": settings.test_model_name,
            ":onboarding_completed": settings.onboarding_completed.map(|v| if v { 1 } else { 0 }),
            ":language": settings.language,
            ":preferred_language": settings.preferred_language,
            ":supported_languages": settings.supported_languages,
            ":auto_backup_enabled": settings.auto_backup_enabled.map(|v| if v { 1 } else { 0 }),
            ":auto_backup_interval": settings.auto_backup_interval,
            ":auto_backup_retention": settings.auto_backup_retention,
            ":last_auto_backup_at": settings.last_auto_backup_at,
        },
    )
    .map_err(AppError::from)?;

    // Update cache with the saved (decrypted) settings
    if let Ok(mut cache) = SETTINGS_CACHE.write() {
        *cache = Some(Arc::new(settings.clone()));
    }

    tracing::info!("Settings saved");
    Ok(())
}

// ── Async wrappers (for use by command layer) ──

/// Async wrapper for getting settings (used by command layer)
pub async fn get_settings() -> AppResult<Arc<Settings>> {
    get_settings_sync()
}

/// Async wrapper for saving settings (used by command layer)
pub async fn save_settings(settings: Settings) -> AppResult<()> {
    save_settings_sync(&settings)
}
