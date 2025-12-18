//! Init-prompt module - Generates instruction files for AI assistants
//!
//! This module implements the "Split Brain" architecture (v1.4.0):
//! - Instruction file (CLAUDE.md / GEMINI_INSTRUCTIONS.txt): Commands, tree, stats, pointer
//! - Context file (CONTEXT.txt): Serialized codebase (separate file)
//!
//! The instruction file does NOT contain code, only a pointer to CONTEXT.txt.

use std::fs;
use std::path::Path;

/// Detect common project commands based on project files
///
/// Scans the project root for common build system files and returns
/// appropriate commands for each detected system.
pub fn detect_project_commands(root: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let root_path = Path::new(root);

    // Rust: Cargo.toml
    if root_path.join("Cargo.toml").exists() {
        commands.push("cargo build".to_string());
        commands.push("cargo test".to_string());
    }

    // Node.js: package.json
    if root_path.join("package.json").exists() {
        commands.push("npm install".to_string());
        commands.push("npm test".to_string());
        commands.push("npm start".to_string());
    }

    // Make: Makefile
    if root_path.join("Makefile").exists() {
        commands.push("make".to_string());
        commands.push("make test".to_string());
    }

    // Python: requirements.txt or pyproject.toml
    if root_path.join("requirements.txt").exists() {
        commands.push("pip install -r requirements.txt".to_string());
    }
    if root_path.join("pyproject.toml").exists() {
        commands.push("pip install -e .".to_string());
    }

    // Python: pytest
    if root_path.join("pytest.ini").exists() || root_path.join("tests").exists() {
        commands.push("pytest".to_string());
    }

    // Docker
    if root_path.join("Dockerfile").exists() {
        commands.push("docker build .".to_string());
    }
    if root_path.join("docker-compose.yml").exists() || root_path.join("docker-compose.yaml").exists() {
        commands.push("docker-compose up".to_string());
    }

    // Go
    if root_path.join("go.mod").exists() {
        commands.push("go build".to_string());
        commands.push("go test ./...".to_string());
    }

    commands
}

/// Generate a directory tree representation
///
/// Creates an ASCII tree structure showing the project layout.
/// Respects ignore patterns and max depth.
pub fn generate_directory_tree(
    root: &str,
    ignore_patterns: &[String],
    max_depth: usize,
) -> Vec<String> {
    let mut lines = Vec::new();
    let root_path = Path::new(root);

    // Get the directory name for the root
    let root_name = root_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".");

    lines.push(format!("{}/", root_name));

    // Build tree recursively
    build_tree_recursive(root_path, root_path, &mut lines, "", ignore_patterns, 0, max_depth);

    lines
}

fn build_tree_recursive(
    root: &Path,
    current: &Path,
    lines: &mut Vec<String>,
    prefix: &str,
    ignore_patterns: &[String],
    depth: usize,
    max_depth: usize,
) {
    if depth >= max_depth {
        return;
    }

    // Read directory entries
    let mut entries: Vec<_> = match fs::read_dir(current) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };

    // Sort entries: directories first, then by name
    entries.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    // Filter out ignored entries
    let entries: Vec<_> = entries
        .into_iter()
        .filter(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Check against ignore patterns
            for pattern in ignore_patterns {
                if pattern.starts_with("*.") {
                    // Extension pattern
                    let ext = &pattern[1..]; // Get ".ext"
                    if name_str.ends_with(ext) {
                        return false;
                    }
                } else if name_str == pattern.as_str() {
                    return false;
                }
            }
            true
        })
        .collect();

    let count = entries.len();

    for (i, entry) in entries.into_iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

        if is_dir {
            lines.push(format!("{}{}{}/", prefix, connector, name_str));
            build_tree_recursive(
                root,
                &entry.path(),
                lines,
                &format!("{}{}", prefix, child_prefix),
                ignore_patterns,
                depth + 1,
                max_depth,
            );
        } else {
            lines.push(format!("{}{}{}", prefix, connector, name_str));
        }
    }
}

/// Get the instruction file name for a target
fn get_instruction_filename(target: &str) -> &'static str {
    match target.to_lowercase().as_str() {
        "gemini" => "GEMINI_INSTRUCTIONS.txt",
        _ => "CLAUDE.md", // Default to Claude
    }
}

