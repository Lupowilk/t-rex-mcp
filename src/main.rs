use alloy::{primitives::{Address, U256}, providers::{Provider, ProviderBuilder}, sol};
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt, handler::server::{router::tool::ToolRouter, wrapper::Parameters}, model::*, tool, tool_handler, tool_router, transport::stdio,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;


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
    #[schemars(description = "ERC-3643 token contract address, 0x-prefixed hex")]
    token: String,
    #[schemars(description = "Sender wallet address, 0x-prefixed hex")]
    from: String,
    #[schemars(description = "Receiver wallet address, 0x-prefixed hex")]
    to: String,
    #[schemars(description = "Transfer amount as a whole number in the token's raw base units (smallest unit, no decimals applied)")]
    amount: String
}

#[derive(Deserialize, JsonSchema)]
pub struct IdentityCheck {
    #[schemars(description = "ERC-3643 token contract address, 0x-prefixed hex")]
    token: String,
    #[schemars(description = "Holder wallet address to look up in the token's identity registry, 0x-prefixed hex")]
    holder: String
}

#[derive(Deserialize, JsonSchema)]
pub struct ClaimTopics {
    #[schemars(description = "ERC-3643 token contract address, 0x-prefixed hex")]
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

        #[tool(description = "Returns the current Ethereum mainnet block number as an object with a block_number field.")]
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

