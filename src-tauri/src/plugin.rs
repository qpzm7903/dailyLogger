//! Plugin system for DailyLogger.
//!
//! This module provides a plugin architecture that allows users to extend
//! the application with custom functionality without modifying core code.
//!
//! # Plugin Lifecycle
//!
//! 1. Plugin is discovered in the plugins directory
//! 2. Plugin is loaded and validated
//! 3. Plugin's `on_load` hook is called
//! 4. Plugin can register hooks and extend functionality
//! 5. Plugin's `on_unload` hook is called when the app shuts down
//!
//! # Plugin Discovery
//!
//! Plugins are discovered by scanning the plugins directory for subdirectories
//! containing a `plugin.json` manifest file. The manifest contains metadata
//! about the plugin and its capabilities.
//!
//! # Example Plugin Manifest (plugin.json)
//!
//! ```json
//! {
//!   "id": "com.example.my-plugin",
//!   "name": "My Plugin",
//!   "version": "1.0.0",
//!   "description": "A custom plugin",
//!   "author": "Author Name",
//!   "min_app_version": "1.18.0",
//!   "enabled": true
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Metadata describing a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin (e.g., "com.example.my-plugin")
    #[serde(default)]
    pub id: String,
    /// Human-readable name of the plugin
    #[serde(default)]
    pub name: String,
    /// Version string (semver recommended)
    #[serde(default = "default_version")]
    pub version: String,
    /// Brief description of what the plugin does
    #[serde(default)]
    pub description: String,
    /// Author or organization name
    #[serde(default)]
    pub author: String,
    /// Minimum DailyLogger version required (semver)
    #[serde(default)]
    pub min_app_version: Option<String>,
    /// Plugin homepage URL
    #[serde(default)]
    pub homepage: Option<String>,
    /// Plugin license
    #[serde(default)]
    pub license: Option<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Plugin manifest loaded from plugin.json file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata
    #[serde(flatten)]
    pub metadata: PluginMetadata,
    /// Whether the plugin is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Plugin entry point (for future scripting support)
    #[serde(default)]
    pub entry_point: Option<String>,
    /// List of plugin dependencies (IDs)
    #[serde(default)]
    pub dependencies: Vec<String>,
}

fn default_enabled() -> bool {
    true
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            metadata: PluginMetadata {
                id: String::new(),
                name: String::new(),
                version: "1.0.0".to_string(),
                description: String::new(),
                author: String::new(),
                min_app_version: None,
                homepage: None,
                license: None,
            },
            enabled: true,
            entry_point: None,
            dependencies: Vec::new(),
        }
    }
}

/// Status of a discovered plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    /// Plugin is ready to be loaded
    Ready,
    /// Plugin is disabled in manifest
    Disabled,
    /// Plugin has an error (e.g., invalid manifest)
    Error(String),
}

/// A discovered plugin with its manifest and status.
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    /// Path to the plugin directory
    pub path: PathBuf,
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Plugin status
    pub status: PluginStatus,
}

use std::path::PathBuf;

/// Discovers plugins in a directory.
///
/// Scans the given directory for subdirectories containing a `plugin.json` manifest.
/// Returns a list of discovered plugins with their manifests and status.
pub fn discover_plugins(plugins_dir: &Path) -> io::Result<Vec<DiscoveredPlugin>> {
    let mut plugins = Vec::new();

    if !plugins_dir.exists() {
        return Ok(plugins);
    }

    for entry in fs::read_dir(plugins_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("plugin.json");
        if !manifest_path.exists() {
            continue;
        }

        let discovered = match load_manifest(&manifest_path) {
            Ok(manifest) => {
                let status = if manifest.enabled {
                    PluginStatus::Ready
                } else {
                    PluginStatus::Disabled
                };
                DiscoveredPlugin {
                    path,
                    manifest,
                    status,
                }
            }
            Err(e) => DiscoveredPlugin {
                path,
                manifest: PluginManifest::default(),
                status: PluginStatus::Error(e),
            },
        };

        plugins.push(discovered);
    }

    Ok(plugins)
}

