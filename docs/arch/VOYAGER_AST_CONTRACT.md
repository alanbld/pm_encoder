# voyager-ast Architecture Contract v1

**Status:** Locked for Phase 1A
**Last Updated:** 2025-12-25

---

## Overview

This document defines the stable API contract and IR schema for `voyager-ast v1`. All implementations MUST conform to this contract. Breaking changes require a major version bump.

---

## IR v1 Schema

### Core Types

```rust
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
}

/// A contiguous region in source code
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: usize,

    /// End byte offset (exclusive)
    pub end: usize,

    /// Start line (1-indexed)
    pub start_line: usize,

    /// End line (1-indexed)
    pub end_line: usize,
}

/// A source region with optional language override (for embedded languages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub span: Span,
    pub language: Option<LanguageId>,
}

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
    pub signature_span: Option<Span>,

    /// Span of the body (for Zoom mode extraction)
    pub body_span: Option<Span>,

    /// Attached documentation comment
    pub doc_comment: Option<Comment>,

    /// Nested declarations (methods in class, etc.)
    pub children: Vec<Declaration>,

    /// Parameters (for functions/methods)
    pub parameters: Vec<Parameter>,

    /// Return type annotation (if present)
    pub return_type: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<String>,
    pub default_value: Option<String>,
    pub span: Span,
}

/// A code block (function body, if body, loop body, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// The block's span
    pub span: Span,

    /// Nested control flow structures
    pub control_flow: Vec<ControlFlow>,

    /// Function/method calls within this block
    pub calls: Vec<Call>,

    /// Comments within this block
    pub comments: Vec<Comment>,

    /// Unknown/unparsed regions within the block
    pub unknown_regions: Vec<UnknownNode>,
}

/// Control flow constructs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFlow {
    pub kind: ControlFlowKind,
    pub span: Span,
    /// The condition expression span (if applicable)
    pub condition_span: Option<Span>,
    /// Child blocks (then/else branches, loop body, match arms)
    pub branches: Vec<Block>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
}

/// Import, require, include, using, or module reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportLike {
    /// What is being imported
    pub source: String,

    /// Kind of import
    pub kind: ImportKind,

    /// Specific items imported (for selective imports)
    pub items: Vec<String>,

    /// Alias if renamed
    pub alias: Option<String>,

    /// Whether this is a type-only import
    pub type_only: bool,

    pub span: Span,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImportKind {
    /// import x from 'y'
    Import,
    /// require('x')
    Require,
    /// #include <x>
    Include,
    /// using namespace x
    Using,
    /// mod x;
    Module,
    /// from x import y
    From,
    Other,
}

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
    pub attached_to: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommentKind {
    /// Single-line comment (// or #)
    Line,
    /// Multi-line block comment (/* */)
    Block,
    /// Documentation comment (/// or /** */)
    Doc,
}

/// A region that couldn't be parsed or is syntactically invalid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownNode {
    pub span: Span,
    /// Optional description of why this region is unknown
    pub reason: Option<String>,
    /// The raw text of the region (for debugging)
    pub raw_text: Option<String>,
}

/// An unparsed block (larger region with syntax errors)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnparsedBlock {
    pub span: Span,
    pub reason: String,
}

/// Language identifier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LanguageId {
    Rust,
    Python,
    TypeScript,
    JavaScript,
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
```

---

## API Contract

### Core Functions

```rust
/// Options for project indexing
#[derive(Debug, Clone, Default)]
pub struct IndexOptions {
    /// Maximum files to process (0 = unlimited)
    pub max_files: usize,

    /// File patterns to include
    pub include_patterns: Vec<String>,

    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,

    /// Whether to extract doc comments
    pub extract_comments: bool,

    /// Whether to follow symbolic links
    pub follow_symlinks: bool,
}

/// Options for symbol zoom
#[derive(Debug, Clone, Default)]
pub struct ZoomOptions {
    /// Maximum depth for nested blocks
    pub max_depth: usize,

    /// Whether to extract calls
    pub extract_calls: bool,

    /// Whether to extract control flow
    pub extract_control_flow: bool,

    /// Include surrounding context lines
    pub context_lines: usize,
}

/// The result of indexing a project (Planetarium View)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetariumModel {
    /// Root path of the indexed project
    pub root: String,

    /// All indexed files, keyed by relative path
    pub files: BTreeMap<String, File>,

    /// Statistics about the indexing run
    pub stats: IndexStats,

    /// Errors encountered during indexing
    pub errors: Vec<IndexError>,
}

/// Statistics from an indexing run
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexStats {
    pub files_processed: usize,
    pub files_skipped: usize,
    pub declarations_found: usize,
    pub imports_found: usize,
    pub unknown_regions: usize,
    pub parse_time_ms: u64,
}

/// An error during indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexError {
    pub path: String,
    pub message: String,
    pub recoverable: bool,
}

/// The result of zooming into a symbol (Microscope View)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroscopeModel {
    /// The file containing the symbol
    pub file_path: String,

    /// The symbol that was zoomed into
    pub symbol: Declaration,

    /// The fully-parsed body block
    pub body: Option<Block>,

    /// Surrounding context (if requested)
    pub context: Option<ContextWindow>,
}

/// Surrounding context for a zoomed symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    /// Lines before the symbol
    pub before: Vec<String>,

    /// Lines after the symbol
    pub after: Vec<String>,
}
```

