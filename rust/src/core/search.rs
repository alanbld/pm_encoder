//! Symbol Resolution for Cross-File Navigation (Fractal Protocol v2)
//!
//! This module provides the ability to find symbol definitions (functions, classes, structs)
//! across a codebase without requiring the user to specify the exact file path.
//!
//! # Example
//! ```ignore
//! use pm_encoder::core::SymbolResolver;
//!
//! let resolver = SymbolResolver::new();
//! let location = resolver.find_function("apply_budget", "/path/to/project")?;
//! println!("Found at {}:{}-{}", location.path, location.start_line, location.end_line);
//! ```

use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;

use super::walker::{DefaultWalker, WalkConfig};
use super::FileWalker;

/// A resolved symbol location in the codebase
#[derive(Debug, Clone)]
pub struct SymbolLocation {
    /// File path relative to project root
    pub path: String,
    /// Line number where the symbol starts (1-indexed)
    pub start_line: usize,
    /// Line number where the symbol ends (1-indexed, inclusive)
    pub end_line: usize,
    /// The symbol name
    pub name: String,
    /// Symbol type (function, class, struct, etc.)
    pub symbol_type: SymbolType,
    /// The signature or first line of the definition
    pub signature: String,
}

/// Type of symbol being resolved
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    Function,
    Class,
    Struct,
    Trait,
    Enum,
    Module,
}

impl std::fmt::Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolType::Function => write!(f, "function"),
            SymbolType::Class => write!(f, "class"),
            SymbolType::Struct => write!(f, "struct"),
            SymbolType::Trait => write!(f, "trait"),
            SymbolType::Enum => write!(f, "enum"),
            SymbolType::Module => write!(f, "module"),
        }
    }
}

lazy_static! {
    // Rust patterns
    static ref RUST_FN: Regex = Regex::new(
        r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)"
    ).unwrap();
    static ref RUST_STRUCT: Regex = Regex::new(
        r"^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+(\w+)"
    ).unwrap();
    static ref RUST_ENUM: Regex = Regex::new(
        r"^\s*(?:pub(?:\([^)]*\))?\s+)?enum\s+(\w+)"
    ).unwrap();
    static ref RUST_TRAIT: Regex = Regex::new(
        r"^\s*(?:pub(?:\([^)]*\))?\s+)?trait\s+(\w+)"
    ).unwrap();
    static ref RUST_IMPL: Regex = Regex::new(
        r"^\s*impl(?:\s*<[^>]*>)?\s+(?:(\w+)\s+for\s+)?(\w+)"
    ).unwrap();

    // Python patterns
    static ref PYTHON_DEF: Regex = Regex::new(
        r"^\s*(?:async\s+)?def\s+(\w+)"
    ).unwrap();
    static ref PYTHON_CLASS: Regex = Regex::new(
        r"^\s*class\s+(\w+)"
    ).unwrap();

    // JavaScript/TypeScript patterns
    static ref JS_FUNCTION: Regex = Regex::new(
        r"^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)"
    ).unwrap();
    static ref JS_CLASS: Regex = Regex::new(
        r"^\s*(?:export\s+)?class\s+(\w+)"
    ).unwrap();
    static ref JS_CONST_FN: Regex = Regex::new(
        r"^\s*(?:export\s+)?(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?(?:\([^)]*\)|[^=])\s*=>"
    ).unwrap();
    static ref JS_METHOD: Regex = Regex::new(
        r"^\s*(?:async\s+)?(\w+)\s*\([^)]*\)\s*\{"
    ).unwrap();

    // Go patterns
    static ref GO_FUNC: Regex = Regex::new(
        r"^\s*func\s+(?:\([^)]+\)\s+)?(\w+)"
    ).unwrap();
    static ref GO_TYPE: Regex = Regex::new(
        r"^\s*type\s+(\w+)\s+(?:struct|interface)"
    ).unwrap();
}

/// Symbol resolver for finding definitions across a codebase
pub struct SymbolResolver {
    ignore_patterns: Vec<String>,
    include_patterns: Vec<String>,
}

impl Default for SymbolResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolResolver {
    /// Create a new symbol resolver with default patterns
    pub fn new() -> Self {
        Self {
            ignore_patterns: vec![
                "*.pyc".to_string(),
                "__pycache__".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "*.min.js".to_string(),
            ],
            include_patterns: Vec::new(),
        }
    }

    /// Create with custom ignore patterns
    pub fn with_ignore(mut self, patterns: Vec<String>) -> Self {
        self.ignore_patterns = patterns;
        self
    }

    /// Create with include patterns
    pub fn with_include(mut self, patterns: Vec<String>) -> Self {
        self.include_patterns = patterns;
        self
    }

