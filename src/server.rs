use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rust_mcp_sdk::mcp_server::{server_runtime, ServerHandler};
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use rust_mcp_sdk::schema::{
    CallToolRequest, CallToolResult, Implementation, InitializeResult, ListResourcesRequest,
    ListResourcesResult, ListToolsRequest, ListToolsResult, ReadResourceRequest,
    ReadResourceResult, Resource, ServerCapabilities, ServerCapabilitiesResources,
    ServerCapabilitiesTools, TextContent, TextResourceContents, Tool, ToolAnnotations,
    ToolInputSchema, LATEST_PROTOCOL_VERSION,
};
use rust_mcp_sdk::{McpServer, StdioTransport, TransportOptions};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{Map, Value};
use tracing::{info, warn};

use crate::settings::Settings;
use crate::signal_cli::SignalCli;

const LIST_CONVERSATIONS_TOOL: &str = "signal_list_conversations";
const SEND_MESSAGE_TOOL: &str = "signal_send_message";
const RESOURCE_OVERVIEW_URI: &str = "resource://signal/overview";

pub struct Server {
    settings: Settings,
    signal_cli: Arc<SignalCli>,
}

impl Server {
    pub async fn new(settings: Settings) -> Result<Self> {
        info!("initializing server components");
        let signal_cli = Arc::new(SignalCli::new(
            settings.signal_cli_path.clone(),
            settings.account.clone(),
        ));
        Ok(Self {
            settings,
            signal_cli,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let transport = StdioTransport::new(TransportOptions::default())
            .map_err(|err| anyhow!("failed to create stdio transport: {err}"))?;

        let server_details = self.build_server_details();
        let handler = SignalMcpHandler::new(self.signal_cli.clone());

        let runtime = server_runtime::create_server(server_details, transport, handler);
        info!("signal MCP server runtime started; waiting for MCP client initialization");

        runtime
            .start()
            .await
            .map_err(|err| anyhow!("mcp runtime error: {err}"))
    }

    fn build_server_details(&self) -> InitializeResult {
        InitializeResult {
            server_info: Implementation {
                name: "signal-mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: Some("Signal MCP Server".to_string()),
            },
            capabilities: self.server_capabilities(),
            instructions: Some(self.server_instructions()),
            meta: None,
            protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        }
    }

    fn server_capabilities(&self) -> ServerCapabilities {
        let mut capabilities = ServerCapabilities::default();
        capabilities.tools = Some(ServerCapabilitiesTools {
            list_changed: Some(false),
        });
        capabilities.resources = Some(ServerCapabilitiesResources {
            list_changed: Some(false),
            subscribe: Some(false),
        });
        capabilities
    }

    fn server_instructions(&self) -> String {
        format!(
            "Expose Signal conversations for account {}. Use `{}` to fetch metadata, `{}` to send messages, or read `{}` for setup guidance.",
            self.settings.account, LIST_CONVERSATIONS_TOOL, SEND_MESSAGE_TOOL, RESOURCE_OVERVIEW_URI
        )
    }
}

struct SignalMcpHandler {
    signal_cli: Arc<SignalCli>,
    tools: Vec<Tool>,
    resources: Vec<ResourceEntry>,
}

#[derive(Debug, Deserialize)]
struct SendMessageArgs {
    recipient: String,
    message: String,
}

struct ResourceEntry {
    descriptor: Resource,
    body: String,
}

impl SignalMcpHandler {
    fn new(signal_cli: Arc<SignalCli>) -> Self {
        let tools = vec![build_list_conversations_tool(), build_send_message_tool()];
        let resources = build_resource_entries();
        Self {
            signal_cli,
            tools,
            resources,
        }
    }

    async fn invoke_list_conversations(
        &self,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        match self.signal_cli.list_chats().await {
            Ok(chats) => {
                if chats.is_empty() {
                    let content =
                        TextContent::new("No Signal conversations found.".to_string(), None, None);
                    Ok(CallToolResult::text_content(vec![content]))
                } else {
                    let mut lines = Vec::with_capacity(chats.len());
                    for chat in chats {
                        let label = chat.name.as_deref().unwrap_or("<unnamed>");
                        lines.push(format!("{} — {}", chat.id, label));
                    }
                    let body = lines.join("\n");
                    let content = TextContent::new(body, None, None);
                    Ok(CallToolResult::text_content(vec![content]))
                }
            }
            Err(err) => {
                warn!(?err, "signal-cli listChats failed from tool invocation");
                Err(CallToolError::from_message(format!(
                    "signal-cli listChats failed: {}",
                    err
                )))
            }
        }
    }

    async fn invoke_send_message(
        &self,
        args: SendMessageArgs,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        if args.message.trim().is_empty() {
            return Err(CallToolError::from_message(
                "message text must not be empty".to_string(),
            ));
        }

        match self
            .signal_cli
            .send_message(&args.recipient, &args.message)
            .await
        {
            Ok(receipt) => {
                let mut lines = vec![
                    format!("Message delivered to {}", args.recipient),
                    format!("signal-cli response: {}", receipt),
                ];
                if receipt.is_empty() {
                    lines.push("No response payload from signal-cli".to_string());
                }
                let content = TextContent::new(lines.join("\n"), None, None);
                Ok(CallToolResult::text_content(vec![content]))
            }
            Err(err) => {
                warn!(?err, "signal-cli send failed from tool invocation");
                Err(CallToolError::from_message(format!(
                    "signal-cli send failed: {}",
                    err
                )))
            }
        }
    }
}

#[async_trait]
impl ServerHandler for SignalMcpHandler {
    async fn handle_list_tools_request(
        &self,
        _request: ListToolsRequest,
        runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, rust_mcp_sdk::schema::RpcError> {
        let method = ListToolsRequest::method_name();
        runtime.assert_server_request_capabilities(&method)?;

        Ok(ListToolsResult {
            tools: self.tools.clone(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_list_resources_request(
        &self,
        _request: ListResourcesRequest,
        runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListResourcesResult, rust_mcp_sdk::schema::RpcError> {
        let method = ListResourcesRequest::method_name();
        runtime.assert_server_request_capabilities(&method)?;

        let resources = self
            .resources
            .iter()
            .map(|entry| entry.descriptor.clone())
            .collect();

        Ok(ListResourcesResult {
            resources,
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let method = CallToolRequest::method_name();
        runtime
            .assert_server_request_capabilities(&method)
            .map_err(CallToolError::new)?;

        let params = request.params;
        let name = params.name;
        match name.as_str() {
            LIST_CONVERSATIONS_TOOL => self.invoke_list_conversations().await,
            SEND_MESSAGE_TOOL => {
                let args = parse_arguments::<SendMessageArgs>(params.arguments)?;
                self.invoke_send_message(args).await
            }
            _ => Err(CallToolError::unknown_tool(name)),
        }
    }

    async fn handle_read_resource_request(
        &self,
        request: ReadResourceRequest,
        runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ReadResourceResult, rust_mcp_sdk::schema::RpcError> {
        let method = ReadResourceRequest::method_name();
        runtime.assert_server_request_capabilities(&method)?;

        let uri = request.params.uri;
        if let Some(entry) = self
            .resources
            .iter()
            .find(|entry| entry.descriptor.uri == uri)
        {
            let contents = TextResourceContents {
                meta: None,
                mime_type: Some("text/markdown".to_string()),
                text: entry.body.clone(),
                uri: uri.clone(),
            };

            Ok(ReadResourceResult {
                contents: vec![contents.into()],
                meta: None,
            })
        } else {
            Err(rust_mcp_sdk::schema::RpcError::invalid_params()
                .with_message(format!("Unknown resource URI: {}", uri)))
        }
    }
}

fn build_list_conversations_tool() -> Tool {
    let mut annotations = ToolAnnotations::default();
    annotations.read_only_hint = Some(true);
    annotations.destructive_hint = Some(false);

    let input_schema = ToolInputSchema::new(Vec::new(), None);

    Tool {
        annotations: Some(annotations),
        description: Some(
            "Return known Signal contacts and groups using signal-cli listContacts/listGroups."
                .into(),
        ),
        input_schema,
        meta: None,
        name: LIST_CONVERSATIONS_TOOL.to_string(),
        output_schema: None,
        title: Some("List Signal Conversations".into()),
    }
}

fn build_send_message_tool() -> Tool {
    let mut annotations = ToolAnnotations::default();
    annotations.read_only_hint = Some(false);
    annotations.destructive_hint = Some(false);

    let mut properties: HashMap<String, Map<String, Value>> = HashMap::new();

    let mut recipient_schema = Map::new();
    recipient_schema.insert("type".to_string(), Value::String("string".into()));
    recipient_schema.insert(
        "description".to_string(),
        Value::String("Signal recipient in E.164 form or group ID".into()),
    );
    properties.insert("recipient".to_string(), recipient_schema);

    let mut message_schema = Map::new();
    message_schema.insert("type".to_string(), Value::String("string".into()));
    message_schema.insert(
        "description".to_string(),
        Value::String("Message body to send".into()),
    );
    properties.insert("message".to_string(), message_schema);

    let input_schema = ToolInputSchema::new(
        vec!["recipient".to_string(), "message".to_string()],
        Some(properties),
    );

    Tool {
        annotations: Some(annotations),
        description: Some("Send a Signal message using signal-cli".into()),
        input_schema,
        meta: None,
        name: SEND_MESSAGE_TOOL.to_string(),
        output_schema: None,
        title: Some("Send Signal Message".into()),
    }
}

fn parse_arguments<T>(
    arguments: Option<Map<String, Value>>,
) -> std::result::Result<T, CallToolError>
where
    T: DeserializeOwned,
{
    let map = arguments.unwrap_or_else(Map::new);
    let value = Value::Object(map);
    serde_json::from_value(value).map_err(|err| CallToolError::from_message(err.to_string()))
}

fn build_resource_entries() -> Vec<ResourceEntry> {
    let overview_body = r#"
# Signal MCP Server Overview

This MCP server bridges a linked Signal account via `signal-cli`.

## Current Tools

- `signal_list_conversations` — lists known contacts and group chats using `signal-cli listContacts`/`listGroups`.
- `signal_send_message` — sends a text message to a phone number or group ID via `signal-cli send`.

## Configuration

Provide a `config.toml` (or `SIGNAL_MCP__*` environment variables) with:

```
account = "+1XXXXXXXXXX"
signal_cli_path = "/path/to/signal-cli"
storage = "./var"
```

The Signal account must already be linked or registered using `signal-cli`.

## Roadmap Highlights

- Fetch and normalize conversation/message history.
- Stream live events from `signal-cli jsonRpc` and expose MCP notifications.
- Attachment handling, search, health checks, and richer telemetry.
"#;

    let descriptor = Resource {
        annotations: None,
        description: Some(
            "Overview of available Signal MCP capabilities and configuration.".into(),
        ),
        meta: None,
        mime_type: Some("text/markdown".into()),
        name: "signal.overview".into(),
        size: None,
        title: Some("Signal MCP Overview".into()),
        uri: RESOURCE_OVERVIEW_URI.into(),
    };

    vec![ResourceEntry {
        descriptor,
        body: overview_body.trim().to_string(),
    }]
}
