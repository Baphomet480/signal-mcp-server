# Signal MCP Server — Technical Plan

## Goal
Build a Model Context Protocol (MCP) server that exposes Signal messaging capabilities with feature parity comparable to `messages-app-mcp`, using `signal-cli` as the underlying transport.

## Core Capabilities
1. **Conversation Management**
   - List 1:1 and group threads, including names, participant roster, last message preview, unread counts.
2. **Message Retrieval**
   - Fetch paginated message history per conversation.
   - Surface metadata (author, timestamps, receipts) and attachments.
3. **Sending Messages**
   - Deliver text and file attachments to individuals or groups.
   - Support quoting/reply-to metadata when available.
4. **Event Streaming**
   - Subscribe to incoming messages, receipts, and typing indicators and forward them as MCP notifications.
5. **Search**
   - Provide keyword search over cached messages (initially local index, with optional full-text backend later).
6. **Attachment Handling**
   - Download/store media securely with short-lived handles exposed through MCP.
7. **Health & Telemetry**
   - Report signal-cli connectivity, registration state, and version drift.

## High-Level Architecture
```
┌─────────────┐      JSON-RPC       ┌─────────────┐
│ MCP Client  │◄──────────────────►│ MCP Server  │
└─────────────┘                   │ (new project)│
                                  └──────┬───────┘
                                         │
                                         ▼
                                ┌────────────────┐
                                │ signal-cli     │
                                │ (daemon / REST)│
                                └────────────────┘
                                         │
                                         ▼
                                ┌────────────────┐
                                │ Signal Service │
                                └────────────────┘
```

## Implementation Steps
1. **Environment Bootstrapping**
   - Package signal-cli (or dockerized signal-cli REST wrapper).
   - Provide CLI scripts for registration/linking.
2. **MCP Server Skeleton**
   - Pick language (Rust preferred for parity with `messages-app-mcp`).
   - Scaffold command handlers, configuration loader, logging, structured errors.
3. **Conversation APIs**
   - Wrap `signal-cli rpc listChats` to build `Conversation` objects.
   - Persist conversation metadata in local cache (SQLite/JSON).
4. **Message APIs**
   - Implement historic fetch via `listMessages` bridge with pagination cursors.
   - Normalize messages, map attachments and quoted replies.
5. **Send Pipeline**
   - Integrate `signal-cli rpc send` for text/attachments.
   - Enforce queueing/backoff to respect Signal rate limits.
6. **Event Stream**
   - Subscribe to `signal-cli listen --json` stream.
   - Translate events into MCP notifications.
7. **Search Index**
   - Incrementally index messages in SQLite FTS.
   - Expose `searchMessages` RPC.
8. **Attachments Service**
   - Store media in a secured cache directory with metadata.
   - Provide download endpoints or base64 streaming via MCP.
9. **Testing & Tooling**
   - Unit tests for transformers.
   - Integration tests with a sandbox Signal account.
   - CLI health command (`mcp-signal check`).
10. **Packaging**
   - Dockerfile bundling signal-cli + MCP server.
   - Example configuration (`config.example.toml`).
   - Documentation for registration, secrets, and deployment.

## Open Questions / Next Steps
- Decide whether to ship built-in search (SQLite FTS) or defer.
- Evaluate need for multi-device management (linking vs dedicated number).
- Consider encryption at rest for the local message cache.
- Determine how to expose profile images (limited due to Signal’s encryption model).
- Plan recurrent updates to match Signal’s server version requirements.
