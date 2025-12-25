//! Tree-sitter Adapter - High-Performance AST Extraction
//!
//! This module provides the Tree-sitter-based implementation of `SyntaxProvider`.
//! It handles parsing for 25 core languages with unified symbol extraction.

use super::{
    Language, NormalizedAst, PluginHook, ProviderStats, Symbol, SymbolKind,
    SymbolVisibility, SyntaxError, SyntaxProvider, Import, ImportKind,
    Location, Span, Parameter,
};
use std::collections::HashMap;

/// Central registry for all syntax providers
///
/// The SyntaxRegistry manages Tree-sitter parsers for all supported languages
/// and provides a unified interface for parsing source code.
pub struct SyntaxRegistry {
    adapter: TreeSitterAdapter,
}

impl SyntaxRegistry {
    /// Create a new syntax registry with all language support
    pub fn new() -> Self {
        Self {
            adapter: TreeSitterAdapter::new(),
        }
    }

    /// Parse source code for a given language
    pub fn parse(&self, source: &str, language: Language) -> Result<NormalizedAst, SyntaxError> {
        self.adapter.parse(source, language)
    }

    /// Parse source code, auto-detecting language from file extension
    pub fn parse_file(&self, source: &str, filename: &str) -> Result<NormalizedAst, SyntaxError> {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let language = Language::from_extension(ext)
            .ok_or_else(|| SyntaxError::UnsupportedLanguage(ext.to_string()))?;

        self.parse(source, language)
    }

    /// Check if a language is supported
    pub fn supports(&self, language: Language) -> bool {
        self.adapter.supports(language)
    }

    /// Get statistics
    pub fn stats(&self) -> ProviderStats {
        self.adapter.stats()
    }
}

impl Default for SyntaxRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Tree-sitter based syntax provider
///
/// This adapter wraps Tree-sitter parsers for all 25 supported languages
/// and provides normalized AST extraction.
pub struct TreeSitterAdapter {
    /// Cached parsers (created lazily)
    parsers: std::sync::Mutex<HashMap<Language, tree_sitter::Parser>>,

    /// Registered plugin hooks
    #[allow(dead_code)]
    hooks: Vec<PluginHook>,

    /// Performance statistics
    stats: std::sync::Mutex<ProviderStats>,
}

impl TreeSitterAdapter {
    /// Create a new Tree-sitter adapter
    pub fn new() -> Self {
        Self {
            parsers: std::sync::Mutex::new(HashMap::new()),
            hooks: Vec::new(),
            stats: std::sync::Mutex::new(ProviderStats::default()),
        }
    }

    /// Get or create a parser for the given language
    fn get_parser(&self, language: Language) -> Result<tree_sitter::Parser, SyntaxError> {
        let ts_language = self.get_tree_sitter_language(language)?;

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&ts_language)
            .map_err(|e| SyntaxError::InitializationError(e.to_string()))?;

