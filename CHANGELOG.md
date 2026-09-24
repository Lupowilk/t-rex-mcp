# Changelog

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning: [SemVer](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-24

Initial release. Read-only MCP server exposing ERC-3643 (T-REX) compliance state as agent-callable tools over stdio, on Ethereum mainnet.

### Added

- `ping` — liveness check.
- `get_block_number` — current mainnet block number.
- `check_token_eligibility` — runs the token's compliance contract `canTransfer(from, to, amount)`.
- `read_identity_registry` — holder's ONCHAINID in the token's Identity Registry.
- `list_claim_topics` — claim topics a token requires.
- Prebuilt binaries attached to the release: macOS (Apple Silicon) and Linux (x86_64), as `.tar.gz`.

### Notes

- Read-only, mainnet only. Requires `ALCHEMY_API_KEY`.
- Verified live in Claude Desktop against a Tokeny T-REX deployment.
- `check_token_eligibility` checks compliance rules only — not identity verification, frozen wallets, paused tokens or balance — and cannot say which rule rejected a transfer.

[0.1.0]: https://github.com/Lupowilk/t-rex-mcp/releases/tag/v0.1.0
