//! voyager-ast: Language-agnostic Structural Indexer
//!
//! This crate provides the "optics layer" for Voyager Observatory, enabling
//! fast, resilient, multi-language structural indexing via Tree-sitter.
//!
//! # Design Philosophy: Telescope, Not Compiler
//!
//! voyager-ast is explicitly designed as an observation instrument:
//! - Best-effort recovery over formal correctness
//! - ~90% structural accuracy is the target, not 100%
//! - Explicit uncertainty via `UnknownNode` markers
//! - Never silently drop content we can't parse
//!
//! # Two Operating Modes
//!
//! 1. **Index (Planetarium)**: Fast project-wide scan
//!    - Top-level declarations, imports, file-level comments
//!    - No intra-function control-flow by default
//!
//! 2. **Zoom (Microscope)**: Deep symbol inspection
//!    - Full body of target symbol
//!    - Nested blocks, control flow, calls, comments
//!
//! # Example
//!
//! ```rust,ignore
//! use voyager_ast::{AstProvider, TreeSitterProvider, IndexOptions};
//! use std::path::Path;
//!
//! let provider = TreeSitterProvider::new();
//! let model = provider.index_project(Path::new("."), &IndexOptions::default())?;
//!
//! for (path, file) in &model.files {
//!     println!("{}: {} declarations", path, file.declarations.len());
//! }
//! ```

pub mod ir;
pub mod error;
pub mod provider;
pub mod adapters;
mod registry;

// Re-export core types for convenience
pub use ir::{
    // Core types
    File, Span, Region,
    // Declarations
    Declaration, DeclarationKind, Visibility, Parameter,
    // Blocks and control flow
    Block, ControlFlow, ControlFlowKind, Call,
    // Imports
    ImportLike, ImportKind,
    // Comments
    Comment, CommentKind,
    // Error recovery
    UnknownNode, UnparsedBlock,
    // Language
    LanguageId,
};

pub use error::AstError;
pub use provider::{
    AstProvider, IndexOptions, ZoomOptions,
    PlanetariumModel, MicroscopeModel, IndexStats, IndexError, ContextWindow,
};
pub use adapters::LanguageAdapter;
pub use registry::AdapterRegistry;

/// Version of the IR schema
pub const IR_VERSION: &str = "v1";

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
