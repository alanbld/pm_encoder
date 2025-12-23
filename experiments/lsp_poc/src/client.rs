//! Robust LSP Client for rust-analyzer
//!
//! Production-ready LSP client with proper error handling, timeouts,
//! and structured communication following JSON-RPC 2.0 protocol.

use std::path::Path;
use std::time::{Duration, Instant};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;

use crate::comparison::{Symbol, SymbolKind};

/// LSP client for communicating with rust-analyzer
pub struct LspClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    initialized: bool,
}

/// LSP DocumentSymbol response structure
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DocumentSymbolResponse {
    /// Flat list of SymbolInformation
    Flat(Vec<SymbolInformation>),
    /// Hierarchical DocumentSymbol tree
    Hierarchical(Vec<DocumentSymbol>),
}

/// LSP SymbolInformation (flat format)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SymbolInformation {
    pub name: String,
    pub kind: u32,
    pub location: Location,
    #[serde(rename = "containerName")]
    pub container_name: Option<String>,
}

/// LSP DocumentSymbol (hierarchical format)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: u32,
    pub range: Range,
    #[serde(rename = "selectionRange")]
    pub selection_range: Range,
    pub children: Option<Vec<DocumentSymbol>>,
}

/// LSP Location
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

/// LSP Range
#[derive(Debug, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// LSP Position
#[derive(Debug, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Metrics collected during LSP operations
#[derive(Debug, Default)]
pub struct LspOperationMetrics {
    pub initialization_time: Duration,
    pub did_open_time: Duration,
    pub symbol_request_time: Duration,
    pub total_time: Duration,
}