    /// Find a function definition by name
    pub fn find_function(&self, name: &str, root: &Path) -> Result<SymbolLocation, String> {
        self.find_symbol(name, SymbolType::Function, root)
    }

    /// Find a class/struct definition by name
    pub fn find_class(&self, name: &str, root: &Path) -> Result<SymbolLocation, String> {
        // Try struct first (Rust), then class (Python/JS)
        self.find_symbol(name, SymbolType::Struct, root)
            .or_else(|_| self.find_symbol(name, SymbolType::Class, root))
    }

    /// Find all matches for a symbol (for disambiguation)
    pub fn find_all(&self, name: &str, symbol_type: SymbolType, root: &Path) -> Vec<SymbolLocation> {
        let mut results = Vec::new();

        let walker = DefaultWalker::new();
        let config = WalkConfig {
            ignore_patterns: self.ignore_patterns.clone(),
            include_patterns: self.include_patterns.clone(),
            max_file_size: 1_048_576, // 1MB
        };

        let entries = match walker.walk(root.to_str().unwrap_or("."), &config) {
            Ok(e) => e,
            Err(_) => return results,
        };

        for entry in entries {
            if let Some(locations) = self.find_in_file(&entry.path, &entry.content, name, symbol_type) {
                results.extend(locations);
            }
        }

        results
    }

    /// Find a single symbol (returns first match or error)
    pub fn find_symbol(&self, name: &str, symbol_type: SymbolType, root: &Path) -> Result<SymbolLocation, String> {
        let walker = DefaultWalker::new();
        let config = WalkConfig {
            ignore_patterns: self.ignore_patterns.clone(),
            include_patterns: self.include_patterns.clone(),
            max_file_size: 1_048_576,
        };

        let entries = walker.walk(root.to_str().unwrap_or("."), &config)
            .map_err(|e| format!("Failed to walk directory: {}", e))?;

        for entry in entries {
            if let Some(locations) = self.find_in_file(&entry.path, &entry.content, name, symbol_type) {
                if let Some(loc) = locations.into_iter().next() {
                    return Ok(loc);
                }
            }
        }

        Err(format!(
            "{} '{}' not found in scanned files. Try checking the name or file patterns.",
            symbol_type, name
        ))
    }

    /// Find symbols in a single file
    fn find_in_file(&self, path: &str, content: &str, name: &str, symbol_type: SymbolType) -> Option<Vec<SymbolLocation>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut results = Vec::new();

        let ext = Path::new(path).extension()?.to_str()?;

        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;