/// Loads a plugin manifest from a file.
pub fn load_manifest(path: &Path) -> Result<PluginManifest, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read manifest: {}", e))?;

    let manifest: PluginManifest =
        serde_json::from_str(&content).map_err(|e| format!("Invalid manifest JSON: {}", e))?;

    // Validate required fields
    if manifest.metadata.id.is_empty() {
        return Err("Plugin ID is required".to_string());
    }
    if manifest.metadata.name.is_empty() {
        return Err("Plugin name is required".to_string());
    }

    Ok(manifest)
}

/// Saves a plugin manifest to a file.
pub fn save_manifest(manifest: &PluginManifest, path: &Path) -> io::Result<()> {
    let content = serde_json::to_string_pretty(manifest)?;
    fs::write(path, content)
}

/// Returns the default plugins directory path.
pub fn get_plugins_directory() -> PathBuf {
    crate::get_app_data_dir().join("plugins")
}

/// Creates the plugins directory if it doesn't exist.
pub fn ensure_plugins_directory() -> io::Result<PathBuf> {
    let plugins_dir = get_plugins_directory();
    if !plugins_dir.exists() {
        fs::create_dir_all(&plugins_dir)?;
    }
    Ok(plugins_dir)
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Plugin info for frontend display.
#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    /// Plugin ID
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Whether the plugin is enabled
    pub enabled: bool,
    /// Plugin status
    pub status: String,
    /// Plugin path
    pub path: String,
}

impl From<DiscoveredPlugin> for PluginInfo {
    fn from(discovered: DiscoveredPlugin) -> Self {
        let status = match discovered.status {
            PluginStatus::Ready => "ready",
            PluginStatus::Disabled => "disabled",
            PluginStatus::Error(_) => "error",
        };
        Self {
            id: discovered.manifest.metadata.id,
            name: discovered.manifest.metadata.name,
            version: discovered.manifest.metadata.version,
            description: discovered.manifest.metadata.description,
            author: discovered.manifest.metadata.author,
            enabled: discovered.manifest.enabled,
            status: status.to_string(),
            path: discovered.path.to_string_lossy().to_string(),
        }
    }
}

/// Tauri command: List all discovered plugins.
#[tauri::command]
pub fn list_discovered_plugins() -> Result<Vec<PluginInfo>, String> {
    let plugins_dir = get_plugins_directory();

    let discovered =
        discover_plugins(&plugins_dir).map_err(|e| format!("Failed to discover plugins: {}", e))?;

    Ok(discovered.into_iter().map(PluginInfo::from).collect())
}

/// Tauri command: Enable a plugin by ID.
#[tauri::command]
pub fn enable_plugin(plugin_id: String) -> Result<(), String> {
    let plugins_dir = get_plugins_directory();
    let discovered =
        discover_plugins(&plugins_dir).map_err(|e| format!("Failed to discover plugins: {}", e))?;

    for plugin in discovered {
        if plugin.manifest.metadata.id == plugin_id {
            let manifest_path = plugin.path.join("plugin.json");
            let mut manifest = plugin.manifest;
            manifest.enabled = true;
            save_manifest(&manifest, &manifest_path)
                .map_err(|e| format!("Failed to save manifest: {}", e))?;
            return Ok(());
        }
    }

    Err(format!("Plugin '{}' not found", plugin_id))
}

/// Tauri command: Disable a plugin by ID.
#[tauri::command]
pub fn disable_plugin(plugin_id: String) -> Result<(), String> {
    let plugins_dir = get_plugins_directory();
    let discovered =
        discover_plugins(&plugins_dir).map_err(|e| format!("Failed to discover plugins: {}", e))?;

    for plugin in discovered {
        if plugin.manifest.metadata.id == plugin_id {
            let manifest_path = plugin.path.join("plugin.json");
            let mut manifest = plugin.manifest;
            manifest.enabled = false;
            save_manifest(&manifest, &manifest_path)
                .map_err(|e| format!("Failed to save manifest: {}", e))?;
            return Ok(());
        }
    }

    Err(format!("Plugin '{}' not found", plugin_id))
}

