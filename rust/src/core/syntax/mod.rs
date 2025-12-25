//! Voyager Observatory - Core Syntax Infrastructure
//!
//! This module provides AST-level parsing capabilities using Tree-sitter,
//! enabling the Fractal Telescope to see beyond raw text into the
//! structural "DNA" of code.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    SyntaxProvider Trait                     │
//! │  ┌─────────────────────────────────────────────────────────┤
//! │  │  parse(source) -> NormalizedAst                         │
//! │  │  language() -> Language                                 │
//! │  │  apply_plugin_hook(hook) -> Result<()>  [Reserved]      │
//! │  └─────────────────────────────────────────────────────────┤
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!          ┌───────────────────┼───────────────────┐
//!          ▼                   ▼                   ▼
//! ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
//! │ TreeSitter      │ │ TreeSitter      │ │ TreeSitter      │
//! │ Rust Adapter    │ │ Python Adapter  │ │ TypeScript...   │
//! └─────────────────┘ └─────────────────┘ └─────────────────┘
//! ```
//!
//! # Supported Languages (25 Core)
//!
//! | Category | Languages |
//! |----------|-----------|
//! | Systems  | Rust, C, C++, Go |
//! | JVM      | Java, Kotlin, Scala |
//! | .NET     | C# |
//! | Scripting| Python, Ruby, PHP, Lua |
//! | Web      | JavaScript, TypeScript, HTML, CSS |
//! | Mobile   | Swift, Kotlin |
//! | Data     | JSON, YAML, TOML, SQL |
//! | DevOps   | Bash, HCL, Dockerfile |
//! | Docs     | Markdown |
//!
//! # Example
//!
//! ```rust,ignore
//! use voyager_observatory::core::syntax::{SyntaxRegistry, Language};
//!
//! let registry = SyntaxRegistry::new();
//! let ast = registry.parse("fn main() { println!(\"Hello\"); }", Language::Rust)?;
//!
//! for symbol in ast.symbols() {
//!     println!("Found: {} at line {}", symbol.name, symbol.location.line);
//! }
//! ```

mod adapter;
mod ast;

pub use adapter::{SyntaxRegistry, TreeSitterAdapter};
pub use ast::{
    NormalizedAst, Symbol, SymbolKind, SymbolVisibility, Import, ImportKind,
    Module, Scope, Location, Span, Parameter, ParseDiagnostic, DiagnosticSeverity,
};

use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during syntax analysis
#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Tree-sitter initialization failed: {0}")]
    InitializationError(String),

    #[error("Plugin hook error: {0}")]
    PluginHookError(String),
}

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    // Systems
    Rust,
    C,
    Cpp,
    Go,

    // JVM
    Java,
    Kotlin,
    Scala,

    // .NET
    CSharp,

    // Scripting
    Python,
    Ruby,
    Php,
    Lua,

    // Web
    JavaScript,
    TypeScript,
    Tsx,
    Html,
    Css,

    // Mobile (Swift uses same as systems, Kotlin above)
    Swift,

    // Data
    Json,
    Yaml,
    Toml,
    Sql,

    // DevOps
    Bash,
    Hcl,
    Dockerfile,

    // Docs
    Markdown,
}

impl Language {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            // Systems
            "rs" => Some(Language::Rust),
            "c" | "h" => Some(Language::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Language::Cpp),
            "go" => Some(Language::Go),

            // JVM
            "java" => Some(Language::Java),
            "kt" | "kts" => Some(Language::Kotlin),
            "scala" | "sc" => Some(Language::Scala),

            // .NET
            "cs" => Some(Language::CSharp),

            // Scripting
            "py" | "pyw" | "pyi" => Some(Language::Python),
            "rb" | "rake" | "gemspec" => Some(Language::Ruby),
            "php" | "phtml" => Some(Language::Php),
            "lua" => Some(Language::Lua),

            // Web
            "js" | "mjs" | "cjs" => Some(Language::JavaScript),
            "ts" | "mts" | "cts" => Some(Language::TypeScript),
            "tsx" => Some(Language::Tsx),
            "html" | "htm" => Some(Language::Html),
            "css" | "scss" | "sass" => Some(Language::Css),

            // Mobile
            "swift" => Some(Language::Swift),

            // Data
            "json" | "jsonc" => Some(Language::Json),
            "yaml" | "yml" => Some(Language::Yaml),
            "toml" => Some(Language::Toml),
            "sql" => Some(Language::Sql),

            // DevOps
            "sh" | "bash" | "zsh" => Some(Language::Bash),
            "tf" | "hcl" => Some(Language::Hcl),
            "dockerfile" => Some(Language::Dockerfile),

            // Docs
            "md" | "markdown" => Some(Language::Markdown),

