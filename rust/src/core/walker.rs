//! Directory traversal for pm_encoder
//!
//! This module provides the FileWalker trait and default implementation
//! for walking directory trees and discovering files.

use crate::core::error::{EncoderError, Result};
use crate::core::models::FileEntry;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;
use std::time::SystemTime;

#[cfg(test)]
use mockall::automock;

/// Trait for file system walking
///
/// This trait allows for mocking in tests and alternative implementations
/// (e.g., virtual file systems, remote sources).
#[cfg_attr(test, automock)]
pub trait FileWalker: Send + Sync {
    /// Walk a directory and return file entries
    fn walk(&self, root: &str, config: &WalkConfig) -> Result<Vec<FileEntry>>;

    /// Check if a path matches ignore patterns
    fn should_ignore(&self, path: &str, patterns: &[String]) -> bool;

    /// Check if a file is too large
    fn is_too_large(&self, size: u64, limit: u64) -> bool {
        size > limit
    }
}

/// Configuration for directory walking
#[derive(Debug, Clone)]
pub struct WalkConfig {
    /// Patterns to ignore
    pub ignore_patterns: Vec<String>,
    /// Patterns to include (empty = all)
    pub include_patterns: Vec<String>,
    /// Maximum file size in bytes
    pub max_file_size: u64,
}

impl Default for WalkConfig {
    fn default() -> Self {
        Self {
            ignore_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "__pycache__".to_string(),
                "*.pyc".to_string(),
                ".DS_Store".to_string(),
                "target".to_string(),
            ],
            include_patterns: vec![],
            max_file_size: 1_048_576,
        }
    }
}

/// Default file walker implementation
pub struct DefaultWalker;

impl DefaultWalker {
    /// Create a new DefaultWalker
    pub fn new() -> Self {
        Self
    }

    /// Build a GlobSet from patterns
    fn build_globset(patterns: &[String]) -> Option<GlobSet> {
        if patterns.is_empty() {
            return None;
        }

        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }
        builder.build().ok()
    }

    /// Check if path matches any pattern
    fn matches_patterns(path: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            // Check for exact match
            if path == pattern {
                return true;
            }

            // Check for directory component match
            for component in path.split('/') {
                if component == pattern {
                    return true;
                }
            }

            // Check for glob match
            if let Ok(glob) = Glob::new(pattern) {
                if let Ok(matcher) = glob.compile_matcher().try_into() {
                    let matcher: globset::GlobMatcher = matcher;
                    if matcher.is_match(path) {
                        return true;
                    }
                }
            }

            // Check for prefix match (directory)
            if path.starts_with(&format!("{}/", pattern)) {
                return true;
            }
        }
        false
    }
}

impl Default for DefaultWalker {
    fn default() -> Self {
        Self::new()
    }
}

impl FileWalker for DefaultWalker {
    fn walk(&self, root: &str, config: &WalkConfig) -> Result<Vec<FileEntry>> {
        let root_path = Path::new(root);
        if !root_path.exists() {
            return Err(EncoderError::DirectoryNotFound {
                path: root_path.to_path_buf(),
            });
        }
        if !root_path.is_dir() {
            return Err(EncoderError::invalid_config(format!(
                "'{}' is not a directory",
                root
            )));
        }

        let include_set = Self::build_globset(&config.include_patterns);
        let mut entries = Vec::new();

        for entry in walkdir::WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            // Skip directories
            if entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path();
            let relative_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Skip ignored files
            if self.should_ignore(&relative_path, &config.ignore_patterns) {
                continue;
            }

            // Check include patterns if specified
            if let Some(ref include_set) = include_set {
                if !include_set.is_match(&relative_path) {
                    continue;
                }
            }

            // Check file size
            let metadata = entry.metadata().ok();
            if let Some(ref meta) = metadata {
                if self.is_too_large(meta.len(), config.max_file_size) {
                    continue;
                }
            }

            // Read file content
            let bytes = match std::fs::read(path) {
                Ok(b) => b,
                Err(_) => continue,
            };

            // Skip binary files
            if is_binary(&bytes) {
                continue;
            }

            // Convert to string
            let content = match read_file_content(&bytes) {
                Some(c) => c,
                None => continue,
            };

            // Get timestamps
            let (mtime, ctime) = metadata
                .map(|m| {
                    let mtime = m.modified()
                        .ok()
                        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    let ctime = m.created()
                        .ok()
                        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(mtime);
                    (mtime, ctime)
                })
                .unwrap_or((0, 0));

            entries.push(FileEntry::new(&relative_path, content).with_timestamps(mtime, ctime));
        }

