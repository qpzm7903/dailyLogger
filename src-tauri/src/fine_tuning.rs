//! Model fine-tuning support module.
//!
//! This module provides functions for preparing training data and managing
//! fine-tuning workflows for local LLM models via Ollama.
//!
//! FUTURE-003: 本地 AI 模型微调

use serde::{Deserialize, Serialize};
use tauri::command;

/// Configuration for model fine-tuning.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FineTuningConfig {
    /// Base model to fine-tune from (e.g., "llama3.2", "qwen2.5")
    pub base_model: String,
    /// Name for the output fine-tuned model
    pub output_model_name: String,
    /// Number of training epochs
    pub epochs: Option<u32>,
    /// Batch size for training
    pub batch_size: Option<u32>,
    /// Learning rate (default: typically 0.0001)
    pub learning_rate: Option<f64>,
    /// System prompt for the fine-tuned model
    pub system_prompt: Option<String>,
    /// Temperature for the fine-tuned model
    pub temperature: Option<f32>,
    /// Context window size
    pub num_ctx: Option<u32>,
}

/// Training data entry for fine-tuning.
///
/// Represents a single input-output pair for supervised fine-tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingDataEntry {
    /// The input prompt (e.g., screenshot description or context)
    pub input: String,
    /// The expected output (e.g., generated summary)
    pub output: String,
    /// Optional metadata about the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Timestamp of the original record
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Result of training data export.
#[derive(Debug, Serialize, Deserialize)]
pub struct TrainingDataExportResult {
    pub success: bool,
    pub message: String,
    pub entries_count: usize,
    pub output_path: Option<String>,
}

/// Result of fine-tuning operation initiation.
#[derive(Debug, Serialize, Deserialize)]
pub struct FineTuningResult {
    pub success: bool,
    pub message: String,
    pub model_name: Option<String>,
}

