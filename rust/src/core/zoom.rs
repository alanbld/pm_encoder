//! Fractal Protocol: Zoom Actions
//!
//! This module implements the interactive zoom feature that allows LLMs to request
//! deeper context for specific code elements.
//!
//! # Protocol
//!
//! When content is truncated, a zoom affordance is embedded:
//! ```text
//! /* ZOOM_AFFORDANCE: pm_encoder --zoom function=apply_budget --budget=1000 */
//! ```
//!
//! The LLM can then request expansion via MCP or CLI.

use crate::core::error::{EncoderError, Result};
use std::fmt;

/// Target type for zoom operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoomTarget {
    /// Zoom into a specific function
    Function(String),
    /// Zoom into a specific class/struct
    Class(String),
    /// Zoom into a module
    Module(String),
    /// Zoom into a file with optional line range
    File {
        path: String,
        start_line: Option<usize>,
        end_line: Option<usize>,
    },
}

impl ZoomTarget {
    /// Parse a zoom target from string format "type=value"
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(EncoderError::InvalidZoomTarget {
                target: s.to_string(),
            });
        }

        let (kind, value) = (parts[0], parts[1]);
        match kind {
            "function" | "fn" => Ok(ZoomTarget::Function(value.to_string())),
            "class" | "struct" => Ok(ZoomTarget::Class(value.to_string())),
            "module" | "mod" => Ok(ZoomTarget::Module(value.to_string())),
            "file" => {
                // Parse file path, optionally with line range: path:start-end
                if let Some(colon_pos) = value.rfind(':') {
                    let path = value[..colon_pos].to_string();
                    let range = &value[colon_pos + 1..];
                    if let Some(dash_pos) = range.find('-') {
                        let start = range[..dash_pos].parse().ok();
                        let end = range[dash_pos + 1..].parse().ok();
                        Ok(ZoomTarget::File {
                            path,
                            start_line: start,
                            end_line: end,
                        })
                    } else {
                        Ok(ZoomTarget::File {
                            path,
                            start_line: range.parse().ok(),
                            end_line: None,
                        })
                    }
                } else {
                    Ok(ZoomTarget::File {
                        path: value.to_string(),
                        start_line: None,
                        end_line: None,
                    })
                }
            }
            _ => Err(EncoderError::InvalidZoomTarget {
                target: s.to_string(),
            }),
        }
    }

    /// Generate the CLI command for this zoom target
    pub fn to_command(&self, budget: Option<usize>) -> String {
        let target_str = match self {
            ZoomTarget::Function(name) => format!("function={}", name),
            ZoomTarget::Class(name) => format!("class={}", name),
            ZoomTarget::Module(name) => format!("module={}", name),
            ZoomTarget::File { path, start_line, end_line } => {
                match (start_line, end_line) {
                    (Some(s), Some(e)) => format!("file={}:{}-{}", path, s, e),
                    (Some(s), None) => format!("file={}:{}", path, s),
                    _ => format!("file={}", path),
                }
            }
        };

        match budget {
            Some(b) => format!("pm_encoder --zoom {} --budget {}", target_str, b),
            None => format!("pm_encoder --zoom {}", target_str),
        }
    }
}

impl fmt::Display for ZoomTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoomTarget::Function(name) => write!(f, "function:{}", name),
            ZoomTarget::Class(name) => write!(f, "class:{}", name),
            ZoomTarget::Module(name) => write!(f, "module:{}", name),
            ZoomTarget::File { path, start_line, end_line } => {
                match (start_line, end_line) {
                    (Some(s), Some(e)) => write!(f, "file:{}[{}-{}]", path, s, e),
                    (Some(s), None) => write!(f, "file:{}[{}]", path, s),
                    _ => write!(f, "file:{}", path),
                }
            }
        }
    }
}

/// Configuration for a zoom operation
#[derive(Debug, Clone)]
pub struct ZoomConfig {
    /// The target to zoom into
    pub target: ZoomTarget,
    /// Token budget for the zoomed content
    pub budget: Option<usize>,
    /// Depth of expansion: "signature", "implementation", or "full"
    pub depth: ZoomDepth,
    /// Include related tests
    pub include_tests: bool,
    /// Context lines around the target
    pub context_lines: usize,
}

/// Depth of zoom expansion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZoomDepth {
    /// Only show signatures/declarations
    Signature,
    /// Show implementation without docstrings
    #[default]
    Implementation,
    /// Show full content including docs and tests
    Full,
}

