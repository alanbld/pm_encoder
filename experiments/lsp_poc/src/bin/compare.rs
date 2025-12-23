//! LSP vs Regex Symbol Extraction Comparison Tool
//!
//! This binary runs a comprehensive comparison between regex-based and LSP-based
//! symbol extraction on the pm_encoder codebase.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use serde::Serialize;
use walkdir::WalkDir;

use lsp_poc::client::LspClient;
use lsp_poc::comparison::RegexExtractor;

/// Result of comparing symbol extraction methods for a single file
#[derive(Debug, Serialize)]
struct ComparisonResult {
    file: String,
    file_size_bytes: u64,
    regex_time_us: u128,
    lsp_time_ms: u128,
    regex_count: usize,
    lsp_count: usize,
    matched_count: usize,
    jaccard_index: f64,
    precision: f64,
    recall: f64,
    f1_score: f64,
    regex_only: String,
    lsp_only: String,
}

/// Aggregate statistics across all files
#[derive(Debug, Default)]
struct AggregateStats {
    total_files: usize,
    total_regex_time_us: u128,
    total_lsp_time_ms: u128,
    total_regex_symbols: usize,
    total_lsp_symbols: usize,
    total_matched: usize,
    jaccard_sum: f64,
    precision_sum: f64,
    recall_sum: f64,
    f1_sum: f64,
}

impl AggregateStats {
    fn avg_jaccard(&self) -> f64 {
        if self.total_files > 0 {
            self.jaccard_sum / self.total_files as f64
        } else {
            0.0
        }
    }

    fn avg_precision(&self) -> f64 {
        if self.total_files > 0 {
            self.precision_sum / self.total_files as f64
        } else {
            0.0
        }
    }

    fn avg_recall(&self) -> f64 {
        if self.total_files > 0 {
            self.recall_sum / self.total_files as f64
        } else {
            0.0
        }
    }

    fn avg_f1(&self) -> f64 {
        if self.total_files > 0 {
            self.f1_sum / self.total_files as f64
        } else {
            0.0
        }
    }

    fn speed_ratio(&self) -> f64 {
        if self.total_regex_time_us > 0 {
            (self.total_lsp_time_ms * 1000) as f64 / self.total_regex_time_us as f64
        } else {
            0.0
        }
    }
}

/// Calculate Jaccard index between two sets
fn jaccard_index(set_a: &HashSet<String>, set_b: &HashSet<String>) -> f64 {
    let intersection = set_a.intersection(set_b).count();
    let union = set_a.union(set_b).count();
    if union > 0 {
        intersection as f64 / union as f64
    } else {
        1.0 // Both empty = perfect match
    }
}

/// Calculate precision, recall, and F1 score
/// Treating LSP as ground truth
fn precision_recall_f1(
    regex_symbols: &HashSet<String>,
    lsp_symbols: &HashSet<String>,
) -> (f64, f64, f64) {
    let true_positives = regex_symbols.intersection(lsp_symbols).count();

    // Precision = TP / (TP + FP) = TP / regex_count
    let precision = if !regex_symbols.is_empty() {
        true_positives as f64 / regex_symbols.len() as f64
    } else {
        1.0
    };

    // Recall = TP / (TP + FN) = TP / lsp_count
    let recall = if !lsp_symbols.is_empty() {
        true_positives as f64 / lsp_symbols.len() as f64
    } else {
        1.0
    };

    // F1 = 2 * (precision * recall) / (precision + recall)
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    (precision, recall, f1)
}

/// Find all Rust source files in a directory
fn find_rust_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |ext| ext == "rs")
                // Skip test directories and generated files
                && !e.path().to_string_lossy().contains("/target/")
        })
        .map(|e| e.into_path())
        .collect()
}

