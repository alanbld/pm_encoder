//! Core IR (Intermediate Representation) Types
//!
//! This module defines the language-agnostic structural model used by
//! voyager-ast. All types are designed for:
//!
//! 1. **Determinism**: Using BTreeMap/BTreeSet for ordered iteration
//! 2. **Serialization**: Full serde support for caching and export
//! 3. **Error Tolerance**: UnknownNode/UnparsedBlock for graceful degradation

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ============================================================================
// Language Identification
// ============================================================================

/// Language identifier for source files
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LanguageId {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Tsx,
    Jsx,
    Abl,
    C,
    Cpp,
    Java,
    Go,
    Ruby,
    Php,
    CSharp,
    Swift,
    Kotlin,
    Scala,
    Html,
    Css,
    Json,
    Yaml,
    Toml,
    Markdown,
    Bash,
    Sql,
    Unknown,
}

impl LanguageId {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Self::Rust,
            "py" | "pyw" | "pyi" => Self::Python,
            "ts" | "mts" | "cts" => Self::TypeScript,
            "tsx" => Self::Tsx,
            "js" | "mjs" | "cjs" => Self::JavaScript,
            "jsx" => Self::Jsx,
            "p" | "i" | "w" | "cls" => Self::Abl,
            "c" | "h" => Self::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => Self::Cpp,
            "java" => Self::Java,
            "go" => Self::Go,
            "rb" | "rake" | "gemspec" => Self::Ruby,
            "php" | "phtml" => Self::Php,
            "cs" => Self::CSharp,
            "swift" => Self::Swift,
            "kt" | "kts" => Self::Kotlin,
            "scala" | "sc" => Self::Scala,
            "html" | "htm" => Self::Html,
            "css" | "scss" | "sass" => Self::Css,
            "json" | "jsonc" => Self::Json,
            "yaml" | "yml" => Self::Yaml,
            "toml" => Self::Toml,
            "md" | "markdown" => Self::Markdown,
            "sh" | "bash" | "zsh" | "ksh" => Self::Bash,
            "sql" => Self::Sql,
            _ => Self::Unknown,
        }
    }

    /// Get canonical file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Rust => "rs",
            Self::Python => "py",
            Self::TypeScript => "ts",
            Self::Tsx => "tsx",
            Self::JavaScript => "js",
            Self::Jsx => "jsx",
            Self::Abl => "p",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Java => "java",
            Self::Go => "go",
            Self::Ruby => "rb",
            Self::Php => "php",
            Self::CSharp => "cs",
            Self::Swift => "swift",
            Self::Kotlin => "kt",
            Self::Scala => "scala",
            Self::Html => "html",
            Self::Css => "css",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Markdown => "md",
            Self::Bash => "sh",
            Self::Sql => "sql",
            Self::Unknown => "",
        }
    }

    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::TypeScript => "TypeScript",
            Self::Tsx => "TSX",
            Self::JavaScript => "JavaScript",
            Self::Jsx => "JSX",
            Self::Abl => "ABL",
            Self::C => "C",
            Self::Cpp => "C++",
            Self::Java => "Java",
            Self::Go => "Go",
            Self::Ruby => "Ruby",
            Self::Php => "PHP",
            Self::CSharp => "C#",
            Self::Swift => "Swift",
            Self::Kotlin => "Kotlin",
            Self::Scala => "Scala",
            Self::Html => "HTML",
            Self::Css => "CSS",
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Toml => "TOML",
            Self::Markdown => "Markdown",
            Self::Bash => "Bash",
            Self::Sql => "SQL",
            Self::Unknown => "Unknown",
        }
    }
}

// ============================================================================
// Span and Region
// ============================================================================

/// A contiguous region in source code
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: usize,

    /// End byte offset (exclusive)
    pub end: usize,

    /// Start line (1-indexed)
    pub start_line: usize,

    /// End line (1-indexed)
    pub end_line: usize,

    /// Start column (0-indexed, in bytes)
    pub start_column: usize,

    /// End column (0-indexed, in bytes)
    pub end_column: usize,
}

