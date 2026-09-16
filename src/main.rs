use alloy::{primitives::{Address, U256}, providers::{Provider, ProviderBuilder}, sol};
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt, handler::server::{router::tool::ToolRouter, wrapper::Parameters}, model::*, tool, tool_handler, tool_router, transport::stdio,
};
use schemars::JsonSchema;
use serde::Deserialize;


sol! {
    #[sol(rpc)]
    contract IToken{
        function compliance() external view returns (address);
        function identityRegistry() external view returns (address);
    }

    #[sol(rpc)]
    contract ICompliance{
        function canTransfer(address _from, address _to, uint256 _amount) external view returns (bool);
    }

     #[sol(rpc)]
     contract IIdentityRegistry{
        function identity(address _userAddress) external view returns (address);
        function topicsRegistry() external view returns (address);
     }

     #[sol(rpc)]
     contract IClaimTopicsRegistry {
         function getClaimTopics() external view returns (uint256[] memory);
     }

}

#[derive(Deserialize, JsonSchema)]
pub struct EligibilityCheck {
    token: String,
    from: String,
    to: String,
    amount: String
}

#[derive(Deserialize, JsonSchema)]
pub struct IdentityCheck {
    token: String,
    holder: String
}


#[derive(Deserialize, JsonSchema)]
pub struct ClaimTopics {
    token: String
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

    #[tool(description = "Checks for eligibility of the token contract")]
    pub async fn check_token_eligibility(&self, tokendetails: Parameters<EligibilityCheck> ) -> Result<CallToolResult, McpError> {
        let token_el = tokendetails.0.token.parse::<Address>()
            .map_err(|e| McpError::internal_error(format!("invalid token address: {e}"), None))?;

        let from_el = tokendetails.0.from.parse::<Address>()
            .map_err(|e| McpError::internal_error(format!("invalid sending address: {e}"), None))?;

        let to_el = tokendetails.0.to.parse::<Address>()
            .map_err(|e| McpError::internal_error(format!("invalid recipient address: {e}"), None))?;

        let amount_el = tokendetails.0.amount.parse::<U256>()
            .map_err(|e| McpError::internal_error(format!("invalid amount: {e}"), None))?;

        let alchemy_key = std::env::var("ALCHEMY_API_KEY")
            .map_err(|e| McpError::internal_error(format!("missing ALCHEMY_API_KEY: {e}"), None))?;

        let url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", alchemy_key);

        let provider = ProviderBuilder::new().connect(&url).await
            .map_err(|e| McpError::internal_error(format!("Connection failure: {e}"), None))?;

        let compliance_address = IToken::new(token_el, &provider).compliance().call().await
            .map_err(|e| McpError::internal_error(format!("No compliance address exists: {e}"), None))?;

        let compliance_contract_check = ICompliance::new(compliance_address, &provider).canTransfer(from_el, to_el, amount_el).call().await
            .map_err(|e| McpError::internal_error(format!("The contract is not compliant, {e}"), None))?;

        Ok(CallToolResult::success(vec![ContentBlock::text(compliance_contract_check.to_string())]))
    }

    #[tool(description = "looks up a holder's ONCHAINID identity contract address in the token's Identity Registry.")]
    pub async fn read_identity_registry(&self, identitydetails: Parameters<IdentityCheck>) -> Result<CallToolResult, McpError> {
        let onchainid_check_contract = identitydetails.0.token.parse::<Address>()
            .map_err(|e| McpError::internal_error(format!("invalid token address, {e}"), None))?;

        let onchain_holder_check = identitydetails.0.holder.parse::<Address>()
            .map_err(|e| McpError::internal_error(format!("invalid holder address, {e}"), None))?;

        let alchemy_key = std::env::var("ALCHEMY_API_KEY")
            .map_err(|e| McpError::internal_error(format!("missing ALCHEMY_API_KEY: {e}"), None))?;

        let url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", alchemy_key);

        let provider = ProviderBuilder::new().connect(&url).await
            .map_err(|e| McpError::internal_error(format!("Connection failure: {e}"), None))?;

        let registry_address_call = IToken::new(onchainid_check_contract, &provider).identityRegistry().call().await
            .map_err(|e| McpError::internal_error(format!("failed to read identity registry, {e}"), None))?;

        let registry_identity_call = IIdentityRegistry::new(registry_address_call, &provider).identity(onchain_holder_check).call().await
            .map_err(|e| McpError::internal_error(format!("failed to read identity registry, {e}"), None))?;

        Ok(CallToolResult::success(vec![ContentBlock::text(registry_identity_call.to_string())]))
        }

    #[tool(description = "looks into the claim topics required by a token's IdentityRegistry")]
    pub async fn list_claim_topics(&self, claimdetails: Parameters<ClaimTopics>) -> Result<CallToolResult, McpError> {
            let token_address = claimdetails.0.token.parse::<Address>()
                .map_err(|e| McpError::internal_error(format!("invalid token address, {e}"), None))?;

            let alchemy_key = std::env::var("ALCHEMY_API_KEY")
                .map_err(|e| McpError::internal_error(format!("missing ALCHEMY_API_KEY: {e}"), None))?;

            let url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", alchemy_key);

            let provider = ProviderBuilder::new().connect(&url).await
                .map_err(|e| McpError::internal_error(format!("Connection failure: {e}"), None))?;

            let registry_address_call = IToken::new(token_address, &provider).identityRegistry().call().await
                .map_err(|e| McpError::internal_error(format!("failed to read identity registry, {e}"), None))?;

            let topics_registry_address = IIdentityRegistry::new(registry_address_call, &provider).topicsRegistry().call().await
                .map_err(|e| McpError::internal_error(format!("failed to read topics registry, {e}"), None))?;

            let read_topics_registry = IClaimTopicsRegistry::new(topics_registry_address, &provider).getClaimTopics().call().await
                .map_err(|e| McpError::internal_error(format!("failed to read topics registry, {e}"), None))?;

            let topics: Vec<String> = read_topics_registry.iter().map(|t| t.to_string()).collect();

            Ok(CallToolResult::success(vec![ContentBlock::text(topics.join(","))]))
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