impl ZoomDepth {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "signature" | "sig" => Some(ZoomDepth::Signature),
            "implementation" | "impl" => Some(ZoomDepth::Implementation),
            "full" => Some(ZoomDepth::Full),
            _ => None,
        }
    }
}

impl Default for ZoomConfig {
    fn default() -> Self {
        Self {
            target: ZoomTarget::Function("main".to_string()),
            budget: Some(1000),
            depth: ZoomDepth::Implementation,
            include_tests: false,
            context_lines: 5,
        }
    }
}

/// A zoom action represents a suggested expansion point
#[derive(Debug, Clone)]
pub struct ZoomAction {
    /// The zoom target
    pub target: ZoomTarget,
    /// Suggested budget
    pub suggested_budget: usize,
    /// Human-readable description
    pub description: String,
    /// The CLI command to execute
    pub command: String,
}

impl ZoomAction {
    /// Create a new zoom action for a function
    pub fn for_function(name: &str, budget: usize) -> Self {
        let target = ZoomTarget::Function(name.to_string());
        let command = target.to_command(Some(budget));
        Self {
            target,
            suggested_budget: budget,
            description: format!("Expand function '{}' ({} tokens)", name, budget),
            command,
        }
    }

    /// Create a new zoom action for a class
    pub fn for_class(name: &str, budget: usize) -> Self {
        let target = ZoomTarget::Class(name.to_string());
        let command = target.to_command(Some(budget));
        Self {
            target,
            suggested_budget: budget,
            description: format!("Expand class '{}' ({} tokens)", name, budget),
            command,
        }
    }

    /// Create a new zoom action for a file
    pub fn for_file(path: &str, budget: usize) -> Self {
        let target = ZoomTarget::File {
            path: path.to_string(),
            start_line: None,
            end_line: None,
        };
        let command = target.to_command(Some(budget));
        Self {
            target,
            suggested_budget: budget,
            description: format!("Expand file '{}' ({} tokens)", path, budget),
            command,
        }
    }

    /// Generate the affordance comment for serialization
    pub fn to_affordance_comment(&self) -> String {
        format!("/* ZOOM_AFFORDANCE: {} */", self.command)
    }

    /// Generate XML representation
    pub fn to_xml(&self) -> String {
        format!(
            "<action type=\"expand\" target=\"{}\" budget=\"{}\" cmd=\"{}\" />",
            self.target, self.suggested_budget, self.command
        )
    }
}

// ============================================================================
// Fractal Protocol v2: Bidirectional Zoom & Sessions
// ============================================================================

/// Direction of zoom operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomDirection {
    /// Expand to show more detail
    Expand,
    /// Collapse to show less detail (structure only)
    Collapse,
}

/// A zoom history entry for undo/redo
#[derive(Debug, Clone)]
pub struct ZoomHistoryEntry {
    /// The zoom target
    pub target: ZoomTarget,
    /// Direction of the zoom
    pub direction: ZoomDirection,
    /// Depth before the zoom (for undo)
    pub previous_depth: ZoomDepth,
    /// Timestamp of the action
    pub timestamp: u64,
}

/// Zoom history for tracking and undoing actions
#[derive(Debug, Clone, Default)]
pub struct ZoomHistory {
    /// Stack of zoom actions (most recent last)
    entries: Vec<ZoomHistoryEntry>,
    /// Current position in history (for redo)
    position: usize,
    /// Maximum history size
    max_size: usize,
}

impl ZoomHistory {
    /// Create a new zoom history with default max size
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            position: 0,
            max_size: 100,
        }
    }

    /// Create with custom max size
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            position: 0,
            max_size,
        }
    }

    /// Record a zoom action
    pub fn record(&mut self, entry: ZoomHistoryEntry) {
        // Truncate any "future" entries if we're not at the end
        self.entries.truncate(self.position);

        // Add the new entry
        self.entries.push(entry);
        self.position = self.entries.len();

        // Enforce max size
        if self.entries.len() > self.max_size {
            self.entries.remove(0);
            self.position = self.entries.len();
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.position < self.entries.len()
    }

    /// Get the entry to undo (moves position back)
    pub fn undo(&mut self) -> Option<&ZoomHistoryEntry> {
        if self.can_undo() {
            self.position -= 1;
            Some(&self.entries[self.position])
        } else {
            None
        }
    }

    /// Get the entry to redo (moves position forward)
    pub fn redo(&mut self) -> Option<&ZoomHistoryEntry> {
        if self.can_redo() {
            let entry = &self.entries[self.position];
            self.position += 1;
            Some(entry)
        } else {
            None
        }
    }

    /// Get all entries
    pub fn entries(&self) -> &[ZoomHistoryEntry] {
        &self.entries
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.position = 0;
    }
}

