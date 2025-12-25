//! Error types for voyager-ast
//!
//! Following the "Telescope, Not Compiler" philosophy, many errors are
//! recoverable and result in partial output rather than total failure.

use crate::ir::{File, LanguageId};
use thiserror::Error;

/// Errors from AST operations
#[derive(Error, Debug, Clone)]
pub enum AstError {
    /// File not found or couldn't be read
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Symbol not found in file
    #[error("Symbol '{symbol}' not found in file '{file}'")]
    SymbolNotFound { file: String, symbol: String },

    /// Language not supported by any adapter
    #[error("Unsupported language: {0:?}")]
    UnsupportedLanguage(LanguageId),

    /// Parse error occurred, but partial results may be available
    #[error("Parse error: {message}")]
    ParseError {
        message: String,
        /// Partial parse result (if any structure could be recovered)
        partial: Option<Box<File>>,
    },

    /// I/O error during file operations
    #[error("I/O error: {0}")]
    IoError(String),

    /// Invalid configuration or options
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Tree-sitter specific error
    #[error("Tree-sitter error: {0}")]
    TreeSitterError(String),

    /// Internal error (should not happen in normal operation)
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl AstError {
    /// Check if this error has partial results available
    pub fn has_partial(&self) -> bool {
        matches!(self, AstError::ParseError { partial: Some(_), .. })
    }

    /// Extract partial results if available
    pub fn take_partial(self) -> Option<File> {
        match self {
            AstError::ParseError { partial: Some(file), .. } => Some(*file),
            _ => None,
        }
    }

    /// Create a parse error with partial results
    pub fn parse_error_with_partial(message: impl Into<String>, file: File) -> Self {
        AstError::ParseError {
            message: message.into(),
            partial: Some(Box::new(file)),
        }
    }

    /// Create a simple parse error without partial results
    pub fn parse_error(message: impl Into<String>) -> Self {
        AstError::ParseError {
            message: message.into(),
            partial: None,
        }
    }
}

/// Result type alias for AstError
pub type Result<T> = std::result::Result<T, AstError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Span;

    #[test]
    fn test_error_display() {
        let err = AstError::FileNotFound("test.rs".to_string());
        assert!(err.to_string().contains("test.rs"));

        let err = AstError::SymbolNotFound {
            file: "main.rs".to_string(),
            symbol: "foo".to_string(),
        };
        assert!(err.to_string().contains("foo"));
        assert!(err.to_string().contains("main.rs"));
    }

    #[test]
    fn test_partial_results() {
        let file = File::new("test.rs".to_string(), LanguageId::Rust);
        let err = AstError::parse_error_with_partial("syntax error", file);

        assert!(err.has_partial());
        let partial = err.take_partial().unwrap();
        assert_eq!(partial.path, "test.rs");
    }

    #[test]
    fn test_no_partial() {
        let err = AstError::parse_error("syntax error");
        assert!(!err.has_partial());
        assert!(err.take_partial().is_none());
    }
}
