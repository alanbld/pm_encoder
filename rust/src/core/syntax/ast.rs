//! Normalized AST - Language-Agnostic Code Representation
//!
//! This module defines a unified data model for representing code structure
//! across all 25 supported languages. The NormalizedAst serves as the
//! "lingua franca" between Tree-sitter parsers and the Voyager Observatory.
//!
//! # Design Philosophy
//!
//! Different languages have wildly different syntax and semantics:
//! - Rust has traits, Python has decorators, TypeScript has interfaces
//! - Some languages use classes, others use modules, others use both
//! - Visibility rules differ (public/private vs. export/import)
//!
//! The NormalizedAst abstracts these differences into a common vocabulary:
//! - **Symbol**: Any named code entity (function, class, variable, etc.)
//! - **Import**: Any dependency on external code
//! - **Module**: Any grouping of related symbols
//! - **Scope**: The visibility/accessibility of a symbol

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A language-agnostic representation of parsed source code
///
/// This is the primary output of syntax analysis. It contains all
/// extracted symbols, imports, and structural information in a
/// normalized format that can be processed uniformly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormalizedAst {
    /// All symbols found in the source
    pub symbols: Vec<Symbol>,

    /// All import/require statements
    pub imports: Vec<Import>,

    /// Module structure (for languages with explicit modules)
    pub modules: Vec<Module>,

    /// File-level documentation
    pub doc_comment: Option<String>,

    /// Language-specific metadata
    pub metadata: HashMap<String, String>,

    /// Parse errors (non-fatal)
    pub errors: Vec<ParseDiagnostic>,
}

impl NormalizedAst {
    /// Create an empty AST
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all symbols of a specific kind
    pub fn symbols_of_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.symbols.iter().filter(|s| s.kind == kind).collect()
    }

    /// Get all public/exported symbols
    pub fn public_symbols(&self) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| matches!(s.visibility, SymbolVisibility::Public | SymbolVisibility::Export))
            .collect()
    }

    /// Get all functions (including methods)
    pub fn functions(&self) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| matches!(s.kind, SymbolKind::Function | SymbolKind::Method))
            .collect()
    }

    /// Get all classes/structs/types
    pub fn types(&self) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| {
                matches!(
                    s.kind,
                    SymbolKind::Class
                        | SymbolKind::Struct
                        | SymbolKind::Interface
                        | SymbolKind::Trait
                        | SymbolKind::Enum
                        | SymbolKind::TypeAlias
                )
            })
            .collect()
    }

    /// Find a symbol by name (first match)
    pub fn find_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|s| s.name == name)
    }

    /// Find all symbols matching a pattern
    pub fn find_symbols(&self, pattern: &str) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| s.name.contains(pattern))
            .collect()
    }

    /// Get the total line count covered by symbols
    pub fn symbol_line_coverage(&self) -> usize {
        self.symbols
            .iter()
            .filter_map(|s| s.span.as_ref())
            .map(|span| span.end_line.saturating_sub(span.start_line) + 1)
            .sum()
    }

    /// Check if the AST has any parse errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Merge another AST into this one
    pub fn merge(&mut self, other: NormalizedAst) {
        self.symbols.extend(other.symbols);
        self.imports.extend(other.imports);
        self.modules.extend(other.modules);
        self.errors.extend(other.errors);

        for (key, value) in other.metadata {
            self.metadata.entry(key).or_insert(value);
        }
    }
}

/// A code symbol (function, class, variable, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// The symbol's name
    pub name: String,

    /// What kind of symbol this is
    pub kind: SymbolKind,

    /// Visibility/accessibility
    pub visibility: SymbolVisibility,

    /// Location in source file
    pub location: Location,

    /// Full span of the symbol (start to end)
    pub span: Option<Span>,

    /// Documentation comment
    pub doc_comment: Option<String>,

    /// Parent symbol (for nested items)
    pub parent: Option<String>,

    /// Child symbols (for containers like classes)
    pub children: Vec<String>,

    /// Type signature (if available)
    pub signature: Option<String>,

    /// Function parameters (for functions/methods)
    pub parameters: Vec<Parameter>,

    /// Return type (for functions/methods)
    pub return_type: Option<String>,

    /// Decorators/attributes (Python decorators, Rust attributes, etc.)
    pub decorators: Vec<String>,

    /// Generic type parameters
    pub type_parameters: Vec<String>,

    /// Language-specific metadata
    pub metadata: HashMap<String, String>,
}

