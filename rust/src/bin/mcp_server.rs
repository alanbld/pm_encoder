//! pm_encoder MCP Server
//!
//! Model Context Protocol server for pm_encoder, allowing AI assistants
//! to serialize codebases directly.
//!
//! Build: cargo build --features mcp --bin pm_encoder_mcp
//! Run:   ./target/debug/pm_encoder_mcp

use pm_encoder::{
    ContextEngine, EncoderConfig, LensManager,
    parse_token_budget, apply_token_budget,
};
use pm_encoder::core::{
    ContextEngine as CoreContextEngine,
    ZoomConfig, ZoomTarget, ZoomDepth,
};
use rmcp::{
    schemars,
    schemars::JsonSchema,
    ServerHandler, ServiceExt,
    handler::server::tool::ToolRouter,
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation, ListToolsResult,
        ServerCapabilities, ServerInfo, Tool, ToolsCapability,
    },
    service::{RequestContext, RoleServer},
};
use serde::Deserialize;
use tokio::io::{stdin, stdout};

/// MCP Server for pm_encoder
#[derive(Clone)]
struct PmEncoderServer {
    tool_router: ToolRouter<Self>,
}

/// Input for get_context tool
#[derive(Debug, Deserialize, JsonSchema)]
struct GetContextParams {
    /// List of files with path and content
    files: Vec<FileInput>,
    /// Optional lens name (architecture, debug, security, minimal, onboarding)
    #[serde(default)]
    lens: Option<String>,
    /// Truncate files to this many lines (0 = no truncation)
    #[serde(default)]
    truncate_lines: Option<usize>,
    /// Maximum token budget (e.g., "100000", "100k", "2M")
    #[serde(default)]
    token_budget: Option<String>,
    /// Budget strategy: "drop", "truncate", or "hybrid"
    #[serde(default)]
    budget_strategy: Option<String>,
}

/// A file with path and content
#[derive(Debug, Deserialize, JsonSchema)]
struct FileInput {
    /// File path (e.g., "src/main.py")
    path: String,
    /// File content
    content: String,
}

/// Input for list_lenses tool (no params needed)
#[derive(Debug, Deserialize, JsonSchema)]
struct ListLensesParams {}

/// Input for zoom_context tool
#[derive(Debug, Deserialize, JsonSchema)]
struct ZoomContextParams {
    /// Root directory to search in
    root: String,
    /// Zoom target type: "fn", "class", "mod", or "file"
    target_type: String,
    /// Target name (function name, class name, module name, or file path)
    target_name: String,
    /// Optional line range for file zoom (e.g., "10-50")
    #[serde(default)]
    line_range: Option<String>,
    /// Zoom depth: "signature", "implementation", or "full"
    #[serde(default)]
    depth: Option<String>,
    /// Token budget for zoomed content
    #[serde(default)]
    token_budget: Option<usize>,
}

impl PmEncoderServer {
    fn new() -> Self {
        // Build the tool router with our tools
        let tool_router = ToolRouter::new()
            .with_route(Self::get_context_route())
            .with_route(Self::list_lenses_route())
            .with_route(Self::zoom_context_route());

        Self { tool_router }
    }

    fn get_context_route() -> rmcp::handler::server::tool::ToolRoute<Self> {
        let tool = Tool::new(
            "get_context",
            "Serialize files into LLM-optimized context using Plus/Minus format. Supports context lenses, token budgeting, and file truncation.",
            rmcp::handler::server::tool::schema_for_type::<GetContextParams>(),
        );

        rmcp::handler::server::tool::ToolRoute::new_dyn(tool, |ctx| {
            Box::pin(async move {
                let params: GetContextParams = rmcp::handler::server::tool::parse_json_object(
                    ctx.arguments.unwrap_or_default(),
                )?;

                // Build config
                let mut config = EncoderConfig::default();

                if let Some(lines) = params.truncate_lines {
                    config.truncate_lines = lines;
                }

                // Create lens manager for priority resolution
                let mut lens_manager = LensManager::new();

                // Apply lens if specified
                if let Some(ref lens_name) = params.lens {
                    lens_manager.apply_lens(lens_name).map_err(|e| {
                        rmcp::ErrorData::invalid_params(
                            format!("Invalid lens '{}': {}", lens_name, e),
                            None,
                        )
                    })?;
                }

                // Convert files to tuples
                let files: Vec<(String, String)> = params
                    .files
                    .into_iter()
                    .map(|f| (f.path, f.content))
                    .collect();

                // Apply token budget if specified
                let selected_files = if let Some(ref budget_str) = params.token_budget {
                    let budget = parse_token_budget(budget_str).map_err(|e| {
                        rmcp::ErrorData::invalid_params(
                            format!("Invalid token budget '{}': {}", budget_str, e),
                            None,
                        )
                    })?;

                    let strategy = params.budget_strategy.as_deref().unwrap_or("drop");
                    let (selected, _report) = apply_token_budget(files, budget, &lens_manager, strategy);
                    selected
                } else {
                    files
                };

                // Create engine with optional lens
                let engine = if let Some(lens_name) = params.lens {
                    ContextEngine::with_lens(config, &lens_name).map_err(|e| {
                        rmcp::ErrorData::invalid_params(
                            format!("Invalid lens '{}': {}", lens_name, e),
                            None,
                        )
                    })?
                } else {
                    ContextEngine::new(config)
                };

                // Generate context
                let context = engine.generate_context(&selected_files);

                Ok(CallToolResult::success(vec![Content::text(context)]))
            })
        })
    }

