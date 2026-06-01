use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router,
};



fn main() {
    //
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