/// Run comparison on a single file
async fn compare_file(
    file_path: &Path,
    regex_extractor: &RegexExtractor,
    lsp_client: &mut LspClient,
) -> Result<ComparisonResult> {
    // Read file
    let content = tokio::fs::read_to_string(file_path)
        .await
        .context(format!("Failed to read: {:?}", file_path))?;

    let file_size = content.len() as u64;

    // Regex extraction
    let regex_start = Instant::now();
    let regex_symbols = regex_extractor.extract(&content);
    let regex_time = regex_start.elapsed();

    // LSP extraction
    let (lsp_symbols, lsp_metrics) = lsp_client.document_symbol(file_path).await?;

    // Convert to name sets for comparison
    let regex_names: HashSet<String> = regex_symbols.iter().map(|s| s.name.clone()).collect();
    let lsp_names: HashSet<String> = lsp_symbols.iter().map(|s| s.name.clone()).collect();

    // Calculate metrics
    let matched_count = regex_names.intersection(&lsp_names).count();
    let jaccard = jaccard_index(&regex_names, &lsp_names);
    let (precision, recall, f1) = precision_recall_f1(&regex_names, &lsp_names);

    // Find discrepancies
    let regex_only: Vec<String> = regex_names.difference(&lsp_names).cloned().collect();
    let lsp_only: Vec<String> = lsp_names.difference(&regex_names).cloned().collect();

    Ok(ComparisonResult {
        file: file_path.to_string_lossy().to_string(),
        file_size_bytes: file_size,
        regex_time_us: regex_time.as_micros(),
        lsp_time_ms: lsp_metrics.total_time.as_millis(),
        regex_count: regex_symbols.len(),
        lsp_count: lsp_symbols.len(),
        matched_count,
        jaccard_index: jaccard,
        precision,
        recall,
        f1_score: f1,
        regex_only: regex_only.join(", "),
        lsp_only: lsp_only.join(", "),
    })
}

/// Print summary statistics
fn print_summary(stats: &AggregateStats) {
    println!("\n{}", "=".repeat(60));
    println!("LSP vs Regex Comparison - Summary");
    println!("{}", "=".repeat(60));
    println!("Files analyzed:        {}", stats.total_files);
    println!("Total regex symbols:   {}", stats.total_regex_symbols);
    println!("Total LSP symbols:     {}", stats.total_lsp_symbols);
    println!("Total matched:         {}", stats.total_matched);
    println!();
    println!("Average Jaccard Index: {:.4}", stats.avg_jaccard());
    println!("Average Precision:     {:.4}", stats.avg_precision());
    println!("Average Recall:        {:.4}", stats.avg_recall());
    println!("Average F1 Score:      {:.4}", stats.avg_f1());
    println!();
    println!(
        "Total regex time:      {} us ({:.2} ms)",
        stats.total_regex_time_us,
        stats.total_regex_time_us as f64 / 1000.0
    );
    println!(
        "Total LSP time:        {} ms ({:.2} s)",
        stats.total_lsp_time_ms,
        stats.total_lsp_time_ms as f64 / 1000.0
    );
    println!("Speed ratio (LSP/regex): {:.0}x", stats.speed_ratio());
    println!("{}", "=".repeat(60));
}

/// Write results to CSV file
fn write_csv(results: &[ComparisonResult], path: &Path) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    for result in results {
        wtr.serialize(result)?;
    }
    wtr.flush()?;
    Ok(())
}

/// Write summary to text file
fn write_summary(stats: &AggregateStats, path: &Path) -> Result<()> {
    use std::io::Write;
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "LSP vs Regex Experiment - Phase 3")?;
    writeln!(file, "=================================")?;
    writeln!(file, "Date: {}", chrono_lite())?;
    writeln!(file, "Files analyzed: {}", stats.total_files)?;
    writeln!(file, "Average accuracy (Jaccard): {:.4}", stats.avg_jaccard())?;
    writeln!(file, "Average precision: {:.4}", stats.avg_precision())?;
    writeln!(file, "Average recall: {:.4}", stats.avg_recall())?;
    writeln!(file, "Average F1 score: {:.4}", stats.avg_f1())?;
    writeln!(
        file,
        "Average regex latency: {:.0} us",
        if stats.total_files > 0 {
            stats.total_regex_time_us as f64 / stats.total_files as f64
        } else {
            0.0
        }
    )?;
    writeln!(
        file,
        "Average LSP latency: {:.0} ms",
        if stats.total_files > 0 {
            stats.total_lsp_time_ms as f64 / stats.total_files as f64
        } else {
            0.0
        }
    )?;
    writeln!(file, "Speed ratio: {:.0}x", stats.speed_ratio())?;
    Ok(())
}