    fn list_lenses_route() -> rmcp::handler::server::tool::ToolRoute<Self> {
        let tool = Tool::new(
            "list_lenses",
            "Get a list of available context lenses with their descriptions.",
            rmcp::handler::server::tool::schema_for_type::<ListLensesParams>(),
        );

        rmcp::handler::server::tool::ToolRoute::new_dyn(tool, |_ctx| {
            Box::pin(async move {
                let lenses = vec![
                    ("architecture", "Signatures only - best for understanding structure"),
                    ("debug", "Full content - for debugging and deep analysis"),
                    ("security", "Auth, crypto, validation focus"),
                    ("minimal", "Entry points only - smallest context"),
                    ("onboarding", "Balanced view for new contributors"),
                ];

                let output = lenses
                    .iter()
                    .map(|(name, desc)| format!("- {}: {}", name, desc))
                    .collect::<Vec<_>>()
                    .join("\n");

                let header = format!(
                    "pm_encoder v{} - Available Lenses:\n\n{}",
                    pm_encoder::version(),
                    output
                );

                Ok(CallToolResult::success(vec![Content::text(header)]))
            })
        })
    }

    fn zoom_context_route() -> rmcp::handler::server::tool::ToolRoute<Self> {
        let tool = Tool::new(
            "zoom_context",
            "Zoom into a specific code element for detailed context. Use after seeing a ZOOM_AFFORDANCE marker in truncated content.",
            rmcp::handler::server::tool::schema_for_type::<ZoomContextParams>(),
        );

        rmcp::handler::server::tool::ToolRoute::new_dyn(tool, |ctx| {
            Box::pin(async move {
                let params: ZoomContextParams = rmcp::handler::server::tool::parse_json_object(
                    ctx.arguments.unwrap_or_default(),
                )?;

                // Parse zoom target
                let target = match params.target_type.to_lowercase().as_str() {
                    "fn" | "function" => ZoomTarget::Function(params.target_name.clone()),
                    "class" | "struct" => ZoomTarget::Class(params.target_name.clone()),
                    "mod" | "module" => ZoomTarget::Module(params.target_name.clone()),
                    "file" => {
                        // Parse optional line range
                        let (start, end) = if let Some(ref range) = params.line_range {
                            if let Some(dash_pos) = range.find('-') {
                                let start: Option<usize> = range[..dash_pos].parse().ok();
                                let end: Option<usize> = range[dash_pos + 1..].parse().ok();
                                (start, end)
                            } else {
                                (range.parse().ok(), None)
                            }
                        } else {
                            (None, None)
                        };
                        ZoomTarget::File {
                            path: params.target_name.clone(),
                            start_line: start,
                            end_line: end,
                        }
                    }
                    _ => {
                        return Err(rmcp::ErrorData::invalid_params(
                            format!(
                                "Invalid target_type '{}'. Use: fn, class, mod, or file",
                                params.target_type
                            ),
                            None,
                        ));
                    }
                };

                // Parse zoom depth
                let depth = params
                    .depth
                    .as_ref()
                    .and_then(|d| ZoomDepth::from_str(d))
                    .unwrap_or(ZoomDepth::Full);

                // Build zoom config
                let zoom_config = ZoomConfig {
                    target,
                    budget: params.token_budget,
                    depth,
                    include_tests: false,
                    context_lines: 5,
                };

                // Create core engine and perform zoom
                let engine = CoreContextEngine::new();
                match engine.zoom(&params.root, &zoom_config) {
                    Ok(content) => Ok(CallToolResult::success(vec![Content::text(content)])),
                    Err(e) => Err(rmcp::ErrorData::invalid_params(
                        format!("Zoom failed: {}", e),
                        None,
                    )),
                }
            })
        })
    }
}

impl ServerHandler for PmEncoderServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability::default()),
                ..Default::default()
            },
            server_info: Implementation {
                name: "pm_encoder".into(),
                version: pm_encoder::version().into(),
                title: Some("pm_encoder Context Serializer".into()),
                icons: None,
                website_url: Some("https://github.com/alanbld/pm_encoder".into()),
            },
            instructions: Some(
                "Use get_context to serialize code files into LLM-optimized context. \
                 Use list_lenses to see available context lenses. \
                 Use zoom_context to expand truncated content (follow ZOOM_AFFORDANCE markers)."
                    .into(),
            ),
        }
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send + '_
    {
        async move {
            Ok(ListToolsResult {
                tools: self.tool_router.list_all(),
                next_cursor: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, rmcp::ErrorData>> + Send + '_
    {
        async move {
            let tool_context =
                rmcp::handler::server::tool::ToolCallContext::new(self, request, context);
            self.tool_router.call(tool_context).await
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the MCP server
    let server = PmEncoderServer::new();

    // Log to stderr so stdout is clean for MCP protocol
    eprintln!("pm_encoder MCP Server v{} starting...", pm_encoder::version());

    // Set up stdio transport for MCP
    let transport = (stdin(), stdout());

    // Serve the MCP protocol
    let service = server.serve(transport).await?;

    // Wait for the client to disconnect
    let _quit_reason = service.waiting().await?;

    Ok(())
}