            if let Some(loc) = self.match_symbol(path, line, line_num, name, symbol_type, ext, &lines) {
                results.push(loc);
            }
        }

        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    }

    /// Match a symbol on a specific line
    fn match_symbol(
        &self,
        path: &str,
        line: &str,
        line_num: usize,
        name: &str,
        symbol_type: SymbolType,
        ext: &str,
        all_lines: &[&str],
    ) -> Option<SymbolLocation> {
        let patterns: Vec<&Regex> = match (ext, symbol_type) {
            ("rs", SymbolType::Function) => vec![&RUST_FN],
            ("rs", SymbolType::Struct) => vec![&RUST_STRUCT],
            ("rs", SymbolType::Enum) => vec![&RUST_ENUM],
            ("rs", SymbolType::Trait) => vec![&RUST_TRAIT],
            ("rs", SymbolType::Class) => vec![&RUST_STRUCT, &RUST_ENUM], // Rust doesn't have classes

            ("py" | "pyw", SymbolType::Function) => vec![&PYTHON_DEF],
            ("py" | "pyw", SymbolType::Class) => vec![&PYTHON_CLASS],

            ("js" | "jsx" | "ts" | "tsx" | "mjs", SymbolType::Function) => {
                vec![&JS_FUNCTION, &JS_CONST_FN, &JS_METHOD]
            }
            ("js" | "jsx" | "ts" | "tsx" | "mjs", SymbolType::Class) => vec![&JS_CLASS],

            ("go", SymbolType::Function) => vec![&GO_FUNC],
            ("go", SymbolType::Class | SymbolType::Struct) => vec![&GO_TYPE],

            _ => return None,
        };

        for pattern in patterns {
            if let Some(caps) = pattern.captures(line) {
                // Get the captured name (group 1, or group 2 for some patterns)
                let captured_name = caps.get(1)
                    .or_else(|| caps.get(2))
                    .map(|m| m.as_str())?;

                if captured_name == name {
                    // Find the end of the symbol (simple heuristic: find closing brace at same indent)
                    let end_line = self.find_block_end(all_lines, line_num - 1, ext);

                    return Some(SymbolLocation {
                        path: path.to_string(),
                        start_line: line_num,
                        end_line,
                        name: name.to_string(),
                        symbol_type,
                        signature: line.trim().to_string(),
                    });
                }
            }
        }

        None
    }

    /// Find the end of a code block (heuristic based on brace/indent matching)
    fn find_block_end(&self, lines: &[&str], start_idx: usize, ext: &str) -> usize {
        if start_idx >= lines.len() {
            return start_idx + 1;
        }

        let start_line = lines[start_idx];
        let start_indent = start_line.len() - start_line.trim_start().len();

        match ext {
            // Brace-based languages
            "rs" | "js" | "jsx" | "ts" | "tsx" | "mjs" | "go" | "c" | "cpp" | "java" => {
                let mut brace_count = 0;
                let mut found_open = false;

                for (i, line) in lines.iter().enumerate().skip(start_idx) {
                    for ch in line.chars() {
                        if ch == '{' {
                            brace_count += 1;
                            found_open = true;
                        } else if ch == '}' {
                            brace_count -= 1;
                            if found_open && brace_count == 0 {
                                return i + 1; // 1-indexed
                            }
                        }
                    }
                }
                // If no closing brace found, estimate ~50 lines
                (start_idx + 50).min(lines.len())
            }

            // Indent-based languages (Python)
            "py" | "pyw" => {
                for (i, line) in lines.iter().enumerate().skip(start_idx + 1) {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let indent = line.len() - line.trim_start().len();
                    if indent <= start_indent && !trimmed.starts_with('#') && !trimmed.starts_with('@') {
                        return i; // 1-indexed (previous line is end)
                    }
                }
                lines.len()
            }

            _ => (start_idx + 30).min(lines.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_function_pattern() {
        let test_cases = vec![
            ("fn main() {", "main"),
            ("pub fn process() {", "process"),
            ("    pub async fn fetch_data() ->", "fetch_data"),
            ("pub(crate) fn internal() {", "internal"),
        ];

        for (line, expected) in test_cases {
            let caps = RUST_FN.captures(line);
            assert!(caps.is_some(), "Failed to match: {}", line);
            assert_eq!(caps.unwrap().get(1).unwrap().as_str(), expected);
        }
    }

    #[test]
    fn test_python_function_pattern() {
        let test_cases = vec![
            ("def hello():", "hello"),
            ("    def process(self):", "process"),
            ("async def fetch():", "fetch"),
        ];

        for (line, expected) in test_cases {
            let caps = PYTHON_DEF.captures(line);
            assert!(caps.is_some(), "Failed to match: {}", line);
            assert_eq!(caps.unwrap().get(1).unwrap().as_str(), expected);
        }
    }

    #[test]
    fn test_javascript_patterns() {
        let test_cases = vec![
            ("function hello() {", "hello", &*JS_FUNCTION),
            ("export async function fetch() {", "fetch", &*JS_FUNCTION),
            ("const handler = () => {", "handler", &*JS_CONST_FN),
            ("export const process = async () => {", "process", &*JS_CONST_FN),
        ];

        for (line, expected, pattern) in test_cases {
            let caps = pattern.captures(line);
            assert!(caps.is_some(), "Failed to match: {}", line);
            assert_eq!(caps.unwrap().get(1).unwrap().as_str(), expected);
        }
    }

    #[test]
    fn test_find_block_end_rust() {
        let resolver = SymbolResolver::new();
        let lines = vec![
            "fn test() {",
            "    let x = 1;",
            "    if x > 0 {",
            "        println!(\"hi\");",
            "    }",
            "}",
            "",
            "fn other() {",
        ];

        let end = resolver.find_block_end(&lines, 0, "rs");
        assert_eq!(end, 6); // Line 6 (1-indexed)
    }

    #[test]
    fn test_find_block_end_python() {
        let resolver = SymbolResolver::new();
        let lines = vec![
            "def test():",
            "    x = 1",
            "    if x > 0:",
            "        print('hi')",
            "",
            "def other():",
        ];

        let end = resolver.find_block_end(&lines, 0, "py");
        assert_eq!(end, 5); // Ends at line 5 (before def other)
    }

    #[test]
    fn test_symbol_location_display() {
        let loc = SymbolLocation {
            path: "src/main.rs".to_string(),
            start_line: 10,
            end_line: 25,
            name: "main".to_string(),
            symbol_type: SymbolType::Function,
            signature: "fn main() {".to_string(),
        };

        assert_eq!(loc.symbol_type.to_string(), "function");
        assert_eq!(loc.name, "main");
    }
}
