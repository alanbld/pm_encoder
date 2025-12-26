//! Plugin Engine - Core Plugin System
//!
//! Orchestrates plugin discovery, loading, and execution.
//! Provides the main interface for the plugin ecosystem.

use std::path::PathBuf;

use super::error::PluginResult;
use super::loader::{PluginLoader, LoadedPlugin, PluginStatus, CURRENT_API_VERSION};

#[cfg(feature = "plugins")]
use super::bridges::vo_table::SharedContributions;

/// Plugin engine state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    /// Engine not initialized
    Uninitialized,
    /// Plugins discovered but not executed
    Discovered,
    /// Plugins executed
    Executed,
    /// Engine disabled (feature not compiled or --no-plugins)
    Disabled,
}

/// The Plugin Engine - manages the plugin lifecycle
pub struct PluginEngine {
    /// Plugin loader
    loader: PluginLoader,
    /// Current engine state
    state: EngineState,
    /// Plugin contributions (after execution)
    #[cfg(feature = "plugins")]
    contributions: Option<SharedContributions>,
}

impl PluginEngine {
    /// Create a new plugin engine
    pub fn new() -> Self {
        Self {
            loader: PluginLoader::new(),
            state: EngineState::Uninitialized,
            #[cfg(feature = "plugins")]
            contributions: None,
        }
    }

    /// Create a disabled engine (for --no-plugins mode)
    pub fn disabled() -> Self {
        Self {
            loader: PluginLoader::with_paths(vec![]),
            state: EngineState::Disabled,
            #[cfg(feature = "plugins")]
            contributions: None,
        }
    }

