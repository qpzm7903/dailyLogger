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
//! # Example Plugin
//!
//! ```ignore
//! use daily_logger_lib::plugin::{Plugin, PluginContext, PluginMetadata};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn metadata(&self) -> PluginMetadata {
//!         PluginMetadata {
//!             name: "My Plugin".to_string(),
//!             version: "1.0.0".to_string(),
//!             description: "A custom plugin".to_string(),
//!             author: "Author Name".to_string(),
//!         }
//!     }
//!
//!     fn on_load(&mut self, context: &PluginContext) -> Result<(), String> {
//!         // Initialize plugin
//!         Ok(())
//!     }
//!
//!     fn on_unload(&mut self) -> Result<(), String> {
//!         // Cleanup
//!         Ok(())
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata describing a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin (e.g., "com.example.my-plugin")
    pub id: String,
    /// Human-readable name of the plugin
    pub name: String,
    /// Version string (semver recommended)
    pub version: String,
    /// Brief description of what the plugin does
    pub description: String,
    /// Author or organization name
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

        manager.register_hook(HookPoint::OnAppStart, Box::new(move |_data| {
            called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }));

        manager.set_enabled(true);
        manager
            .trigger_hook(HookPoint::OnAppStart, &HookData::AppLifecycle {
                version: "1.0.0".to_string(),
            })
            .unwrap();

        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_plugin_manager_hooks_disabled() {
        let mut manager = PluginManager::new();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        manager.register_hook(HookPoint::OnAppStart, Box::new(move |_data| {
            called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }));

        manager.set_enabled(false);
        manager
            .trigger_hook(HookPoint::OnAppStart, &HookData::AppLifecycle {
                version: "1.0.0".to_string(),
            })
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
}