        Ok(entries)
    }

    fn should_ignore(&self, path: &str, patterns: &[String]) -> bool {
        Self::matches_patterns(path, patterns)
    }
}

/// Check if content appears to be binary
pub fn is_binary(content: &[u8]) -> bool {
    // Empty is not binary
    if content.is_empty() {
        return false;
    }

    // Check first 8KB for null bytes (common binary indicator)
    let check_len = content.len().min(8192);
    content[..check_len].contains(&0)
}

/// Read file content, handling encoding
pub fn read_file_content(bytes: &[u8]) -> Option<String> {
    // Try UTF-8 first
    if let Ok(s) = std::str::from_utf8(bytes) {
        // Normalize line endings
        return Some(s.replace("\r\n", "\n"));
    }

    // Try lossy conversion
    let s = String::from_utf8_lossy(bytes);
    if s.chars().filter(|c| *c == '\u{FFFD}').count() < s.len() / 10 {
        Some(s.replace("\r\n", "\n"))
    } else {
        None // Too many replacement characters, likely binary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_walk_config_default() {
        let config = WalkConfig::default();
        assert!(config.ignore_patterns.contains(&".git".to_string()));
        assert_eq!(config.max_file_size, 1_048_576);
    }

    #[test]
    fn test_is_binary_empty() {
        assert!(!is_binary(&[]));
    }

    #[test]
    fn test_is_binary_with_null() {
        assert!(is_binary(&[0x00, 0x01, 0x02]));
    }

    #[test]
    fn test_is_binary_text() {
        assert!(!is_binary(b"Hello, world!"));
    }

    #[test]
    fn test_read_file_content_utf8() {
        let content = read_file_content(b"Hello, world!");
        assert_eq!(content, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_read_file_content_crlf() {
        let content = read_file_content(b"line1\r\nline2");
        assert_eq!(content, Some("line1\nline2".to_string()));
    }

    #[test]
    fn test_default_walker_nonexistent() {
        let walker = DefaultWalker::new();
        let config = WalkConfig::default();
        let result = walker.walk("/nonexistent/path/xyz", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_walker_walk() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, world!").unwrap();

        let walker = DefaultWalker::new();
        let config = WalkConfig::default();
        let entries = walker.walk(temp_dir.path().to_str().unwrap(), &config).unwrap();

        assert_eq!(entries.len(), 1);
        assert!(entries[0].path.ends_with("test.txt"));
        assert_eq!(entries[0].content, "Hello, world!");
    }

    #[test]
    fn test_should_ignore() {
        let walker = DefaultWalker::new();
        assert!(walker.should_ignore(".git/config", &vec![".git".to_string()]));
        assert!(walker.should_ignore("node_modules/pkg/index.js", &vec!["node_modules".to_string()]));
        assert!(!walker.should_ignore("src/main.rs", &vec![".git".to_string()]));
    }

    #[test]
    fn test_is_too_large() {
        let walker = DefaultWalker::new();
        assert!(walker.is_too_large(2_000_000, 1_000_000));
        assert!(!walker.is_too_large(500_000, 1_000_000));
    }

    #[test]
    fn test_matches_patterns_glob() {
        assert!(DefaultWalker::matches_patterns("test.pyc", &vec!["*.pyc".to_string()]));
        assert!(!DefaultWalker::matches_patterns("test.py", &vec!["*.pyc".to_string()]));
    }
}