    /// Add a custom plugin search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.loader.add_path(path);
    }

    /// Get current engine state
    pub fn state(&self) -> EngineState {
        self.state
    }

    /// Check if plugins feature is available
    pub fn is_available() -> bool {
        cfg!(feature = "plugins")
    }

    /// Get API version
    pub fn api_version() -> &'static str {
        CURRENT_API_VERSION
    }

    /// Discover plugins from configured paths
    pub fn discover(&mut self) -> &[LoadedPlugin] {
        if self.state == EngineState::Disabled {
            return &[];
        }

        self.loader.discover();
        self.state = EngineState::Discovered;
        self.loader.plugins()
    }

    /// Execute all discovered plugins
    #[cfg(feature = "plugins")]
    pub fn execute(&mut self) -> PluginResult<()> {
        if self.state == EngineState::Disabled {
            return Ok(());
        }

        if self.state == EngineState::Uninitialized {
            self.discover();
        }

        self.contributions = Some(self.loader.execute_all()?);
        self.state = EngineState::Executed;
        Ok(())
    }

    /// Execute (no-op when plugins feature is disabled)
    #[cfg(not(feature = "plugins"))]
    pub fn execute(&mut self) -> PluginResult<()> {
        self.state = EngineState::Disabled;
        Ok(())
    }

    /// Get discovered plugins
    pub fn plugins(&self) -> &[LoadedPlugin] {
        self.loader.plugins()
    }

    /// Get enabled plugin names
    pub fn plugin_names(&self) -> Vec<&str> {
        self.loader.plugin_names()
    }

    /// Get plugin count
    pub fn plugin_count(&self) -> usize {
        self.loader.enabled_plugins().len()
    }

    /// Get contributions from executed plugins
    #[cfg(feature = "plugins")]
    pub fn contributions(&self) -> Option<&SharedContributions> {
        self.contributions.as_ref()
    }

    /// Get metric value by name
    #[cfg(feature = "plugins")]
    pub fn get_metric(&self, name: &str) -> Option<f64> {
        self.contributions.as_ref().and_then(|c| {
            c.lock().ok().and_then(|contribs| {
                contribs.metrics.get(name).map(|m| m.value)
            })
        })
    }

    /// Get tags for a node
    #[cfg(feature = "plugins")]
    pub fn get_tags(&self, node_id: &str) -> Vec<String> {
        self.contributions
            .as_ref()
            .and_then(|c| {
                c.lock().ok().map(|contribs| {
                    contribs.tags.get(node_id).cloned().unwrap_or_default()
                })
            })
            .unwrap_or_default()
    }

    /// Get all log entries
    #[cfg(feature = "plugins")]
    pub fn get_logs(&self) -> Vec<super::bridges::vo_table::LogEntry> {
        self.contributions
            .as_ref()
            .and_then(|c| {
                c.lock().ok().map(|contribs| contribs.logs.clone())
            })
            .unwrap_or_default()
    }

    /// Generate summary for Mission Log
    pub fn summary(&self) -> String {
        if self.state == EngineState::Disabled {
            return String::from("🔌 External optics: Disabled");
        }

        let plugin_count = self.plugin_count();
        if plugin_count == 0 {
            return String::from("🔌 No external optics detected.");
        }

        let mut output = format!("🔌 External Optics: {} community plugin{} loaded\n",
            plugin_count,
            if plugin_count == 1 { "" } else { "s" }
        );

        for name in self.plugin_names() {
            output.push_str(&format!("   ├─ {}\n", name));
        }

        output.push_str("🛡️ Plugin sandbox: Active (10MB memory, 100ms timeout)\n");
        output
    }

    /// Generate status for --plugins list command
    pub fn list_status(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Plugin API Version: {}\n", CURRENT_API_VERSION));
        output.push_str(&format!("Feature Status: {}\n",
            if Self::is_available() { "Enabled" } else { "Disabled" }
        ));
        output.push_str("\nSearch Paths:\n");

        for path in self.loader.search_paths() {
            let status = if path.exists() { "✓" } else { "✗" };
            output.push_str(&format!("  {} {}\n", status, path.display()));
        }

        output.push_str(&format!("\nDiscovered Plugins: {}\n", self.loader.plugins().len()));

        for plugin in self.loader.plugins() {
            let status_icon = match &plugin.status {
                PluginStatus::Loaded => "✓",
                PluginStatus::Executed => "✓",
                PluginStatus::Disabled => "○",
                PluginStatus::LoadError(_) => "✗",
                PluginStatus::ExecutionError(_) => "✗",
            };

            output.push_str(&format!("  {} {} (priority: {})\n",
                status_icon,
                plugin.entry.name,
                plugin.entry.priority
            ));

            if let PluginStatus::LoadError(e) | PluginStatus::ExecutionError(e) = &plugin.status {
                output.push_str(&format!("      Error: {}\n", e));
            }
        }

        output
    }
}

impl Default for PluginEngine {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = PluginEngine::new();
        assert_eq!(engine.state(), EngineState::Uninitialized);
    }

    #[test]
    fn test_engine_disabled() {
        let engine = PluginEngine::disabled();
        assert_eq!(engine.state(), EngineState::Disabled);
    }

    #[test]
    fn test_api_version() {
        assert_eq!(PluginEngine::api_version(), "3.0");
    }

    #[test]
    fn test_summary_no_plugins() {
        let mut engine = PluginEngine::new();
        engine.discover();
        let summary = engine.summary();
        assert!(summary.contains("No external optics"));
    }

    #[test]
    fn test_summary_disabled() {
        let engine = PluginEngine::disabled();
        let summary = engine.summary();
        assert!(summary.contains("Disabled"));
    }

    #[cfg(feature = "plugins")]
    #[test]
    fn test_engine_with_plugins() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();

        // Create manifest
        let manifest = serde_json::json!({
            "vo_api_version": "3.0",
            "plugins": [{
                "name": "test-plugin",
                "file": "test.lua",
                "enabled": true,
                "priority": 100
            }]
        });
        std::fs::write(plugins_dir.join("manifest.json"), manifest.to_string()).unwrap();
        std::fs::write(plugins_dir.join("test.lua"), "vo.log('info', 'Hello!')").unwrap();

        let mut engine = PluginEngine::new();
        engine.add_search_path(plugins_dir);
        engine.discover();

        assert_eq!(engine.plugin_count(), 1);
        assert_eq!(engine.state(), EngineState::Discovered);
    }
}
