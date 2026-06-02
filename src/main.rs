use rmcp::{
    ErrorData as McpError, ServerHandler,ServiceExt,
    handler::server::router::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router, transport::stdio,
};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mcp_server = TRexServer::new().serve(stdio()).await?;
    mcp_server.waiting().await?;
    Ok(())
}

#[derive(Clone)]
pub struct TRexServer {
    tool_router: ToolRouter<TRexServer>,
}

#[tool_router]
impl TRexServer {

    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router()
        }
    }

    #[tool(description = "Ping-pong check for client")]
    pub async fn ping(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("pong")]))
    }
}

#[tool_handler]
impl ServerHandler for TRexServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(Implementation::from_build_env())
    }
}
