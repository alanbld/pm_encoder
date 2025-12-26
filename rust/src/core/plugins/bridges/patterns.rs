//! Pre-compiled Patterns Bridge
//!
//! Provides pre-compiled regex patterns for common language constructs.
//! These patterns are accessible via `vo.patterns.*` in Lua plugins.

#[cfg(feature = "plugins")]
use mlua::{Lua, Table, Result as LuaResult};

/// Create the patterns table with common regex patterns
#[cfg(feature = "plugins")]
pub fn create_patterns_table(lua: &Lua) -> LuaResult<Table> {
    let patterns = lua.create_table()?;

    // Rust patterns
    patterns.set("rust_fn", r#"fn\s+(\w+)"#)?;
    patterns.set("rust_struct", r#"struct\s+(\w+)"#)?;
    patterns.set("rust_enum", r#"enum\s+(\w+)"#)?;
    patterns.set("rust_impl", r#"impl(?:<[^>]+>)?\s+(?:(\w+)\s+for\s+)?(\w+)"#)?;
    patterns.set("rust_trait", r#"trait\s+(\w+)"#)?;
    patterns.set("rust_mod", r#"mod\s+(\w+)"#)?;
    patterns.set("rust_use", r#"use\s+([^;]+)"#)?;
    patterns.set("rust_const", r#"const\s+(\w+)"#)?;
    patterns.set("rust_static", r#"static\s+(\w+)"#)?;
    patterns.set("rust_type", r#"type\s+(\w+)"#)?;

    // Python patterns
    patterns.set("python_def", r#"def\s+(\w+)"#)?;
    patterns.set("python_class", r#"class\s+(\w+)"#)?;
    patterns.set("python_import", r#"(?:from\s+[\w.]+\s+)?import\s+(.+)"#)?;
    patterns.set("python_decorator", r#"@(\w+)"#)?;
    patterns.set("python_async_def", r#"async\s+def\s+(\w+)"#)?;

    // JavaScript/TypeScript patterns
    patterns.set("js_function", r#"function\s+(\w+)"#)?;
    patterns.set("js_const", r#"const\s+(\w+)"#)?;
    patterns.set("js_let", r#"let\s+(\w+)"#)?;
    patterns.set("js_class", r#"class\s+(\w+)"#)?;
    patterns.set("js_arrow", r#"(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s*)?\([^)]*\)\s*=>"#)?;
    patterns.set("js_import", r#"import\s+(?:\{[^}]+\}|\*\s+as\s+\w+|\w+)\s+from\s+["']([^"']+)["']"#)?;
    patterns.set("js_export", r#"export\s+(?:default\s+)?(?:function|class|const|let|var)\s+(\w+)"#)?;

    // Go patterns
    patterns.set("go_func", r#"func\s+(?:\([^)]+\)\s+)?(\w+)"#)?;
    patterns.set("go_type", r#"type\s+(\w+)"#)?;
    patterns.set("go_struct", r#"type\s+(\w+)\s+struct"#)?;
    patterns.set("go_interface", r#"type\s+(\w+)\s+interface"#)?;
    patterns.set("go_package", r#"package\s+(\w+)"#)?;
    patterns.set("go_import", r#"import\s+(?:\(\s*)?["']([^"']+)["']"#)?;

    // Java patterns
    patterns.set("java_class", r#"(?:public|private|protected)?\s*class\s+(\w+)"#)?;
    patterns.set("java_interface", r#"(?:public|private|protected)?\s*interface\s+(\w+)"#)?;
    patterns.set("java_method", r#"(?:public|private|protected)?\s*(?:static\s+)?(?:\w+\s+)+(\w+)\s*\("#)?;
    patterns.set("java_import", r#"import\s+([^;]+)"#)?;
    patterns.set("java_package", r#"package\s+([^;]+)"#)?;

    // Common patterns
    patterns.set("todo_comment", r#"(?://|#|/\*)\s*TODO[:\s]"#)?;
    patterns.set("fixme_comment", r#"(?://|#|/\*)\s*FIXME[:\s]"#)?;
    patterns.set("hack_comment", r#"(?://|#|/\*)\s*HACK[:\s]"#)?;
    patterns.set("note_comment", r#"(?://|#|/\*)\s*NOTE[:\s]"#)?;
    patterns.set("url", r#"https?://[^\s<>"{}|\\^`\[\]]+"#)?;
    patterns.set("email", r#"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"#)?;

    Ok(patterns)
}

#[cfg(all(test, feature = "plugins"))]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_patterns_table_creation() {
        let lua = Lua::new();
        let patterns = create_patterns_table(&lua).unwrap();

        // Verify some patterns exist
        let rust_fn: String = patterns.get("rust_fn").unwrap();
        assert_eq!(rust_fn, r#"fn\s+(\w+)"#);

        let python_def: String = patterns.get("python_def").unwrap();
        assert_eq!(python_def, r#"def\s+(\w+)"#);
    }
}