            _ => None,
        }
    }

    /// Get the canonical file extension for this language
    pub fn extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Go => "go",
            Language::Java => "java",
            Language::Kotlin => "kt",
            Language::Scala => "scala",
            Language::CSharp => "cs",
            Language::Python => "py",
            Language::Ruby => "rb",
            Language::Php => "php",
            Language::Lua => "lua",
            Language::JavaScript => "js",
            Language::TypeScript => "ts",
            Language::Tsx => "tsx",
            Language::Html => "html",
            Language::Css => "css",
            Language::Swift => "swift",
            Language::Json => "json",
            Language::Yaml => "yaml",
            Language::Toml => "toml",
            Language::Sql => "sql",
            Language::Bash => "sh",
            Language::Hcl => "tf",
            Language::Dockerfile => "dockerfile",
            Language::Markdown => "md",
        }
    }

    /// Get human-readable language name
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::Scala => "Scala",
            Language::CSharp => "C#",
            Language::Python => "Python",
            Language::Ruby => "Ruby",
            Language::Php => "PHP",
            Language::Lua => "Lua",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Tsx => "TSX",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Swift => "Swift",
            Language::Json => "JSON",
            Language::Yaml => "YAML",
            Language::Toml => "TOML",
            Language::Sql => "SQL",
            Language::Bash => "Bash",
            Language::Hcl => "HCL",
            Language::Dockerfile => "Dockerfile",
            Language::Markdown => "Markdown",
        }
    }
}

/// Plugin hook definition (reserved for Phase 2 Lua ecosystem)
///
/// This structure defines the interface for external plugins to extend
/// syntax analysis capabilities. In Phase 2, Lua scripts will be able
/// to register hooks that modify or enhance the AST extraction process.
#[derive(Debug, Clone)]
pub struct PluginHook {
    /// Unique identifier for the hook
    pub id: String,

    /// Human-readable description
    pub description: String,

    /// Hook priority (lower = earlier execution)
    pub priority: i32,

    /// Reserved: Lua script path or inline code
    #[allow(dead_code)]
    lua_source: Option<String>,
}

/// The core trait for syntax analysis providers
///
/// This trait defines the interface that all syntax analyzers must implement.
/// The Tree-sitter adapter is the primary implementation, but the design
/// allows for alternative backends or custom analyzers.
pub trait SyntaxProvider: Send + Sync {
    /// Parse source code and extract a normalized AST
    ///
    /// # Arguments
    /// * `source` - The source code to parse
    /// * `language` - The programming language
    ///
    /// # Returns
    /// A `NormalizedAst` containing all extracted symbols, imports, and structure
    fn parse(&self, source: &str, language: Language) -> Result<NormalizedAst, SyntaxError>;

    /// Get the languages supported by this provider
    fn supported_languages(&self) -> &[Language];

    /// Check if a specific language is supported
    fn supports(&self, language: Language) -> bool {
        self.supported_languages().contains(&language)
    }

    /// Apply a plugin hook to modify parsing behavior
    ///
    /// # Phase 2 Reserved
    ///
    /// This method is reserved for the Phase 2 Lua plugin ecosystem.
    /// Currently returns `Ok(())` for all inputs.
    ///
    /// In Phase 2, hooks will be able to:
    /// - Add custom symbol extractors
    /// - Modify AST traversal order
    /// - Inject metadata into symbols
    /// - Filter or transform extracted data
    fn apply_plugin_hook(&mut self, _hook: PluginHook) -> Result<(), SyntaxError> {
        // Reserved for Phase 2 Lua ecosystem
        Ok(())
    }

    /// Get statistics about parsing performance
    fn stats(&self) -> ProviderStats {
        ProviderStats::default()
    }
}

/// Statistics about syntax provider performance
#[derive(Debug, Clone, Default)]
pub struct ProviderStats {
    /// Total files parsed
    pub files_parsed: usize,

    /// Total symbols extracted
    pub symbols_extracted: usize,

    /// Total parse time in milliseconds
    pub total_parse_time_ms: u64,

    /// Cache hit rate (0.0 - 1.0)
    pub cache_hit_rate: f64,

    /// Per-language statistics
    pub by_language: HashMap<Language, LanguageStats>,
}

/// Per-language parsing statistics
#[derive(Debug, Clone, Default)]
pub struct LanguageStats {
    pub files: usize,
    pub symbols: usize,
    pub avg_parse_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("tsx"), Some(Language::Tsx));
        assert_eq!(Language::from_extension("go"), Some(Language::Go));
        assert_eq!(Language::from_extension("java"), Some(Language::Java));
        assert_eq!(Language::from_extension("unknown"), None);
    }

    #[test]
    fn test_language_extension_roundtrip() {
        let languages = [
            Language::Rust,
            Language::Python,
            Language::TypeScript,
            Language::Go,
            Language::Java,
        ];

        for lang in languages {
            let ext = lang.extension();
            let recovered = Language::from_extension(ext);
            assert_eq!(recovered, Some(lang), "Roundtrip failed for {:?}", lang);
        }
    }

    #[test]
    fn test_language_names() {
        assert_eq!(Language::Rust.name(), "Rust");
        assert_eq!(Language::CSharp.name(), "C#");
        assert_eq!(Language::Cpp.name(), "C++");
        assert_eq!(Language::TypeScript.name(), "TypeScript");
    }

    #[test]
    fn test_plugin_hook_creation() {
        let hook = PluginHook {
            id: "test-hook".to_string(),
            description: "A test hook".to_string(),
            priority: 100,
            lua_source: None,
        };

        assert_eq!(hook.id, "test-hook");
        assert_eq!(hook.priority, 100);
    }
}
