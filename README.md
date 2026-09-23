# t-rex-mcp
An MCP server in Rust exposing on-chain RWA compliance state as agent-callable tools.
Currently supports ERC-3643 (T-REX). ERC-7943 (uRWA) support is planned.

## Why
Compliance primitives like identity registries, claim topics, transfer restrictions, and frozen balances are readable on-chain but awkward to reason about. This server makes them queryable by an AI agent.

ERC-3643 is the dominant framework for regulated securities tokens. ERC-7943 went Final (May 2026) as the neutral interface, sharing `canTransfer` semantics by design.

No public MCP server for ERC-3643 that we're aware of (verified Sept 2026).

## Scope
Read-only. Ethereum mainnet only. stdio transport. MCP spec 2026-07-28 via rmcp 3.x.

## Tools
| Tool | Inputs | Returns | Status |
|---|---|---|---|
| `ping` | none | `"pong"` (text) | working |
| `get_block_number` | none | `{"block_number": <latest mainnet block>}` | working |
| `check_token_eligibility` | `token`, `from`, `to`, `amount` | `{"can_transfer": true}` | working |
| `read_identity_registry` | `token`, `holder` | `{"registered": true, "onchainid": "0x7714…"}`, or `{"registered": false, "onchainid": null}` | working |
| `list_claim_topics` | `token` | `{"topics": ["10101010000101"]}` | working |
| `simulate_transfer` | | | v0.2 |
| `query_transfer_restrictions` | | | v0.2 |

All inputs are strings. Addresses are `0x…` hex. `amount` is a whole number in the token's smallest unit (no decimals applied).

Results come back as `structuredContent`, a JSON object. Claim topic IDs are strings.

### Limitations of `check_token_eligibility`
It asks only the token's compliance contract (`canTransfer`). Those rules vary by token.
A real T-REX transfer also checks things this tool does **not**:
- whether the recipient is verified in the identity registry
- whether either wallet, or the tokens, are frozen
- whether the token is paused
- whether the sender has enough balance

So `can_transfer: true` means "compliance rules allow it", not "the transfer will succeed". The contract also does not say *which* rule caused a `false`.

## Quick start guide

### Prerequisites
- Rust (stable), installed via [rustup](https://rustup.rs)
- An [Alchemy](https://www.alchemy.com) API key for Ethereum mainnet (free tier is enough)
- [Claude Desktop](https://claude.ai/download)

### 1. Build
```bash
git clone https://github.com/Lupowilk/t-rex-mcp.git
cd t-rex-mcp
cargo build --release
```
The binary is at `target/release/t-rex-mcp`.

### 2. Add to Claude Desktop
Open the config file (macOS):
```
~/Library/Application Support/Claude/claude_desktop_config.json
```
Add the server. Use the **absolute** path to your binary:
```json
{
  "mcpServers": {
    "t-rex-mcp": {
      "command": "/absolute/path/to/t-rex-mcp/target/release/t-rex-mcp",
      "env": {
        "ALCHEMY_API_KEY": "your_key_here"
      }
    }
  }
}
```
If the file already has other servers, add `t-rex-mcp` inside the existing `mcpServers` object.

The server reads `ALCHEMY_API_KEY` from this `env` block. It does not load a `.env` file.
Pass the key only, not the full RPC URL.

### 3. Restart Claude Desktop
Fully quit with **Cmd+Q** (closing the window is not enough), then reopen. The five tools
should appear in the tools menu.

After any code change, run `cargo build --release` and fully quit again.
A stale tool list looks exactly like a missing tool.

### 4. Examples
Example token: BLABS, a Tokeny T-REX deployment on mainnet.
```
Token:  0x6fb975af85262ee9d0f7ce3db83172db8e4295b6
Holder: 0xa9b0c51b01e79b5ceab77577f04d136edbbb3420
```
Ask Claude:
> What claim topics does token 0x6fb975af85262ee9d0f7ce3db83172db8e4295b6 require?

> What is the ONCHAINID of holder 0xa9b0c51b01e79b5ceab77577f04d136edbbb3420 on that token?

## Architecture
![check_token_eligibility dataflow](docs/architecture.svg)

## References
- [ERC-3643](https://eips.ethereum.org/EIPS/eip-3643) · [ERC-7943](https://eips.ethereum.org/EIPS/eip-7943) · [T-REX](https://github.com/TokenySolutions/T-REX) · [MCP](https://modelcontextprotocol.io) · [rmcp](https://github.com/modelcontextprotocol/rust-sdk)