/// Tauri command: Open the plugins directory in file explorer.
#[tauri::command]
pub fn open_plugins_directory() -> Result<String, String> {
    let plugins_dir = ensure_plugins_directory()
        .map_err(|e| format!("Failed to create plugins directory: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&plugins_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&plugins_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&plugins_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    Ok(plugins_dir.to_string_lossy().to_string())
}

/// Context provided to plugins during lifecycle hooks.
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Path to the plugin's data directory
    pub data_dir: std::path::PathBuf,
    /// Path to the plugin's config file
    pub config_path: std::path::PathBuf,
    /// Application data directory
    pub app_data_dir: std::path::PathBuf,
}

/// Result of a hook execution.
pub type HookResult<T> = Result<T, String>;

/// Core plugin trait that all plugins must implement.
pub trait Plugin: Send + Sync {
    /// Returns the plugin's metadata.
    fn metadata(&self) -> PluginMetadata;

    /// Called when the plugin is loaded.
    ///
    /// Use this to initialize resources, register hooks, etc.
    fn on_load(&mut self, _context: &PluginContext) -> HookResult<()> {
        Ok(())
    }

    /// Called when the plugin is unloaded.
    ///
    /// Use this to cleanup resources.
    fn on_unload(&mut self) -> HookResult<()> {
        Ok(())
    }
}

/// Hook points where plugins can extend functionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookPoint {
    /// Called before a report is generated
    BeforeReportGenerate,
    /// Called after a report is generated
    AfterReportGenerate,
    /// Called before a record is saved
    BeforeRecordSave,
    /// Called after a record is saved
    AfterRecordSave,
    /// Called when a screenshot is captured
    OnScreenshotCapture,
    /// Called when a quick note is created
    OnQuickNote,
    /// Called when the app starts
    OnAppStart,
    /// Called when the app shuts down
    OnAppShutdown,
}

/// Data passed to hooks during execution.
#[derive(Debug, Clone)]
pub enum HookData {
    /// Report generation data
    ReportGenerate {
        /// Report type (daily, weekly, monthly, custom, comparison)
        report_type: String,
        /// Report content (for AfterReportGenerate)
        content: Option<String>,
        /// Output path
        output_path: Option<std::path::PathBuf>,
    },
    /// Record save data
    RecordSave {
        /// Record ID (for AfterRecordSave)
        record_id: Option<i64>,
        /// Record content
        content: String,
        /// Source type (auto, manual)
        source_type: String,
    },
    /// Screenshot capture data
    ScreenshotCapture {
        /// Screenshot path
        path: std::path::PathBuf,
        /// Analysis result (if available)
        analysis: Option<String>,
    },
    /// Quick note data
    QuickNote {
        /// Note content
        content: String,
    },
    /// App lifecycle data
    AppLifecycle {
        /// App version
        version: String,
    },
}

/// Function type for hook callbacks.
pub type HookCallback = Box<dyn Fn(&HookData) -> HookResult<()> + Send + Sync>;