impl LspClient {
    /// Check if rust-analyzer is available
    pub fn is_available() -> bool {
        std::process::Command::new("rust-analyzer")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Create a new LSP client by spawning rust-analyzer
    pub async fn new() -> Result<Self> {
        let mut child = Command::new("rust-analyzer")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn rust-analyzer. Is it installed?")?;

        let stdin = child.stdin.take().context("Failed to open stdin")?;
        let stdout = BufReader::new(child.stdout.take().context("Failed to open stdout")?);

        Ok(Self {
            child,
            stdin,
            stdout,
            next_id: 1,
            initialized: false,
        })
    }

    /// Initialize the LSP server with the given root path
    pub async fn initialize(&mut self, root_path: &Path) -> Result<Duration> {
        let start = Instant::now();

        let root_uri = format!(
            "file://{}",
            root_path
                .canonicalize()
                .context("Failed to canonicalize root path")?
                .display()
        );

        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": root_uri,
                "capabilities": {
                    "textDocument": {
                        "documentSymbol": {
                            "hierarchicalDocumentSymbolSupport": true
                        }
                    }
                }
            }
        });

        self.next_id += 1;

        let response = timeout(Duration::from_secs(10), self.send_request(&request))
            .await
            .context("LSP initialization timeout")??;

        // Validate response has result
        if response.get("result").is_none() {
            if let Some(error) = response.get("error") {
                anyhow::bail!("LSP initialization error: {:?}", error);
            }
            anyhow::bail!("LSP initialization failed: no result in response");
        }

        // Send initialized notification
        let initialized = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });
        self.send_notification(&initialized).await?;

        self.initialized = true;
        Ok(start.elapsed())
    }

    /// Request document symbols for a file
    pub async fn document_symbol(&mut self, file_path: &Path) -> Result<(Vec<Symbol>, LspOperationMetrics)> {
        if !self.initialized {
            anyhow::bail!("LSP client not initialized");
        }

        let mut metrics = LspOperationMetrics::default();
        let total_start = Instant::now();

        let file_path = file_path
            .canonicalize()
            .context(format!("Failed to canonicalize path: {:?}", file_path))?;
        let file_uri = format!("file://{}", file_path.display());

        // Read file content
        let content = tokio::fs::read_to_string(&file_path)
            .await
            .context(format!("Failed to read file: {:?}", file_path))?;

        // Send textDocument/didOpen
        let did_open_start = Instant::now();
        let did_open = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": &file_uri,
                    "languageId": "rust",
                    "version": 1,
                    "text": content
                }
            }
        });
        self.send_notification(&did_open).await?;
        metrics.did_open_time = did_open_start.elapsed();

        // Small delay to allow server to process
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Send textDocument/documentSymbol request
        let symbol_start = Instant::now();
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": "textDocument/documentSymbol",
            "params": {
                "textDocument": {
                    "uri": &file_uri
                }
            }
        });
        self.next_id += 1;

        let response = timeout(Duration::from_secs(30), self.send_request(&request))
            .await
            .context("Symbol request timeout")??;
        metrics.symbol_request_time = symbol_start.elapsed();

        // Parse symbols from response
        let symbols = self.parse_symbol_response(&response)?;

        // Send textDocument/didClose
        let did_close = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": {
                "textDocument": {
                    "uri": &file_uri
                }
            }
        });
        self.send_notification(&did_close).await?;

        metrics.total_time = total_start.elapsed();
        Ok((symbols, metrics))
    }

    /// Shutdown the LSP server gracefully
    pub async fn shutdown(mut self) -> Result<()> {
        // Send shutdown request
        let shutdown = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": "shutdown",
            "params": null
        });
        self.next_id += 1;

        let _ = timeout(Duration::from_secs(5), self.send_request(&shutdown)).await;

        // Send exit notification
        let exit = json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": null
        });
        let _ = self.send_notification(&exit).await;

        // Kill the process if it's still running
        let _ = self.child.kill().await;

        Ok(())
    }

    /// Send a JSON-RPC request and wait for response
    async fn send_request(&mut self, request: &Value) -> Result<Value> {
        let message = serde_json::to_string(request)?;
        let frame = format!("Content-Length: {}\r\n\r\n{}", message.len(), message);

        self.stdin.write_all(frame.as_bytes()).await?;
        self.stdin.flush().await?;

        // Read response
        self.read_response().await
    }

    /// Send a JSON-RPC notification (no response expected)
    async fn send_notification(&mut self, notification: &Value) -> Result<()> {
        let message = serde_json::to_string(notification)?;
        let frame = format!("Content-Length: {}\r\n\r\n{}", message.len(), message);

        self.stdin.write_all(frame.as_bytes()).await?;
        self.stdin.flush().await?;

        Ok(())
    }

    /// Read a JSON-RPC response from the server
    async fn read_response(&mut self) -> Result<Value> {
        loop {
            // Read headers
            let mut content_length: Option<usize> = None;
            loop {
                let mut line = String::new();
                self.stdout.read_line(&mut line).await?;
                let line = line.trim();

                if line.is_empty() {
                    break;
                }

                if let Some(value) = line.strip_prefix("Content-Length: ") {
                    content_length = Some(value.parse()?);
                }
            }

            let content_length = content_length.context("Missing Content-Length header")?;

            // Read body
            let mut body = vec![0u8; content_length];
            self.stdout.read_exact(&mut body).await?;

            let response: Value = serde_json::from_slice(&body)?;

            // Skip notifications (no id field) - we only want responses
            if response.get("id").is_some() {
                return Ok(response);
            }
        }
    }

    /// Parse document symbol response into our Symbol type
    fn parse_symbol_response(&self, response: &Value) -> Result<Vec<Symbol>> {
        let result = response.get("result").context("No result in response")?;

        // Handle null result (no symbols)
        if result.is_null() {
            return Ok(Vec::new());
        }

        let mut symbols = Vec::new();

        // Try to parse as DocumentSymbol array (hierarchical)
        if let Ok(doc_symbols) = serde_json::from_value::<Vec<DocumentSymbol>>(result.clone()) {
            self.flatten_document_symbols(&doc_symbols, &mut symbols);
            return Ok(symbols);
        }

        // Try to parse as SymbolInformation array (flat)
        if let Ok(sym_infos) = serde_json::from_value::<Vec<SymbolInformation>>(result.clone()) {
            for info in sym_infos {
                symbols.push(Symbol {
                    name: info.name,
                    kind: lsp_kind_to_symbol_kind(info.kind),
                    line: info.location.range.start.line as usize + 1,
                });
            }
            return Ok(symbols);
        }

        // Empty array is valid
        if result.as_array().map(|a| a.is_empty()).unwrap_or(false) {
            return Ok(Vec::new());
        }

        anyhow::bail!("Failed to parse document symbol response: {:?}", result);
    }

    /// Flatten hierarchical DocumentSymbol tree into flat list
    fn flatten_document_symbols(&self, symbols: &[DocumentSymbol], output: &mut Vec<Symbol>) {
        for sym in symbols {
            output.push(Symbol {
                name: sym.name.clone(),
                kind: lsp_kind_to_symbol_kind(sym.kind),
                line: sym.selection_range.start.line as usize + 1,
            });

            // Recursively process children
            if let Some(children) = &sym.children {
                self.flatten_document_symbols(children, output);
            }
        }
    }

    /// Check if the client is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

