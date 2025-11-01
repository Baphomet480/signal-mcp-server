# Signal MCP Server

Early scaffold for a Model Context Protocol server that bridges Signal messaging via `signal-cli`.

## Status

Prototype bootstrap. Core event loop and handlers are placeholders.

## Configuration

Create a `config.toml` in the project root or provide environment variables with the `SIGNAL_MCP__` prefix.

```toml
account = "+10000000000"
signal_cli_path = "/usr/local/bin/signal-cli"
storage = "./var"
```

## Development

- Requires Rust (edition 2021) and `signal-cli`.
- The MCP layer uses [`rust-mcp-sdk`](https://crates.io/crates/rust-mcp-sdk) with the stdio transport targeting the 2025-06-18 protocol schema.
- Run `cargo build` or `cargo check` to verify the crate.
- More documentation coming as features land.

## MCP Interface

- **Transport:** stdio (suitable for use with MCP inspectors or clients that spawn the server as a subprocess).
- **Tool:** `signal_list_conversations` — returns Signal contact and group identifiers via `signal-cli listContacts`/`listGroups`.
- **Tool:** `signal_send_message` — sends a text message using `signal-cli send` (requires `recipient` and `message` arguments).
- **Resource:** `resource://signal/overview` — markdown overview of configuration, available tools, and roadmap.

## Roadmap

- Flesh out MCP request/response schema and wire to a JSON-RPC transport.
- Implement conversation and message queries backed by `signal-cli` RPC.
- Build send pipeline with rate limiting and attachment handling.
- Stream live Signal events into MCP notifications.
- Add integration tests against a sandbox Signal account.