impl Span {
    /// Create a new span
    pub fn new(start: usize, end: usize, start_line: usize, end_line: usize) -> Self {
        Self {
            start,
            end,
            start_line,
            end_line,
            start_column: 0,
            end_column: 0,
        }
    }

    /// Check if this span contains a byte offset
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Check if this span contains a line number
    pub fn contains_line(&self, line: usize) -> bool {
        line >= self.start_line && line <= self.end_line
    }

    /// Get the length in bytes
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// A source region with optional language override (for embedded languages)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Region {
    pub span: Span,
    pub language: Option<LanguageId>,
}

// ============================================================================
// File
// ============================================================================

/// A parsed source file with its structural elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    /// Canonical path to the file
    pub path: String,

    /// Detected language identifier
    pub language: LanguageId,

    /// Top-level declarations (functions, classes, structs, etc.)
    pub declarations: Vec<Declaration>,

    /// Import statements
    pub imports: Vec<ImportLike>,

    /// File-level and attached comments
    pub comments: Vec<Comment>,

    /// Regions that couldn't be parsed
    pub unknown_regions: Vec<UnknownNode>,

    /// Byte range of the entire file
    pub span: Span,

    /// Additional metadata (BTreeMap for determinism)
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

impl File {
    /// Create a new empty file
    pub fn new(path: String, language: LanguageId) -> Self {
        Self {
            path,
            language,
            declarations: Vec::new(),
            imports: Vec::new(),
            comments: Vec::new(),
            unknown_regions: Vec::new(),
            span: Span::default(),
            metadata: BTreeMap::new(),
        }
    }

    /// Check if the file has any parse errors
    pub fn has_errors(&self) -> bool {
        !self.unknown_regions.is_empty()
    }

    /// Get the total number of declarations (including nested)
    pub fn total_declarations(&self) -> usize {
        fn count_nested(decls: &[Declaration]) -> usize {
            decls.iter().map(|d| 1 + count_nested(&d.children)).sum()
        }
        count_nested(&self.declarations)
    }
}

// ============================================================================
// Declaration
// ============================================================================

/// A named declaration (function, class, struct, type, constant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Declaration {
    /// The declaration's name
    pub name: String,

    /// What kind of declaration this is
    pub kind: DeclarationKind,

    /// Visibility (public, private, etc.)
    pub visibility: Visibility,

    /// The full span of the declaration
    pub span: Span,

    /// Span of just the signature/header (for display)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_span: Option<Span>,

    /// Span of the body (for Zoom mode extraction)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_span: Option<Span>,

    /// Attached documentation comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_comment: Option<Comment>,

    /// Nested declarations (methods in class, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Declaration>,

    /// Parameters (for functions/methods)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,

    /// Return type annotation (if present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,

    /// Additional metadata
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

impl Declaration {
    /// Create a new declaration
    pub fn new(name: String, kind: DeclarationKind, span: Span) -> Self {
        Self {
            name,
            kind,
            visibility: Visibility::Unknown,
            span,
            signature_span: None,
            body_span: None,
            doc_comment: None,
            children: Vec::new(),
            parameters: Vec::new(),
            return_type: None,
            metadata: BTreeMap::new(),
        }
    }

    /// Create a unique identifier for this declaration
    pub fn id(&self) -> String {
        format!("{}:{}:{}", self.kind.as_str(), self.name, self.span.start_line)
    }
}

/// Kind of declaration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeclarationKind {
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Interface,
    Trait,
    Type,
    Constant,
    Variable,
    Module,
    Namespace,
    Impl,
    Macro,
    Other,
}

impl DeclarationKind {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Method => "method",
            Self::Class => "class",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Interface => "interface",
            Self::Trait => "trait",
            Self::Type => "type",
            Self::Constant => "constant",
            Self::Variable => "variable",
            Self::Module => "module",
            Self::Namespace => "namespace",
            Self::Impl => "impl",
            Self::Macro => "macro",
            Self::Other => "other",
        }
    }
}

/// Visibility of a declaration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    #[default]
    Unknown,
}

/// A function/method parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_annotation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    pub span: Span,
}

// ============================================================================
// Block and Control Flow
// ============================================================================