        Ok(parser)
    }

    /// Get the Tree-sitter language for a given Language enum
    fn get_tree_sitter_language(
        &self,
        language: Language,
    ) -> Result<tree_sitter::Language, SyntaxError> {
        let ts_lang = match language {
            // Core supported languages (17 grammars)
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Language::Go => tree_sitter_go::LANGUAGE.into(),
            Language::Java => tree_sitter_java::LANGUAGE.into(),
            Language::C => tree_sitter_c::LANGUAGE.into(),
            Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Language::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
            Language::Ruby => tree_sitter_ruby::LANGUAGE.into(),
            Language::Html => tree_sitter_html::LANGUAGE.into(),
            Language::Css => tree_sitter_css::LANGUAGE.into(),
            Language::Json => tree_sitter_json::LANGUAGE.into(),
            Language::Bash => tree_sitter_bash::LANGUAGE.into(),

            // Reserved for future grammars (Phase 1B)
            // Note: TOML and Markdown require tree-sitter version updates
            Language::Toml
            | Language::Markdown
            | Language::Php
            | Language::Swift
            | Language::Kotlin
            | Language::Scala
            | Language::Yaml
            | Language::Lua
            | Language::Sql
            | Language::Hcl
            | Language::Dockerfile => {
                return Err(SyntaxError::UnsupportedLanguage(format!(
                    "{} grammar not yet available (Phase 1B)",
                    language.name()
                )));
            }
        };

        Ok(ts_lang)
    }

    /// Extract symbols from a parsed tree
    fn extract_symbols(
        &self,
        tree: &tree_sitter::Tree,
        source: &[u8],
        language: Language,
    ) -> NormalizedAst {
        let mut ast = NormalizedAst::new();
        let root = tree.root_node();

        // Extract based on language family
        match language {
            Language::Rust => self.extract_rust_symbols(&mut ast, root, source),
            Language::Python => self.extract_python_symbols(&mut ast, root, source),
            Language::JavaScript | Language::TypeScript | Language::Tsx => {
                self.extract_js_symbols(&mut ast, root, source)
            }
            Language::Go => self.extract_go_symbols(&mut ast, root, source),
            Language::Java | Language::Kotlin | Language::Scala => {
                self.extract_jvm_symbols(&mut ast, root, source)
            }
            Language::C | Language::Cpp => self.extract_c_symbols(&mut ast, root, source),
            Language::CSharp => self.extract_csharp_symbols(&mut ast, root, source),
            Language::Ruby => self.extract_ruby_symbols(&mut ast, root, source),
            Language::Php => self.extract_php_symbols(&mut ast, root, source),
            Language::Swift => self.extract_swift_symbols(&mut ast, root, source),
            _ => self.extract_generic_symbols(&mut ast, root, source),
        }

        ast
    }

    // ========================================================================
    // Language-Specific Extractors
    // ========================================================================

    fn extract_rust_symbols(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_item" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let visibility = self.rust_visibility(&child);
                        let mut symbol = Symbol::new(name, SymbolKind::Function, self.node_location(name_node));
                        symbol.visibility = visibility;
                        symbol.span = Some(self.node_span(child));
                        symbol.parameters = self.extract_rust_params(&child, source);
                        symbol.return_type = child
                            .child_by_field_name("return_type")
                            .map(|n| self.node_text(n, source));
                        ast.symbols.push(symbol);
                    }
                }
                "struct_item" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Struct, self.node_location(name_node));
                        symbol.visibility = self.rust_visibility(&child);
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "enum_item" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Enum, self.node_location(name_node));
                        symbol.visibility = self.rust_visibility(&child);
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "trait_item" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Trait, self.node_location(name_node));
                        symbol.visibility = self.rust_visibility(&child);
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "impl_item" => {
                    // Extract methods from impl blocks
                    self.extract_rust_impl_methods(ast, child, source);
                }
                "mod_item" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Module, self.node_location(name_node));
                        symbol.visibility = self.rust_visibility(&child);
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "use_declaration" => {
                    self.extract_rust_use(ast, child, source);
                }
                "const_item" | "static_item" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let kind = if child.kind() == "const_item" {
                            SymbolKind::Constant
                        } else {
                            SymbolKind::Variable
                        };
                        let mut symbol = Symbol::new(name, kind, self.node_location(name_node));
                        symbol.visibility = self.rust_visibility(&child);
                        ast.symbols.push(symbol);
                    }
                }
                "type_alias" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::TypeAlias, self.node_location(name_node));
                        symbol.visibility = self.rust_visibility(&child);
                        ast.symbols.push(symbol);
                    }
                }
                "macro_definition" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let symbol = Symbol::new(name, SymbolKind::Macro, self.node_location(name_node));
                        ast.symbols.push(symbol);
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_rust_impl_methods(
        &self,
        ast: &mut NormalizedAst,
        impl_node: tree_sitter::Node,
        source: &[u8],
    ) {
        let type_name = impl_node
            .child_by_field_name("type")
            .map(|n| self.node_text(n, source));

        let mut cursor = impl_node.walk();
        for child in impl_node.children(&mut cursor) {
            if child.kind() == "declaration_list" {
                let mut inner_cursor = child.walk();
                for method in child.children(&mut inner_cursor) {
                    if method.kind() == "function_item" {
                        if let Some(name_node) = method.child_by_field_name("name") {
                            let name = self.node_text(name_node, source);
                            let mut symbol = Symbol::new(name, SymbolKind::Method, self.node_location(name_node));
                            symbol.visibility = self.rust_visibility(&method);
                            symbol.parent = type_name.clone();
                            symbol.span = Some(self.node_span(method));
                            symbol.parameters = self.extract_rust_params(&method, source);
                            ast.symbols.push(symbol);
                        }
                    }
                }
            }
        }
    }

    fn extract_rust_params(&self, func_node: &tree_sitter::Node, source: &[u8]) -> Vec<Parameter> {
        let mut params = Vec::new();

        if let Some(params_node) = func_node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                if child.kind() == "parameter" {
                    if let Some(pattern) = child.child_by_field_name("pattern") {
                        let name = self.node_text(pattern, source);
                        let type_ann = child
                            .child_by_field_name("type")
                            .map(|n| self.node_text(n, source));
                        params.push(Parameter {
                            name,
                            type_annotation: type_ann,
                            default_value: None,
                            is_rest: false,
                            is_keyword_only: false,
                        });
                    }
                }
            }
        }

        params
    }

    fn extract_rust_use(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let source_text = self.node_text(node, source);
        // Simplified: extract the path from use statement
        let import = Import {
            source: source_text,
            kind: ImportKind::Selective,
            alias: None,
            items: Vec::new(),
            location: self.node_location(node),
            type_only: false,
        };
        ast.imports.push(import);
    }

    fn rust_visibility(&self, node: &tree_sitter::Node) -> SymbolVisibility {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                return SymbolVisibility::Public;
            }
        }
        SymbolVisibility::Private
    }

    fn extract_python_symbols(
        &self,
        ast: &mut NormalizedAst,
        node: tree_sitter::Node,
        source: &[u8],
    ) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let kind = if name.starts_with("__") && name.ends_with("__") {
                            SymbolKind::Method // Dunder method
                        } else {
                            SymbolKind::Function
                        };
                        let mut symbol = Symbol::new(name.clone(), kind, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        symbol.visibility = if name.starts_with('_') && !name.starts_with("__") {
                            SymbolVisibility::Private
                        } else {
                            SymbolVisibility::Public
                        };
                        symbol.decorators = self.extract_python_decorators(&child, source);
                        symbol.parameters = self.extract_python_params(&child, source);
                        ast.symbols.push(symbol);
                    }
                }
                "class_definition" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name.clone(), SymbolKind::Class, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        symbol.visibility = if name.starts_with('_') {
                            SymbolVisibility::Private
                        } else {
                            SymbolVisibility::Public
                        };
                        symbol.decorators = self.extract_python_decorators(&child, source);
                        ast.symbols.push(symbol);

                        // Extract methods
                        self.extract_python_class_methods(ast, child, source, name);
                    }
                }
                "import_statement" | "import_from_statement" => {
                    self.extract_python_import(ast, child, source);
                }
                "assignment" => {
                    // Top-level assignments can be constants
                    if let Some(left) = child.child_by_field_name("left") {
                        if left.kind() == "identifier" {
                            let name = self.node_text(left, source);
                            if name.chars().all(|c| c.is_uppercase() || c == '_') {
                                let symbol = Symbol::new(name, SymbolKind::Constant, self.node_location(left));
                                ast.symbols.push(symbol);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_python_class_methods(
        &self,
        ast: &mut NormalizedAst,
        class_node: tree_sitter::Node,
        source: &[u8],
        class_name: String,
    ) {
        if let Some(body) = class_node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "function_definition" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let kind = if name == "__init__" {
                            SymbolKind::Constructor
                        } else {
                            SymbolKind::Method
                        };
                        let mut symbol = Symbol::new(name.clone(), kind, self.node_location(name_node));
                        symbol.parent = Some(class_name.clone());
                        symbol.span = Some(self.node_span(child));
                        symbol.visibility = if name.starts_with('_') && !name.starts_with("__") {
                            SymbolVisibility::Private
                        } else {
                            SymbolVisibility::Public
                        };
                        symbol.decorators = self.extract_python_decorators(&child, source);
                        ast.symbols.push(symbol);
                    }
                }
            }
        }
    }

    fn extract_python_decorators(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
    ) -> Vec<String> {
        let mut decorators = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                decorators.push(self.node_text(child, source));
            }
        }

        decorators
    }

    fn extract_python_params(&self, func_node: &tree_sitter::Node, source: &[u8]) -> Vec<Parameter> {
        let mut params = Vec::new();

        if let Some(params_node) = func_node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                match child.kind() {
                    "identifier" => {
                        let name = self.node_text(child, source);
                        if name != "self" && name != "cls" {
                            params.push(Parameter {
                                name,
                                type_annotation: None,
                                default_value: None,
                                is_rest: false,
                                is_keyword_only: false,
                            });
                        }
                    }
                    "typed_parameter" | "default_parameter" | "typed_default_parameter" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = self.node_text(name_node, source);
                            if name != "self" && name != "cls" {
                                let type_ann = child
                                    .child_by_field_name("type")
                                    .map(|n| self.node_text(n, source));
                                let default = child
                                    .child_by_field_name("value")
                                    .map(|n| self.node_text(n, source));
                                params.push(Parameter {
                                    name,
                                    type_annotation: type_ann,
                                    default_value: default,
                                    is_rest: false,
                                    is_keyword_only: false,
                                });
                            }
                        }
                    }
                    "list_splat_pattern" => {
                        if let Some(name_node) = child.child(0) {
                            params.push(Parameter {
                                name: self.node_text(name_node, source),
                                type_annotation: None,
                                default_value: None,
                                is_rest: true,
                                is_keyword_only: false,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        params
    }

    fn extract_python_import(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let import = Import {
            source: self.node_text(node, source),
            kind: if node.kind() == "import_from_statement" {
                ImportKind::Selective
            } else {
                ImportKind::Module
            },
            alias: None,
            items: Vec::new(),
            location: self.node_location(node),
            type_only: false,
        };
        ast.imports.push(import);
    }

    fn extract_js_symbols(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Function, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        symbol.visibility = SymbolVisibility::Private;
                        ast.symbols.push(symbol);
                    }
                }
                "class_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name.clone(), SymbolKind::Class, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);

                        // Extract methods
                        self.extract_js_class_members(ast, child, source, name);
                    }
                }
                "interface_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Interface, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "type_alias_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::TypeAlias, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "enum_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Enum, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "export_statement" => {
                    self.extract_js_export(ast, child, source);
                }
                "import_statement" => {
                    self.extract_js_import(ast, child, source);
                }
                "lexical_declaration" | "variable_declaration" => {
                    self.extract_js_variable(ast, child, source);
                }
                _ => {}
            }
        }
    }

    fn extract_js_class_members(
        &self,
        ast: &mut NormalizedAst,
        class_node: tree_sitter::Node,
        source: &[u8],
        class_name: String,
    ) {
        if let Some(body) = class_node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                match child.kind() {
                    "method_definition" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = self.node_text(name_node, source);
                            let kind = if name == "constructor" {
                                SymbolKind::Constructor
                            } else {
                                SymbolKind::Method
                            };
                            let mut symbol = Symbol::new(name, kind, self.node_location(name_node));
                            symbol.parent = Some(class_name.clone());
                            symbol.span = Some(self.node_span(child));
                            ast.symbols.push(symbol);
                        }
                    }
                    "public_field_definition" | "field_definition" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = self.node_text(name_node, source);
                            let mut symbol = Symbol::new(name, SymbolKind::Field, self.node_location(name_node));
                            symbol.parent = Some(class_name.clone());
                            ast.symbols.push(symbol);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn extract_js_export(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Function, self.node_location(name_node));
                        symbol.visibility = SymbolVisibility::Export;
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "class_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Class, self.node_location(name_node));
                        symbol.visibility = SymbolVisibility::Export;
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_js_import(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let import = Import {
            source: self.node_text(node, source),
            kind: ImportKind::Module,
            alias: None,
            items: Vec::new(),
            location: self.node_location(node),
            type_only: false,
        };
        ast.imports.push(import);
    }

    fn extract_js_variable(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(name_node, source);
                    let kind = if name.chars().all(|c| c.is_uppercase() || c == '_') {
                        SymbolKind::Constant
                    } else {
                        SymbolKind::Variable
                    };
                    let symbol = Symbol::new(name, kind, self.node_location(name_node));
                    ast.symbols.push(symbol);
                }
            }
        }
    }

    fn extract_go_symbols(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name.clone(), SymbolKind::Function, self.node_location(name_node));
                        symbol.visibility = if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                            SymbolVisibility::Public
                        } else {
                            SymbolVisibility::Private
                        };
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "method_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name.clone(), SymbolKind::Method, self.node_location(name_node));
                        symbol.visibility = if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                            SymbolVisibility::Public
                        } else {
                            SymbolVisibility::Private
                        };
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "type_declaration" => {
                    self.extract_go_type(ast, child, source);
                }
                "import_declaration" => {
                    let import = Import {
                        source: self.node_text(child, source),
                        kind: ImportKind::Module,
                        alias: None,
                        items: Vec::new(),
                        location: self.node_location(child),
                        type_only: false,
                    };
                    ast.imports.push(import);
                }
                _ => {}
            }
        }
    }

    fn extract_go_type(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_spec" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(name_node, source);
                    let type_node = child.child_by_field_name("type");
                    let kind = match type_node.map(|n| n.kind()) {
                        Some("struct_type") => SymbolKind::Struct,
                        Some("interface_type") => SymbolKind::Interface,
                        _ => SymbolKind::TypeAlias,
                    };
                    let mut symbol = Symbol::new(name.clone(), kind, self.node_location(name_node));
                    symbol.visibility = if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                        SymbolVisibility::Public
                    } else {
                        SymbolVisibility::Private
                    };
                    symbol.span = Some(self.node_span(child));
                    ast.symbols.push(symbol);
                }
            }
        }
    }

    fn extract_jvm_symbols(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "class_declaration" | "class_definition" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name.clone(), SymbolKind::Class, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                        self.extract_jvm_class_members(ast, child, source, name);
                    }
                }
                "interface_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Interface, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "enum_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Enum, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "import_declaration" => {
                    let import = Import {
                        source: self.node_text(child, source),
                        kind: ImportKind::Module,
                        alias: None,
                        items: Vec::new(),
                        location: self.node_location(child),
                        type_only: false,
                    };
                    ast.imports.push(import);
                }
                _ => {}
            }
        }
    }

    fn extract_jvm_class_members(
        &self,
        ast: &mut NormalizedAst,
        class_node: tree_sitter::Node,
        source: &[u8],
        class_name: String,
    ) {
        if let Some(body) = class_node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                match child.kind() {
                    "method_declaration" | "function_declaration" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = self.node_text(name_node, source);
                            let mut symbol = Symbol::new(name, SymbolKind::Method, self.node_location(name_node));
                            symbol.parent = Some(class_name.clone());
                            symbol.span = Some(self.node_span(child));
                            ast.symbols.push(symbol);
                        }
                    }
                    "constructor_declaration" => {
                        let mut symbol = Symbol::new(
                            class_name.clone(),
                            SymbolKind::Constructor,
                            self.node_location(child),
                        );
                        symbol.parent = Some(class_name.clone());
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                    "field_declaration" => {
                        let mut inner_cursor = child.walk();
                        for decl in child.children(&mut inner_cursor) {
                            if decl.kind() == "variable_declarator" {
                                if let Some(name_node) = decl.child_by_field_name("name") {
                                    let name = self.node_text(name_node, source);
                                    let mut symbol = Symbol::new(name, SymbolKind::Field, self.node_location(name_node));
                                    symbol.parent = Some(class_name.clone());
                                    ast.symbols.push(symbol);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn extract_c_symbols(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_definition" | "function_declarator" => {
                    if let Some(declarator) = child.child_by_field_name("declarator") {
                        if let Some(name_node) = declarator.child_by_field_name("declarator") {
                            let name = self.node_text(name_node, source);
                            let mut symbol = Symbol::new(name, SymbolKind::Function, self.node_location(name_node));
                            symbol.span = Some(self.node_span(child));
                            ast.symbols.push(symbol);
                        }
                    }
                }
                "struct_specifier" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Struct, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "enum_specifier" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Enum, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "type_definition" => {
                    if let Some(declarator) = child.child_by_field_name("declarator") {
                        let name = self.node_text(declarator, source);
                        let mut symbol = Symbol::new(name, SymbolKind::TypeAlias, self.node_location(declarator));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "preproc_include" | "preproc_import" => {
                    let import = Import {
                        source: self.node_text(child, source),
                        kind: ImportKind::Module,
                        alias: None,
                        items: Vec::new(),
                        location: self.node_location(child),
                        type_only: false,
                    };
                    ast.imports.push(import);
                }
                "preproc_def" | "preproc_function_def" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let symbol = Symbol::new(name, SymbolKind::Macro, self.node_location(name_node));
                        ast.symbols.push(symbol);
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_csharp_symbols(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        // Similar to JVM but with C# specific nodes
        self.extract_jvm_symbols(ast, node, source);
    }

    fn extract_ruby_symbols(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "method" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Function, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "class" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Class, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "module" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Module, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_php_symbols(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Function, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "class_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Class, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "interface_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Interface, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "trait_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Trait, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_swift_symbols(&self, ast: &mut NormalizedAst, node: tree_sitter::Node, source: &[u8]) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Function, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "class_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Class, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "struct_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Struct, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "protocol_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Interface, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                "enum_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(name_node, source);
                        let mut symbol = Symbol::new(name, SymbolKind::Enum, self.node_location(name_node));
                        symbol.span = Some(self.node_span(child));
                        ast.symbols.push(symbol);
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_generic_symbols(
        &self,
        ast: &mut NormalizedAst,
        node: tree_sitter::Node,
        source: &[u8],
    ) {
        // Generic extractor for languages without specific handlers
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            // Look for common patterns
            let kind_str = child.kind();
            if kind_str.contains("function") || kind_str.contains("method") {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(name_node, source);
                    let mut symbol = Symbol::new(name, SymbolKind::Function, self.node_location(name_node));
                    symbol.span = Some(self.node_span(child));
                    ast.symbols.push(symbol);
                }
            } else if kind_str.contains("class") || kind_str.contains("struct") {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(name_node, source);
                    let kind = if kind_str.contains("struct") {
                        SymbolKind::Struct
                    } else {
                        SymbolKind::Class
                    };
                    let mut symbol = Symbol::new(name, kind, self.node_location(name_node));
                    symbol.span = Some(self.node_span(child));
                    ast.symbols.push(symbol);
                }
            }
        }
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    fn node_text(&self, node: tree_sitter::Node, source: &[u8]) -> String {
        node.utf8_text(source).unwrap_or("").to_string()
    }

    fn node_location(&self, node: tree_sitter::Node) -> Location {
        let start = node.start_position();
        Location {
            line: start.row + 1,
            column: start.column + 1,
            offset: node.start_byte(),
        }
    }

    fn node_span(&self, node: tree_sitter::Node) -> Span {
        let start = node.start_position();
        let end = node.end_position();
        Span {
            start_line: start.row + 1,
            start_column: start.column + 1,
            end_line: end.row + 1,
            end_column: end.column + 1,
            start_offset: node.start_byte(),
            end_offset: node.end_byte(),
        }
    }
}

impl Default for TreeSitterAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxProvider for TreeSitterAdapter {
    fn parse(&self, source: &str, language: Language) -> Result<NormalizedAst, SyntaxError> {
        let mut parser = self.get_parser(language)?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| SyntaxError::ParseError {
                line: 0,
                column: 0,
                message: "Failed to parse source".to_string(),
            })?;

        let ast = self.extract_symbols(&tree, source.as_bytes(), language);

        // Update stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.files_parsed += 1;
            stats.symbols_extracted += ast.symbols.len();
        }

        Ok(ast)
    }

    fn supported_languages(&self) -> &[Language] {
        // Core supported languages (17 grammars in Phase 1A)
        &[
            Language::Rust,
            Language::Python,
            Language::JavaScript,
            Language::TypeScript,
            Language::Tsx,
            Language::Go,
            Language::Java,
            Language::C,
            Language::Cpp,
            Language::CSharp,
            Language::Ruby,
            Language::Html,
            Language::Css,
            Language::Json,
            Language::Toml,
            Language::Bash,
            Language::Markdown,
        ]
    }

    fn stats(&self) -> ProviderStats {
        self.stats.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_registry_creation() {
        let registry = SyntaxRegistry::new();
        assert!(registry.supports(Language::Rust));
        assert!(registry.supports(Language::Python));
        assert!(registry.supports(Language::TypeScript));
    }

    #[test]
    fn test_parse_rust_function() {
        let registry = SyntaxRegistry::new();
        let source = r#"
            pub fn hello_world(name: &str) -> String {
                format!("Hello, {}!", name)
            }
        "#;

        let ast = registry.parse(source, Language::Rust).unwrap();
        assert!(!ast.symbols.is_empty());

        let func = ast.find_symbol("hello_world");
        assert!(func.is_some());
        let func = func.unwrap();
        assert_eq!(func.kind, SymbolKind::Function);
        assert_eq!(func.visibility, SymbolVisibility::Public);
    }

    #[test]
    fn test_parse_rust_struct() {
        let registry = SyntaxRegistry::new();
        let source = r#"
            pub struct Config {
                name: String,
                value: i32,
            }
        "#;

        let ast = registry.parse(source, Language::Rust).unwrap();
        let config = ast.find_symbol("Config");
        assert!(config.is_some());
        assert_eq!(config.unwrap().kind, SymbolKind::Struct);
    }

    #[test]
    fn test_parse_python_class() {
        let registry = SyntaxRegistry::new();
        let source = r#"
class Calculator:
    """A simple calculator."""

    def __init__(self, value: int = 0):
        self.value = value

    def add(self, x: int) -> int:
        return self.value + x
        "#;

        let ast = registry.parse(source, Language::Python).unwrap();

        // Should find class
        let calc = ast.find_symbol("Calculator");
        assert!(calc.is_some());
        assert_eq!(calc.unwrap().kind, SymbolKind::Class);

        // Should find methods
        let init = ast.find_symbol("__init__");
        assert!(init.is_some());
        assert_eq!(init.unwrap().kind, SymbolKind::Constructor);

        let add = ast.find_symbol("add");
        assert!(add.is_some());
        assert_eq!(add.unwrap().kind, SymbolKind::Method);
    }

    #[test]
    fn test_parse_typescript_interface() {
        let registry = SyntaxRegistry::new();
        let source = r#"
interface User {
    id: number;
    name: string;
    email?: string;
}

export function createUser(data: Partial<User>): User {
    return { id: 1, name: data.name || 'Anonymous', ...data };
}
        "#;

        let ast = registry.parse(source, Language::TypeScript).unwrap();

        // Should find interface
        let user = ast.find_symbol("User");
        assert!(user.is_some());
        assert_eq!(user.unwrap().kind, SymbolKind::Interface);

        // Should find exported function
        let create = ast.find_symbol("createUser");
        assert!(create.is_some());
    }

    #[test]
    fn test_parse_go_struct_and_methods() {
        let registry = SyntaxRegistry::new();
        let source = r#"
package main

type Server struct {
    Host string
    Port int
}

func (s *Server) Start() error {
    return nil
}

func NewServer(host string, port int) *Server {
    return &Server{Host: host, Port: port}
}
        "#;

        let ast = registry.parse(source, Language::Go).unwrap();

        // Should find struct
        let server = ast.find_symbol("Server");
        assert!(server.is_some());
        assert_eq!(server.unwrap().kind, SymbolKind::Struct);

        // Should find function
        let new_server = ast.find_symbol("NewServer");
        assert!(new_server.is_some());
        assert_eq!(new_server.unwrap().visibility, SymbolVisibility::Public);

        // Should find method
        let start = ast.find_symbol("Start");
        assert!(start.is_some());
    }

    #[test]
    fn test_parse_file_auto_detect() {
        let registry = SyntaxRegistry::new();
        let source = "fn main() {}";

        let ast = registry.parse_file(source, "main.rs").unwrap();
        assert!(!ast.symbols.is_empty());
    }

    #[test]
    fn test_stats_tracking() {
        let registry = SyntaxRegistry::new();

        registry.parse("fn a() {}", Language::Rust).unwrap();
        registry.parse("fn b() {}", Language::Rust).unwrap();

        let stats = registry.stats();
        assert_eq!(stats.files_parsed, 2);
    }
}