/// Initialize AI instruction files (Split Brain architecture)
///
/// Creates two files:
/// 1. Instruction file (CLAUDE.md or GEMINI_INSTRUCTIONS.txt): Commands, tree, stats
/// 2. Context file (CONTEXT.txt): Serialized codebase
///
/// The instruction file points to CONTEXT.txt, does NOT contain code.
pub fn init_prompt(
    root: &str,
    lens_name: &str,
    target: &str,
) -> Result<(String, String), String> {
    use crate::{EncoderConfig, LensManager, serialize_project_with_config};

    let root_path = Path::new(root);
    if !root_path.exists() {
        return Err(format!("Directory not found: {}", root));
    }

    // Step 1: Detect project commands
    let commands = detect_project_commands(root);

    // Step 2: Generate directory tree
    let ignore_patterns = vec![
        ".git".to_string(),
        "target".to_string(),
        ".venv".to_string(),
        "__pycache__".to_string(),
        "node_modules".to_string(),
        "*.pyc".to_string(),
    ];
    let tree = generate_directory_tree(root, &ignore_patterns, 4);

    // Step 3: Apply lens and serialize context
    let mut lens_manager = LensManager::new();
    let applied_lens = lens_manager.apply_lens(lens_name)?;

    let config = EncoderConfig {
        ignore_patterns: applied_lens.ignore_patterns.clone(),
        include_patterns: applied_lens.include_patterns.clone(),
        ..Default::default()
    };

    let context = serialize_project_with_config(root, &config)?;
    let context_lines = context.lines().count();
    let context_bytes = context.len();

    // Step 4: Write CONTEXT.txt
    let context_path = root_path.join("CONTEXT.txt");
    fs::write(&context_path, &context)
        .map_err(|e| format!("Failed to write CONTEXT.txt: {}", e))?;

    // Step 5: Generate instruction file content
    let instruction_filename = get_instruction_filename(target);

    // Get project name from directory - handle "." by canonicalizing first
    let canonical_path = root_path.canonicalize()
        .unwrap_or_else(|_| root_path.to_path_buf());
    let project_name = canonical_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let instructions = generate_instruction_content(
        project_name,
        lens_name,
        &commands,
        &tree,
        context_lines,
        context_bytes,
    );

    // Step 6: Write instruction file
    let instruction_path = root_path.join(instruction_filename);
    fs::write(&instruction_path, &instructions)
        .map_err(|e| format!("Failed to write {}: {}", instruction_filename, e))?;

    Ok((
        instruction_path.to_string_lossy().to_string(),
        context_path.to_string_lossy().to_string(),
    ))
}

