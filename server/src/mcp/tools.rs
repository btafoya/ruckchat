//! MCP tool definitions and handlers.
//!
//! All tools are scoped by the authenticated caller. Errors are returned as
//! tool-level results so that clients can render them, while malformed inputs
//! are returned as protocol-level [`rmcp::Error`]s.

use crate::services::{McpService, PostMessageResult};
use rmcp::model::{CallToolResult, ContentBlock, ErrorData, TextContent, Tool};
use ruckchat_domain::ConversationType;
use ruckchat_id::{MessageId, OrganizationId, UserId};
use serde_json::{Map, Value};
use std::sync::Arc;
use uuid::Uuid;

/// All tools exposed by the RuckChat MCP server.
#[must_use]
pub fn all_tools() -> Vec<Tool> {
    vec![
        list_channels_tool(),
        list_direct_messages_tool(),
        get_messages_tool(),
        search_messages_tool(),
        post_message_tool(),
        get_user_profile_tool(),
    ]
}

/// Dispatches a tool call to the appropriate handler.
pub async fn handle_tool_call(
    mcp: &McpService,
    caller_id: UserId,
    name: &str,
    arguments: Option<&Map<String, Value>>,
) -> Result<CallToolResult, ErrorData> {
    let empty = Map::new();
    let args = arguments.unwrap_or(&empty);

    match name {
        "list_channels" => list_channels(mcp, caller_id, args).await,
        "list_direct_messages" => list_direct_messages(mcp, caller_id, args).await,
        "get_messages" => get_messages(mcp, caller_id, args).await,
        "search_messages" => search_messages(mcp, caller_id, args).await,
        "post_message" => post_message(mcp, caller_id, args).await,
        "get_user_profile" => get_user_profile(mcp, caller_id, args).await,
        _ => Err(ErrorData::invalid_params(
            format!("unknown tool: {name}"),
            None,
        )),
    }
}

fn list_channels_tool() -> Tool {
    Tool::new(
        "list_channels",
        "List channels visible to the caller in an organization.",
        schema(
            &["organization_id"],
            &[("organization_id", json_schema_uuid())],
        ),
    )
}

async fn list_channels(
    mcp: &McpService,
    caller_id: UserId,
    args: &Map<String, Value>,
) -> Result<CallToolResult, ErrorData> {
    let organization_id = required_uuid(args, "organization_id")?;
    match mcp
        .list_channels(caller_id, OrganizationId::from_uuid(organization_id))
        .await
    {
        Ok(channels) => Ok(json_tool_result(&channels)),
        Err(err) => Ok(tool_error(err)),
    }
}

fn list_direct_messages_tool() -> Tool {
    Tool::new(
        "list_direct_messages",
        "List direct message conversations for the caller in an organization.",
        schema(
            &["organization_id"],
            &[("organization_id", json_schema_uuid())],
        ),
    )
}

async fn list_direct_messages(
    mcp: &McpService,
    caller_id: UserId,
    args: &Map<String, Value>,
) -> Result<CallToolResult, ErrorData> {
    let organization_id = required_uuid(args, "organization_id")?;
    match mcp
        .list_direct_messages(caller_id, OrganizationId::from_uuid(organization_id))
        .await
    {
        Ok(conversations) => Ok(json_tool_result(&conversations)),
        Err(err) => Ok(tool_error(err)),
    }
}

fn get_messages_tool() -> Tool {
    Tool::new(
        "get_messages",
        "Fetch recent messages from a conversation the caller can read.",
        schema(
            &["conversation_id", "conversation_type"],
            &[
                ("conversation_id", json_schema_uuid()),
                ("conversation_type", json_schema_conversation_type()),
                ("limit", json_schema_limit()),
                ("offset", json_schema_offset()),
            ],
        ),
    )
}

async fn get_messages(
    mcp: &McpService,
    caller_id: UserId,
    args: &Map<String, Value>,
) -> Result<CallToolResult, ErrorData> {
    let conversation_id = required_uuid(args, "conversation_id")?;
    let conversation_type = required_conversation_type(args, "conversation_type")?;
    let limit = optional_i64(args, "limit", 50);
    let offset = optional_i64(args, "offset", 0);

    match mcp
        .get_messages(
            caller_id,
            conversation_id,
            conversation_type,
            crate::services::dto::Pagination { limit, offset },
        )
        .await
    {
        Ok(messages) => Ok(json_tool_result(&messages)),
        Err(err) => Ok(tool_error(err)),
    }
}

fn search_messages_tool() -> Tool {
    Tool::new(
        "search_messages",
        "Search message content visible to the caller in an organization.",
        schema(
            &["organization_id", "query"],
            &[
                ("organization_id", json_schema_uuid()),
                ("query", json_schema_string("Search query.")),
                ("limit", json_schema_limit()),
                ("offset", json_schema_offset()),
            ],
        ),
    )
}

async fn search_messages(
    mcp: &McpService,
    caller_id: UserId,
    args: &Map<String, Value>,
) -> Result<CallToolResult, ErrorData> {
    let organization_id = required_uuid(args, "organization_id")?;
    let query = required_str(args, "query")?;
    let limit = optional_i64(args, "limit", 50);
    let offset = optional_i64(args, "offset", 0);

    match mcp
        .search_messages(
            caller_id,
            OrganizationId::from_uuid(organization_id),
            query,
            crate::services::dto::Pagination { limit, offset },
        )
        .await
    {
        Ok(messages) => Ok(json_tool_result(&messages)),
        Err(err) => Ok(tool_error(err)),
    }
}

