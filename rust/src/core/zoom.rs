//! Fractal Protocol: Zoom Actions
//!
//! This module implements the interactive zoom feature that allows LLMs to request
//! deeper context for specific code elements.
//!
//! # Protocol
//!
//! When content is truncated, a zoom affordance is embedded:
//! ```text
//! /* ZOOM_AFFORDANCE: pm_encoder --zoom function=apply_budget --budget=1000 */
//! ```
//!
//! The LLM can then request expansion via MCP or CLI.

use crate::core::error::{EncoderError, Result};
use std::fmt;

/// Target type for zoom operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoomTarget {
    /// Zoom into a specific function
    Function(String),
    /// Zoom into a specific class/struct
    Class(String),
    /// Zoom into a module
    Module(String),
    /// Zoom into a file with optional line range
    File {
        path: String,
        start_line: Option<usize>,
        end_line: Option<usize>,
    },
}

impl ZoomTarget {
    /// Parse a zoom target from string format "type=value"
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(EncoderError::InvalidZoomTarget {
                target: s.to_string(),
            });
        }

        let (kind, value) = (parts[0], parts[1]);
        match kind {
            "function" | "fn" => Ok(ZoomTarget::Function(value.to_string())),
            "class" | "struct" => Ok(ZoomTarget::Class(value.to_string())),
            "module" | "mod" => Ok(ZoomTarget::Module(value.to_string())),
            "file" => {
                // Parse file path, optionally with line range: path:start-end
                if let Some(colon_pos) = value.rfind(':') {
                    let path = value[..colon_pos].to_string();
                    let range = &value[colon_pos + 1..];
                    if let Some(dash_pos) = range.find('-') {
                        let start = range[..dash_pos].parse().ok();
                        let end = range[dash_pos + 1..].parse().ok();
                        Ok(ZoomTarget::File {
                            path,
                            start_line: start,
                            end_line: end,
                        })
                    } else {
                        Ok(ZoomTarget::File {
                            path,
                            start_line: range.parse().ok(),
                            end_line: None,
                        })
                    }
                } else {
                    Ok(ZoomTarget::File {
                        path: value.to_string(),
                        start_line: None,
                        end_line: None,
                    })
                }
            }
            _ => Err(EncoderError::InvalidZoomTarget {
                target: s.to_string(),
            }),
        }
    }

    /// Generate the CLI command for this zoom target
    pub fn to_command(&self, budget: Option<usize>) -> String {
        let target_str = match self {
            ZoomTarget::Function(name) => format!("function={}", name),
            ZoomTarget::Class(name) => format!("class={}", name),
            ZoomTarget::Module(name) => format!("module={}", name),
            ZoomTarget::File { path, start_line, end_line } => {
                match (start_line, end_line) {
                    (Some(s), Some(e)) => format!("file={}:{}-{}", path, s, e),
                    (Some(s), None) => format!("file={}:{}", path, s),
                    _ => format!("file={}", path),
                }
            }
        };

        match budget {
            Some(b) => format!("pm_encoder --zoom {} --budget {}", target_str, b),
            None => format!("pm_encoder --zoom {}", target_str),
        }
    }
}

impl fmt::Display for ZoomTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoomTarget::Function(name) => write!(f, "function:{}", name),
            ZoomTarget::Class(name) => write!(f, "class:{}", name),
            ZoomTarget::Module(name) => write!(f, "module:{}", name),
            ZoomTarget::File { path, start_line, end_line } => {
                match (start_line, end_line) {
                    (Some(s), Some(e)) => write!(f, "file:{}[{}-{}]", path, s, e),
                    (Some(s), None) => write!(f, "file:{}[{}]", path, s),
                    _ => write!(f, "file:{}", path),
                }
            }
        }
    }
}

/// Configuration for a zoom operation
#[derive(Debug, Clone)]
pub struct ZoomConfig {
    /// The target to zoom into
    pub target: ZoomTarget,
    /// Token budget for the zoomed content
    pub budget: Option<usize>,
    /// Depth of expansion: "signature", "implementation", or "full"
    pub depth: ZoomDepth,
    /// Include related tests
    pub include_tests: bool,
    /// Context lines around the target
    pub context_lines: usize,
}

/// Depth of zoom expansion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZoomDepth {
    /// Only show signatures/declarations
    Signature,
    /// Show implementation without docstrings
    #[default]
    Implementation,
    /// Show full content including docs and tests
    Full,
}

