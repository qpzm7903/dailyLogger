mod records;
mod schema;
mod settings;
pub mod tags;

use once_cell::sync::Lazy;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::command;

pub use schema::init_database;
// Re-export all public items from settings module (including Tauri command generated types)
pub use settings::*;
// Re-export all public items from records module
pub use records::*;
// Re-export all public items from tags module (including Tauri command generated types)
pub use tags::*;

#[cfg(test)]
pub use schema::init_test_database;

pub static DB_CONNECTION: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub model_name: Option<String>,
    pub screenshot_interval: Option<i32>,
    pub summary_time: Option<String>,
    pub obsidian_path: Option<String>,
    pub auto_capture_enabled: Option<bool>,
    pub last_summary_path: Option<String>,
    pub summary_model_name: Option<String>,
    pub analysis_prompt: Option<String>,
    pub summary_prompt: Option<String>,
    pub change_threshold: Option<i32>,
    pub max_silent_minutes: Option<i32>,
    // 新增字段：日报标题格式
    pub summary_title_format: Option<String>,
    // 新增字段：是否包含手动记录
    pub include_manual_records: Option<bool>,
    // SMART-001: 窗口过滤配置
    pub window_whitelist: Option<String>,
    pub window_blacklist: Option<String>,
    pub use_whitelist_only: Option<bool>,
    // SMART-002: 自动调整静默阈值配置
    pub auto_adjust_silent: Option<bool>,
    pub silent_adjustment_paused_until: Option<String>,
    // SMART-003: 工作时间自动识别配置
    pub auto_detect_work_time: Option<bool>,
    pub use_custom_work_time: Option<bool>,
    pub custom_work_time_start: Option<String>, // "HH:MM" format
    pub custom_work_time_end: Option<String>,
    pub learned_work_time: Option<String>, // JSON: {"periods": [{"start": 9, "end": 12}, ...]}
    // SMART-004: 多显示器支持配置
    pub capture_mode: Option<String>, // "primary" | "secondary" | "all"
    pub selected_monitor_index: Option<i32>, // For "secondary" mode
    // AI-004: 工作分类标签配置
    pub tag_categories: Option<String>, // JSON: Vec<String> of custom tag categories
    // AI-005: Ollama 本地模型支持
    pub is_ollama: Option<bool>,
    // REPORT-001: 周报生成配置
    pub weekly_report_prompt: Option<String>,
    pub weekly_report_day: Option<i32>, // 0=周一, 6=周日
    pub last_weekly_report_path: Option<String>,
    // REPORT-002: 月报生成配置
    pub monthly_report_prompt: Option<String>,
    pub last_monthly_report_path: Option<String>,
    // REPORT-003: 自定义报告周期配置
    pub custom_report_prompt: Option<String>,
    pub last_custom_report_path: Option<String>,
    // DATA-006: 多 Obsidian Vault 支持
    pub obsidian_vaults: Option<String>, // JSON: [{"name":"x","path":"y","is_default":true}]
    // REPORT-004: 对比报告配置
    pub comparison_report_prompt: Option<String>,
    // INT-002: Logseq 导出支持
    pub logseq_graphs: Option<String>, // JSON: [{"name":"x","path":"y","is_default":true}]
    // INT-001: Notion 导出支持
    pub notion_api_key: Option<String>, // Notion integration secret (encrypted)
    pub notion_database_id: Option<String>, // Notion database ID to write pages to
    // INT-004: Slack 通知配置
    pub slack_webhook_url: Option<String>, // Slack Incoming Webhook URL
    // INT-004: DingTalk 通知配置
    pub dingtalk_webhook_url: Option<String>, // DingTalk Robot Webhook URL
    // FEAT-006: 仅截图模式 (#65)
    pub capture_only_mode: Option<bool>, // Only capture screenshots without AI analysis
    // AI-006: 自定义 API Headers (#68)
    pub custom_headers: Option<String>, // JSON: Vec<CustomHeader>
    // EXP-002: 截图质量过滤
    pub quality_filter_enabled: Option<bool>,
    pub quality_filter_threshold: Option<f64>,
    // SESSION-001: 工作时段管理
    pub session_gap_minutes: Option<i32>, // 时段间隔阈值（分钟），默认 30
}