/// A code block (function body, if body, loop body, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Block {
    /// The block's span
    pub span: Span,

    /// Nested control flow structures
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_flow: Vec<ControlFlow>,

    /// Function/method calls within this block
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<Call>,

    /// Comments within this block
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<Comment>,

    /// Unknown/unparsed regions within the block
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknown_regions: Vec<UnknownNode>,

    /// Nested declarations (lambdas, inner functions, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nested_declarations: Vec<Declaration>,
}

/// Control flow constructs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFlow {
    pub kind: ControlFlowKind,
    pub span: Span,
    /// The condition expression span (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_span: Option<Span>,
    /// Child blocks (then/else branches, loop body, match arms)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<Block>,
}

/// Kind of control flow
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ControlFlowKind {
    If,
    Else,
    ElseIf,
    Match,
    Switch,
    For,
    While,
    Loop,
    Try,
    Catch,
    Finally,
    With,
    Return,
    Break,
    Continue,
    Other,
}

/// A function or method call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    /// The callee expression (function name, method chain, etc.)
    pub callee: String,

    /// Span of the entire call expression
    pub span: Span,

    /// Number of arguments
    pub argument_count: usize,

    /// Whether this is a method call
    #[serde(default)]
    pub is_method: bool,
}

// ============================================================================
// Import
// ============================================================================

/// Import, require, include, using, or module reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportLike {
    /// What is being imported (module path, file, etc.)
    pub source: String,

    /// Kind of import
    pub kind: ImportKind,

    /// Specific items imported (for selective imports)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,

    /// Alias if renamed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,

    /// Whether this is a type-only import
    #[serde(default)]
    pub type_only: bool,

    pub span: Span,
}

/// Kind of import statement
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImportKind {
    /// import x from 'y' or import 'y'
    Import,
    /// require('x')
    Require,
    /// #include <x> or #include "x"
    Include,
    /// using namespace x
    Using,
    /// mod x; or mod x { }
    Module,
    /// from x import y
    From,
    /// use x::y
    Use,
    Other,
}

// ============================================================================
// Comment
// ============================================================================

/// A comment in source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// The comment text (without delimiters)
    pub text: String,

    /// Kind of comment
    pub kind: CommentKind,

    pub span: Span,

    /// The declaration this comment is attached to (if any)
    /// Uses "Nearest Preceding Node" heuristic
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attached_to: Option<String>,
}

/// Kind of comment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CommentKind {
    /// Single-line comment (// or #)
    Line,
    /// Multi-line block comment (/* */)
    Block,
    /// Documentation comment (/// or /** */)
    Doc,
}

// ============================================================================
// Error Recovery Types
// ============================================================================

/// A region that couldn't be parsed or is syntactically invalid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownNode {
    pub span: Span,
    /// Optional description of why this region is unknown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The raw text of the region (for debugging, may be truncated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_text: Option<String>,
}

/// An unparsed block (larger region with syntax errors)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnparsedBlock {
    pub span: Span,
    pub reason: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(LanguageId::from_extension("rs"), LanguageId::Rust);
        assert_eq!(LanguageId::from_extension("py"), LanguageId::Python);
        assert_eq!(LanguageId::from_extension("ts"), LanguageId::TypeScript);
        assert_eq!(LanguageId::from_extension("tsx"), LanguageId::Tsx);
        assert_eq!(LanguageId::from_extension("p"), LanguageId::Abl);
        assert_eq!(LanguageId::from_extension("xyz"), LanguageId::Unknown);
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(10, 20, 1, 2);
        assert!(span.contains(10));
        assert!(span.contains(15));
        assert!(!span.contains(20)); // exclusive end
        assert!(!span.contains(5));
    }

    #[test]
    fn test_declaration_id() {
        let decl = Declaration::new(
            "my_function".to_string(),
            DeclarationKind::Function,
            Span::new(0, 100, 5, 10),
        );
        assert_eq!(decl.id(), "function:my_function:5");
    }

    #[test]
    fn test_file_serialization_deterministic() {
        let file = File::new("test.rs".to_string(), LanguageId::Rust);
        let json1 = serde_json::to_string(&file).unwrap();
        let json2 = serde_json::to_string(&file).unwrap();
        assert_eq!(json1, json2, "Serialization must be deterministic");
    }
}
