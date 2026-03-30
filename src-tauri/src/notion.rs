//! Notion API integration for exporting reports.
//!
//! This module provides functionality to write reports to Notion databases
//! using the Notion API.

use crate::create_http_client;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use tauri::command;

/// Notion API base URL
const NOTION_API_BASE: &str = "https://api.notion.com/v1";

/// Notion API version
const NOTION_API_VERSION: &str = "2022-06-28";

/// Maximum number of blocks per append request (Notion API limit)
const MAX_BLOCKS_PER_REQUEST: usize = 100;

/// Maximum characters per rich text element (Notion API limit)
const MAX_RICH_TEXT_LENGTH: usize = 2000;

/// Response from Notion API when creating a page
#[derive(Debug, Deserialize)]
struct NotionPageResponse {
    id: String,
    url: Option<String>,
}

/// Response from Notion API when retrieving a database
#[derive(Debug, Deserialize)]
struct NotionDatabaseResponse {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    title: Vec<NotionRichText>,
    properties: serde_json::Value,
}

/// Notion rich text object
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotionRichText {
    #[serde(rename = "type")]
    text_type: String,
    text: NotionTextContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<NotionAnnotations>,
}

/// Notion text content
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotionTextContent {
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    link: Option<serde_json::Value>,
}

/// Notion text annotations for formatting
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotionAnnotations {
    #[serde(default)]
    bold: bool,
    #[serde(default)]
    italic: bool,
    #[serde(default)]
    strikethrough: bool,
    #[serde(default)]
    underline: bool,
    #[serde(default)]
    code: bool,
    #[serde(default)]
    color: String,
}

impl NotionAnnotations {
    fn new() -> Self {
        Self {
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            code: false,
            color: "default".to_string(),
        }
    }
}

/// Notion block object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(flatten)]
    content: serde_json::Map<String, serde_json::Value>,
}

impl NotionBlock {
    /// Create a paragraph block
    pub fn paragraph(rich_text: Vec<NotionRichText>) -> Self {
        let mut content = serde_json::Map::new();
        content.insert(
            "rich_text".to_string(),
            serde_json::to_value(rich_text).unwrap_or(serde_json::Value::Null),
        );
        Self {
            block_type: "paragraph".to_string(),
            content,
        }
    }

    /// Create a heading block
    pub fn heading(level: u8, rich_text: Vec<NotionRichText>) -> Self {
        let block_type = format!("heading_{}", level);
        let mut content = serde_json::Map::new();
        content.insert(
            "rich_text".to_string(),
            serde_json::to_value(rich_text).unwrap_or(serde_json::Value::Null),
        );
        Self {
            block_type,
            content,
        }
    }

    /// Create a bulleted list item
    pub fn bulleted_list_item(rich_text: Vec<NotionRichText>) -> Self {
        let mut content = serde_json::Map::new();
        content.insert(
            "rich_text".to_string(),
            serde_json::to_value(rich_text).unwrap_or(serde_json::Value::Null),
        );
        Self {
            block_type: "bulleted_list_item".to_string(),
            content,
        }
    }

    /// Create a numbered list item
    pub fn numbered_list_item(rich_text: Vec<NotionRichText>) -> Self {
        let mut content = serde_json::Map::new();
        content.insert(
            "rich_text".to_string(),
            serde_json::to_value(rich_text).unwrap_or(serde_json::Value::Null),
        );
        Self {
            block_type: "numbered_list_item".to_string(),
            content,
        }
    }

    /// Create a code block
    pub fn code(rich_text: Vec<NotionRichText>, language: Option<&str>) -> Self {
        let mut content = serde_json::Map::new();
        content.insert(
            "rich_text".to_string(),
            serde_json::to_value(rich_text).unwrap_or(serde_json::Value::Null),
        );
        content.insert(
            "language".to_string(),
            serde_json::to_value(language.unwrap_or("plain text"))
                .unwrap_or(serde_json::Value::Null),
        );
        Self {
            block_type: "code".to_string(),
            content,
        }
    }