impl ZoomDepth {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "signature" | "sig" => Some(ZoomDepth::Signature),
            "implementation" | "impl" => Some(ZoomDepth::Implementation),
            "full" => Some(ZoomDepth::Full),
            _ => None,
        }
    }
}

impl Default for ZoomConfig {
    fn default() -> Self {
        Self {
            target: ZoomTarget::Function("main".to_string()),
            budget: Some(1000),
            depth: ZoomDepth::Implementation,
            include_tests: false,
            context_lines: 5,
        }
    }
}

/// A zoom action represents a suggested expansion point
#[derive(Debug, Clone)]
pub struct ZoomAction {
    /// The zoom target
    pub target: ZoomTarget,
    /// Suggested budget
    pub suggested_budget: usize,
    /// Human-readable description
    pub description: String,
    /// The CLI command to execute
    pub command: String,
}

impl ZoomAction {
    /// Create a new zoom action for a function
    pub fn for_function(name: &str, budget: usize) -> Self {
        let target = ZoomTarget::Function(name.to_string());
        let command = target.to_command(Some(budget));
        Self {
            target,
            suggested_budget: budget,
            description: format!("Expand function '{}' ({} tokens)", name, budget),
            command,
        }
    }

    /// Create a new zoom action for a class
    pub fn for_class(name: &str, budget: usize) -> Self {
        let target = ZoomTarget::Class(name.to_string());
        let command = target.to_command(Some(budget));
        Self {
            target,
            suggested_budget: budget,
            description: format!("Expand class '{}' ({} tokens)", name, budget),
            command,
        }
    }

    /// Create a new zoom action for a file
    pub fn for_file(path: &str, budget: usize) -> Self {
        let target = ZoomTarget::File {
            path: path.to_string(),
            start_line: None,
            end_line: None,
        };
        let command = target.to_command(Some(budget));
        Self {
            target,
            suggested_budget: budget,
            description: format!("Expand file '{}' ({} tokens)", path, budget),
            command,
        }
    }

    /// Generate the affordance comment for serialization
    pub fn to_affordance_comment(&self) -> String {
        format!("/* ZOOM_AFFORDANCE: {} */", self.command)
    }

    /// Generate XML representation
    pub fn to_xml(&self) -> String {
        format!(
            "<action type=\"expand\" target=\"{}\" budget=\"{}\" cmd=\"{}\" />",
            self.target, self.suggested_budget, self.command
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_target_parse_function() {
        let target = ZoomTarget::parse("function=apply_budget").unwrap();
        assert!(matches!(target, ZoomTarget::Function(name) if name == "apply_budget"));
    }

    #[test]
    fn test_zoom_target_parse_file_with_range() {
        let target = ZoomTarget::parse("file=src/main.rs:10-50").unwrap();
        if let ZoomTarget::File { path, start_line, end_line } = target {
            assert_eq!(path, "src/main.rs");
            assert_eq!(start_line, Some(10));
            assert_eq!(end_line, Some(50));
        } else {
            panic!("Expected File target");
        }
    }

    #[test]
    fn test_zoom_target_to_command() {
        let target = ZoomTarget::Function("process".to_string());
        assert_eq!(
            target.to_command(Some(1000)),
            "pm_encoder --zoom function=process --budget 1000"
        );
    }

    #[test]
    fn test_zoom_action_for_function() {
        let action = ZoomAction::for_function("main", 500);
        assert!(action.command.contains("function=main"));
        assert!(action.command.contains("--budget 500"));
    }

    #[test]
    fn test_zoom_action_affordance_comment() {
        let action = ZoomAction::for_function("test", 1000);
        let comment = action.to_affordance_comment();
        assert!(comment.starts_with("/* ZOOM_AFFORDANCE:"));
        assert!(comment.ends_with("*/"));
    }

    #[test]
    fn test_zoom_action_xml() {
        let action = ZoomAction::for_class("DataProcessor", 2000);
        let xml = action.to_xml();
        assert!(xml.contains("type=\"expand\""));
        assert!(xml.contains("DataProcessor"));
        assert!(xml.contains("2000"));
    }

    #[test]
    fn test_zoom_depth_from_str() {
        assert_eq!(ZoomDepth::from_str("signature"), Some(ZoomDepth::Signature));
        assert_eq!(ZoomDepth::from_str("full"), Some(ZoomDepth::Full));
        assert_eq!(ZoomDepth::from_str("invalid"), None);
    }
}