/// Simple date string without chrono dependency
fn chrono_lite() -> String {
    // Use system time for basic timestamp
    format!("{:?}", std::time::SystemTime::now())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("LSP vs Regex Symbol Extraction Comparison");
    println!("==========================================\n");

    // Check if rust-analyzer is available
    if !LspClient::is_available() {
        eprintln!("ERROR: rust-analyzer not found in PATH");
        eprintln!("Install with: rustup component add rust-analyzer");
        std::process::exit(1);
    }

    // Determine source directory (default to pm_encoder/rust/src)
    let src_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Try to find pm_encoder rust src relative to current dir
            let candidates = [
                PathBuf::from("../../rust/src"),
                PathBuf::from("../rust/src"),
                PathBuf::from("./rust/src"),
                PathBuf::from("./src"),
            ];

            for candidate in &candidates {
                if candidate.exists() {
                    return candidate.clone();
                }
            }

            PathBuf::from("./src")
        });

    if !src_dir.exists() {
        eprintln!("ERROR: Source directory not found: {:?}", src_dir);
        eprintln!("Usage: compare [SOURCE_DIR]");
        std::process::exit(1);
    }

    println!("Source directory: {:?}", src_dir.canonicalize()?);

    // Find all Rust files
    let rust_files = find_rust_files(&src_dir);
    println!("Found {} Rust files\n", rust_files.len());

    if rust_files.is_empty() {
        eprintln!("No Rust files found in {:?}", src_dir);
        std::process::exit(1);
    }

    // Initialize LSP client
    println!("Starting rust-analyzer...");
    let mut lsp_client = LspClient::new().await?;

    let root_path = src_dir.canonicalize()?;
    let init_time = lsp_client.initialize(&root_path).await?;
    println!("rust-analyzer initialized in {:?}\n", init_time);

    // Initialize regex extractor
    let regex_extractor = RegexExtractor::new();

    // Run comparison on each file
    let mut results = Vec::new();
    let mut stats = AggregateStats::default();

    for (i, file_path) in rust_files.iter().enumerate() {
        print!(
            "[{}/{}] Processing: {} ... ",
            i + 1,
            rust_files.len(),
            file_path.file_name().unwrap_or_default().to_string_lossy()
        );

        match compare_file(file_path, &regex_extractor, &mut lsp_client).await {
            Ok(result) => {
                println!(
                    "OK (regex: {} symbols in {}us, LSP: {} symbols in {}ms, Jaccard: {:.2})",
                    result.regex_count,
                    result.regex_time_us,
                    result.lsp_count,
                    result.lsp_time_ms,
                    result.jaccard_index
                );

                // Update aggregate stats
                stats.total_files += 1;
                stats.total_regex_time_us += result.regex_time_us;
                stats.total_lsp_time_ms += result.lsp_time_ms;
                stats.total_regex_symbols += result.regex_count;
                stats.total_lsp_symbols += result.lsp_count;
                stats.total_matched += result.matched_count;
                stats.jaccard_sum += result.jaccard_index;
                stats.precision_sum += result.precision;
                stats.recall_sum += result.recall;
                stats.f1_sum += result.f1_score;

                results.push(result);
            }
            Err(e) => {
                println!("FAILED: {}", e);
            }
        }
    }

    // Shutdown LSP client
    println!("\nShutting down rust-analyzer...");
    lsp_client.shutdown().await?;

    // Print summary
    print_summary(&stats);

    // Write outputs
    let output_dir = PathBuf::from(".");
    let csv_path = output_dir.join("comparison_results.csv");
    let summary_path = output_dir.join("experiment_summary.txt");

    write_csv(&results, &csv_path)?;
    println!("\nResults written to: {:?}", csv_path);

    write_summary(&stats, &summary_path)?;
    println!("Summary written to: {:?}", summary_path);

    // Validation against decision matrix
    println!("\n{}", "=".repeat(60));
    println!("Validation Against Decision Matrix");
    println!("{}", "=".repeat(60));

    let avg_regex_us = if stats.total_files > 0 {
        stats.total_regex_time_us / stats.total_files as u128
    } else {
        0
    };

    let avg_lsp_ms = if stats.total_files > 0 {
        stats.total_lsp_time_ms / stats.total_files as u128
    } else {
        0
    };

    // Check thresholds
    let regex_ok = avg_regex_us < 100;
    let lsp_ok = avg_lsp_ms < 50;
    let accuracy_ok = stats.avg_jaccard() > 0.95;

    println!(
        "Regex < 100us per file:  {} (avg: {}us)",
        if regex_ok { "PASS" } else { "FAIL" },
        avg_regex_us
    );
    println!(
        "LSP < 50ms per file:     {} (avg: {}ms)",
        if lsp_ok { "PASS" } else { "FAIL" },
        avg_lsp_ms
    );
    println!(
        "Jaccard > 0.95:          {} (avg: {:.4})",
        if accuracy_ok { "PASS" } else { "FAIL" },
        stats.avg_jaccard()
    );
    println!("{}", "=".repeat(60));

    Ok(())
}