    /// Create a quote block
    pub fn quote(rich_text: Vec<NotionRichText>) -> Self {
        let mut content = serde_json::Map::new();
        content.insert(
            "rich_text".to_string(),
            serde_json::to_value(rich_text).unwrap_or(serde_json::Value::Null),
        );
        Self {
            block_type: "quote".to_string(),
            content,
        }
    }

    /// Create a divider block
    pub fn divider() -> Self {
        Self {
            block_type: "divider".to_string(),
            content: serde_json::Map::new(),
        }
    }
}

/// Create a rich text element with optional formatting
fn create_rich_text_element(content: &str, annotations: &NotionAnnotations) -> NotionRichText {
    let truncated = if content.len() > MAX_RICH_TEXT_LENGTH {
        &content[..MAX_RICH_TEXT_LENGTH]
    } else {
        content
    };

    let has_formatting =
        annotations.bold || annotations.italic || annotations.strikethrough || annotations.code;

    NotionRichText {
        text_type: "text".to_string(),
        text: NotionTextContent {
            content: truncated.to_string(),
            link: None,
        },
        annotations: if has_formatting {
            Some(annotations.clone())
        } else {
            None
        },
    }
}

/// Builder for accumulating rich text with formatting state
struct RichTextBuilder {
    segments: Vec<NotionRichText>,
    current_text: String,
    annotations: NotionAnnotations,
}

impl RichTextBuilder {
    fn new() -> Self {
        Self {
            segments: Vec::new(),
            current_text: String::new(),
            annotations: NotionAnnotations::new(),
        }
    }

    fn push_text(&mut self, text: &str) {
        self.current_text.push_str(text);
    }

    fn push_code(&mut self, text: &str) {
        // Flush current text first
        self.flush();
        // Add inline code as separate segment
        let mut code_annotations = NotionAnnotations::new();
        code_annotations.code = true;
        self.segments
            .push(create_rich_text_element(text, &code_annotations));
    }

    fn set_bold(&mut self, bold: bool) {
        self.flush();
        self.annotations.bold = bold;
    }

    fn set_italic(&mut self, italic: bool) {
        self.flush();
        self.annotations.italic = italic;
    }

    fn set_strikethrough(&mut self, strikethrough: bool) {
        self.flush();
        self.annotations.strikethrough = strikethrough;
    }

    fn flush(&mut self) {
        if !self.current_text.is_empty() {
            self.segments.push(create_rich_text_element(
                &self.current_text,
                &self.annotations,
            ));
            self.current_text.clear();
        }
    }

    fn build(&mut self) -> Vec<NotionRichText> {
        self.flush();
        if self.segments.is_empty() {
            vec![NotionRichText::default()]
        } else {
            self.segments.clone()
        }
    }

    fn clear(&mut self) {
        self.segments.clear();
        self.current_text.clear();
        self.annotations = NotionAnnotations::new();
    }

    fn is_empty(&self) -> bool {
        self.segments.is_empty() && self.current_text.is_empty()
    }
}

