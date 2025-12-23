//! Mock LSP Server for Testing
//!
//! Implements a minimal JSON-RPC 2.0 server that responds to LSP initialize requests.
//! Uses stdio for communication (stdin/stdout) following the LSP specification.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Read, Write};

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
struct Request {
    jsonrpc: String,
    id: Option<u64>,
    method: String,
    #[serde(default)]
    params: Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
struct Response {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ResponseError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Serialize)]
struct ResponseError {
    code: i32,
    message: String,
}

impl Response {
    fn success(id: u64, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: u64, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(ResponseError {
                code,
                message: message.to_string(),
            }),
        }
    }
}

/// Read a single LSP message from stdin.
/// LSP messages have the format:
/// ```
/// Content-Length: <length>\r\n
/// \r\n
/// <json-body>
/// ```
fn read_message(reader: &mut BufReader<io::Stdin>) -> io::Result<Option<String>> {
    // Read headers until empty line
    let mut content_length: Option<usize> = None;

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;

        if bytes_read == 0 {
            // EOF
            return Ok(None);
        }

        let line = line.trim_end_matches(|c| c == '\r' || c == '\n');

        if line.is_empty() {
            // End of headers
            break;
        }

        if let Some(value) = line.strip_prefix("Content-Length: ") {
            content_length = value.trim().parse().ok();
        }
        // Ignore other headers (like Content-Type)
    }

    let content_length = match content_length {
        Some(len) => len,
        None => {
            eprintln!("[mock_lsp] Missing Content-Length header");
            return Ok(None);
        }
    };

    // Read the JSON body
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;

    String::from_utf8(body)
        .map(Some)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Write an LSP message to stdout.
fn write_message(response: &Response) -> io::Result<()> {
    let body = serde_json::to_string(response)?;
    let message = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);

    let mut stdout = io::stdout().lock();
    stdout.write_all(message.as_bytes())?;
    stdout.flush()?;

    eprintln!("[mock_lsp] Sent response: {}", body);
    Ok(())
}

/// Handle an incoming request and generate a response.
fn handle_request(request: &Request) -> Option<Response> {
    let id = request.id?; // Notifications (no id) don't get responses

    eprintln!("[mock_lsp] Handling method: {}", request.method);

    match request.method.as_str() {
        "initialize" => {
            let result = json!({
                "capabilities": {
                    "textDocumentSync": 1,
                    "documentSymbolProvider": true,
                    "foldingRangeProvider": true
                },
                "serverInfo": {
                    "name": "mock-lsp",
                    "version": "0.1.0"
                }
            });
            Some(Response::success(id, result))
        }
        "shutdown" => {
            Some(Response::success(id, Value::Null))
        }
        _ => {
            // Method not found
            Some(Response::error(id, -32601, "Method not found"))
        }
    }
}

fn main() {
    eprintln!("[mock_lsp] Starting Mock LSP Server v0.1.0");

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);

    loop {
        match read_message(&mut reader) {
            Ok(Some(body)) => {
                eprintln!("[mock_lsp] Received: {}", body);

                match serde_json::from_str::<Request>(&body) {
                    Ok(request) => {
                        if let Some(response) = handle_request(&request) {
                            if let Err(e) = write_message(&response) {
                                eprintln!("[mock_lsp] Failed to write response: {}", e);
                                break;
                            }
                        }

                        // Handle exit after shutdown
                        if request.method == "shutdown" {
                            eprintln!("[mock_lsp] Shutdown requested, exiting");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("[mock_lsp] Failed to parse request: {}", e);
                        // Send parse error response
                        let response = Response::error(0, -32700, "Parse error");
                        let _ = write_message(&response);
                    }
                }
            }
            Ok(None) => {
                eprintln!("[mock_lsp] EOF or missing Content-Length, exiting");
                break;
            }
            Err(e) => {
                eprintln!("[mock_lsp] Read error: {}", e);
                break;
            }
        }
    }

    eprintln!("[mock_lsp] Server stopped");
}