/// Convert LSP SymbolKind (number) to our SymbolKind enum
fn lsp_kind_to_symbol_kind(kind: u32) -> SymbolKind {
    // LSP SymbolKind enum values
    match kind {
        1 => SymbolKind::Unknown,   // File
        2 => SymbolKind::Mod,       // Module
        3 => SymbolKind::Unknown,   // Namespace
        4 => SymbolKind::Unknown,   // Package
        5 => SymbolKind::Class,     // Class
        6 => SymbolKind::Method,    // Method
        7 => SymbolKind::Unknown,   // Property
        8 => SymbolKind::Unknown,   // Field
        9 => SymbolKind::Unknown,   // Constructor
        10 => SymbolKind::Enum,     // Enum
        11 => SymbolKind::Trait,    // Interface
        12 => SymbolKind::Function, // Function
        13 => SymbolKind::Unknown,  // Variable
        14 => SymbolKind::Const,    // Constant
        15 => SymbolKind::Unknown,  // String
        16 => SymbolKind::Unknown,  // Number
        17 => SymbolKind::Unknown,  // Boolean
        18 => SymbolKind::Unknown,  // Array
        19 => SymbolKind::Unknown,  // Object
        20 => SymbolKind::Unknown,  // Key
        21 => SymbolKind::Unknown,  // Null
        22 => SymbolKind::Unknown,  // EnumMember
        23 => SymbolKind::Struct,   // Struct
        24 => SymbolKind::Unknown,  // Event
        25 => SymbolKind::Unknown,  // Operator
        26 => SymbolKind::Type,     // TypeParameter
        _ => SymbolKind::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_kind_conversion() {
        assert_eq!(lsp_kind_to_symbol_kind(12), SymbolKind::Function);
        assert_eq!(lsp_kind_to_symbol_kind(23), SymbolKind::Struct);
        assert_eq!(lsp_kind_to_symbol_kind(10), SymbolKind::Enum);
        assert_eq!(lsp_kind_to_symbol_kind(11), SymbolKind::Trait);
        assert_eq!(lsp_kind_to_symbol_kind(2), SymbolKind::Mod);
    }

    #[test]
    fn test_is_available() {
        // This test just verifies the function runs without panic
        let _available = LspClient::is_available();
    }

    #[tokio::test]
    async fn test_lsp_client_initialization() {
        if !LspClient::is_available() {
            eprintln!("SKIPPED: rust-analyzer not available");
            return;
        }

        let mut client = LspClient::new().await.expect("Failed to create client");
        assert!(!client.is_initialized());

        let root = std::env::current_dir().unwrap();
        let init_time = client.initialize(&root).await.expect("Failed to initialize");

        assert!(client.is_initialized());
        assert!(init_time < Duration::from_secs(10));

        client.shutdown().await.expect("Failed to shutdown");
    }
}