/// Convert Markdown content to Notion blocks.
///
/// This function parses Markdown using pulldown-cmark and converts it to
/// Notion Block format that can be appended to a page.
pub fn markdown_to_notion_blocks(markdown: &str) -> Vec<NotionBlock> {
    let parser = Parser::new(markdown);
    let mut blocks = Vec::new();
    let mut rich_text_builder = RichTextBuilder::new();
    let mut in_code_block = false;
    let mut code_language: Option<String> = None;
    let mut code_content = String::new();
    let mut in_blockquote = false;
    let mut blockquote_builder = RichTextBuilder::new();
    let mut is_numbered_list = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level: _, .. }) => {
                // Flush any pending paragraph
                if !rich_text_builder.is_empty() {
                    blocks.push(NotionBlock::paragraph(rich_text_builder.build()));
                }
                rich_text_builder.clear();
            }
            Event::End(TagEnd::Heading(level)) => {
                let rich_text = rich_text_builder.build();
                if !(rich_text.is_empty()
                    || rich_text.len() == 1 && rich_text[0].text.content.is_empty())
                {
                    let heading_level = match level {
                        pulldown_cmark::HeadingLevel::H1 => 1,
                        pulldown_cmark::HeadingLevel::H2 => 2,
                        pulldown_cmark::HeadingLevel::H3 => 3,
                        pulldown_cmark::HeadingLevel::H4 => 4,
                        pulldown_cmark::HeadingLevel::H5 => 5,
                        pulldown_cmark::HeadingLevel::H6 => 6,
                    };
                    blocks.push(NotionBlock::heading(heading_level, rich_text));
                }
                rich_text_builder.clear();
            }
            Event::Start(Tag::Paragraph) => {
                rich_text_builder.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                if in_blockquote {
                    // Blockquote handles its own content
                } else if !rich_text_builder.is_empty() {
                    blocks.push(NotionBlock::paragraph(rich_text_builder.build()));
                }
                rich_text_builder.clear();
            }
            Event::Start(Tag::List(start_number)) => {
                is_numbered_list = start_number.is_some();
            }
            Event::End(TagEnd::List(_)) => {
                is_numbered_list = false;
            }
            Event::Start(Tag::Item) => {
                rich_text_builder.clear();
            }
            Event::End(TagEnd::Item) => {
                let rich_text = rich_text_builder.build();
                if !(rich_text.is_empty()
                    || rich_text.len() == 1 && rich_text[0].text.content.is_empty())
                {
                    if is_numbered_list {
                        blocks.push(NotionBlock::numbered_list_item(rich_text));
                    } else {
                        blocks.push(NotionBlock::bulleted_list_item(rich_text));
                    }
                }
                rich_text_builder.clear();
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_language = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                    _ => None,
                };
                code_content.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let text = code_content.trim();
                if !text.is_empty() {
                    let rich_text = vec![create_rich_text_element(text, &NotionAnnotations::new())];
                    blocks.push(NotionBlock::code(rich_text, code_language.as_deref()));
                }
                code_content.clear();
                code_language = None;
            }
            Event::Start(Tag::BlockQuote(_)) => {
                in_blockquote = true;
                blockquote_builder.clear();
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                in_blockquote = false;
                let rich_text = blockquote_builder.build();
                if !(rich_text.is_empty()
                    || rich_text.len() == 1 && rich_text[0].text.content.is_empty())
                {
                    blocks.push(NotionBlock::quote(rich_text));
                }
                blockquote_builder.clear();
            }
            Event::Start(Tag::Strong) => {
                if in_blockquote {
                    blockquote_builder.set_bold(true);
                } else {
                    rich_text_builder.set_bold(true);
                }
            }
            Event::End(TagEnd::Strong) => {
                if in_blockquote {
                    blockquote_builder.set_bold(false);
                } else {
                    rich_text_builder.set_bold(false);
                }
            }
            Event::Start(Tag::Emphasis) => {
                if in_blockquote {
                    blockquote_builder.set_italic(true);
                } else {
                    rich_text_builder.set_italic(true);
                }
            }
            Event::End(TagEnd::Emphasis) => {
                if in_blockquote {
                    blockquote_builder.set_italic(false);
                } else {
                    rich_text_builder.set_italic(false);
                }
            }
            Event::Start(Tag::Strikethrough) => {
                if in_blockquote {
                    blockquote_builder.set_strikethrough(true);
                } else {
                    rich_text_builder.set_strikethrough(true);
                }
            }
            Event::End(TagEnd::Strikethrough) => {
                if in_blockquote {
                    blockquote_builder.set_strikethrough(false);
                } else {
                    rich_text_builder.set_strikethrough(false);
                }
            }
            Event::Text(text) => {
                if in_code_block {
                    code_content.push_str(&text);
                } else if in_blockquote {
                    blockquote_builder.push_text(&text);
                } else {
                    rich_text_builder.push_text(&text);
                }
            }
            Event::Code(text) => {
                // Inline code
                if in_blockquote {
                    blockquote_builder.push_code(&text);
                } else {
                    rich_text_builder.push_code(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_code_block {
                    code_content.push('\n');
                } else if in_blockquote {
                    blockquote_builder.push_text(" ");
                } else {
                    rich_text_builder.push_text(" ");
                }
            }
            Event::Rule => {
                blocks.push(NotionBlock::divider());
            }
            _ => {}
        }
    }

    // Flush any remaining content
    if !rich_text_builder.is_empty() {
        blocks.push(NotionBlock::paragraph(rich_text_builder.build()));
    }

    blocks
}