/// A saved zoom session
#[derive(Debug, Clone)]
pub struct ZoomSession {
    /// Session name
    pub name: String,
    /// Active zoom targets with their depths
    pub active_zooms: Vec<(ZoomTarget, ZoomDepth)>,
    /// Zoom history
    pub history: ZoomHistory,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
}

impl ZoomSession {
    /// Create a new empty session
    pub fn new(name: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            name: name.to_string(),
            active_zooms: Vec::new(),
            history: ZoomHistory::new(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Add a zoom to the session
    pub fn add_zoom(&mut self, target: ZoomTarget, depth: ZoomDepth) {
        // Record in history
        self.history.record(ZoomHistoryEntry {
            target: target.clone(),
            direction: ZoomDirection::Expand,
            previous_depth: ZoomDepth::Signature,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });

        // Check if target already exists
        if let Some(pos) = self.active_zooms.iter().position(|(t, _)| t == &target) {
            self.active_zooms[pos].1 = depth;
        } else {
            self.active_zooms.push((target, depth));
        }

        self.touch();
    }

    /// Remove a zoom (collapse)
    pub fn remove_zoom(&mut self, target: &ZoomTarget) -> bool {
        if let Some(pos) = self.active_zooms.iter().position(|(t, _)| t == target) {
            let (_, prev_depth) = self.active_zooms.remove(pos);

            // Record in history
            self.history.record(ZoomHistoryEntry {
                target: target.clone(),
                direction: ZoomDirection::Collapse,
                previous_depth: prev_depth,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            });

            self.touch();
            true
        } else {
            false
        }
    }

    /// Update modified timestamp
    fn touch(&mut self) {
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Check if a target is zoomed
    pub fn is_zoomed(&self, target: &ZoomTarget) -> bool {
        self.active_zooms.iter().any(|(t, _)| t == target)
    }

    /// Get zoom depth for a target
    pub fn get_depth(&self, target: &ZoomTarget) -> Option<ZoomDepth> {
        self.active_zooms.iter()
            .find(|(t, _)| t == target)
            .map(|(_, d)| *d)
    }

    /// Get count of active zooms
    pub fn zoom_count(&self) -> usize {
        self.active_zooms.len()
    }
}

/// Session store for managing multiple sessions
#[derive(Debug, Default)]
pub struct ZoomSessionStore {
    sessions: std::collections::HashMap<String, ZoomSession>,
    active_session: Option<String>,
}

impl ZoomSessionStore {
    /// Create a new session store
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
            active_session: None,
        }
    }

    /// Create a new session
    pub fn create_session(&mut self, name: &str) -> &mut ZoomSession {
        let session = ZoomSession::new(name);
        self.sessions.insert(name.to_string(), session);
        self.active_session = Some(name.to_string());
        self.sessions.get_mut(name).unwrap()
    }

    /// Get a session by name
    pub fn get_session(&self, name: &str) -> Option<&ZoomSession> {
        self.sessions.get(name)
    }

    /// Get mutable session by name
    pub fn get_session_mut(&mut self, name: &str) -> Option<&mut ZoomSession> {
        self.sessions.get_mut(name)
    }

    /// Get active session
    pub fn active(&self) -> Option<&ZoomSession> {
        self.active_session.as_ref().and_then(|n| self.sessions.get(n))
    }

    /// Get mutable active session
    pub fn active_mut(&mut self) -> Option<&mut ZoomSession> {
        if let Some(name) = self.active_session.clone() {
            self.sessions.get_mut(&name)
        } else {
            None
        }
    }

    /// Set active session
    pub fn set_active(&mut self, name: &str) -> bool {
        if self.sessions.contains_key(name) {
            self.active_session = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// List all session names
    pub fn list_sessions(&self) -> Vec<&str> {
        self.sessions.keys().map(|s| s.as_str()).collect()
    }

    /// Delete a session
    pub fn delete_session(&mut self, name: &str) -> bool {
        if self.sessions.remove(name).is_some() {
            if self.active_session.as_deref() == Some(name) {
                self.active_session = None;
            }
            true
        } else {
            false
        }
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Fractal v2 Tests - TDD: Written first, implementation follows
    // ========================================================================

    // --- ZoomHistory Tests ---

    #[test]
    fn test_zoom_history_record_and_undo() {
        let mut history = ZoomHistory::new();

        let entry = ZoomHistoryEntry {
            target: ZoomTarget::Function("test".to_string()),
            direction: ZoomDirection::Expand,
            previous_depth: ZoomDepth::Signature,
            timestamp: 12345,
        };

        history.record(entry);
        assert_eq!(history.position(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        let undone = history.undo().unwrap();
        assert!(matches!(&undone.target, ZoomTarget::Function(n) if n == "test"));
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_zoom_history_redo() {
        let mut history = ZoomHistory::new();

        history.record(ZoomHistoryEntry {
            target: ZoomTarget::Function("first".to_string()),
            direction: ZoomDirection::Expand,
            previous_depth: ZoomDepth::Signature,
            timestamp: 1,
        });

        history.undo();
        assert!(history.can_redo());

        let redone = history.redo().unwrap();
        assert!(matches!(&redone.target, ZoomTarget::Function(n) if n == "first"));
        assert!(!history.can_redo());
    }

    #[test]
    fn test_zoom_history_max_size() {
        let mut history = ZoomHistory::with_max_size(3);

        for i in 0..5 {
            history.record(ZoomHistoryEntry {
                target: ZoomTarget::Function(format!("fn{}", i)),
                direction: ZoomDirection::Expand,
                previous_depth: ZoomDepth::Signature,
                timestamp: i as u64,
            });
        }

        assert_eq!(history.entries().len(), 3);
        // Should have fn2, fn3, fn4 (oldest removed)
        assert!(matches!(&history.entries()[0].target, ZoomTarget::Function(n) if n == "fn2"));
    }

    #[test]
    fn test_zoom_history_truncate_on_new_action() {
        let mut history = ZoomHistory::new();

        // Record 3 actions
        for i in 0..3 {
            history.record(ZoomHistoryEntry {
                target: ZoomTarget::Function(format!("fn{}", i)),
                direction: ZoomDirection::Expand,
                previous_depth: ZoomDepth::Signature,
                timestamp: i as u64,
            });
        }

        // Undo twice
        history.undo();
        history.undo();

        // Record new action - should truncate "future"
        history.record(ZoomHistoryEntry {
            target: ZoomTarget::Function("new".to_string()),
            direction: ZoomDirection::Expand,
            previous_depth: ZoomDepth::Signature,
            timestamp: 10,
        });

        assert_eq!(history.entries().len(), 2); // fn0 and new
        assert!(!history.can_redo());
    }

    // --- ZoomSession Tests ---

    #[test]
    fn test_zoom_session_create() {
        let session = ZoomSession::new("test-session");
        assert_eq!(session.name, "test-session");
        assert_eq!(session.zoom_count(), 0);
        assert!(session.created_at > 0);
    }

    #[test]
    fn test_zoom_session_add_zoom() {
        let mut session = ZoomSession::new("test");

        session.add_zoom(
            ZoomTarget::Function("main".to_string()),
            ZoomDepth::Full,
        );

        assert_eq!(session.zoom_count(), 1);
        assert!(session.is_zoomed(&ZoomTarget::Function("main".to_string())));
        assert_eq!(session.get_depth(&ZoomTarget::Function("main".to_string())), Some(ZoomDepth::Full));
    }

    #[test]
    fn test_zoom_session_remove_zoom() {
        let mut session = ZoomSession::new("test");

        let target = ZoomTarget::Function("test".to_string());
        session.add_zoom(target.clone(), ZoomDepth::Full);
        assert!(session.is_zoomed(&target));

        let removed = session.remove_zoom(&target);
        assert!(removed);
        assert!(!session.is_zoomed(&target));
        assert_eq!(session.zoom_count(), 0);
    }

    #[test]
    fn test_zoom_session_update_existing_zoom() {
        let mut session = ZoomSession::new("test");
        let target = ZoomTarget::Function("fn".to_string());

        session.add_zoom(target.clone(), ZoomDepth::Signature);
        session.add_zoom(target.clone(), ZoomDepth::Full);

        assert_eq!(session.zoom_count(), 1); // Still only one entry
        assert_eq!(session.get_depth(&target), Some(ZoomDepth::Full)); // Updated depth
    }

    #[test]
    fn test_zoom_session_history_integration() {
        let mut session = ZoomSession::new("test");

        session.add_zoom(ZoomTarget::Function("a".to_string()), ZoomDepth::Full);
        session.add_zoom(ZoomTarget::Function("b".to_string()), ZoomDepth::Full);
        session.remove_zoom(&ZoomTarget::Function("a".to_string()));

        assert_eq!(session.history.entries().len(), 3);
        assert!(session.history.can_undo());
    }

    // --- ZoomSessionStore Tests ---

    #[test]
    fn test_session_store_create_and_get() {
        let mut store = ZoomSessionStore::new();

        store.create_session("session1");
        assert_eq!(store.session_count(), 1);

        let session = store.get_session("session1").unwrap();
        assert_eq!(session.name, "session1");
    }

    #[test]
    fn test_session_store_active_session() {
        let mut store = ZoomSessionStore::new();

        store.create_session("s1");
        store.create_session("s2");

        // Creating a session makes it active
        assert_eq!(store.active().unwrap().name, "s2");

        store.set_active("s1");
        assert_eq!(store.active().unwrap().name, "s1");
    }

    #[test]
    fn test_session_store_list_sessions() {
        let mut store = ZoomSessionStore::new();

        store.create_session("alpha");
        store.create_session("beta");
        store.create_session("gamma");

        let names = store.list_sessions();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
        assert!(names.contains(&"gamma"));
    }

    #[test]
    fn test_session_store_delete_session() {
        let mut store = ZoomSessionStore::new();

        store.create_session("to-delete");
        assert_eq!(store.session_count(), 1);

        let deleted = store.delete_session("to-delete");
        assert!(deleted);
        assert_eq!(store.session_count(), 0);
        assert!(store.active().is_none());
    }

    // --- ZoomDirection Tests ---

    #[test]
    fn test_zoom_direction_expand() {
        let dir = ZoomDirection::Expand;
        assert_eq!(dir, ZoomDirection::Expand);
    }

    #[test]
    fn test_zoom_direction_collapse() {
        let dir = ZoomDirection::Collapse;
        assert_eq!(dir, ZoomDirection::Collapse);
    }

    // ========================================================================
    // Original v1 Tests
    // ========================================================================

    #[test]
    fn test_zoom_target_parse_function() {
        let target = ZoomTarget::parse("function=apply_budget").unwrap();
        assert!(matches!(target, ZoomTarget::Function(name) if name == "apply_budget"));
    }

    #[test]
    fn test_zoom_target_parse_file_with_range() {
        let target = ZoomTarget::parse("file=src/main.rs:10-50").unwrap();
        if let ZoomTarget::File { path, start_line, end_line } = target {
            assert_eq!(path, "src/main.rs");
            assert_eq!(start_line, Some(10));
            assert_eq!(end_line, Some(50));
        } else {
            panic!("Expected File target");
        }
    }

    #[test]
    fn test_zoom_target_to_command() {
        let target = ZoomTarget::Function("process".to_string());
        assert_eq!(
            target.to_command(Some(1000)),
            "pm_encoder --zoom function=process --budget 1000"
        );
    }

    #[test]
    fn test_zoom_action_for_function() {
        let action = ZoomAction::for_function("main", 500);
        assert!(action.command.contains("function=main"));
        assert!(action.command.contains("--budget 500"));
    }

    #[test]
    fn test_zoom_action_affordance_comment() {
        let action = ZoomAction::for_function("test", 1000);
        let comment = action.to_affordance_comment();
        assert!(comment.starts_with("/* ZOOM_AFFORDANCE:"));
        assert!(comment.ends_with("*/"));
    }

    #[test]
    fn test_zoom_action_xml() {
        let action = ZoomAction::for_class("DataProcessor", 2000);
        let xml = action.to_xml();
        assert!(xml.contains("type=\"expand\""));
        assert!(xml.contains("DataProcessor"));
        assert!(xml.contains("2000"));
    }

    #[test]
    fn test_zoom_depth_from_str() {
        assert_eq!(ZoomDepth::from_str("signature"), Some(ZoomDepth::Signature));
        assert_eq!(ZoomDepth::from_str("full"), Some(ZoomDepth::Full));
        assert_eq!(ZoomDepth::from_str("invalid"), None);
    }
}