fn post_message_tool() -> Tool {
    Tool::new(
        "post_message",
        "Post a message to a channel or DM conversation. Requires confirmed: true when MCP_REQUIRE_CONFIRMATION is enabled.",
        schema(
            &["conversation_id", "conversation_type", "content"],
            &[
                ("conversation_id", json_schema_uuid()),
                ("conversation_type", json_schema_conversation_type()),
                ("content", json_schema_string("Message content.")),
                ("parent_id", json_schema_optional_uuid()),
                ("confirmed", json_schema_boolean()),
            ],
        ),
    )
}

async fn post_message(
    mcp: &McpService,
    caller_id: UserId,
    args: &Map<String, Value>,
) -> Result<CallToolResult, ErrorData> {
    let conversation_id = required_uuid(args, "conversation_id")?;
    let conversation_type = required_conversation_type(args, "conversation_type")?;
    let content = required_str(args, "content")?.to_string();
    let parent_id = optional_uuid(args, "parent_id").map(MessageId::from_uuid);
    let confirmed = optional_bool(args, "confirmed", false);

    match mcp
        .post_message(
            caller_id,
            conversation_id,
            conversation_type,
            content,
            parent_id,
            confirmed,
        )
        .await
    {
        Ok(PostMessageResult::Posted(message)) => Ok(json_tool_result(&message)),
        Ok(PostMessageResult::ConfirmationRequired { .. }) => Ok(CallToolResult::success(vec![
            ContentBlock::Text(TextContent::new(
                "Confirmation required before posting. Call post_message again with confirmed: true."
                    .to_string(),
            )),
        ])),
        Err(err) => Ok(tool_error(err)),
    }
}

fn get_user_profile_tool() -> Tool {
    Tool::new(
        "get_user_profile",
        "Read a user profile if it is visible to the caller.",
        schema(&["user_id"], &[("user_id", json_schema_uuid())]),
    )
}

async fn get_user_profile(
    mcp: &McpService,
    caller_id: UserId,
    args: &Map<String, Value>,
) -> Result<CallToolResult, ErrorData> {
    let user_id = required_uuid(args, "user_id")?;
    match mcp
        .get_user_profile(caller_id, UserId::from_uuid(user_id))
        .await
    {
        Ok(user) => Ok(json_tool_result(&user)),
        Err(err) => Ok(tool_error(err)),
    }
}

fn json_tool_result<T: serde::Serialize>(value: &T) -> CallToolResult {
    let text = serde_json::to_string_pretty(value)
        .unwrap_or_else(|err| format!("{{ \"serialization_error\": \"{}\" }}", err));
    CallToolResult::success(vec![ContentBlock::Text(TextContent::new(text))])
}

fn tool_error(err: ruckchat_common::Error) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::Text(TextContent::new(err.to_string()))])
}

fn schema(
    required: &[&'static str],
    properties: &[(&'static str, Value)],
) -> Arc<Map<String, Value>> {
    let mut map = Map::new();
    let props: Map<String, Value> = properties
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.clone()))
        .collect();
    map.insert("type".to_string(), Value::String("object".to_string()));
    map.insert("properties".to_string(), Value::Object(props));
    if !required.is_empty() {
        map.insert(
            "required".to_string(),
            Value::Array(
                required
                    .iter()
                    .map(|s| Value::String((*s).to_string()))
                    .collect(),
            ),
        );
    }
    Arc::new(map)
}

fn json_schema_uuid() -> Value {
    serde_json::json!({ "type": "string", "format": "uuid" })
}

fn json_schema_optional_uuid() -> Value {
    serde_json::json!({ "type": "string", "format": "uuid" })
}

fn json_schema_string(description: &'static str) -> Value {
    serde_json::json!({ "type": "string", "description": description })
}

fn json_schema_conversation_type() -> Value {
    serde_json::json!({ "type": "string", "enum": ["channel", "dm"] })
}

fn json_schema_limit() -> Value {
    serde_json::json!({ "type": "integer", "minimum": 1, "maximum": 100, "default": 50 })
}

fn json_schema_offset() -> Value {
    serde_json::json!({ "type": "integer", "minimum": 0, "default": 0 })
}

fn json_schema_boolean() -> Value {
    serde_json::json!({ "type": "boolean" })
}

fn required_str<'a>(args: &'a Map<String, Value>, key: &str) -> Result<&'a str, ErrorData> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| ErrorData::invalid_params(format!("missing {key}"), None))
}

fn required_uuid(args: &Map<String, Value>, key: &str) -> Result<Uuid, ErrorData> {
    let s = required_str(args, key)?;
    s.parse::<Uuid>()
        .map_err(|err| ErrorData::invalid_params(format!("invalid {key}: {err}"), None))
}

fn optional_uuid(args: &Map<String, Value>, key: &str) -> Option<Uuid> {
    args.get(key)
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<Uuid>().ok())
}

fn required_conversation_type(
    args: &Map<String, Value>,
    key: &str,
) -> Result<ConversationType, ErrorData> {
    let s = required_str(args, key)?;
    s.parse::<ConversationType>().map_err(|_| {
        ErrorData::invalid_params(format!("invalid {key}: must be 'channel' or 'dm'"), None)
    })
}

fn optional_i64(args: &Map<String, Value>, key: &str, default: i64) -> i64 {
    args.get(key).and_then(Value::as_i64).unwrap_or(default)
}

fn optional_bool(args: &Map<String, Value>, key: &str, default: bool) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(default)
}