/// Split blocks into chunks respecting Notion's API limit
fn chunk_blocks(blocks: Vec<NotionBlock>) -> Vec<Vec<NotionBlock>> {
    blocks
        .chunks(MAX_BLOCKS_PER_REQUEST)
        .map(|chunk| chunk.to_vec())
        .collect()
}

/// Get the title property name from a Notion database
async fn get_title_property_name(api_key: &str, database_id: &str) -> Result<String, String> {
    let url = format!("{}/databases/{}", NOTION_API_BASE, database_id);
    let client =
        create_http_client(&url, 30).map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Notion-Version", NOTION_API_VERSION)
        .send()
        .await
        .map_err(|e| format!("Failed to get database: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Notion API error: {} - {}", status, error_text));
    }

    let db: NotionDatabaseResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse database response: {}", e))?;

    // Find the title property in the database schema
    if let serde_json::Value::Object(props) = db.properties {
        for (name, value) in props {
            if let serde_json::Value::Object(prop) = value {
                if prop.get("type") == Some(&serde_json::Value::String("title".to_string())) {
                    return Ok(name);
                }
            }
        }
    }

    // Default to "Name" if not found
    Ok("Name".to_string())
}

/// Check if Notion is configured in settings
pub fn is_notion_configured(settings: &crate::memory_storage::Settings) -> bool {
    settings
        .notion_api_key
        .as_ref()
        .is_some_and(|k| !k.is_empty())
        && settings
            .notion_database_id
            .as_ref()
            .is_some_and(|id| !id.is_empty())
}

/// Write a report to Notion as a new page in the configured database.
///
/// Returns the URL of the created page on success, or None if Notion is not configured
/// or the write fails.
pub async fn write_report_to_notion(
    settings: &crate::memory_storage::Settings,
    title: &str,
    content: &str,
) -> Option<String> {
    let api_key = settings.notion_api_key.as_ref()?;
    let database_id = settings.notion_database_id.as_ref()?;

    if api_key.is_empty() || database_id.is_empty() {
        return None;
    }

    // Get the correct title property name
    let title_property = get_title_property_name(api_key, database_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to get title property name, using default: {}", e);
            "Name".to_string()
        });

    let url = format!("{}/pages", NOTION_API_BASE);
    let client = create_http_client(&url, 30).ok()?;

    // Create page with title
    let body = serde_json::json!({
        "parent": {
            "database_id": database_id
        },
        "properties": {
            title_property: {
                "title": [
                    {
                        "text": {
                            "content": title
                        }
                    }
                ]
            }
        }
    });

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Notion-Version", NOTION_API_VERSION)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;

    let page = match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<NotionPageResponse>().await {
                    Ok(page) => page,
                    Err(e) => {
                        tracing::warn!("Failed to parse Notion response: {}", e);
                        return None;
                    }
                }
            } else {
                let status = resp.status();
                let error_text = resp.text().await.unwrap_or_default();
                tracing::warn!("Failed to create Notion page: {} - {}", status, error_text);
                return None;
            }
        }
        Err(e) => {
            tracing::warn!("Failed to call Notion API: {}", e);
            return None;
        }
    };

    // Convert content to Notion blocks and append to the page
    if !content.is_empty() {
        let blocks = markdown_to_notion_blocks(content);
        let chunks = chunk_blocks(blocks);

        let blocks_url = format!("{}/blocks/{}/children", NOTION_API_BASE, page.id);

        for chunk in chunks {
            let blocks_body = serde_json::json!({
                "children": chunk
            });

            let blocks_response = client
                .patch(&blocks_url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Notion-Version", NOTION_API_VERSION)
                .header("Content-Type", "application/json")
                .json(&blocks_body)
                .send()
                .await;

            if let Err(e) = blocks_response {
                tracing::warn!("Failed to append blocks to Notion page: {}", e);
                // Continue trying other chunks
            }
        }
    }

    let page_url = page
        .url
        .unwrap_or_else(|| format!("https://notion.so/{}", page.id.replace("-", "")));
    tracing::info!("Report written to Notion: {}", page_url);
    Some(page_url)
}