impl Symbol {
    /// Create a new symbol with minimal information
    pub fn new(name: impl Into<String>, kind: SymbolKind, location: Location) -> Self {
        Self {
            name: name.into(),
            kind,
            visibility: SymbolVisibility::default(),
            location,
            span: None,
            doc_comment: None,
            parent: None,
            children: Vec::new(),
            signature: None,
            parameters: Vec::new(),
            return_type: None,
            decorators: Vec::new(),
            type_parameters: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Check if this symbol is a container (can have children)
    pub fn is_container(&self) -> bool {
        matches!(
            self.kind,
            SymbolKind::Class
                | SymbolKind::Struct
                | SymbolKind::Interface
                | SymbolKind::Trait
                | SymbolKind::Module
                | SymbolKind::Namespace
                | SymbolKind::Enum
        )
    }

    /// Check if this symbol is callable
    pub fn is_callable(&self) -> bool {
        matches!(
            self.kind,
            SymbolKind::Function | SymbolKind::Method | SymbolKind::Constructor
        )
    }

    /// Get the fully qualified name (parent.name)
    pub fn qualified_name(&self) -> String {
        match &self.parent {
            Some(parent) => format!("{}.{}", parent, self.name),
            None => self.name.clone(),
        }
    }
}

/// The type of a code symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    // Callables
    Function,
    Method,
    Constructor,
    Lambda,

    // Types
    Class,
    Struct,
    Interface,
    Trait,
    Enum,
    TypeAlias,

    // Data
    Variable,
    Constant,
    Field,
    Property,
    EnumVariant,

    // Containers
    Module,
    Namespace,
    Package,

    // Other
    Macro,
    Decorator,
    Attribute,
    Unknown,
}

impl SymbolKind {
    /// Get a short label for this kind
    pub fn label(&self) -> &'static str {
        match self {
            SymbolKind::Function => "fn",
            SymbolKind::Method => "method",
            SymbolKind::Constructor => "ctor",
            SymbolKind::Lambda => "lambda",
            SymbolKind::Class => "class",
            SymbolKind::Struct => "struct",
            SymbolKind::Interface => "interface",
            SymbolKind::Trait => "trait",
            SymbolKind::Enum => "enum",
            SymbolKind::TypeAlias => "type",
            SymbolKind::Variable => "var",
            SymbolKind::Constant => "const",
            SymbolKind::Field => "field",
            SymbolKind::Property => "prop",
            SymbolKind::EnumVariant => "variant",
            SymbolKind::Module => "mod",
            SymbolKind::Namespace => "ns",
            SymbolKind::Package => "pkg",
            SymbolKind::Macro => "macro",
            SymbolKind::Decorator => "decorator",
            SymbolKind::Attribute => "attr",
            SymbolKind::Unknown => "?",
        }
    }

    /// Get the icon/emoji for this kind
    pub fn icon(&self) -> &'static str {
        match self {
            SymbolKind::Function | SymbolKind::Method => "⚡",
            SymbolKind::Constructor => "🔨",
            SymbolKind::Lambda => "λ",
            SymbolKind::Class => "📦",
            SymbolKind::Struct => "🔳",
            SymbolKind::Interface | SymbolKind::Trait => "🔌",
            SymbolKind::Enum => "📋",
            SymbolKind::TypeAlias => "📐",
            SymbolKind::Variable => "📝",
            SymbolKind::Constant => "🔒",
            SymbolKind::Field | SymbolKind::Property => "•",
            SymbolKind::EnumVariant => "◦",
            SymbolKind::Module | SymbolKind::Namespace | SymbolKind::Package => "📁",
            SymbolKind::Macro => "⚙️",
            SymbolKind::Decorator | SymbolKind::Attribute => "@",
            SymbolKind::Unknown => "?",
        }
    }
}

/// Visibility/accessibility of a symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SymbolVisibility {
    /// Public to all (Rust pub, TypeScript export, Python __all__)
    Public,

    /// Exported from module (similar to Public but explicit)
    Export,

    /// Private to containing scope
    Private,

    /// Protected (accessible to subclasses)
    Protected,

    /// Internal (package/crate private)
    Internal,

    /// Visibility not specified or not applicable
    #[default]
    Unspecified,
}

/// An import/require statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    /// What is being imported
    pub source: String,

    /// How it's imported
    pub kind: ImportKind,

    /// Alias if renamed (import X as Y)
    pub alias: Option<String>,

    /// Specific items imported (for selective imports)
    pub items: Vec<String>,

    /// Location in source
    pub location: Location,

    /// Whether this is a type-only import (TypeScript)
    pub type_only: bool,
}

/// The kind of import statement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportKind {
    /// Import everything (import * from X, use X::*)
    Wildcard,

    /// Import specific items (from X import a, b)
    Selective,

    /// Import the module itself (import X)
    Module,

    /// Re-export (export { X } from Y)
    ReExport,

    /// Side-effect import (import 'polyfill')
    SideEffect,
}

