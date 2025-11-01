# Signal MCP Server — Agent Notes

## Overview
This repository houses an MCP-compatible service that exposes Signal messaging via `signal-cli`. The goal is feature parity with `messages-app-mcp`, enabling agents to:
- Enumerate Signal conversations (1:1 + groups)
- Fetch and search message history with attachments
- Send messages and media
- Stream incoming events (messages, receipts, typing)

## Key Components
- **signal-cli**: The underlying CLI/JSON-RPC client. Must be installed, registered, and kept up to date.
- **MCP Server**: Wraps signal-cli, normalizes data to the MCP schema, and emits notifications.
- **Storage**: Local cache (e.g., SQLite + filesystem) for message snapshots and attachments.
- **Config**: `.toml` or `.yaml` file with signal-cli path, account number, storage directories, queue tuning.

## Agent Workflow Tips
1. Always ensure signal-cli is running and authenticated before invoking MCP commands.
2. When debugging, check both the MCP server logs and signal-cli logs.
3. Attachments may be large—stream or chunk them to avoid memory issues.
4. Respect Signal’s rate limits; queue outgoing messages when necessary.
5. Update signal-cli promptly to avoid registration lockouts.

## Security Considerations
- Treat the signal-cli data directory as sensitive (contains encryption keys).
- Never commit registration codes or linked device secrets.
- Encourage operators to use a dedicated Signal number or device for automation.

## Helpful References
- signal-cli docs: https://github.com/AsamK/signal-cli
- Signal job queue guidelines: https://support.signal.org/ (rate limiting notes)
- Model Context Protocol spec: https://modelcontextprotocol.io/

Keep this file updated as architecture decisions evolve.