/// Test Notion API connection
/// Returns Ok(true) if connection is successful, Ok(false) if Notion is not configured,
/// or Err with error message if connection fails.
#[command]
pub async fn test_notion_connection() -> Result<bool, String> {
    let settings = crate::memory_storage::get_settings_sync()?;

    let api_key = match settings.notion_api_key {
        Some(ref key) if !key.is_empty() => key,
        _ => return Ok(false), // Not configured
    };

    let database_id = match settings.notion_database_id {
        Some(ref id) if !id.is_empty() => id,
        _ => return Ok(false), // Not configured
    };

    let url = format!("{}/databases/{}", NOTION_API_BASE, database_id);
    let client =
        create_http_client(&url, 30).map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Try to retrieve the database to verify access
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Notion-Version", NOTION_API_VERSION)
        .send()
        .await
        .map_err(|e| format!("Connection error: {}", e))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Notion API error: {} - {}", status, error_text))
    }
}

/// Write a report to Notion synchronously (wrapper for async function)
/// This is used in report generation functions that need to write to Notion
pub fn write_report_to_notion_sync(
    settings: &crate::memory_storage::Settings,
    title: &str,
    content: &str,
) -> Option<String> {
    // For synchronous contexts, we use tokio::runtime
    // Since this is called from async functions in synthesis, we should use the async version
    // This sync version is provided for potential future use
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(write_report_to_notion(settings, title, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_notion_configured_returns_false_when_no_api_key() {
        let settings = crate::memory_storage::Settings {
            notion_api_key: None,
            notion_database_id: Some("test-db-id".to_string()),
            ..Default::default()
        };
        assert!(!is_notion_configured(&settings));
    }

    #[test]
    fn is_notion_configured_returns_false_when_no_database_id() {
        let settings = crate::memory_storage::Settings {
            notion_api_key: Some("test-api-key".to_string()),
            notion_database_id: None,
            ..Default::default()
        };
        assert!(!is_notion_configured(&settings));
    }

    #[test]
    fn is_notion_configured_returns_false_when_empty() {
        let settings = crate::memory_storage::Settings {
            notion_api_key: Some("".to_string()),
            notion_database_id: Some("".to_string()),
            ..Default::default()
        };
        assert!(!is_notion_configured(&settings));
    }

    #[test]
    fn is_notion_configured_returns_true_when_configured() {
        let settings = crate::memory_storage::Settings {
            notion_api_key: Some("secret-key".to_string()),
            notion_database_id: Some("db-id".to_string()),
            ..Default::default()
        };
        assert!(is_notion_configured(&settings));
    }

    // Markdown to Notion block conversion tests

    #[test]
    fn converts_heading_1() {
        let markdown = "# Main Title";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "heading_1");
    }

    #[test]
    fn converts_heading_2() {
        let markdown = "## Section Title";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "heading_2");
    }

    #[test]
    fn converts_heading_3() {
        let markdown = "### Subsection Title";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "heading_3");
    }

    #[test]
    fn converts_paragraph() {
        let markdown = "This is a simple paragraph.";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "paragraph");
    }

    #[test]
    fn converts_bulleted_list() {
        let markdown = "- Item 1\n- Item 2\n- Item 3";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 3);
        for block in &blocks {
            assert_eq!(block.block_type, "bulleted_list_item");
        }
    }

    #[test]
    fn converts_numbered_list() {
        let markdown = "1. First item\n2. Second item\n3. Third item";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 3);
        for block in &blocks {
            assert_eq!(block.block_type, "numbered_list_item");
        }
    }

    #[test]
    fn converts_code_block() {
        let markdown = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "code");
    }

    #[test]
    fn converts_blockquote() {
        let markdown = "> This is a quote";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "quote");
    }

    #[test]
    fn converts_inline_code() {
        let markdown = "This has `inline code` in it.";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "paragraph");
        // Check that rich_text contains code annotation
        let content = &blocks[0].content;
        let rich_text = content.get("rich_text").unwrap();
        assert!(rich_text.to_string().contains("code"));
    }

    #[test]
    fn converts_bold_text() {
        let markdown = "This is **bold** text.";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "paragraph");
        // Check that rich_text contains bold annotation
        let content = &blocks[0].content;
        let rich_text = content.get("rich_text").unwrap();
        let rich_text_str = rich_text.to_string();
        assert!(rich_text_str.contains("bold") && rich_text_str.contains("true"));
    }

    #[test]
    fn converts_italic_text() {
        let markdown = "This is *italic* text.";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "paragraph");
        // Check that rich_text contains italic annotation
        let content = &blocks[0].content;
        let rich_text = content.get("rich_text").unwrap();
        let rich_text_str = rich_text.to_string();
        assert!(rich_text_str.contains("italic") && rich_text_str.contains("true"));
    }

    #[test]
    fn converts_bold_and_italic() {
        let markdown = "This is **bold** and *italic* text.";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "paragraph");
    }

    #[test]
    fn converts_multiple_elements() {
        let markdown = r#"# Title

This is a paragraph.

## Section

- List item 1
- List item 2

> A quote

```javascript
console.log("Hello");
```
"#;
        let blocks = markdown_to_notion_blocks(markdown);
        assert!(blocks.len() >= 5); // At least 5 blocks
        assert!(blocks.iter().any(|b| b.block_type == "heading_1"));
        assert!(blocks.iter().any(|b| b.block_type == "heading_2"));
        assert!(blocks.iter().any(|b| b.block_type == "paragraph"));
        assert!(blocks.iter().any(|b| b.block_type == "bulleted_list_item"));
        assert!(blocks.iter().any(|b| b.block_type == "quote"));
        assert!(blocks.iter().any(|b| b.block_type == "code"));
    }

    #[test]
    fn handles_empty_input() {
        let blocks = markdown_to_notion_blocks("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn handles_plain_text_only() {
        let markdown = "Just some plain text without any formatting.";
        let blocks = markdown_to_notion_blocks(markdown);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "paragraph");
    }

    #[test]
    fn chunk_blocks_respects_limit() {
        let mut blocks = Vec::new();
        for i in 0..250 {
            blocks.push(NotionBlock::paragraph(vec![NotionRichText {
                text_type: "text".to_string(),
                text: NotionTextContent {
                    content: format!("Block {}", i),
                    link: None,
                },
                annotations: None,
            }]));
        }

        let chunks = chunk_blocks(blocks);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= MAX_BLOCKS_PER_REQUEST);
        }
    }

    #[test]
    fn create_plain_text_truncates_long_content() {
        let long_text = "a".repeat(3000);
        let annotations = NotionAnnotations::new();
        let rich_text = create_rich_text_element(&long_text, &annotations);
        assert!(rich_text.text.content.len() <= MAX_RICH_TEXT_LENGTH);
    }
}
