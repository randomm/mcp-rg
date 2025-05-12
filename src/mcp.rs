use async_trait::async_trait;
use rust_mcp_sdk::{
    McpServer,
    mcp_server::{
        ServerHandler,
        server_runtime,
    },
};
use rust_mcp_schema::{
    Implementation,
    InitializeResult,
    ServerCapabilities,
    ServerCapabilitiesTools,
    ListToolsRequest,
    ListToolsResult,
    CallToolRequest,
    CallToolResult,
    TextContent,
    Tool,
    ToolInputSchema,
    schema_utils::CallToolError,
    LATEST_PROTOCOL_VERSION,
};
use rust_mcp_transport::{
    TransportOptions,
    StdioTransport,
};
use serde_json::{json, Map, Value};
use tracing::{debug, info};
use crate::{
    config::Config,
    error::AppError,
    ripgrep::{RipgrepSearcher, SearchOptions},
};
use std::sync::Arc;
use std::collections::HashMap;

pub struct MCPServer {
    searcher: Arc<RipgrepSearcher>,
}

impl MCPServer {
    pub fn new(config: Config) -> Self {
        let searcher = Arc::new(RipgrepSearcher::new(config.files_root.clone()));
        Self { searcher }
    }
    
    pub async fn run(&self) -> Result<(), AppError> {
        // Create server details with the MCP protocol version
        let server_details = InitializeResult {
            server_info: Implementation {
                name: "ripgrep-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: ServerCapabilities {
                tools: Some(ServerCapabilitiesTools { list_changed: None }),
                ..Default::default()
            },
            meta: None,
            instructions: Some("Ripgrep MCP server for code search".to_string()),
            protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        };
        
        // Create a server handler with our implementation
        let handler = RipgrepServerHandler {
            searcher: self.searcher.clone(),
        };
        
        // Create a transport with default options
        let transport_opt = TransportOptions::default();
        let transport = StdioTransport::new(transport_opt)
            .map_err(|e| AppError::MCPError(format!("Failed to create transport: {}", e)))?;
        
        info!("Starting MCP server");
        
        // Create and start the server
        let server = server_runtime::create_server(server_details, transport, handler);
        McpServer::start(&server).await
            .map_err(|e| AppError::MCPError(format!("Server error: {}", e)))?;
        
        Ok(())
    }
}

// Server handler implementation
#[derive(Debug)]
struct RipgrepServerHandler {
    searcher: Arc<RipgrepSearcher>,
}

#[async_trait]
impl ServerHandler for RipgrepServerHandler {
    // Handle tool listing
    async fn handle_list_tools_request(
        &self,
        _request: ListToolsRequest,
        _runtime: &dyn rust_mcp_sdk::McpServer,
    ) -> Result<ListToolsResult, rust_mcp_schema::RpcError> {
        let mut properties = HashMap::new();
        
        // Create pattern property
        let mut pattern_prop = Map::new();
        pattern_prop.insert("type".to_string(), json!("string"));
        pattern_prop.insert("description".to_string(), json!("Search pattern"));
        
        // Create path property
        let mut path_prop = Map::new();
        path_prop.insert("type".to_string(), json!("string"));
        path_prop.insert("description".to_string(), json!("Relative path within root directory"));
        
        // Create fixed_strings property
        let mut fixed_strings_prop = Map::new();
        fixed_strings_prop.insert("type".to_string(), json!("boolean"));
        fixed_strings_prop.insert("description".to_string(), json!("Use fixed strings instead of regex"));
        
        // Add to properties map
        properties.insert("pattern".to_string(), pattern_prop);
        properties.insert("path".to_string(), path_prop);
        properties.insert("fixed_strings".to_string(), fixed_strings_prop);
        
        // Create the tool with input schema
        let search_tool = Tool {
            name: "search".to_string(),
            description: Some("Search code using ripgrep".to_string()),
            input_schema: ToolInputSchema::new(
                vec!["pattern".to_string()], 
                Some(properties)
            ),
        };
        
        Ok(ListToolsResult {
            tools: vec![search_tool],
            meta: None,
            next_cursor: None,
        })
    }
    
    // Handle tool calls
    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        _runtime: &dyn rust_mcp_sdk::McpServer,
    ) -> Result<CallToolResult, CallToolError> {
        debug!(?request, "Received search request");
        
        match request.params.name.as_str() {
            "search" => {
                // Parse the search options from the parameters
                let options: SearchOptions = match request.params.arguments {
                    Some(args) => serde_json::from_value(Value::Object(args))
                        .map_err(|e| {
                            let err_msg = format!("Invalid parameters: {}", e);
                            CallToolError::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, err_msg))
                        })?,
                    None => {
                        let err_msg = "Missing required arguments for search".to_string();
                        return Err(CallToolError::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, err_msg)));
                    }
                };
                
                // Execute the search
                let result = self.searcher.search(&options).await
                    .map_err(|e| {
                        let err_msg = format!("Search failed: {}", e);
                        CallToolError::new(std::io::Error::new(std::io::ErrorKind::Other, err_msg))
                    })?;
                
                // Convert the result to JSON
                let result_json = serde_json::to_string_pretty(&result).map_err(|e| {
                    let err_msg = format!("JSON serialization error: {}", e);
                    CallToolError::new(std::io::Error::new(std::io::ErrorKind::Other, err_msg))
                })?;
                
                // Create text content
                let text_content = TextContent::new(result_json, None);
                
                // Create call tool result with content
                let mut content = Vec::new();
                content.push(text_content.into());
                
                Ok(CallToolResult {
                    content,
                    is_error: None,
                    meta: None,
                })
            },
            _ => {
                Err(CallToolError::unknown_tool(format!("Unknown tool: {}", request.params.name)))
            },
        }
    }
}