/// A module/namespace declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// Module name
    pub name: String,

    /// Nested path (for nested modules)
    pub path: Vec<String>,

    /// Location in source
    pub location: Location,

    /// Documentation
    pub doc_comment: Option<String>,

    /// Visibility
    pub visibility: SymbolVisibility,
}

/// A scope or block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    /// Scope kind (function, class, block, etc.)
    pub kind: String,

    /// Span of the scope
    pub span: Span,

    /// Parent scope index
    pub parent: Option<usize>,

    /// Symbols defined in this scope
    pub symbols: Vec<usize>,
}

/// Location in source code (single point)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    /// Line number (1-indexed)
    pub line: usize,

    /// Column number (1-indexed)
    pub column: usize,

    /// Byte offset in source
    pub offset: usize,
}

impl Location {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }
}

/// Span in source code (range)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Start line (1-indexed)
    pub start_line: usize,

    /// Start column (1-indexed)
    pub start_column: usize,

    /// End line (1-indexed)
    pub end_line: usize,

    /// End column (1-indexed)
    pub end_column: usize,

    /// Start byte offset
    pub start_offset: usize,

    /// End byte offset
    pub end_offset: usize,
}

impl Span {
    pub fn new(
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
    ) -> Self {
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
            start_offset: 0,
            end_offset: 0,
        }
    }

    /// Get the number of lines in this span
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }

    /// Check if this span contains a location
    pub fn contains(&self, loc: &Location) -> bool {
        if loc.line < self.start_line || loc.line > self.end_line {
            return false;
        }
        if loc.line == self.start_line && loc.column < self.start_column {
            return false;
        }
        if loc.line == self.end_line && loc.column > self.end_column {
            return false;
        }
        true
    }
}

/// A function/method parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,

    /// Type annotation (if present)
    pub type_annotation: Option<String>,

    /// Default value (if present)
    pub default_value: Option<String>,

    /// Is this a rest/variadic parameter?
    pub is_rest: bool,

    /// Is this a keyword-only parameter? (Python)
    pub is_keyword_only: bool,
}

/// A parse diagnostic (error or warning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseDiagnostic {
    /// Severity level
    pub severity: DiagnosticSeverity,

    /// Error message
    pub message: String,

    /// Location
    pub location: Location,

    /// Span of affected code
    pub span: Option<Span>,
}

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_ast_new() {
        let ast = NormalizedAst::new();
        assert!(ast.symbols.is_empty());
        assert!(ast.imports.is_empty());
        assert!(!ast.has_errors());
    }

    #[test]
    fn test_symbol_creation() {
        let loc = Location::new(1, 1, 0);
        let symbol = Symbol::new("my_function", SymbolKind::Function, loc);

        assert_eq!(symbol.name, "my_function");
        assert_eq!(symbol.kind, SymbolKind::Function);
        assert!(symbol.is_callable());
        assert!(!symbol.is_container());
    }

    #[test]
    fn test_symbol_qualified_name() {
        let loc = Location::new(5, 1, 100);
        let mut symbol = Symbol::new("method", SymbolKind::Method, loc);
        symbol.parent = Some("MyClass".to_string());

        assert_eq!(symbol.qualified_name(), "MyClass.method");
    }

    #[test]
    fn test_ast_find_symbols() {
        let mut ast = NormalizedAst::new();

        ast.symbols.push(Symbol::new("calculate_total", SymbolKind::Function, Location::default()));
        ast.symbols.push(Symbol::new("calculate_tax", SymbolKind::Function, Location::default()));
        ast.symbols.push(Symbol::new("other_func", SymbolKind::Function, Location::default()));

        let matches = ast.find_symbols("calculate");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(10, 1, 20, 50);

        assert!(span.contains(&Location::new(15, 25, 0)));
        assert!(span.contains(&Location::new(10, 1, 0)));
        assert!(span.contains(&Location::new(20, 50, 0)));
        assert!(!span.contains(&Location::new(9, 1, 0)));
        assert!(!span.contains(&Location::new(21, 1, 0)));
    }

    #[test]
    fn test_symbol_kind_labels() {
        assert_eq!(SymbolKind::Function.label(), "fn");
        assert_eq!(SymbolKind::Class.label(), "class");
        assert_eq!(SymbolKind::Trait.label(), "trait");
    }

    #[test]
    fn test_ast_merge() {
        let mut ast1 = NormalizedAst::new();
        ast1.symbols.push(Symbol::new("a", SymbolKind::Function, Location::default()));

        let mut ast2 = NormalizedAst::new();
        ast2.symbols.push(Symbol::new("b", SymbolKind::Function, Location::default()));

        ast1.merge(ast2);
        assert_eq!(ast1.symbols.len(), 2);
    }
}