/// Manager for plugins and hooks.
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    hooks: HashMap<HookPoint, Vec<HookCallback>>,
    enabled: bool,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// Creates a new empty plugin manager.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            hooks: HashMap::new(),
            enabled: false,
        }
    }

    /// Enables or disables the plugin system.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether the plugin system is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Registers a plugin with the manager.
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> HookResult<()> {
        let id = plugin.metadata().id.clone();
        if self.plugins.contains_key(&id) {
            return Err(format!("Plugin '{}' is already registered", id));
        }
        self.plugins.insert(id, plugin);
        Ok(())
    }

    /// Unregisters a plugin by ID.
    pub fn unregister_plugin(&mut self, id: &str) -> HookResult<()> {
        self.plugins
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| format!("Plugin '{}' not found", id))
    }

    /// Returns a list of all registered plugin metadata.
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }

    /// Registers a hook callback for a specific hook point.
    pub fn register_hook(&mut self, point: HookPoint, callback: HookCallback) {
        self.hooks.entry(point).or_default().push(callback);
    }

    /// Triggers a hook point, calling all registered callbacks.
    pub fn trigger_hook(&self, point: HookPoint, data: &HookData) -> HookResult<()> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(callbacks) = self.hooks.get(&point) {
            for callback in callbacks {
                callback(data)?;
            }
        }
        Ok(())
    }

    /// Loads all plugins and calls their on_load hooks.
    pub fn load_all(&mut self, context: &PluginContext) -> HookResult<()> {
        for plugin in self.plugins.values_mut() {
            plugin.on_load(context)?;
        }
        self.enabled = true;
        Ok(())
    }

    /// Unloads all plugins and calls their on_unload hooks.
    pub fn unload_all(&mut self) -> HookResult<()> {
        self.enabled = false;
        for plugin in self.plugins.values_mut() {
            plugin.on_unload()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        metadata: PluginMetadata,
        loaded: bool,
    }

    impl TestPlugin {
        fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    id: "test.plugin".to_string(),
                    name: "Test Plugin".to_string(),
                    version: "1.0.0".to_string(),
                    description: "A test plugin".to_string(),
                    author: "Test Author".to_string(),
                    min_app_version: None,
                    homepage: None,
                    license: None,
                },
                loaded: false,
            }
        }
    }

    impl Plugin for TestPlugin {
        fn metadata(&self) -> PluginMetadata {
            self.metadata.clone()
        }

        fn on_load(&mut self, _context: &PluginContext) -> HookResult<()> {
            self.loaded = true;
            Ok(())
        }

        fn on_unload(&mut self) -> HookResult<()> {
            self.loaded = false;
            Ok(())
        }
    }

    #[test]
    fn test_plugin_manager_register() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new());
        assert!(manager.register_plugin(plugin).is_ok());
        assert_eq!(manager.list_plugins().len(), 1);
    }

    #[test]
    fn test_plugin_manager_duplicate_register() {
        let mut manager = PluginManager::new();
        let plugin1 = Box::new(TestPlugin::new());
        let plugin2 = Box::new(TestPlugin::new());
        assert!(manager.register_plugin(plugin1).is_ok());
        assert!(manager.register_plugin(plugin2).is_err());
    }

    #[test]
    fn test_plugin_manager_unregister() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new());
        manager.register_plugin(plugin).unwrap();
        assert!(manager.unregister_plugin("test.plugin").is_ok());
        assert!(manager.list_plugins().is_empty());
    }

    #[test]
    fn test_plugin_manager_unregister_not_found() {
        let mut manager = PluginManager::new();
        assert!(manager.unregister_plugin("nonexistent").is_err());
    }

    #[test]
    fn test_plugin_manager_hooks() {
        let mut manager = PluginManager::new();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        manager.register_hook(
            HookPoint::OnAppStart,
            Box::new(move |_data| {
                called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }),
        );

        manager.set_enabled(true);
        manager
            .trigger_hook(
                HookPoint::OnAppStart,
                &HookData::AppLifecycle {
                    version: "1.0.0".to_string(),
                },
            )
            .unwrap();

        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_plugin_manager_hooks_disabled() {
        let mut manager = PluginManager::new();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        manager.register_hook(
            HookPoint::OnAppStart,
            Box::new(move |_data| {
                called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }),
        );

        manager.set_enabled(false);
        manager
            .trigger_hook(
                HookPoint::OnAppStart,
                &HookData::AppLifecycle {
                    version: "1.0.0".to_string(),
                },
            )
            .unwrap();

        assert!(!called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_plugin_lifecycle() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new());
        manager.register_plugin(plugin).unwrap();

        let context = PluginContext {
            data_dir: std::path::PathBuf::from("/tmp/test"),
            config_path: std::path::PathBuf::from("/tmp/test/config.json"),
            app_data_dir: std::path::PathBuf::from("/tmp/app"),
        };

        manager.load_all(&context).unwrap();
        assert!(manager.is_enabled());

        manager.unload_all().unwrap();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_plugin_metadata_serialization() {
        let metadata = PluginMetadata {
            id: "test.plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            min_app_version: Some("1.0.0".to_string()),
            homepage: Some("https://example.com".to_string()),
            license: Some("MIT".to_string()),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: PluginMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.name, deserialized.name);
        assert_eq!(metadata.version, deserialized.version);
    }

    #[test]
    fn test_plugin_manifest_serialization() {
        let manifest = PluginManifest {
            metadata: PluginMetadata {
                id: "com.example.test".to_string(),
                name: "Test Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A test plugin".to_string(),
                author: "Test Author".to_string(),
                min_app_version: Some("1.18.0".to_string()),
                homepage: None,
                license: Some("MIT".to_string()),
            },
            enabled: true,
            entry_point: Some("index.js".to_string()),
            dependencies: vec!["com.example.other".to_string()],
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: PluginManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(manifest.metadata.id, deserialized.metadata.id);
        assert_eq!(manifest.enabled, deserialized.enabled);
        assert_eq!(manifest.entry_point, deserialized.entry_point);
        assert_eq!(manifest.dependencies.len(), deserialized.dependencies.len());
    }

    #[test]
    fn test_load_manifest_valid() {
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("plugin.json");

        let manifest_content = r#"{
            "id": "com.example.test",
            "name": "Test Plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "author": "Test Author",
            "enabled": true
        }"#;

        let mut file = std::fs::File::create(&manifest_path).unwrap();
        file.write_all(manifest_content.as_bytes()).unwrap();

        let loaded = load_manifest(&manifest_path).unwrap();
        assert_eq!(loaded.metadata.id, "com.example.test");
        assert_eq!(loaded.metadata.name, "Test Plugin");
        assert!(loaded.enabled);
    }

    #[test]
    fn test_load_manifest_missing_id() {
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("plugin.json");

        let manifest_content = r#"{
            "name": "Test Plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "author": "Test Author"
        }"#;

        let mut file = std::fs::File::create(&manifest_path).unwrap();
        file.write_all(manifest_content.as_bytes()).unwrap();

        let result = load_manifest(&manifest_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Plugin ID is required"));
    }

    #[test]
    fn test_load_manifest_invalid_json() {
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("plugin.json");

        let mut file = std::fs::File::create(&manifest_path).unwrap();
        file.write_all(b"not valid json").unwrap();

        let result = load_manifest(&manifest_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid manifest JSON"));
    }

    #[test]
    fn test_discover_plugins_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let plugins = discover_plugins(temp_dir.path()).unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_discover_plugins_with_valid_plugin() {
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();
        let plugin_dir = temp_dir.path().join("test-plugin");
        std::fs::create_dir(&plugin_dir).unwrap();

        let manifest_path = plugin_dir.join("plugin.json");
        let manifest_content = r#"{
            "id": "com.example.test",
            "name": "Test Plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "author": "Test Author"
        }"#;

        let mut file = std::fs::File::create(&manifest_path).unwrap();
        file.write_all(manifest_content.as_bytes()).unwrap();

        let plugins = discover_plugins(temp_dir.path()).unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].manifest.metadata.id, "com.example.test");
        assert_eq!(plugins[0].status, PluginStatus::Ready);
    }

    #[test]
    fn test_discover_plugins_disabled_plugin() {
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();
        let plugin_dir = temp_dir.path().join("disabled-plugin");
        std::fs::create_dir(&plugin_dir).unwrap();

        let manifest_path = plugin_dir.join("plugin.json");
        let manifest_content = r#"{
            "id": "com.example.disabled",
            "name": "Disabled Plugin",
            "version": "1.0.0",
            "description": "A disabled plugin",
            "author": "Test Author",
            "enabled": false
        }"#;

        let mut file = std::fs::File::create(&manifest_path).unwrap();
        file.write_all(manifest_content.as_bytes()).unwrap();

        let plugins = discover_plugins(temp_dir.path()).unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].status, PluginStatus::Disabled);
    }

    #[test]
    fn test_discover_plugins_invalid_manifest() {
        let temp_dir = tempfile::tempdir().unwrap();
        let plugin_dir = temp_dir.path().join("broken-plugin");
        std::fs::create_dir(&plugin_dir).unwrap();

        let manifest_path = plugin_dir.join("plugin.json");
        std::fs::write(&manifest_path, "not json").unwrap();

        let plugins = discover_plugins(temp_dir.path()).unwrap();
        assert_eq!(plugins.len(), 1);
        assert!(matches!(plugins[0].status, PluginStatus::Error(_)));
    }

    #[test]
    fn test_save_manifest() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("plugin.json");

        let manifest = PluginManifest {
            metadata: PluginMetadata {
                id: "com.example.test".to_string(),
                name: "Test Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A test plugin".to_string(),
                author: "Test Author".to_string(),
                min_app_version: None,
                homepage: None,
                license: None,
            },
            enabled: true,
            entry_point: None,
            dependencies: Vec::new(),
        };

        save_manifest(&manifest, &manifest_path).unwrap();

        let loaded = load_manifest(&manifest_path).unwrap();
        assert_eq!(loaded.metadata.id, manifest.metadata.id);
    }
}