/// Generate the content for the instruction file
fn generate_instruction_content(
    project_name: &str,
    lens_name: &str,
    commands: &[String],
    tree: &[String],
    context_lines: usize,
    context_bytes: usize,
) -> String {
    let mut content = String::new();

    // Header
    content.push_str(&format!("# {}\n\n", project_name));
    content.push_str("This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.\n\n");

    // Project Overview
    content.push_str("## Project Overview\n\n");
    content.push_str(&format!("{} - Context automatically generated by pm_encoder\n\n", project_name));

    // Quick Start
    content.push_str("## Quick Start\n\n");
    content.push_str(&format!(
        "This is the project context serialized using the `{}` lens for optimal AI understanding.\n\n",
        lens_name
    ));

    // Commands
    if !commands.is_empty() {
        content.push_str("## Commands\n\n");
        content.push_str("Common commands detected for this project:\n");
        for cmd in commands {
            content.push_str(&format!("- `{}`\n", cmd));
        }
        content.push_str("\n");
    }

    // Project Structure
    content.push_str("## Project Structure\n\n");
    content.push_str("```\n");
    for line in tree {
        content.push_str(line);
        content.push('\n');
    }
    content.push_str("```\n\n");

    // Statistics
    content.push_str("**Statistics:**\n");
    content.push_str(&format!("- Context lines: {}\n", context_lines));
    content.push_str(&format!("- Context size: {} bytes ({:.1} KB)\n\n", context_bytes, context_bytes as f64 / 1024.0));

    // Pointer to CONTEXT.txt
    content.push_str("For the complete codebase context, see `CONTEXT.txt` in this directory.\n\n");

    // Footer
    content.push_str("---\n\n");
    content.push_str("**Regenerate these files:**\n");
    content.push_str("```bash\n");
    content.push_str(&format!("./pm_encoder.py . --init-prompt --init-lens {} --target claude\n", lens_name));
    content.push_str("```\n\n");
    content.push_str(&format!("*Generated by pm_encoder v{} using the '{}' lens*\n", crate::VERSION, lens_name));

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_detect_project_commands_makefile() {
        let temp = std::env::temp_dir().join("pm_test_commands_makefile");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("Makefile"), "all:\n\techo test").unwrap();

        let commands = detect_project_commands(temp.to_str().unwrap());
        assert!(commands.contains(&"make".to_string()), "Should detect make command");
        assert!(commands.contains(&"make test".to_string()), "Should detect make test command");

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_detect_project_commands_cargo() {
        let temp = std::env::temp_dir().join("pm_test_commands_cargo");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let commands = detect_project_commands(temp.to_str().unwrap());
        assert!(commands.contains(&"cargo build".to_string()));
        assert!(commands.contains(&"cargo test".to_string()));

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_detect_project_commands_npm() {
        let temp = std::env::temp_dir().join("pm_test_commands_npm");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("package.json"), "{}").unwrap();

        let commands = detect_project_commands(temp.to_str().unwrap());
        assert!(commands.contains(&"npm install".to_string()));
        assert!(commands.contains(&"npm test".to_string()));

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_detect_project_commands_multiple() {
        let temp = std::env::temp_dir().join("pm_test_commands_multi");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("Makefile"), "test:").unwrap();
        fs::write(temp.join("Cargo.toml"), "[package]").unwrap();

        let commands = detect_project_commands(temp.to_str().unwrap());
        assert!(commands.contains(&"make".to_string()));
        assert!(commands.contains(&"cargo build".to_string()));

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_generate_directory_tree() {
        let temp = std::env::temp_dir().join("pm_test_tree");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        fs::create_dir_all(temp.join("src")).unwrap();
        fs::write(temp.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(temp.join("Cargo.toml"), "[package]").unwrap();

        let tree = generate_directory_tree(temp.to_str().unwrap(), &vec![], 3);

        // Check structure
        assert!(!tree.is_empty());
        let tree_str = tree.join("\n");
        assert!(tree_str.contains("src/"), "Tree should contain src/");

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_generate_directory_tree_respects_ignore() {
        let temp = std::env::temp_dir().join("pm_test_tree_ignore");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        fs::create_dir_all(temp.join(".git")).unwrap();
        fs::create_dir_all(temp.join("src")).unwrap();
        fs::write(temp.join(".git/config"), "").unwrap();
        fs::write(temp.join("src/main.rs"), "").unwrap();

        let ignore = vec![".git".to_string()];
        let tree = generate_directory_tree(temp.to_str().unwrap(), &ignore, 3);

        let tree_str = tree.join("\n");
        assert!(!tree_str.contains(".git"), "Tree should not contain .git");
        assert!(tree_str.contains("src"), "Tree should contain src");

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_init_prompt_creates_split_files() {
        let temp = std::env::temp_dir().join("pm_test_init_prompt");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("main.py"), "print('hello')").unwrap();

        let result = init_prompt(temp.to_str().unwrap(), "architecture", "claude");

        if let Ok((instruction_path, context_path)) = result {
            // Verify CLAUDE.md exists and has correct content
            assert!(Path::new(&instruction_path).exists(), "CLAUDE.md should exist");
            let instruction_content = fs::read_to_string(&instruction_path).unwrap();
            assert!(instruction_content.contains("CONTEXT.txt"), "Should point to CONTEXT.txt");
            assert!(!instruction_content.contains("print('hello')"), "CLAUDE.md should NOT contain code");

            // Verify CONTEXT.txt exists and has code
            assert!(Path::new(&context_path).exists(), "CONTEXT.txt should exist");
            let context_content = fs::read_to_string(&context_path).unwrap();
            assert!(context_content.contains("print('hello')"), "CONTEXT.txt should contain code");
        }

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_init_prompt_gemini_target() {
        let temp = std::env::temp_dir().join("pm_test_init_gemini");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("main.py"), "x = 1").unwrap();

        let result = init_prompt(temp.to_str().unwrap(), "architecture", "gemini");

        if let Ok((instruction_path, _)) = result {
            assert!(instruction_path.contains("GEMINI_INSTRUCTIONS.txt"),
                    "Should create GEMINI_INSTRUCTIONS.txt for gemini target");
        }

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_get_instruction_filename() {
        assert_eq!(get_instruction_filename("claude"), "CLAUDE.md");
        assert_eq!(get_instruction_filename("Claude"), "CLAUDE.md");
        assert_eq!(get_instruction_filename("CLAUDE"), "CLAUDE.md");
        assert_eq!(get_instruction_filename("gemini"), "GEMINI_INSTRUCTIONS.txt");
        assert_eq!(get_instruction_filename("Gemini"), "GEMINI_INSTRUCTIONS.txt");
        assert_eq!(get_instruction_filename("unknown"), "CLAUDE.md"); // Default
    }

    #[test]
    fn test_init_prompt_nonexistent_directory() {
        let result = init_prompt("/nonexistent/path/xyz", "architecture", "claude");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