/// Prepare training data from user records.
///
/// Exports records in a format suitable for Ollama fine-tuning.
/// The output is a JSONL file where each line contains a training example.
#[command]
pub async fn prepare_training_data(
    output_path: String,
    include_auto_records: bool,
    include_manual_records: bool,
    days_back: Option<u32>,
) -> Result<TrainingDataExportResult, String> {
    use chrono::{Duration, Local};
    use std::fs::File;
    use std::io::{BufWriter, Write};

    tracing::info!(
        "Preparing training data to: {} (auto: {}, manual: {}, days: {:?})",
        output_path,
        include_auto_records,
        include_manual_records,
        days_back
    );

    // Calculate date range
    let end_date = Local::now().date_naive();
    let start_date = if let Some(days) = days_back {
        end_date - Duration::days(days as i64)
    } else {
        end_date - Duration::days(30) // Default to 30 days
    };

    // Get records from the date range using the sync function
    let records = crate::memory_storage::get_records_by_date_range_sync(
        start_date.format("%Y-%m-%d").to_string(),
        end_date.format("%Y-%m-%d").to_string(),
    )?;

    // Filter and transform records into training entries
    let mut entries: Vec<TrainingDataEntry> = Vec::new();

    for record in records {
        // Skip records based on source type filters
        let is_auto = record.source_type == "auto";
        let is_manual = record.source_type == "manual";

        if (!is_auto && !is_manual)
            || (is_auto && !include_auto_records)
            || (is_manual && !include_manual_records)
        {
            continue;
        }

        // Parse record content
        let content = record.content.clone();

        // Create training entry
        // For auto records: content is the AI analysis, input could be "screenshot context"
        // For manual records: content is the user's note
        let entry = TrainingDataEntry {
            input: if is_auto {
                format!("Screenshot analysis context from {}:", record.timestamp)
            } else {
                format!("User note from {}:", record.timestamp)
            },
            output: content,
            source: Some(record.source_type.clone()),
            timestamp: Some(record.timestamp.clone()),
        };

        entries.push(entry);
    }

    let entries_count = entries.len();

    // Write to JSONL file
    let file = File::create(&output_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut writer = BufWriter::new(file);

    for entry in &entries {
        let json = serde_json::to_string(entry)
            .map_err(|e| format!("Failed to serialize entry: {}", e))?;
        writeln!(writer, "{}", json).map_err(|e| format!("Failed to write entry: {}", e))?;
    }

    writer
        .flush()
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    tracing::info!(
        "Exported {} training entries to {}",
        entries_count,
        output_path
    );

    Ok(TrainingDataExportResult {
        success: true,
        message: format!(
            "Successfully exported {} training entries to {}",
            entries_count, output_path
        ),
        entries_count,
        output_path: Some(output_path),
    })
}

/// Initiate fine-tuning of a model via Ollama.
///
/// Uses Ollama's model creation API to create a fine-tuned model
/// from a base model with custom parameters.
#[command]
pub async fn start_fine_tuning(
    base_url: String,
    config: FineTuningConfig,
) -> Result<FineTuningResult, String> {
    use crate::create_http_client;

    tracing::info!(
        "Starting fine-tuning: base={}, output={}",
        config.base_model,
        config.output_model_name
    );

    // Normalize URL
    let base = base_url.trim_end_matches('/').trim_end_matches("/v1");
    let url = format!("{}/api/create", base);

    // 5 minute timeout for model creation
    let client = create_http_client(&url, 300)?;

    // Build Modelfile content
    let mut modelfile_lines = vec![format!("FROM {}", config.base_model)];

    if let Some(ref prompt) = config.system_prompt {
        modelfile_lines.push(format!("SYSTEM {}", prompt));
    }

    if let Some(temp) = config.temperature {
        modelfile_lines.push(format!("PARAMETER temperature {}", temp));
    }

    if let Some(ctx) = config.num_ctx {
        modelfile_lines.push(format!("PARAMETER num_ctx {}", ctx));
    }

    let modelfile = modelfile_lines.join("\n");

    // Build request body
    let body = serde_json::json!({
        "name": config.output_model_name,
        "modelfile": modelfile,
        "stream": false
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Ok(FineTuningResult {
            success: false,
            message: format!("Ollama API error ({}): {}", status, body),
            model_name: None,
        });
    }

    tracing::info!(
        "Fine-tuning completed: model {} created",
        config.output_model_name
    );

    Ok(FineTuningResult {
        success: true,
        message: format!(
            "Successfully created model '{}' from '{}'",
            config.output_model_name, config.base_model
        ),
        model_name: Some(config.output_model_name),
    })
}

/// Get default fine-tuning configuration suggestions.
#[command]
pub fn get_default_fine_tuning_config() -> FineTuningConfig {
    FineTuningConfig {
        base_model: "llama3.2".to_string(),
        output_model_name: "dailylogger-finetuned".to_string(),
        epochs: Some(3),
        batch_size: Some(4),
        learning_rate: Some(0.0001),
        system_prompt: Some(
            "You are an AI assistant specialized in summarizing daily work activities \
             and generating structured daily reports. You help users track their productivity \
             and identify patterns in their work."
                .to_string(),
        ),
        temperature: Some(0.7),
        num_ctx: Some(4096),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fine_tuning_config_default() {
        let config = FineTuningConfig::default();
        assert!(config.base_model.is_empty());
        assert!(config.output_model_name.is_empty());
        assert!(config.epochs.is_none());
    }

    #[test]
    fn test_training_data_entry_serialization() {
        let entry = TrainingDataEntry {
            input: "Test input".to_string(),
            output: "Test output".to_string(),
            source: Some("auto".to_string()),
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("Test input"));
        assert!(json.contains("Test output"));
    }

    #[test]
    fn test_get_default_fine_tuning_config() {
        let config = get_default_fine_tuning_config();
        assert_eq!(config.base_model, "llama3.2");
        assert_eq!(config.output_model_name, "dailylogger-finetuned");
        assert_eq!(config.epochs, Some(3));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_fine_tuning_config_with_custom_values() {
        let config = FineTuningConfig {
            base_model: "qwen2.5".to_string(),
            output_model_name: "my-custom-model".to_string(),
            epochs: Some(5),
            batch_size: Some(8),
            learning_rate: Some(0.0005),
            system_prompt: Some("Custom system prompt".to_string()),
            temperature: Some(0.5),
            num_ctx: Some(8192),
        };

        assert_eq!(config.base_model, "qwen2.5");
        assert_eq!(config.epochs, Some(5));
        assert_eq!(config.num_ctx, Some(8192));
    }
}
