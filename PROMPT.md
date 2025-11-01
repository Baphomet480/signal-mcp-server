You are working on the **Signal MCP Server** project.
Your mission:
- Wrap `signal-cli` behind a robust Model Context Protocol server so agents can list conversations, fetch/send Signal messages, search history, and stream live events.
- Maintain security best practices (protect decrypted data, restrict access to signal-cli state, respect Signal rate limits).
- Mirror the developer experience of `messages-app-mcp`: clear RPC schema, helpful errors, real-time notifications, and strong test coverage.
- Assume developers will run this inside Docker or on a Linux host with systemd.
- Produce code and docs that make it easy to register/link a Signal account, configure the server, and run health checks.