       Ok(CallToolResult::structured(json!({ "block_number": block_number })))
    }

    #[tool(description = "Checks whether a transfer passes the token's ERC-3643 compliance contract (canTransfer). Returns an object with a can_transfer boolean. It checks the compliance rules (e.g. country restrictions, transfer limits), but does not check whether the recipient is verified in the identity registry, whether either wallet is frozen, whether the token is paused, or whether the sender has enough balance, so true does not guarantee the transfer will succeed. Amount is in raw base units.")]
    pub async fn check_token_eligibility(&self, tokendetails: Parameters<EligibilityCheck> ) -> Result<CallToolResult, McpError> {
        let token_el = tokendetails.0.token.parse::<Address>()
            .map_err(|e| McpError::invalid_params(format!("Invalid token address (expected 0x-prefixed hex): {e}"), None))?;

        let from_el = tokendetails.0.from.parse::<Address>()
            .map_err(|e| McpError::invalid_params(format!("Invalid sender address (expected 0x-prefixed hex): {e}"), None))?;

        let to_el = tokendetails.0.to.parse::<Address>()
            .map_err(|e| McpError::invalid_params(format!("Invalid recipient address (expected 0x-prefixed hex): {e}"), None))?;

        let amount_el = tokendetails.0.amount.parse::<U256>()
            .map_err(|e| McpError::invalid_params(format!("Invalid amount (expected a whole number in base units): {e}"), None))?;

        let alchemy_key = std::env::var("ALCHEMY_API_KEY")
            .map_err(|e| McpError::internal_error(format!("Server is not configured (ALCHEMY_API_KEY is missing): {e}"), None))?;

        let url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", alchemy_key);

        let provider = ProviderBuilder::new().connect(&url).await
            .map_err(|e| McpError::internal_error(format!("Could not set up the Ethereum RPC client (invalid RPC URL): {e}"), None))?;

        let compliance_address = IToken::new(token_el, &provider).compliance().call().await
            .map_err(|e| McpError::internal_error(format!("Could not read the token's compliance contract (compliance() call failed: token may not be ERC-3643, or the RPC/API key is unreachable): {e}"), None))?;

        let compliance_contract_check = ICompliance::new(compliance_address, &provider).canTransfer(from_el, to_el, amount_el).call().await
            .map_err(|e| McpError::internal_error(format!("Could not check transfer eligibility (compliance canTransfer call failed): {e}"), None))?;


                Ok(CallToolResult::structured(json!({ "can_transfer": compliance_contract_check })))
    }

    #[tool(description = "looks up a holder's ONCHAINID identity contract address in the token's Identity Registry.")]
    pub async fn read_identity_registry(&self, identitydetails: Parameters<IdentityCheck>) -> Result<CallToolResult, McpError> {
        let onchainid_check_contract = identitydetails.0.token.parse::<Address>()
            .map_err(|e| McpError::invalid_params(format!("Invalid token address (expected 0x-prefixed hex): {e}"), None))?;

        let onchain_holder_check = identitydetails.0.holder.parse::<Address>()
            .map_err(|e| McpError::invalid_params(format!("Invalid holder address (expected 0x-prefixed hex): {e}"), None))?;

        let alchemy_key = std::env::var("ALCHEMY_API_KEY")
            .map_err(|e| McpError::internal_error(format!("Server is not configured (ALCHEMY_API_KEY is missing): {e}"), None))?;

        let url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", alchemy_key);

        let provider = ProviderBuilder::new().connect(&url).await
            .map_err(|e| McpError::internal_error(format!("Could not set up the Ethereum RPC client (invalid RPC URL): {e}"), None))?;

        let registry_address_call = IToken::new(onchainid_check_contract, &provider).identityRegistry().call().await
            .map_err(|e| McpError::internal_error(format!("Could not read the token's identity registry address (identityRegistry() call failed: token may not be ERC-3643, or the RPC/API key is unreachable): {e}"), None))?;

        let registry_identity_call = IIdentityRegistry::new(registry_address_call, &provider).identity(onchain_holder_check).call().await
            .map_err(|e| McpError::internal_error(format!("Could not read the holder's ONCHAINID (identity() call failed: identity registry call reverted, or the RPC/API key is unreachable): {e}"), None))?;

        if registry_identity_call == Address::ZERO {
            return Ok(CallToolResult::structured(json!({ "registered": false, "onchainid": null })));
        }
        Ok(CallToolResult::structured(json!({ "registered": true, "onchainid": registry_identity_call.to_string() })))
        }

    #[tool(description = "looks into the claim topics required by a token's IdentityRegistry")]
    pub async fn list_claim_topics(&self, claimdetails: Parameters<ClaimTopics>) -> Result<CallToolResult, McpError> {
        let token_address = claimdetails.0.token.parse::<Address>()
            .map_err(|e| McpError::invalid_params(format!("Invalid token address (expected 0x-prefixed hex): {e}"), None))?;

        let alchemy_key = std::env::var("ALCHEMY_API_KEY")
            .map_err(|e| McpError::internal_error(format!("Server is not configured (ALCHEMY_API_KEY is missing): {e}"), None))?;

        let url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", alchemy_key);

        let provider = ProviderBuilder::new().connect(&url).await
            .map_err(|e| McpError::internal_error(format!("Could not set up the Ethereum RPC client (invalid RPC URL): {e}"), None))?;

        let registry_address_call = IToken::new(token_address, &provider).identityRegistry().call().await
            .map_err(|e| McpError::internal_error(format!("Could not read the token's identity registry address (identityRegistry() call failed: token may not be ERC-3643, or the RPC/API key is unreachable): {e}"), None))?;

        let topics_registry_address = IIdentityRegistry::new(registry_address_call, &provider).topicsRegistry().call().await
            .map_err(|e| McpError::internal_error(format!("Could not read the claim topics registry address (topicsRegistry() call failed: identity registry call reverted, or the RPC/API key is unreachable): {e}"), None))?;

        let read_topics_registry = IClaimTopicsRegistry::new(topics_registry_address, &provider).getClaimTopics().call().await
            .map_err(|e| McpError::internal_error(format!("Could not read the required claim topics (getClaimTopics() call failed: claim topics registry call reverted, or the RPC/API key is unreachable): {e}"), None))?;

        let topics: Vec<String> = read_topics_registry.iter().map(|t| t.to_string()).collect();

        Ok(CallToolResult::structured(json!({"topics": topics })))
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
