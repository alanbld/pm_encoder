/// Language analyzers for extracting metadata from source files
pub mod rust_analyzer;

pub use rust_analyzer::RustAnalyzer;

/// Result of file analysis containing extracted metadata
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub language: String,
    pub classes: Vec<String>,
    pub functions: Vec<String>,
    pub imports: Vec<String>,
    pub entry_points: Vec<String>,
    pub config_keys: Vec<String>,
    pub documentation: Vec<String>,
    pub markers: Vec<String>,
    pub category: String,
    pub critical_sections: Vec<(usize, usize)>,
}

impl AnalysisResult {
    /// Create a new empty analysis result
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
            classes: Vec::new(),
            functions: Vec::new(),
            imports: Vec::new(),
            entry_points: Vec::new(),
            config_keys: Vec::new(),
            documentation: Vec::new(),
            markers: Vec::new(),
            category: "library".to_string(),
            critical_sections: Vec::new(),
        }
    }
}

/// Trait for language analyzers
pub trait LanguageAnalyzer {
    /// Analyze source code content and extract metadata
    fn analyze(&self, content: &str, file_path: &str) -> AnalysisResult;

    /// Get supported file extensions
    fn supported_extensions(&self) -> &[&str];

    /// Get language name
    fn language_name(&self) -> &str;
}
