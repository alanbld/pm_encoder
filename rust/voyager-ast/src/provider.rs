//! AST Provider Trait and Core Models
//!
//! This module defines the primary interface for voyager-ast:
//! - `AstProvider` trait for implementations
//! - `PlanetariumModel` for project-wide indexing
//! - `MicroscopeModel` for symbol zoom

use crate::error::{AstError, Result};
use crate::ir::{Block, Declaration, File, LanguageId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

// ============================================================================
// Options
// ============================================================================

/// Options for project indexing (Planetarium mode)
#[derive(Debug, Clone, Default)]
pub struct IndexOptions {
    /// Maximum files to process (0 = unlimited)
    pub max_files: usize,

    /// File patterns to include (glob syntax)
    pub include_patterns: Vec<String>,

    /// File patterns to exclude (glob syntax)
    pub exclude_patterns: Vec<String>,

    /// Whether to extract doc comments
    pub extract_comments: bool,

    /// Whether to follow symbolic links
    pub follow_symlinks: bool,

    /// Languages to include (empty = all supported)
    pub languages: Vec<LanguageId>,

    /// Whether to extract nested declarations in Index mode
    pub extract_nested: bool,
}

/// Options for symbol zoom (Microscope mode)
#[derive(Debug, Clone)]
pub struct ZoomOptions {
    /// Maximum depth for nested blocks
    pub max_depth: usize,

    /// Whether to extract function/method calls
    pub extract_calls: bool,

    /// Whether to extract control flow structures
    pub extract_control_flow: bool,

    /// Include surrounding context lines (before/after)
    pub context_lines: usize,

    /// Whether to include nested declarations
    pub extract_nested: bool,
}

impl Default for ZoomOptions {
    fn default() -> Self {
        Self {
            max_depth: 10,
            extract_calls: true,
            extract_control_flow: true,
            context_lines: 0,
            extract_nested: true,
        }
    }
}

// ============================================================================
// Planetarium Model (Index Result)
// ============================================================================

/// The result of indexing a project (Planetarium View)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetariumModel {
    /// Root path of the indexed project
    pub root: String,

    /// All indexed files, keyed by relative path (BTreeMap for determinism)
    pub files: BTreeMap<String, File>,

    /// Statistics about the indexing run
    pub stats: IndexStats,

    /// Errors encountered during indexing
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<IndexError>,
}

impl PlanetariumModel {
    /// Create a new empty model
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            files: BTreeMap::new(),
            stats: IndexStats::default(),
            errors: Vec::new(),
        }
    }

    /// Get all declarations across all files
    pub fn all_declarations(&self) -> impl Iterator<Item = (&str, &Declaration)> {
        self.files.iter().flat_map(|(path, file)| {
            file.declarations.iter().map(move |d| (path.as_str(), d))
        })
    }

    /// Find declarations by name
    pub fn find_by_name(&self, name: &str) -> Vec<(&str, &Declaration)> {
        self.all_declarations()
            .filter(|(_, d)| d.name == name)
            .collect()
    }

    /// Get total declaration count
    pub fn total_declarations(&self) -> usize {
        self.files.values().map(|f| f.total_declarations()).sum()
    }
}

/// Statistics from an indexing run
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexStats {
    /// Number of files processed
    pub files_processed: usize,

    /// Number of files skipped (binary, too large, etc.)
    pub files_skipped: usize,

    /// Total declarations found
    pub declarations_found: usize,

    /// Total imports found
    pub imports_found: usize,

    /// Number of unknown/error regions
    pub unknown_regions: usize,

    /// Parse time in milliseconds
    pub parse_time_ms: u64,

    /// Per-language statistics
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_language: BTreeMap<String, LanguageStats>,
}

/// Per-language statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageStats {
    pub files: usize,
    pub declarations: usize,
    pub imports: usize,
}

/// An error that occurred during indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexError {
    /// Path to the file that caused the error
    pub path: String,

    /// Error message
    pub message: String,

    /// Whether parsing could partially recover
    pub recoverable: bool,
}

// ============================================================================
// Microscope Model (Zoom Result)
// ============================================================================

/// The result of zooming into a symbol (Microscope View)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroscopeModel {
    /// The file containing the symbol
    pub file_path: String,

    /// The symbol that was zoomed into
    pub symbol: Declaration,

    /// The fully-parsed body block (if the symbol has a body)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Block>,

    /// Surrounding context (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ContextWindow>,

    /// Source code of the symbol
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_text: Option<String>,
}

/// Surrounding context for a zoomed symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    /// Lines before the symbol
    pub before: Vec<String>,

    /// Lines after the symbol
    pub after: Vec<String>,
}

// ============================================================================
// Provider Trait
// ============================================================================

/// The core trait for AST providers
///
/// This trait defines the two primary operations:
/// 1. `index_project` - Planetarium view (project-wide scan)
/// 2. `zoom_into` - Microscope view (symbol deep-dive)
pub trait AstProvider: Send + Sync {
    /// Index an entire project, returning a Planetarium model
    ///
    /// This performs a shallow scan of all files, extracting:
    /// - Top-level declarations
    /// - Import statements
    /// - File-level comments
    ///
    /// # Arguments
    /// * `root` - Root directory to index
    /// * `options` - Indexing options
    fn index_project(&self, root: &Path, options: &IndexOptions) -> Result<PlanetariumModel>;

    /// Zoom into a specific symbol, returning a Microscope model
    ///
    /// This performs a deep parse of the target symbol, extracting:
    /// - Full body with nested blocks
    /// - Control flow structures
    /// - Function/method calls
    /// - Inline comments
    ///
    /// # Arguments
    /// * `file_path` - Path to the file containing the symbol
    /// * `symbol_id` - Identifier for the symbol (from Declaration::id())
    /// * `options` - Zoom options
    fn zoom_into(
        &self,
        file_path: &Path,
        symbol_id: &str,
        options: &ZoomOptions,
    ) -> Result<MicroscopeModel>;

    /// Parse a single file (used internally and for testing)
    ///
    /// # Arguments
    /// * `source` - Source code to parse
    /// * `language` - Language of the source
    fn parse_file(&self, source: &str, language: LanguageId) -> Result<File>;

    /// Get the list of supported languages
    fn supported_languages(&self) -> &[LanguageId];

    /// Check if a language is supported
    fn supports(&self, language: LanguageId) -> bool {
        self.supported_languages().contains(&language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planetarium_model_determinism() {
        let mut model = PlanetariumModel::new("/test");
        model.files.insert(
            "a.rs".to_string(),
            File::new("a.rs".to_string(), LanguageId::Rust),
        );
        model.files.insert(
            "b.py".to_string(),
            File::new("b.py".to_string(), LanguageId::Python),
        );

        let json1 = serde_json::to_string(&model).unwrap();
        let json2 = serde_json::to_string(&model).unwrap();
        assert_eq!(json1, json2, "Model serialization must be deterministic");
    }

    #[test]
    fn test_index_options_default() {
        let opts = IndexOptions::default();
        assert_eq!(opts.max_files, 0);
        assert!(!opts.extract_comments);
    }

    #[test]
    fn test_zoom_options_default() {
        let opts = ZoomOptions::default();
        assert!(opts.extract_calls);
        assert!(opts.extract_control_flow);
        assert_eq!(opts.max_depth, 10);
    }
}
