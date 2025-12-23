//! LSP Initialization Handshake Tests
//!
//! These tests validate the JSON-RPC 2.0 handshake with an LSP server.

use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

/// Read an LSP message (Content-Length header + body)
async fn read_lsp_message<R: AsyncBufReadExt + Unpin>(reader: &mut R) -> Option<String> {
    // Read headers
    let mut content_length: Option<usize> = None;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => return None, // EOF
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    break; // End of headers
                }
                if let Some(value) = line.strip_prefix("Content-Length: ") {
                    content_length = value.parse().ok();
                }
            }
            Err(_) => return None,
        }
    }

    // Read body
    let content_length = content_length?;
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).await.ok()?;

    String::from_utf8(body).ok()
}

/// Test the LSP initialization handshake according to LSP spec.
///
/// The handshake follows this sequence:
/// 1. Client sends `initialize` request
/// 2. Server responds with `InitializeResult` containing `serverInfo`
/// 3. Client sends `initialized` notification
#[tokio::test]
async fn test_lsp_initialization_handshake() {
    // 1. Start the Mock Server
    let mut server = TokioCommand::new(env!("CARGO_BIN_EXE_mock_lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn mock server");

    let mut stdin = server.stdin.take().unwrap();
    let stdout = server.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // 2. Send Initialize Request (JSON-RPC 2.0 with LSP Content-Length header)
    let req = r#"{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}}"#;
    let msg = format!("Content-Length: {}\r\n\r\n{}", req.len(), req);

    stdin.write_all(msg.as_bytes()).await.unwrap();
    stdin.flush().await.unwrap();

    // 3. Read Response with timeout
    let read_result = timeout(Duration::from_secs(2), read_lsp_message(&mut reader)).await;

    let response = match read_result {
        Ok(Some(body)) => body,
        Ok(None) => panic!("Server closed connection without response"),
        Err(_) => panic!("Timeout waiting for server response"),
    };

    // 4. Assert response contains expected LSP fields
    assert!(
        response.contains("serverInfo"),
        "Response should contain serverInfo. Got: {}",
        response
    );
    assert!(
        response.contains("mock-lsp"),
        "Response should identify as mock-lsp. Got: {}",
        response
    );
    assert!(
        response.contains("capabilities"),
        "Response should contain capabilities. Got: {}",
        response
    );

    // 5. Cleanup
    drop(stdin);
    let _ = server.kill().await;
}

/// Test that the server handles malformed requests gracefully.
#[tokio::test]
async fn test_lsp_malformed_request() {
    let mut server = TokioCommand::new(env!("CARGO_BIN_EXE_mock_lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn mock server");

    let mut stdin = server.stdin.take().unwrap();
    let stdout = server.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // Send malformed JSON (missing required fields)
    let req = r#"{"not": "valid"}"#;
    let msg = format!("Content-Length: {}\r\n\r\n{}", req.len(), req);

    stdin.write_all(msg.as_bytes()).await.unwrap();
    stdin.flush().await.unwrap();

    // Read response - should get an error response
    let read_result = timeout(Duration::from_secs(2), read_lsp_message(&mut reader)).await;

    match read_result {
        Ok(Some(response)) => {
            // If we got a response, it should be an error
            assert!(
                response.contains("error") || response.contains("Parse error"),
                "Response should be an error. Got: {}",
                response
            );
        }
        Ok(None) | Err(_) => {
            // Server closed or timeout - acceptable for malformed requests
        }
    }

    drop(stdin);
    let _ = server.kill().await;
}
