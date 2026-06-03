# Rust MCP Server for ERC-3643 compliance
The repository contains a simple MCP server exposing ERC-3643 / T-REX compliance state as tools for an AI agent.

## Why
This serves as a training ground for the author, and no public MCP server exists for ERC-3643 today.
ERC-3643 is the dominant institutional RWA tokenization standard with $32B+ tokenized per the ERC-3643 Association. We want to give agents the ability to reason about on-chain compliance.

## Requirements
- read-only, no writes
- Ethereum mainnet only, no multichain
- no custody integration
- v0.1 will support 3 exemplar tools: `check_eligibility`, `read_identity_registry`, `list_claim_topics`

## Links
- ERC-3643 spec: https://eips.ethereum.org/EIPS/eip-3643
- MCP: https://modelcontextprotocol.io
- T-REX reference impl: https://github.com/TokenySolutions/T-REX