### Provider Trait

```rust
/// The core trait for AST providers
pub trait AstProvider: Send + Sync {
    /// Index an entire project, returning a Planetarium model
    fn index_project(
        &self,
        root: &Path,
        options: &IndexOptions
    ) -> Result<PlanetariumModel, AstError>;

    /// Zoom into a specific symbol, returning a Microscope model
    fn zoom_into(
        &self,
        file_path: &Path,
        symbol_id: &str,
        options: &ZoomOptions,
    ) -> Result<MicroscopeModel, AstError>;

    /// Parse a single file (used internally)
    fn parse_file(
        &self,
        source: &str,
        language: LanguageId,
    ) -> Result<File, AstError>;

    /// Get supported languages
    fn supported_languages(&self) -> &[LanguageId];
}

/// Errors from AST operations
#[derive(Debug, Clone)]
pub enum AstError {
    /// File not found
    FileNotFound(String),

    /// Symbol not found in file
    SymbolNotFound { file: String, symbol: String },

    /// Language not supported
    UnsupportedLanguage(LanguageId),

    /// Parse error (but partial results may be available)
    ParseError { message: String, partial: Option<File> },

    /// I/O error
    IoError(String),
}
```

---

## Determinism Guarantees

1. **Ordered Collections**: All maps and sets use `BTreeMap`/`BTreeSet` for deterministic iteration order.

2. **Stable Serialization**: JSON output is sorted by keys and uses consistent formatting.

3. **No Non-Deterministic Dependencies**: No random number generators, no current-time dependencies in IR generation.

4. **Reproducible Results**: Given the same source file and options, the output IR is byte-identical across runs and platforms.

---

## Comment Attachment Heuristic

The "Nearest Preceding Node" heuristic for attaching comments to declarations:

1. A comment is attached to the **next declaration** if:
   - It appears on the line immediately before the declaration, OR
   - It appears on the same line as the declaration

2. Doc comments (`///`, `/** */`) are always attached to the following declaration.

3. Comments more than 1 blank line before a declaration remain unattached.

4. Trailing comments (on the same line after a statement) attach to that statement.

---

## Language Adapter Contract

Each language adapter must implement:

```rust
pub trait LanguageAdapter: Send + Sync {
    /// The language this adapter handles
    fn language(&self) -> LanguageId;

    /// Get the Tree-sitter language
    fn tree_sitter_language(&self) -> tree_sitter::Language;

    /// Extract declarations from a parse tree (Index mode)
    fn extract_declarations(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<Declaration>;

    /// Extract imports from a parse tree
    fn extract_imports(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Vec<ImportLike>;

    /// Extract the body of a declaration (Zoom mode)
    fn extract_body(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        declaration: &Declaration,
    ) -> Option<Block>;

    /// Determine visibility from a node
    fn extract_visibility(
        &self,
        node: &tree_sitter::Node,
        source: &str,
    ) -> Visibility;
}
```

---

## WASM Compatibility

The `voyager-ast` crate MUST be compatible with `wasm32-unknown-unknown`:

1. **No File I/O**: All file reading is external; the crate receives `&str` source.
2. **No std::fs**: Use feature flags to disable any file system access.
3. **No System Calls**: No spawning processes, no environment access.
4. **Memory-Bounded**: Single-file parsing uses bounded memory.

---

## Versioning

- **IR Schema**: `v1` – frozen for this document
- **API Surface**: `v1` – backward compatible changes only in 1.x
- **Adapters**: May be versioned independently

Breaking changes require:
1. Major version bump
2. Migration guide
3. Deprecation period for old APIs

---
