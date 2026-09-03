use alloy::{providers::{Provider, ProviderBuilder}, sol};
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::router::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router, transport::stdio,
};


sol! {
    #[sol(rpc)]
    contract IToken{
        function compliance() external view returns (address);
    }
}


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
        Ok(CallToolResult::success(vec![ContentBlock::text("pong")]))
    }

    #[tool(description = "Return ETH current block number using Alchemy")]
    pub async fn get_block_number(&self) -> Result<CallToolResult, McpError> {
       // read env + Alchemy key
       let alchemy_key = std::env::var("ALCHEMY_API_KEY")
           .map_err(|e| McpError::internal_error(format!("missing ALCHEMY_API_KEY: {e}"), None))?;
       let url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", alchemy_key);
       // import block
       let provider = ProviderBuilder::new().connect(&url).await
           .map_err(|e| McpError::internal_error(format!("Connection failure: {e}"), None))?;
       let block_number = provider.get_block_number().await
           .map_err(|e| McpError::internal_error(format!("Failed to fetch block number: {e}"), None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(block_number.to_string())]))
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