/// AI-006: Custom API Header for various API providers (OpenRouter, Azure, Claude, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomHeader {
    pub key: String,
    pub value: String,
    pub sensitive: bool, // Whether the value should be encrypted
}

/// AI-006: Preset header templates for common API providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderPreset {
    pub name: String,
    pub headers: Vec<CustomHeader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// AI-006: Get preset header templates for common API providers
pub fn get_header_presets() -> Vec<HeaderPreset> {
    vec![
        HeaderPreset {
            name: "OpenRouter".to_string(),
            headers: vec![
                CustomHeader {
                    key: "HTTP-Referer".to_string(),
                    value: "https://dailylogger.app".to_string(),
                    sensitive: false,
                },
                CustomHeader {
                    key: "X-Title".to_string(),
                    value: "DailyLogger".to_string(),
                    sensitive: false,
                },
            ],
            note: None,
        },
        HeaderPreset {
            name: "Azure OpenAI".to_string(),
            headers: vec![CustomHeader {
                key: "api-key".to_string(),
                value: String::new(),
                sensitive: true,
            }],
            note: Some("api-key header replaces Authorization header".to_string()),
        },
        HeaderPreset {
            name: "Claude API".to_string(),
            headers: vec![CustomHeader {
                key: "anthropic-version".to_string(),
                value: "2023-06-01".to_string(),
                sensitive: false,
            }],
            note: None,
        },
    ]
}

/// DATA-006: Vault entry for multi-vault support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianVault {
    pub name: String,
    pub path: String,
    pub is_default: bool,
}

/// INT-002: Logseq graph entry for multi-graph support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogseqGraph {
    pub name: String,
    pub path: String,
    pub is_default: bool,
}

impl Settings {
    /// Get the effective Obsidian output path.
    /// Checks `obsidian_vaults` for the default vault first, falls back to `obsidian_path`.
    pub fn get_obsidian_output_path(&self) -> Result<String, String> {
        // Try obsidian_vaults first
        if let Some(ref vaults_json) = self.obsidian_vaults {
            if let Ok(vaults) = serde_json::from_str::<Vec<ObsidianVault>>(vaults_json) {
                if let Some(default_vault) = vaults.iter().find(|v| v.is_default) {
                    if !default_vault.path.trim().is_empty() {
                        return Ok(default_vault.path.clone());
                    }
                }
                // If no default, use the first vault
                if let Some(first_vault) = vaults.first() {
                    if !first_vault.path.trim().is_empty() {
                        return Ok(first_vault.path.clone());
                    }
                }
            }
        }

        // Fall back to legacy obsidian_path
        self.obsidian_path
            .clone()
            .filter(|p| !p.trim().is_empty())
            .ok_or_else(|| "Obsidian path not configured".to_string())
    }

    /// INT-002: Get the effective Logseq output path.
    /// Checks `logseq_graphs` for the default graph first.
    pub fn get_logseq_output_path(&self) -> Result<String, String> {
        // Try logseq_graphs
        if let Some(ref graphs_json) = self.logseq_graphs {
            if let Ok(graphs) = serde_json::from_str::<Vec<LogseqGraph>>(graphs_json) {
                if let Some(default_graph) = graphs.iter().find(|g| g.is_default) {
                    if !default_graph.path.trim().is_empty() {
                        return Ok(default_graph.path.clone());
                    }
                }
                // If no default, use the first graph
                if let Some(first_graph) = graphs.first() {
                    if !first_graph.path.trim().is_empty() {
                        return Ok(first_graph.path.clone());
                    }
                }
            }
        }

        Err("Logseq path not configured".to_string())
    }
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub context_window: Option<u64>,
    pub error: Option<String>,
}

/// Get model information including context window
#[command]
pub async fn get_model_info(
    api_base_url: String,
    api_key: String,
    model_name: String,
) -> Result<ModelInfo, String> {
    // OpenAI compatible API /models endpoint
    let url = if api_base_url.ends_with('/') {
        format!("{}models/{}", api_base_url, model_name)
    } else {
        format!("{}/models/{}", api_base_url, model_name)
    };

    // Create HTTP client with proxy bypass for local URLs
    let client = crate::create_http_client(&url, 30)?;

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let json: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({}));

            // OpenAI returns format: {"id": "gpt-4o", "context_window": 128000}
            // Or in some APIs it's max_tokens
            let context_window = json
                .get("context_window")
                .or_else(|| json.get("max_tokens"))
                .or_else(|| {
                    // Some APIs return it in model_info
                    json.get("model_info")
                        .and_then(|mi| mi.get("context_window"))
                })
                .and_then(|v| v.as_u64());

            Ok(ModelInfo {
                model_id: model_name,
                context_window,
                error: None,
            })
        }
        Ok(resp) => {
            let status = resp.status();
            Ok(ModelInfo {
                model_id: model_name,
                context_window: None,
                error: Some(format!("无法获取模型信息 (状态: {})", status)),
            })
        }
        Err(e) => Ok(ModelInfo {
            model_id: model_name,
            context_window: None,
            error: Some(format!("请求失败: {}", e)),
        }),
    }
}

#[cfg(test)]
mod tests_ai_006 {
    use super::*;

    #[test]
    fn test_custom_header_serialization() {
        let header = CustomHeader {
            key: "X-Custom-Header".to_string(),
            value: "test-value".to_string(),
            sensitive: false,
        };
        let json = serde_json::to_string(&header).unwrap();
        assert!(json.contains("X-Custom-Header"));
        assert!(json.contains("test-value"));
        assert!(json.contains("\"sensitive\":false"));
    }

    #[test]
    fn test_custom_header_deserialization() {
        let json = r#"{"key":"Authorization","value":"Bearer token","sensitive":true}"#;
        let header: CustomHeader = serde_json::from_str(json).unwrap();
        assert_eq!(header.key, "Authorization");
        assert_eq!(header.value, "Bearer token");
        assert!(header.sensitive);
    }

    #[test]
    fn test_custom_headers_vec_serialization() {
        let headers = vec![
            CustomHeader {
                key: "HTTP-Referer".to_string(),
                value: "https://dailylogger.app".to_string(),
                sensitive: false,
            },
            CustomHeader {
                key: "api-key".to_string(),
                value: "secret-key".to_string(),
                sensitive: true,
            },
        ];
        let json = serde_json::to_string(&headers).unwrap();
        assert!(json.contains("HTTP-Referer"));
        assert!(json.contains("api-key"));
        assert!(json.contains("secret-key"));
    }

    #[test]
    fn test_custom_headers_vec_deserialization() {
        let json = r#"[{"key":"X-Title","value":"DailyLogger","sensitive":false}]"#;
        let headers: Vec<CustomHeader> = serde_json::from_str(json).unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].key, "X-Title");
        assert_eq!(headers[0].value, "DailyLogger");
        assert!(!headers[0].sensitive);
    }

    #[test]
    fn test_header_presets() {
        let presets = get_header_presets();
        assert!(!presets.is_empty());

        // Check OpenRouter preset
        let openrouter = presets.iter().find(|p| p.name == "OpenRouter");
        assert!(openrouter.is_some());
        let openrouter = openrouter.unwrap();
        assert_eq!(openrouter.headers.len(), 2);

        // Check Azure OpenAI preset
        let azure = presets.iter().find(|p| p.name == "Azure OpenAI");
        assert!(azure.is_some());
        let azure = azure.unwrap();
        assert_eq!(azure.headers.len(), 1);
        assert!(azure.headers[0].sensitive);

        // Check Claude API preset
        let claude = presets.iter().find(|p| p.name == "Claude API");
        assert!(claude.is_some());
    }

    #[test]
    fn test_settings_default_custom_headers() {
        let settings = Settings::default();
        assert!(
            settings.custom_headers.is_none() || settings.custom_headers == Some("[]".to_string())
        );
    }
}
